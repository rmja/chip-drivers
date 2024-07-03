use core::sync::atomic::Ordering;

use atat::{asynch::AtatClient, AtatCmd};
use core::fmt::Write as _;
use embassy_time::{with_timeout, Duration, Instant};
use embedded_io_async::{Read, Write};
use embedded_nal_async::{SocketAddr, TcpConnect};
use heapless::String;

use crate::{
    commands::{
        tcpip::{ReadData, SendData, StartConnection, WriteData, MAX_WRITE},
        urc::Urc,
    },
    device::Handle,
    SimcomUrcChannel,
};

use super::{DataService, SocketError, SOCKET_STATE_DROPPED, SOCKET_STATE_USED};

impl<'buf, 'dev, 'sub, AtCl: AtatClient + 'static> TcpConnect
    for DataService<'buf, 'dev, 'sub, AtCl>
{
    type Error = SocketError;

    type Connection<'a> = TcpSocket<'buf, 'dev, 'sub, AtCl> where Self : 'a;

    async fn connect<'a>(
        &'a self,
        remote: SocketAddr,
    ) -> Result<Self::Connection<'a>, Self::Error> {
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

pub struct TcpSocket<'buf, 'dev, 'sub, AtCl: AtatClient> {
    id: usize,
    handle: &'dev Handle<'sub, AtCl>,
    urc_channel: &'buf SimcomUrcChannel,
}

impl<'buf, 'dev, 'sub, AtCl: AtatClient + 'static> TcpSocket<'buf, 'dev, 'sub, AtCl> {
    pub(crate) fn try_new(
        handle: &'dev Handle<'sub, AtCl>,
        urc_channel: &'buf SimcomUrcChannel,
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
                .await
                .map_err(|_| SocketError::UnableToConnect)?;

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
        self.drain_background_urcs_and_ensure_in_use()
            .map_err(|_| SocketError::Test1)?;
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
                .await
                .map_err(|_| SocketError::UnableToRead)?;

            urc_subscription
        };

        let mut no_data_response_received = false;

        let mut timeout_instant = Instant::now() + Duration::from_secs(60);
        'wait_for_data: while let Some(timeout) =
            timeout_instant.checked_duration_since(Instant::now())
        {
            // Wait for next urc
            let urc = match with_timeout(timeout, urc_subscription.next_message_pure()).await {
                Ok(urc) => urc,
                Err(_) => {
                    break 'wait_for_data;
                }
            };

            self.drain_background_urcs_and_ensure_in_use()
                .map_err(|_| SocketError::Test2)?;

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
                            .await
                            .map_err(|_| SocketError::UnableToRead)?;

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

        error!("[{}] Timeout while reading data", self.id);
        self.handle.socket_state[self.id].store(SOCKET_STATE_DROPPED, Ordering::Release);
        Err(SocketError::ReadTimeout)
    }

    async fn write(&mut self, buf: &[u8]) -> Result<usize, SocketError> {
        if buf.is_empty() {
            self.drain_background_urcs_and_ensure_in_use()
                .map_err(|_| SocketError::Test3)?;
            return Ok(0);
        }

        self.wait_ongoing_write().await?;

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

        // Indicate that we are currently writing
        self.handle.busy_writing[self.id].store(true, Ordering::Release);

        // Write the data buffer
        client.send(&WriteData { buf: &buf[..len] }).await?;

        debug!("[{}] Wrote {} bytes", self.id, len);

        Ok(len)
    }

    // Wait for an ongoing write to complete.
    // This completion depends on the selection in AT+CIPQSEND
    // * Normal mode: This completes when the server receives the data
    // * Quick Send mode: This completes when the modem has received the data
    async fn wait_ongoing_write(&mut self) -> Result<(), SocketError> {
        let mut urc_subscription = self.urc_channel.subscribe().unwrap();

        self.drain_background_urcs_and_ensure_in_use()
            .map_err(|_| SocketError::Test4)?;

        if !self.handle.busy_writing[self.id].load(Ordering::Acquire) {
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

            self.drain_background_urcs_and_ensure_in_use()
                .map_err(|_| SocketError::Test5)?;

            if !self.handle.busy_writing[self.id].load(Ordering::Acquire) {
                trace!("[{}] Data is now written", self.id);
                return Ok(());
            } else {
                trace!("[{}] Data is not yet written", self.id);
            }
        }

        Err(SocketError::WriteTimeout)
    }
}

impl<AtCl: AtatClient> embedded_io::ErrorType for TcpSocket<'_, '_, '_, AtCl> {
    type Error = SocketError;
}

impl<AtCl: AtatClient + 'static> Read for TcpSocket<'_, '_, '_, AtCl> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, SocketError> {
        match self.read(buf).await {
            Ok(len) => Ok(len),
            Err(SocketError::ReadTimeout) => Ok(0),
            Err(e) => Err(e),
        }
    }
}

impl<AtCl: AtatClient + 'static> Write for TcpSocket<'_, '_, '_, AtCl> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, SocketError> {
        self.write(buf).await
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        // We do not do any buffering in the data so all writes are sent to the uart immediately
        // We cannot wait for the modem to reply "SEND OK" using wait_ongoing_write()
        // as this can cause deadlocks if the application does flush().await before it starts read().await.
        Ok(())
    }
}

impl<AtCl: AtatClient> Drop for TcpSocket<'_, '_, '_, AtCl> {
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

    use atat::AtatIngress;
    use embedded_hal::digital::{ErrorType, OutputPin};
    use embedded_nal_async::{IpAddr, Ipv4Addr, SocketAddr};
    use static_cell::make_static;

    use crate::{
        device::{SocketState, SOCKET_STATE_UNKNOWN, SOCKET_STATE_UNUSED},
        services::serial_mock::{RxMock, SerialMock},
        SimcomConfig, SimcomDevice, SimcomIngress, SimcomResponseSlot, MAX_SOCKETS,
    };

    use super::*;

    struct Config(ResetPin);
    struct ResetPin(bool);

    impl SimcomConfig for Config {
        type ResetPin = ResetPin;

        fn reset_pin(&mut self) -> &mut Self::ResetPin {
            &mut self.0
        }
    }

    impl OutputPin for ResetPin {
        fn set_low(&mut self) -> Result<(), Self::Error> {
            self.0 = false;
            Ok(())
        }

        fn set_high(&mut self) -> Result<(), Self::Error> {
            self.0 = true;
            Ok(())
        }
    }

    impl ErrorType for ResetPin {
        type Error = Infallible;
    }

    macro_rules! setup_atat {
        () => {{
            let ingress_buf = make_static!([0; 128]);
            static RES_SLOT: SimcomResponseSlot<128> = SimcomResponseSlot::new();
            let device_buf = make_static!([0; 128]);
            static URC_CHANNEL: SimcomUrcChannel = SimcomUrcChannel::new();
            static SERIAL: SerialMock = SerialMock::new();
            let (tx, rx) = SERIAL.split();
            let ingress = SimcomIngress::new(ingress_buf, &RES_SLOT, &URC_CHANNEL);
            let config = Config(ResetPin(true));
            let device = SimcomDevice::new(tx, &RES_SLOT, device_buf, &URC_CHANNEL, config);
            (ingress, device, rx)
        }};
    }

    async fn _hello_world_example() {
        const INGRESS_BUF_SIZE: usize = 128;
        static RES_SLOT: SimcomResponseSlot<INGRESS_BUF_SIZE> = SimcomResponseSlot::new();
        let device_buf = make_static!([0u8; 128]);
        static URC_CHANNEL: SimcomUrcChannel = SimcomUrcChannel::new();
        static SERIAL: SerialMock = SerialMock::new();
        let (tx, _rx) = SERIAL.split();
        let config = Config(ResetPin(true));
        let mut device = SimcomDevice::new(tx, &RES_SLOT, device_buf, &URC_CHANNEL, config);

        // Run in a different task
        // let ingress = SimcomIngress::new(&RES_SLOT, &URC_CHANNEL);
        // ingress.read_from(rx);

        device.network().attach(None).await.unwrap();

        device.reset().await.unwrap();
        device.setup().await.unwrap();

        let mut network = device.network();
        network.attach(None).await.unwrap();

        let _tcp = device.data("internet".into()).await.unwrap();
    }

    async fn connect<'buf, 'dev, 'sub, AtCl: AtatClient + 'static, Config: SimcomConfig>(
        ingress: &mut impl AtatIngress,
        device: &'dev mut SimcomDevice<'buf, 'sub, AtCl, Config>,
        serial: &mut RxMock<'_>,
        id: usize,
    ) -> TcpSocket<'buf, 'dev, 'sub, AtCl> {
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
        let sent = async {
            // Expect StartConnection request
            let sent = with_timeout(Duration::from_millis(100), serial.next_message_pure())
                .await
                .unwrap();

            ingress.write(b"\r\nOK\r\n").await;
            ingress
                .write(format!("\r\n{}, CONNECT OK\r\n", id).as_bytes())
                .await;

            sent
        };

        let (socket, sent) = tokio::join!(socket, sent);

        assert_eq!(
            format!("AT+CIPSTART={},\"TCP\",\"127.0.0.1\",\"8080\"\r", id).as_bytes(),
            &sent
        );

        socket
    }

    #[tokio::test]
    async fn can_connect() {
        let (mut ingress, mut device, mut serial) = setup_atat!();
        connect(&mut ingress, &mut device, &mut serial, 5).await;
    }

    #[tokio::test]
    async fn can_read_available_data() {
        let (mut ingress, mut device, mut serial) = setup_atat!();
        let mut socket = connect(&mut ingress, &mut device, &mut serial, 5).await;

        let read = async {
            let mut buf = [0; 16];
            socket.read(&mut buf).await.unwrap()
        };
        let sent = async {
            // Expect ReadData request
            let sent = with_timeout(Duration::from_millis(100), serial.next_message_pure())
                .await
                .unwrap();

            ingress
                .write(b"\r\n+CIPRXGET: 2,5,8,0\r\nHTTP\r\n\r\n")
                .await;
            ingress.write(b"\r\nOK\r\n").await;

            sent
        };

        let (read, sent) = tokio::join!(read, sent);

        assert_eq!(8, read);
        assert_eq!(b"AT+CIPRXGET=2,5,16\r", sent.as_slice());
    }

    #[tokio::test]
    async fn can_read_data_with_data_available_before_read_data() {
        let (mut ingress, mut device, mut serial) = setup_atat!();
        let mut socket = connect(&mut ingress, &mut device, &mut serial, 5).await;

        let read = async {
            let mut buf = [0; 16];
            socket.read(&mut buf).await.unwrap()
        };
        let sent = async {
            // Expect ReadData request
            let sent = with_timeout(Duration::from_millis(100), serial.next_message_pure())
                .await
                .unwrap();

            ingress.write(b"\r\n+CIPRXGET: 1,5\r\n").await; // Transmitted by modem before it understands our read request
            ingress
                .write(b"\r\n+CIPRXGET: 2,5,8,0\r\nHTTP\r\n\r\n")
                .await;
            ingress.write(b"\r\nOK\r\n").await;

            sent
        };

        let (read, sent) = tokio::join!(read, sent);

        assert_eq!(8, read);
        assert_eq!(b"AT+CIPRXGET=2,5,16\r", sent.as_slice());
    }

    #[tokio::test]
    async fn can_read_data_with_no_data_initially_available_retrying() {
        let (mut ingress, mut device, mut serial) = setup_atat!();
        let mut socket = connect(&mut ingress, &mut device, &mut serial, 5).await;

        let read = async {
            let mut buf = [0; 16];
            socket.read(&mut buf).await.unwrap()
        };
        let sent = async {
            // Expect ReadData request
            let sent0 = with_timeout(Duration::from_millis(100), serial.next_message_pure())
                .await
                .unwrap();

            ingress.write(b"\r\n+CIPRXGET: 2,5,0,0\r\n").await; // There is no data available
            ingress.write(b"\r\nOK\r\n").await;

            ingress.write(b"\r\n+CIPRXGET: 1,5\r\n").await; // Data becomes available

            // Expect ReadData request
            let sent1 = with_timeout(Duration::from_millis(100), serial.next_message_pure())
                .await
                .unwrap();

            ingress
                .write(b"\r\n+CIPRXGET: 2,5,8,0\r\nHTTP\r\n\r\n")
                .await;
            ingress.write(b"\r\nOK\r\n").await;

            (sent0, sent1)
        };

        let (read, sent) = tokio::join!(read, sent);

        assert_eq!(8, read);
        assert_eq!(b"AT+CIPRXGET=2,5,16\r", sent.0.as_slice());
        assert_eq!(b"AT+CIPRXGET=2,5,16\r", sent.1.as_slice());
    }
}
