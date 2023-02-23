use core::sync::atomic::Ordering;

use atat::{AtatCmd, Error, asynch::AtatClient};
use embassy_time::{Timer, Duration};
use core::fmt::Write as _;
use embedded_io::{
    asynch::{Read, Write},
    Io,
};
use embedded_nal_async::{SocketAddr, TcpConnect};
use heapless::String;

use crate::{
    commands::{
        tcpip::{ReadData, SendData, StartConnection, WriteData},
        urc::Urc,
    },
    device::{Handle, CONNECTED_STATE_CONNECTED, CONNECTED_STATE_UNKNOWN, CONNECTED_STATE_FAILED},
};

use super::{DataService, SocketError, SOCKET_STATE_DROPPED, SOCKET_STATE_USED};

// There is an example implementation of TcpConnect here: https://github.com/drogue-iot/esp8266-at-driver/blob/c49a6b469da6991b680a166e7c5e236d5fb4c560/src/lib.rs

impl<'a, AtCl: AtatClient> TcpConnect for DataService<'a, AtCl> {
    type Error = SocketError;

    type Connection<'m> = TcpSocket<'m, AtCl> where Self : 'm;

    async fn connect<'m>(&'m self, remote: SocketAddr) -> Result<Self::Connection<'m>, Self::Error>
    where
        Self: 'm,
    {
        // Close any sockets that have been dropped
        self.close_dropped_sockets().await;

        let socket = TcpSocket::try_new(self.handle)?;
        info!("[{}] Socket created", socket.id);

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

        const TRIALS: u32 = StartConnection::MAX_TIMEOUT_MS / 200;
        for trial in 1..=TRIALS {
            debug!("[{}] Testing connection status trial #{}", socket.id, trial);
            {
                let mut client = self.handle.client.lock().await;
                client.try_read_urc_with::<Urc, _>(|urc, _| self.handle.handle_urc(&urc));
            }

            if self.handle.connected_state[socket.id].load(Ordering::Relaxed) != CONNECTED_STATE_UNKNOWN {
                break;
            }

            Timer::after(Duration::from_millis(200)).await;
        }

        match self.handle.connected_state[socket.id].load(Ordering::Relaxed) {
            CONNECTED_STATE_CONNECTED => Ok(socket),
            CONNECTED_STATE_FAILED => Err(SocketError::UnableToConnect),
            _ => Err(SocketError::ConnectTimeout)
        }
    }
}

pub struct TcpSocket<'a, AtCl: AtatClient> {
    id: usize,
    handle: &'a Handle<AtCl>,
}

impl<'a, AtCl: AtatClient> TcpSocket<'a, AtCl> {
    pub(crate) fn try_new(
        handle: &'a Handle<AtCl>,
    ) -> Result<Self, SocketError> {
        let id = handle.take_unused()?;
        Ok(Self {
            id,
            handle,
        })
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

impl<AtCl: AtatClient> Io for TcpSocket<'_, AtCl> {
    type Error = SocketError;
}

impl<AtCl: AtatClient> Read for TcpSocket<'_, AtCl> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, SocketError> {
        self.ensure_in_use()?;

        if buf.is_empty() {
            return Ok(0);
        }

        self.read_inner(buf).await.map(|(len, _)| len)
    }
}

impl<AtCl: AtatClient> Write for TcpSocket<'_, AtCl> {
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
        self.handle.is_flushed[self.id].store(false, Ordering::Release);

        // Write the data buffer
        client.send(&WriteData { buf }).await?;

        Ok(len)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.ensure_in_use()?;
        if self.handle.is_flushed[self.id].load(Ordering::Acquire) {
            return Ok(());
        }

        const TRIALS: u32 = WriteData::MAX_TIMEOUT_MS / 100;
        for _ in 0..TRIALS {
            let mut client = self.handle.client.lock().await;
            client.try_read_urc_with::<Urc, _>(|urc, _| self.handle.handle_urc(&urc));

            // The socket may have closed while handling urc
            self.ensure_in_use()?;

            if self.handle.is_flushed[self.id].load(Ordering::Acquire) {
                trace!("SEND OK RECEIVED");
                return Ok(());
            }

            Timer::after(Duration::from_millis(100)).await;
        }

        Err(SocketError::WriteTimeout)
    }
}

impl<AtCl: AtatClient> Drop for TcpSocket<'_, AtCl> {
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
