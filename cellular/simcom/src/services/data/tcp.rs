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
    services::data::SOCKET_STATE_UNUSED,
};

use super::{DataService, SocketError, SOCKET_STATE_DROPPED, SOCKET_STATE_USED};

// There is an example implementation of TcpConnect here: https://github.com/drogue-iot/esp8266-at-driver/blob/c49a6b469da6991b680a166e7c5e236d5fb4c560/src/lib.rs

impl<'a, At: AtatClient, Delay: DelayUs + Clone> TcpConnect for DataService<'a, At, Delay> {
    type Error = SocketError;

    type Connection<'m> = TcpSocket<'m, At, Delay> where Self : 'm;

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
                client.try_read_urc_with::<Urc, _>(|urc| match urc {
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
    flushed: bool,
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
            flushed: true,
        }
    }

    fn ensure_in_use(&self) -> Result<(), SocketError> {
        if self.handle.socket_state[self.id].load(Ordering::Acquire) == SOCKET_STATE_USED {
            Ok(())
        } else {
            Err(SocketError::Closed)
        }
    }
}

impl<At: AtatClient, Delay: DelayUs> Io for TcpSocket<'_, At, Delay> {
    type Error = SocketError;
}

impl<At: AtatClient, Delay: DelayUs> Read for TcpSocket<'_, At, Delay> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, SocketError> {
        self.ensure_in_use()?;
        if buf.is_empty() {
            return Ok(0);
        }

        const MAX_HEADER_LEN: usize = "\r\n+CIPRXGET: 1,1,4444,4444\r\n".len();
        const TAIL_LEN: usize = "\r\nOK\r\n".len();
        let len = usize::min(
            At::max_response_len() - MAX_HEADER_LEN - TAIL_LEN,
            buf.len(),
        );

        let mut client = self.handle.client.lock().await;

        let response = client
            .send(&ReadData::new(self.id, &mut buf[..len]))
            .await?;

        Ok(response.data_len)
    }
}

impl<At: AtatClient, Delay: DelayUs> Write for TcpSocket<'_, At, Delay> {
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

                    let info = client.send(&ReadData::new(self.id, &mut [][..])).await?;
                    if info.pending_len > 0 {
                        debug!("CME ERROR 3 during write because of pending data");
                        SocketError::MustReadBeforeWrite
                    } else {
                        SocketError::Atat(Error::CmeError(e))
                    }
                }
                e => SocketError::Atat(e),
            });
        }

        client.send(&WriteData { buf }).await?;

        self.flushed = false;

        Ok(len)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.ensure_in_use()?;
        if self.flushed {
            return Ok(());
        }

        const TRIALS: u32 = WriteData::MAX_TIMEOUT_MS / 100;
        for _ in 0..TRIALS {
            let mut must_read = false;
            let mut client = self.handle.client.lock().await;
            client.try_read_urc_with::<Urc, _>(|urc| match urc {
                Urc::SendOk(id) if id == self.id => {
                    self.flushed = true;
                    true
                }
                Urc::Closed(id) if id == self.id => {
                    warn!("[{}] Socket closed", id);
                    true
                }
                Urc::DataAvailable(id) if id == self.id => {
                    must_read = true;
                    true
                }
                urc => self.handle.handle_urc(&urc),
            });

            if must_read {
                return Err(SocketError::MustReadBeforeWrite);
            }

            if self.flushed {
                break;
            }

            self.delay.delay_ms(100).await.unwrap();
        }

        if self.flushed {
            trace!("SEND OK RECEIVED");
            Ok(())
        } else {
            self.handle.socket_state[self.id].store(SOCKET_STATE_UNUSED, Ordering::Release);
            Err(SocketError::Closed)
        }
    }
}

impl<At: AtatClient, Delay: DelayUs> Drop for TcpSocket<'_, At, Delay> {
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
