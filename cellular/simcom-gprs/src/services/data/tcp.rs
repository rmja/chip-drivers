use core::sync::atomic::Ordering;

use atat::{asynch::AtatClient, AtatCmd, Error};
use core::fmt::Write as _;
use embassy_time::{Duration, Timer};
use embedded_io::{
    asynch::{Read, Write},
    Io,
};
use embedded_nal_async::{SocketAddr, TcpConnect};
use futures::{future::select, pin_mut};
use heapless::String;

use crate::{
    commands::{
        tcpip::{ReadData, SendData, StartConnection, WriteData},
        urc::Urc,
    },
    device::{Handle, CONNECTED_STATE_CONNECTED, CONNECTED_STATE_FAILED, CONNECTED_STATE_UNKNOWN},
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

            if self.handle.connected_state[socket.id].load(Ordering::Relaxed)
                != CONNECTED_STATE_UNKNOWN
            {
                break;
            }

            Timer::after(Duration::from_millis(200)).await;
        }

        match self.handle.connected_state[socket.id].load(Ordering::Relaxed) {
            CONNECTED_STATE_CONNECTED => Ok(socket),
            CONNECTED_STATE_FAILED => Err(SocketError::UnableToConnect),
            _ => Err(SocketError::ConnectTimeout),
        }
    }
}

pub struct TcpSocket<'a, AtCl: AtatClient> {
    id: usize,
    handle: &'a Handle<AtCl>,
}

impl<'a, AtCl: AtatClient> TcpSocket<'a, AtCl> {
    pub(crate) fn try_new(handle: &'a Handle<AtCl>) -> Result<Self, SocketError> {
        let id = handle.take_unused()?;
        Ok(Self { id, handle })
    }
}

impl<AtCl: AtatClient> Handle<AtCl> {
    fn ensure_in_use(&self, id: usize) -> Result<(), SocketError> {
        if self.socket_state[id].load(Ordering::Acquire) == SOCKET_STATE_USED {
            Ok(())
        } else {
            Err(SocketError::Closed)
        }
    }

    async fn read(&self, id: usize, buf: &mut [u8]) -> Result<(usize, usize), SocketError> {
        const MAX_READ: usize = 1460;
        const MAX_HEADER_LEN: usize = "\r\n+CIPRXGET: 1,1,4444,4444\r\n".len();
        const TAIL_LEN: usize = "\r\nOK\r\n".len();
        let max_len = usize::min(
            usize::min(buf.len(), MAX_READ),
            AtCl::max_urc_len() - MAX_HEADER_LEN - TAIL_LEN,
        );

        {
            let mut client = self.client.lock().await;
            client.send(&ReadData { id, max_len }).await?;
        }

        let mut result = None;

        loop {
            let handled = {
                let mut client = self.client.lock().await;
                client.try_read_urc_with::<Urc, _>(|urc, urc_buf| match urc {
                    Urc::ReadData(r) => {
                        // The data is in the end of the urc buffer
                        let offset = urc_buf.len() - r.data_len;
                        buf[..r.data_len].copy_from_slice(&urc_buf[offset..]);
                        result = Some(r);
                        true
                    }
                    urc => self.handle_urc(&urc),
                })
            };

            // The socket may have closed while handling urc
            self.ensure_in_use(id)?;

            if let Some(result) = result {
                self.data_available[id].store(result.pending_len > 0, Ordering::Release);
                return Ok((result.data_len, result.pending_len));
            }

            if !handled {
                Timer::after(Duration::from_millis(10)).await;
            }
        }
    }

    async fn wait_for_data_available(&self, id: usize) -> Result<(), SocketError> {
        if self.data_available[id].load(Ordering::Acquire) {
            trace!("[{}] Data is already available", id);
            return Ok(());
        }

        const TRIALS: u32 = 10_000 / 100; // Read timeout: 10 seconds

        for trial in 1..=TRIALS {
            {
                let mut client = self.client.lock().await;
                client.try_read_urc_with::<Urc, _>(|urc, _| self.handle_urc(&urc));
            }

            // The socket may have closed while handling urc
            self.ensure_in_use(id)?;

            if self.data_available[id].load(Ordering::Acquire) {
                trace!("[{}] Data found to be available in {} trials", id, trial);
                return Ok(());
            } else {
                trace!("[{}] Data is not yet available, trial #{}", id, trial);
            }

            Timer::after(Duration::from_millis(100)).await;
        }

        Err(SocketError::ReadTimeout)
    }
}

impl<AtCl: AtatClient> Io for TcpSocket<'_, AtCl> {
    type Error = SocketError;
}

impl<'a, AtCl: AtatClient> Read for TcpSocket<'a, AtCl> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, SocketError> {
        self.handle.ensure_in_use(self.id)?;

        if buf.is_empty() {
            return Ok(0);
        }

        {
            let data_available = self.handle.wait_for_data_available(self.id);
            let read_data = self.handle.read(self.id, buf);

            pin_mut!(data_available);
            pin_mut!(read_data);

            match select(data_available, read_data).await {
                futures::future::Either::Left(_) => {}
                futures::future::Either::Right((result, data_available)) => {
                    // ReadData was first, lets see if there was actually any data
                    let (len, _pending) = result?;
                    if len > 0 {
                        return Ok(len);
                    } else {
                        // Wait for data to actually become available
                        data_available.await?;
                    }
                }
            }
        }

        // Data is now available
        let len = self
            .handle
            .read(self.id, buf)
            .await
            .map(|(len, _pending)| len)?;
        assert!(len > 0);
        Ok(len)
    }
}

impl<'a, AtCl: AtatClient + 'a> Write for TcpSocket<'a, AtCl> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, SocketError> {
        self.handle.ensure_in_use(self.id)?;
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

                    let (_, pending) = self.handle.read(self.id, &mut []).await?;
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
        self.handle.ensure_in_use(self.id)?;
        if self.handle.is_flushed[self.id].load(Ordering::Acquire) {
            return Ok(());
        }

        const TRIALS: u32 = WriteData::MAX_TIMEOUT_MS / 100;
        for _ in 0..TRIALS {
            {
                let mut client = self.handle.client.lock().await;
                client.try_read_urc_with::<Urc, _>(|urc, _| self.handle.handle_urc(&urc));
            }

            // The socket may have closed while handling urc
            self.handle.ensure_in_use(self.id)?;

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