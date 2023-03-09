mod impls;
mod responses;
mod types;

use super::NoResponse;
use atat::atat_derive::AtatCmd;
pub use responses::*;
pub use types::*;

/// 8.2.1 AT+CIPMUX Start Up Multi-IP Connection
#[derive(Clone, AtatCmd)]
#[at_cmd("+CIPMUX", NoResponse, termination = "\r")]
pub struct StartMultiIpConnection {
    pub n: MultiIpValue,
}

/// 8.2.2 AT+CIPSTART Start Up TCP or UDP Connection
#[derive(Clone, AtatCmd)]
#[at_cmd("+CIPSTART", NoResponse, timeout_ms = 75_000, termination = "\r")]
pub struct StartConnection<'a> {
    pub id: usize,
    #[at_arg(len = 3)]
    pub mode: &'a str,
    #[at_arg(len = 15)]
    pub ip: &'a str,
    #[at_arg(len = 5)]
    pub port: &'a str,
}

/// 8.2.3 AT+CIPSEND Send Data Through TCP or UDP Connection
#[derive(Clone, AtatCmd)]
#[at_cmd("+CIPSEND", NoResponse, termination = "\r")]
pub struct SendData {
    pub id: usize,
    pub len: Option<usize>,
}

#[derive(Clone)]
pub struct WriteData<'a> {
    pub buf: &'a [u8],
}

/// 8.2.6 AT+CIPCLOSE Close TCP or UDP Connection.
#[derive(Clone)]
pub struct CloseConnection {
    pub id: usize,
}

/// 8.2.7 AT+CIPSHUT Deactivate GPRS PDP Context
#[derive(Clone, AtatCmd)]
#[at_cmd("+CIPSHUT", NoResponse, timeout_ms = 65_000, termination = "\r")]
pub struct DeactivateGprsPdpContext;

/// 8.2.9 AT+CSTT Start Task and Set APN, USER NAME, PASSWORD
#[derive(Clone, AtatCmd)]
#[at_cmd("+CSTT", NoResponse, termination = "\r")]
pub struct StartTaskAndSetApn<'a> {
    #[at_arg(len = 16)]
    pub apn: &'a str,
    #[at_arg(len = 16)]
    pub username: &'a str,
    #[at_arg(len = 16)]
    pub password: &'a str,
}

/// 8.2.10 AT+CIICR Bring Up Wireless Connection with GPRS or CSD
#[derive(Clone, AtatCmd)]
#[at_cmd("+CIICR", NoResponse, timeout_ms = 85_000, termination = "\r")]
pub struct BringUpWireless;

/// 8.2.11 AT+CIFSR Get Local IP Address
///
/// AT+CIFSR replies with the local IP without a terminating OK.
/// We therefore append an AT command to ensure that OK is sent.
#[derive(Clone, AtatCmd)]
#[at_cmd("+CIFSR\rAT", LocalIP, termination = "\r")]
pub struct GetLocalIP;

/// 8.2.12 AT+CIPSTATUS Query Current Connection Status
///
/// AT+CIPSTATUS replies with an OK before the actual status table.
/// The actual connection status must therefore be read using a subsequent `ReadConnectionStatus`
#[derive(Clone, AtatCmd)]
#[at_cmd("+CIPSTATUS", ConnectionStatus, termination = "\r")]
pub struct GetConnectionStatus {
    pub id: usize,
}

/// 8.2.14 AT+CDNSGIP Query the IP Address of Given Domain Name
#[derive(Clone, AtatCmd)]
#[at_cmd("+CDNSGIP", NoResponse, termination = "\r")]
pub struct ResolveHostIp<'a> {
    #[at_arg(len = 128)]
    pub host: &'a str,
}

/// 8.2.26 AT+CIPRXGET Get Data from Network Manually
#[derive(Clone, AtatCmd)]
#[at_cmd("+CIPRXGET=1", NoResponse, termination = "\r")]
pub struct SetManualRxGetMode;

/// 8.2.26 AT+CIPRXGET Get Data from Network Manually
///
/// Note: the response for this command is typically
/// +CIPRXGET ..data
/// OK
///
/// However, it seems as if the response may be
/// +CIPRXGET ..data
/// SOME URC
/// OK
///
/// This is the reason why we consider the response from AT+CIPRXGET as
/// `NoResponse` and the +CIPRXGET part a Urc
#[derive(Clone, AtatCmd)]
#[at_cmd("+CIPRXGET=2,", NoResponse, value_sep = false, termination = "\r")]
pub struct ReadData {
    pub id: usize,
    // The maximum number of bytes to read in the range 0..1460
    pub max_len: usize,
}

#[cfg(test)]
mod tests {
    use assert_hex::assert_eq_hex;
    use atat::{asynch::AtatClient, AtatCmd, AtatIngress, AtatUrcChannel, DigestResult, Digester};

    use crate::{commands::urc::Urc, Device, SimcomAtatBuffers, SimcomDigester};

    use super::*;

    macro_rules! setup_atat {
        () => {{
            static mut BUFFERS: SimcomAtatBuffers<128, 512> = SimcomAtatBuffers::new();
            let buffers = unsafe { &mut BUFFERS };
            Device::from_buffers(buffers, Vec::new())
        }};
    }

    #[test]
    fn can_start_multi_ip_connection() {
        let cmd = StartMultiIpConnection {
            n: MultiIpValue::MultiIpConnection,
        };
        assert_eq_hex!(b"AT+CIPMUX=1\r", cmd.as_bytes());
    }

    #[test]
    fn can_start_connection() {
        let cmd = StartConnection {
            id: 2,
            mode: "TCP",
            ip: "google.com",
            port: "80",
        };
        assert_eq_hex!(
            b"AT+CIPSTART=2,\"TCP\",\"google.com\",\"80\"\r",
            cmd.as_bytes()
        );
    }

    #[test]
    fn can_send_data() {
        let cmd = SendData {
            id: 2,
            len: Some(10),
        };
        assert_eq_hex!(b"AT+CIPSEND=2,10\r", cmd.as_bytes());
    }

    #[test]
    fn can_close_connection() {
        let cmd = CloseConnection { id: 2 };
        assert_eq_hex!(b"AT+CIPCLOSE=2\r", cmd.as_bytes());

        let mut digester = SimcomDigester::new();
        assert_eq!(
            (DigestResult::Response(Ok(b"2, CLOSE OK")), 15),
            digester.digest(b"\r\n2, CLOSE OK\r\n")
        );
    }

    #[test]
    fn can_deactivate_gprs_pdp_context() {
        let cmd = DeactivateGprsPdpContext;
        assert_eq_hex!(b"AT+CIPSHUT\r", cmd.as_bytes());

        let mut digester = SimcomDigester::new();
        assert_eq!(
            (DigestResult::Response(Ok(b"")), 11),
            digester.digest(b"\r\nSHUT OK\r\n")
        );
    }

    #[test]
    fn can_start_task_and_set_apn() {
        let cmd = StartTaskAndSetApn {
            apn: &"internet",
            username: &"",
            password: &"",
        };
        assert_eq_hex!(b"AT+CSTT=\"internet\",\"\",\"\"\r", cmd.as_bytes());
    }

    #[test]
    fn can_bring_up_wireless() {
        let cmd = BringUpWireless;
        assert_eq_hex!(b"AT+CIICR\r", cmd.as_bytes());
    }

    #[tokio::test]
    async fn can_get_local_ip() {
        let cmd = GetLocalIP;
        assert_eq_hex!(b"AT+CIFSR\rAT\r", cmd.as_bytes());

        let (mut ingress, device) = setup_atat!();

        ingress.write(b"\r\n10.0.109.44\r\n").await;
        ingress.write(b"\r\nOK\r\n").await;

        let mut at_client = device.handle.client.lock().await;
        let response = at_client.send(&cmd).await.unwrap();
        assert_eq!(b"10.0.109.44", response.ip.as_ref());
    }

    #[tokio::test]
    async fn can_get_connection_status_initial() {
        let cmd = GetConnectionStatus { id: 2 };
        assert_eq_hex!(b"AT+CIPSTATUS=2\r", cmd.as_bytes());

        let (mut ingress, device) = setup_atat!();

        ingress
            .write(b"\r\n+CIPSTATUS: 2,,\"\",\"\",\"\",\"INITIAL\"\r\n\r\nOK\r\n")
            .await;

        let mut at_client = device.handle.client.lock().await;
        let response = at_client.send(&cmd).await.unwrap();
        assert_eq!(2, response.id);
        assert_eq!("", response.mode);
        assert_eq!("", response.ip);
        assert_eq!("", response.port);
        assert_eq!(ClientState::Initial, response.state);
    }

    #[tokio::test]
    async fn can_get_connection_status_connected() {
        let cmd = GetConnectionStatus { id: 2 };
        assert_eq_hex!(b"AT+CIPSTATUS=2\r", cmd.as_bytes());

        let (mut ingress, device) = setup_atat!();

        ingress.write(
            b"\r\n+CIPSTATUS: 2,0,\"TCP\",\"123.123.123.123\",\"80\",\"CONNECTED\"\r\n\r\nOK\r\n",
        ).await;

        let mut at_client = device.handle.client.lock().await;
        let response = at_client.send(&cmd).await.unwrap();
        assert_eq!(2, response.id);
        assert_eq!("TCP", response.mode);
        assert_eq!("123.123.123.123", response.ip);
        assert_eq!("80", response.port);
        assert_eq!(ClientState::Connected, response.state);
    }

    #[tokio::test]
    async fn can_resolve_host_ip() {
        let cmd = ResolveHostIp {
            host: "utiliread.dk",
        };
        assert_eq_hex!(b"AT+CDNSGIP=\"utiliread.dk\"\r", cmd.as_bytes());

        let (mut ingress, device) = setup_atat!();

        let mut subscription = device.urc_channel.subscribe().unwrap();

        ingress.write(b"\r\nOK\r\n").await;
        ingress
            .write(b"\r\n+CDNSGIP: 1,\"utiliread.dk\",\"1.2.3.4\"\r\n")
            .await;

        let mut at_client = device.handle.client.lock().await;
        _ = at_client.send(&cmd).await.unwrap();
        if let Urc::IpLookup(res) = subscription.try_next_message_pure().unwrap() {
            assert_eq!("utiliread.dk", res.host);
            assert_eq!("1.2.3.4", res.ip);
        } else {
            panic!("Invalid URC");
        }

        assert_eq!(0, subscription.available());
    }

    #[test]
    fn can_set_manual_rx_get_mode() {
        let cmd = SetManualRxGetMode;
        assert_eq_hex!(b"AT+CIPRXGET=1\r", cmd.as_bytes());
    }

    #[tokio::test]
    async fn can_read_data() {
        let cmd = ReadData { id: 5, max_len: 16 };
        assert_eq_hex!(b"AT+CIPRXGET=2,5,16\r", cmd.as_bytes());

        let (mut ingress, device) = setup_atat!();

        let mut subscription = device.urc_channel.subscribe().unwrap();

        ingress
            .write(b"\r\n+CIPRXGET: 2,5,8,0\r\nHTTP\r\n\r\n")
            .await;
        ingress.write(b"\r\nOK\r\n").await;

        let mut at_client = device.handle.client.lock().await;
        at_client.send(&cmd).await.unwrap();
        if let Urc::ReadData(data) = subscription.try_next_message_pure().unwrap() {
            assert_eq!(5, data.id);
            assert_eq!(8, data.data_len);
            assert_eq!(0, data.pending_len);
            assert_eq!(b"HTTP\r\n\r\n", data.data.take().unwrap().as_slice());
        } else {
            panic!("Invalid URC");
        }

        assert_eq!(0, subscription.available());
    }
}
