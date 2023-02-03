use core::sync::atomic::Ordering;

use atat::{AtatCmd, Error};
use core::fmt::Write as _;
use embedded_hal_async::delay::DelayUs;
use embedded_io::{
    asynch::{Read, Write},
    Io,
};
use embedded_nal_async::{SocketAddr, TcpConnect};
use heapless::String;

use crate::{
    atat_async::AtatClient,
    commands::{
        tcpip::{ReadData, SendData, StartConnection, WriteData},
        urc::Urc,
    },
    device::Handle,
};

use super::{DataService, SocketError, SOCKET_STATE_DROPPED, SOCKET_STATE_USED};

// There is an example implementation of TcpConnect here: https://github.com/drogue-iot/esp8266-at-driver/blob/c49a6b469da6991b680a166e7c5e236d5fb4c560/src/lib.rs

impl<'a, AtCl: AtatClient, Delay: DelayUs + Clone> TcpConnect for DataService<'a, AtCl, Delay> {
    type Error = SocketError;

    type Connection<'m> = TcpSocket<'m, AtCl, Delay> where Self : 'm;

    async fn connect<'m>(&'m self, remote: SocketAddr) -> Result<Self::Connection<'m>, Self::Error>
    where
        Self: 'm,
    {
        // Close any sockets that have been dropped
        self.close_dropped_sockets().await;

        let id = self.take_socket_id()?;
        let socket = TcpSocket::new(id, self.handle, self.delay.clone());
        debug!("[{}] Socket created", id);

        let mut ip = String::<15>::new();
        write!(ip, "{}", remote.ip()).unwrap();

        let mut port = String::<5>::new();
        write!(port, "{}", remote.port()).unwrap();

        {
            let mut client = self.handle.client.lock().await;

            client
                .send(&StartConnection {
                    id: socket.id,
                    mode: "TCP",
                    ip: &ip,
                    port: &port,
                })
                .await?;
        }

        let mut connected = None;
        let mut delay = self.delay.clone();
        const TRIALS: u32 = StartConnection::MAX_TIMEOUT_MS / 200;
        for _ in 0..TRIALS {
            {
                let mut client = self.handle.client.lock().await;
                client.try_read_urc_with::<Urc, _>(|urc, _| match urc {
                    Urc::ConnectOk(x) if x == id => {
                        connected = Some(true);
                        true
                    }
                    Urc::ConnectFail(x) if x == id => {
                        connected = Some(false);
                        true
                    }
                    urc => self.handle.handle_urc(&urc),
                });
            }

            if connected.is_some() {
                break;
            }

            delay.delay_ms(200).await.unwrap();
        }

        if let Some(connected) = connected {
            if connected {
                Ok(socket)
            } else {
                Err(SocketError::UnableToConnect)
            }
        } else {
            Err(SocketError::ConnectTimeout)
        }
    }
}

pub struct TcpSocket<'a, AtCl: AtatClient, Delay: DelayUs> {
    id: usize,
    handle: &'a Handle<AtCl>,
    delay: Delay,
}

impl<'a, AtCl: AtatClient, Delay: DelayUs> TcpSocket<'a, AtCl, Delay> {
    pub(crate) fn new(
        id: usize,
        // state: &'a SocketState,
        handle: &'a Handle<AtCl>,
        delay: Delay,
    ) -> Self {
        Self {
            id,
            // state,
            handle,
            delay,
        }
    }

    fn ensure_in_use(&self) -> Result<(), SocketError> {
        if self.handle.socket_state[self.id].load(Ordering::Acquire) == SOCKET_STATE_USED {
            Ok(())
        } else {
            Err(SocketError::Closed)
        }
    }

    async fn read_inner(&mut self, buf: &mut [u8]) -> Result<(usize, usize), SocketError> {
        const MAX_READ: usize = 1460;
        const MAX_HEADER_LEN: usize = "\r\n+CIPRXGET: 1,1,4444,4444\r\n".len();
        const TAIL_LEN: usize = "\r\nOK\r\n".len();
        let max_len = usize::min(
            usize::min(buf.len(), MAX_READ),
            AtCl::max_urc_len() - MAX_HEADER_LEN - TAIL_LEN,
        );

        let mut client = self.handle.client.lock().await;

        client
            .send(&ReadData {
                id: self.id,
                max_len,
            })
            .await?;

        let mut result = None;

        loop {
            client.try_read_urc_with::<Urc, _>(|urc, urc_buf| match urc {
                Urc::ReadData(r) => {
                    // The data is in the end of the urc buffer
                    let offset = urc_buf.len() - r.data_len;
                    buf[..r.data_len].copy_from_slice(&urc_buf[offset..]);
                    result = Some(r);
                    true
                }
                urc => self.handle.handle_urc(&urc),
            });

            // The socket may have closed while handling urc
            self.ensure_in_use()?;

            if let Some(result) = result {
                return Ok((result.data_len, result.pending_len));
            }
        }
    }
}

impl<AtCl: AtatClient, Delay: DelayUs> Io for TcpSocket<'_, AtCl, Delay> {
    type Error = SocketError;
}

impl<AtCl: AtatClient, Delay: DelayUs> Read for TcpSocket<'_, AtCl, Delay> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, SocketError> {
        self.ensure_in_use()?;

        if buf.is_empty() {
            return Ok(0);
        }

        self.read_inner(buf).await.map(|(len, _)| len)
    }
}

impl<AtCl: AtatClient, Delay: DelayUs> Write for TcpSocket<'_, AtCl, Delay> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, SocketError> {
        self.ensure_in_use()?;
        self.flush().await?;

        const MAX_WRITE: usize = 1024; // This is the value reported by AT+CIPSEND?
        let len = usize::min(buf.len(), MAX_WRITE);

        let mut client = self.handle.client.lock().await;
        if let Err(error) = client
            .send(&SendData {
                id: self.id,
                len: Some(len),
            })
            .await
        {
            return Err(match error {
                Error::CmeError(e) if e == 3.into() => {
                    // Determine if there is data available
                    // CME ERROR 3 means invalid operation, and we cannot write when there is data available

                    let (_, pending) = self.read_inner(&mut []).await?;
                    if pending > 0 {
                        debug!("CME ERROR 3 during write because of pending data");
                        SocketError::MustReadBeforeWrite
                    } else {
                        SocketError::Atat(Error::CmeError(e))
                    }
                }
                e => SocketError::Atat(e),
            });
        }

        // We have received prompt and are ready to write data
        
        // Clear the flushed flag
        self.handle.flushed[self.id].store(false, Ordering::Release);

        // Write the data buffer
        client.send(&WriteData { buf }).await?;

        Ok(len)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.ensure_in_use()?;
        if self.handle.flushed[self.id].load(Ordering::Acquire) {
            return Ok(());
        }

        const TRIALS: u32 = WriteData::MAX_TIMEOUT_MS / 100;
        for _ in 0..TRIALS {
            let mut data_available = false;
            let mut client = self.handle.client.lock().await;
            client.try_read_urc_with::<Urc, _>(|urc, _| match urc {
                Urc::DataAvailable(id) if id == self.id => {
                    data_available = true;
                    true
                }
                urc => self.handle.handle_urc(&urc),
            });

            // The socket may have closed while handling urc
            self.ensure_in_use()?;

            if self.handle.flushed[self.id].load(Ordering::Acquire) {
                trace!("SEND OK RECEIVED");
                return Ok(());
            } else if data_available {
                return Err(SocketError::MustReadBeforeWrite);
            }

            self.delay.delay_ms(100).await.unwrap();
        }

        Err(SocketError::WriteTimeout)
    }
}

impl<AtCl: AtatClient, Delay: DelayUs> Drop for TcpSocket<'_, AtCl, Delay> {
    fn drop(&mut self) {
        // Only set DROPPED state if the connection is not already closed
        if self.handle.socket_state[self.id]
            .compare_exchange(
                SOCKET_STATE_USED,
                SOCKET_STATE_DROPPED,
                Ordering::AcqRel,
                Ordering::Relaxed,
            )
            .is_ok()
        {
            warn!("[{}] Socket dropped", self.id);
        };
    }
}
