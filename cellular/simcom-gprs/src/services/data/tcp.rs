use core::sync::atomic::Ordering;

use atat::{asynch::AtatClient, AtatCmd, AtatUrcChannel, Error};
use core::fmt::Write as _;
use embassy_time::{with_timeout, Duration, Instant};
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
    device::Handle,
};

use super::{DataService, SocketError, SOCKET_STATE_DROPPED, SOCKET_STATE_USED};

impl<'buf, 'dev, 'sub, AtCl: AtatClient, AtUrcCh: AtatUrcChannel<Urc>> TcpConnect
    for DataService<'buf, 'dev, 'sub, AtCl, AtUrcCh>
{
    type Error = SocketError;

    type Connection<'m> = TcpSocket<'buf, 'dev, 'sub, AtCl, AtUrcCh> where Self : 'm;

    async fn connect<'m>(&'m self, remote: SocketAddr) -> Result<Self::Connection<'m>, Self::Error>
    where
        Self: 'm,
    {
        self.handle.drain_background_urcs();

        // Close any sockets that have been dropped
        self.close_dropped_sockets().await;

        let mut socket = TcpSocket::try_new(self.handle, self.urc_channel)?;
        info!("[{}] Socket created", socket.id);

        let mut ip = String::<15>::new();
        write!(ip, "{}", remote.ip()).unwrap();

        let mut port = String::<5>::new();
        write!(port, "{}", remote.port()).unwrap();

        socket.connect(&ip, &port).await?;
        Ok(socket)
    }
}

pub struct TcpSocket<'buf, 'dev, 'sub, AtCl: AtatClient, AtUrcCh: AtatUrcChannel<Urc>> {
    id: usize,
    handle: &'dev Handle<'sub, AtCl>,
    urc_channel: &'buf AtUrcCh,
    max_urc_len: usize,
}

impl<'buf, 'dev, 'sub, AtCl: AtatClient, AtUrcCh: AtatUrcChannel<Urc>>
    TcpSocket<'buf, 'dev, 'sub, AtCl, AtUrcCh>
{
    pub(crate) fn try_new(
        handle: &'dev Handle<'sub, AtCl>,
        urc_channel: &'buf AtUrcCh,
    ) -> Result<Self, SocketError> {
        let id = handle.take_unused()?;
        Ok(Self {
            id,
            handle,
            urc_channel,
            max_urc_len: AtUrcCh::max_urc_len(),
        })
    }

    async fn connect(&mut self, ip: &str, port: &str) -> Result<(), SocketError> {
        self.handle.drain_background_urcs();

        let mut urc_subscription = self.urc_channel.subscribe().unwrap();

        {
            let mut client = self.handle.client.lock().await;

            client
                .send(&StartConnection {
                    id: self.id,
                    mode: "TCP",
                    ip,
                    port,
                })
                .await?;
        }

        let timeout_instant =
            Instant::now() + Duration::from_millis(StartConnection::MAX_TIMEOUT_MS as u64);
        while let Some(timeout) = timeout_instant.checked_duration_since(Instant::now()) {
            // Wait for next urc
            let urc = with_timeout(timeout, urc_subscription.next_message_pure())
                .await
                .map_err(|_| SocketError::ConnectTimeout)?;
            self.handle.drain_background_urcs();

            match urc {
                Urc::ConnectOk(id) if id == self.id => return Ok(()),
                Urc::ConnectFail(id) if id == self.id => return Err(SocketError::UnableToConnect),
                _ => {}
            }
        }

        Err(SocketError::ConnectTimeout)
    }

    fn ensure_in_use(&self) -> Result<(), SocketError> {
        if self.handle.socket_state[self.id].load(Ordering::Acquire) == SOCKET_STATE_USED {
            Ok(())
        } else {
            Err(SocketError::Closed)
        }
    }

    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, SocketError> {
        self.handle.drain_background_urcs();
        self.ensure_in_use()?;
        if buf.is_empty() {
            return Ok(0);
        }

        const MAX_READ: usize = 1460;
        const MAX_HEADER_LEN: usize = "\r\n+CIPRXGET: 1,1,4444,4444\r\n".len();
        const TAIL_LEN: usize = "\r\nOK\r\n".len();
        let max_len = usize::min(
            usize::min(buf.len(), MAX_READ),
            self.max_urc_len - MAX_HEADER_LEN - TAIL_LEN,
        );

        let mut urc_subscription = {
            let mut client = self.handle.client.lock().await;
            let subscription = self.urc_channel.subscribe().unwrap();

            client
                .send(&ReadData {
                    id: self.id,
                    max_len,
                })
                .await?;

            subscription
        };
        let mut no_data_response_received = false;

        let timeout_instant = Instant::now() + Duration::from_secs(10);
        while let Some(timeout) = timeout_instant.checked_duration_since(Instant::now()) {
            // Wait for next urc
            let urc = with_timeout(timeout, urc_subscription.next_message_pure())
                .await
                .map_err(|_| SocketError::ReadTimeout)?;
            self.handle.drain_background_urcs();

            // The socket may have closed while handling urc
            self.ensure_in_use()?;

            match urc {
                Urc::ReadData(r) if r.id == self.id => {
                    if r.data_len > 0 {
                        buf[..r.data_len].copy_from_slice(r.data.take().unwrap().as_slice());
                        return Ok(r.data_len);
                    }

                    // There was no data - start waiting for the DataAvailable urc
                    no_data_response_received = true;
                }
                Urc::DataAvailable(id) if id == self.id => {
                    // Re-request data now when we know that it is available
                    // Only do so if we have not yet processed the ReadData urc
                    if no_data_response_received {
                        let mut client = self.handle.client.lock().await;
                        client
                            .send(&ReadData {
                                id: self.id,
                                max_len,
                            })
                            .await?;
                    }
                }
                _ => {}
            }
        }

        Err(SocketError::ReadTimeout)
    }

    async fn write(&mut self, buf: &[u8]) -> Result<usize, SocketError> {
        self.handle.drain_background_urcs();
        self.ensure_in_use()?;
        self.wait_ongoing_write().await?;

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
                    if self.handle.data_available[self.id].load(Ordering::Acquire) {
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
        self.handle.data_written[self.id].store(false, Ordering::Release);

        // Write the data buffer
        client.send(&WriteData { buf }).await?;

        debug!("[{}] Wrote {} bytes", self.id, buf.len());

        Ok(len)
    }

    async fn wait_ongoing_write(&mut self) -> Result<(), SocketError> {
        self.handle.drain_background_urcs();

        if self.handle.data_written[self.id].load(Ordering::Acquire) {
            trace!("[{}] Data already written", self.id);
            return Ok(());
        }

        let mut urc_subscription = self.urc_channel.subscribe().unwrap();

        let timeout_instant =
            Instant::now() + Duration::from_millis(WriteData::MAX_TIMEOUT_MS as u64);
        while let Some(timeout) = timeout_instant.checked_duration_since(Instant::now()) {
            // Wait for next urc
            with_timeout(timeout, urc_subscription.next_message_pure())
                .await
                .map_err(|_| SocketError::WriteTimeout)?;
            self.handle.drain_background_urcs();

            // The socket may have closed while handling urc
            self.ensure_in_use()?;

            if self.handle.data_written[self.id].load(Ordering::Acquire) {
                trace!("[{}] Data was written", self.id);
                return Ok(());
            } else {
                trace!("[{}] Data is not yet written", self.id);
            }
        }

        Err(SocketError::WriteTimeout)
    }
}

impl<AtCl: AtatClient, AtUrcCh: AtatUrcChannel<Urc>> Io for TcpSocket<'_, '_, '_, AtCl, AtUrcCh> {
    type Error = SocketError;
}

impl<AtCl: AtatClient, AtUrcCh: AtatUrcChannel<Urc>> Read for TcpSocket<'_, '_, '_, AtCl, AtUrcCh> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, SocketError> {
        self.read(buf).await
    }
}

impl<AtCl: AtatClient, AtUrcCh: AtatUrcChannel<Urc>> Write
    for TcpSocket<'_, '_, '_, AtCl, AtUrcCh>
{
    async fn write(&mut self, buf: &[u8]) -> Result<usize, SocketError> {
        self.write(buf).await
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        // We do not do any buffering in the data so all writes are sent to the uart immediately
        // We cannot wait for the modem to reply "SENT OK" using wait_data_written()
        // as this can cause deadlocks if the application does flush().await before it starts read().await.
        Ok(())
    }
}

impl<AtCl: AtatClient, AtUrcCh: AtatUrcChannel<Urc>> Drop for TcpSocket<'_, '_, '_, AtCl, AtUrcCh> {
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
