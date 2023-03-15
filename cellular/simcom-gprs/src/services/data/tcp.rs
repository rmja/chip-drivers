use core::sync::atomic::Ordering;

use atat::{asynch::AtatClient, AtatCmd, AtatUrcChannel};
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
        })
    }

    async fn connect(&mut self, ip: &str, port: &str) -> Result<(), SocketError> {
        self.handle.drain_background_urcs();

        let mut urc_subscription = {
            let mut client = self.handle.client.lock().await;
            let urc_subscription = self.urc_channel.subscribe().unwrap();

            client
                .send(&StartConnection {
                    id: self.id,
                    mode: "TCP",
                    ip,
                    port,
                })
                .await?;

            urc_subscription
        };

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

    fn drain_background_urcs_and_ensure_in_use(&self) -> Result<(), SocketError> {
        self.handle.drain_background_urcs();

        if self.handle.socket_state[self.id].load(Ordering::Acquire) == SOCKET_STATE_USED {
            Ok(())
        } else {
            Err(SocketError::Closed)
        }
    }

    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, SocketError> {
        self.drain_background_urcs_and_ensure_in_use()?;
        if buf.is_empty() {
            return Ok(0);
        }

        const MAX_READ: usize = 1460;
        const MAX_HEADER_LEN: usize = "\r\n+CIPRXGET: 1,1,4444,4444\r\n".len();
        const TAIL_LEN: usize = "\r\nOK\r\n".len();
        let max_len = usize::min(
            usize::min(buf.len(), MAX_READ),
            self.handle.max_urc_len - MAX_HEADER_LEN - TAIL_LEN,
        );

        let mut urc_subscription = {
            let mut client = self.handle.client.lock().await;
            let urc_subscription = self.urc_channel.subscribe().unwrap();

            trace!("[{}] Sending ReadData", self.id);

            client
                .send(&ReadData {
                    id: self.id,
                    max_len,
                })
                .await?;

            urc_subscription
        };

        let mut no_data_response_received = false;

        let mut timeout_instant = Instant::now() + Duration::from_secs(10);
        while let Some(timeout) = timeout_instant.checked_duration_since(Instant::now()) {
            // Wait for next urc
            let urc = with_timeout(timeout, urc_subscription.next_message_pure())
                .await
                .map_err(|_| SocketError::ReadTimeout)?;

            self.drain_background_urcs_and_ensure_in_use()?;

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
                        debug!("[{}] Re-sending data read request", id);

                        let mut client = self.handle.client.lock().await;

                        // Drain all messages in subscription before re-sending ReadData
                        let mut cnt = 0;
                        while urc_subscription.try_next_message_pure().is_some() {
                            cnt += 1;
                        }
                        trace!(
                            "[{}] Drained {} messages before re-sending data read request",
                            id,
                            cnt
                        );

                        trace!("[{}] Sending ReadData", id);

                        client
                            .send(&ReadData {
                                id: self.id,
                                max_len,
                            })
                            .await?;

                        // Reset timeout to ensure that we in fact read the response
                        timeout_instant = Instant::now() + Duration::from_secs(10);
                    } else {
                        debug!(
                            "[{}] Data available urc received before read data response urc",
                            id
                        );
                    }
                }
                _ => {}
            }
        }

        Err(SocketError::ReadTimeout)
    }

    async fn write(&mut self, buf: &[u8]) -> Result<usize, SocketError> {
        self.wait_ongoing_write().await?;

        const MAX_WRITE: usize = 1024; // This is the value reported by AT+CIPSEND?
        let len = usize::min(buf.len(), MAX_WRITE);

        let mut client = self.handle.client.lock().await;
        // Hold client all the way from request prompt to actually writing data
        client
            .send(&SendData {
                id: self.id,
                len: Some(len),
            })
            .await?;

        // We have received prompt and are ready to write data

        // Clear the flushed flag
        self.handle.data_written[self.id].store(false, Ordering::Release);

        // Write the data buffer
        client.send(&WriteData { buf }).await?;

        debug!("[{}] Wrote {} bytes", self.id, buf.len());

        Ok(len)
    }

    async fn wait_ongoing_write(&mut self) -> Result<(), SocketError> {
        let mut urc_subscription = self.urc_channel.subscribe().unwrap();

        self.drain_background_urcs_and_ensure_in_use()?;

        if self.handle.data_written[self.id].load(Ordering::Acquire) {
            trace!("[{}] Data already written", self.id);
            return Ok(());
        }

        let timeout_instant =
            Instant::now() + Duration::from_millis(WriteData::MAX_TIMEOUT_MS as u64);
        while let Some(timeout) = timeout_instant.checked_duration_since(Instant::now()) {
            // Wait for next urc
            with_timeout(timeout, urc_subscription.next_message_pure())
                .await
                .map_err(|_| SocketError::WriteTimeout)?;

            self.drain_background_urcs_and_ensure_in_use()?;

            if self.handle.data_written[self.id].load(Ordering::Acquire) {
                trace!("[{}] Data is now written", self.id);
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

#[cfg(test)]
mod tests {
    use core::convert::Infallible;

    use atat::{
        bbqueue::{
            framed::{FrameConsumer, FrameProducer},
            BBBuffer,
        },
        AtatIngress,
    };
    use embedded_nal_async::{IpAddr, Ipv4Addr, SocketAddr};
    use tokio::task::yield_now;

    use crate::{
        device::{SocketState, SOCKET_STATE_UNKNOWN, SOCKET_STATE_UNUSED},
        Device, SimcomAtatBuffers, MAX_SOCKETS,
    };

    use super::*;

    struct FrameWriter<'a, const N: usize>(FrameProducer<'a, N>);

    impl<const N: usize> Io for FrameWriter<'_, N> {
        type Error = Infallible;
    }

    impl<const N: usize> Write for FrameWriter<'_, N> {
        async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
            let len = buf.len();
            let mut grant = self.0.grant(len).unwrap();
            grant[..len].copy_from_slice(buf);
            grant.commit(len);
            yield_now().await;
            Ok(len)
        }
    }

    macro_rules! setup_atat {
        () => {{
            static mut BUFFERS: SimcomAtatBuffers<128, 512> = SimcomAtatBuffers::new();
            static SERIAL: BBBuffer<1000> = BBBuffer::new();
            let buffers = unsafe { &mut BUFFERS };
            let (producer, consumer) = SERIAL.try_split_framed().unwrap();
            let (ingress, device) = Device::from_buffers(buffers, FrameWriter(producer));
            (ingress, device, consumer)
        }};
    }

    async fn connect<'buf, 'dev, 'sub, AtCl: AtatClient, AtUrcCh: AtatUrcChannel<Urc>>(
        ingress: &mut impl AtatIngress,
        device: &'dev mut Device<'buf, 'sub, AtCl, AtUrcCh>,
        serial: &mut FrameConsumer<'_, 1000>,
        id: usize,
    ) -> TcpSocket<'buf, 'dev, 'sub, AtCl, AtUrcCh> {
        for _ in 0..MAX_SOCKETS {
            device
                .handle
                .socket_state
                .push(SocketState::new(SOCKET_STATE_UNKNOWN))
                .unwrap();
        }
        device.handle.socket_state[id].store(SOCKET_STATE_UNUSED, Ordering::Relaxed);

        let data = DataService::new(&device.handle, device.urc_channel);

        let socket = async {
            data.connect(SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                8080,
            ))
            .await
            .unwrap()
        };
        let receive = async {
            // Expect StartConnection request
            let request = with_timeout(Duration::from_millis(100), serial.read_async())
                .await
                .unwrap()
                .unwrap();
            assert_eq!(
                format!("AT+CIPSTART={},\"TCP\",\"127.0.0.1\",\"8080\"\r", id).as_bytes(),
                request.as_ref()
            );
            request.release();

            ingress.write(b"\r\nOK\r\n").await;
            ingress
                .write(format!("\r\n{}, CONNECT OK\r\n", id).as_bytes())
                .await;
        };

        tokio::join!(socket, receive).0
    }

    #[tokio::test]
    async fn can_read_available_data() {
        let (mut ingress, mut device, mut serial) = setup_atat!();
        let mut socket = connect(&mut ingress, &mut device, &mut serial, 5).await;

        let read = async {
            let mut buf = [0; 16];
            assert_eq!(8, socket.read(&mut buf).await.unwrap());
        };
        let receive = async {
            // Expect ReadData request
            let request = with_timeout(Duration::from_millis(100), serial.read_async())
                .await
                .unwrap()
                .unwrap();
            assert_eq!(b"AT+CIPRXGET=2,5,16\r", request.as_ref());
            request.release();

            ingress
                .write(b"\r\n+CIPRXGET: 2,5,8,0\r\nHTTP\r\n\r\n")
                .await;
            ingress.write(b"\r\nOK\r\n").await;
        };

        tokio::join!(read, receive);
    }

    #[tokio::test]
    async fn can_read_data_with_data_available_before_read_data() {
        let (mut ingress, mut device, mut serial) = setup_atat!();
        let mut socket = connect(&mut ingress, &mut device, &mut serial, 5).await;

        let read = async {
            let mut buf = [0; 16];
            assert_eq!(8, socket.read(&mut buf).await.unwrap());
        };
        let receive = async {
            // Expect ReadData request
            let request = with_timeout(Duration::from_millis(100), serial.read_async())
                .await
                .unwrap()
                .unwrap();
            assert_eq!(b"AT+CIPRXGET=2,5,16\r", request.as_ref());
            request.release();

            ingress.write(b"\r\n+CIPRXGET: 1,5\r\n").await; // Transmitted by modem before it understands our read request
            ingress
                .write(b"\r\n+CIPRXGET: 2,5,8,0\r\nHTTP\r\n\r\n")
                .await;
            ingress.write(b"\r\nOK\r\n").await;
        };

        tokio::join!(read, receive);
    }

    #[tokio::test]
    async fn can_read_data_with_no_data_initially_available_retrying() {
        let (mut ingress, mut device, mut serial) = setup_atat!();
        let mut socket = connect(&mut ingress, &mut device, &mut serial, 5).await;

        let read = async {
            let mut buf = [0; 16];
            assert_eq!(8, socket.read(&mut buf).await.unwrap());
        };
        let receive = async {
            // Expect ReadData request
            let request = with_timeout(Duration::from_millis(100), serial.read_async())
                .await
                .unwrap()
                .unwrap();
            assert_eq!(b"AT+CIPRXGET=2,5,16\r", request.as_ref());
            request.release();

            ingress.write(b"\r\n+CIPRXGET: 2,5,0,0\r\n").await; // There is no data available
            ingress.write(b"\r\nOK\r\n").await;

            ingress.write(b"\r\n+CIPRXGET: 1,5\r\n").await; // Data becomes available

            // Expect ReadData request
            let request = with_timeout(Duration::from_millis(100), serial.read_async())
                .await
                .unwrap()
                .unwrap();
            assert_eq!(b"AT+CIPRXGET=2,5,16\r", request.as_ref());
            request.release();

            ingress
                .write(b"\r\n+CIPRXGET: 2,5,8,0\r\nHTTP\r\n\r\n")
                .await;
            ingress.write(b"\r\nOK\r\n").await;
        };

        tokio::join!(read, receive);
    }
}
