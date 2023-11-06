mod impls;
mod responses;
mod types;

use super::NoResponse;
use atat::atat_derive::AtatCmd;
pub use responses::*;
pub use types::*;

/// 8.2.1 AT+CIPMUX Start Up Multi-IP Connection
#[derive(AtatCmd)]
#[at_cmd("+CIPMUX", NoResponse, termination = "\r")]
pub struct StartMultiIpConnection {
    pub n: MultiIpValue,
}

/// 8.2.2 AT+CIPSTART Start Up TCP or UDP Connection
#[derive(AtatCmd)]
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
#[derive(AtatCmd)]
#[at_cmd("+CIPSEND", NoResponse, termination = "\r")]
pub struct SendData {
    pub id: usize,
    pub len: Option<usize>,
}

pub struct WriteData<'a> {
    pub buf: &'a [u8],
}

/// 8.2.6 AT+CIPCLOSE Close TCP or UDP Connection.
pub struct CloseConnection {
    pub id: usize,
}

/// 8.2.7 AT+CIPSHUT Deactivate GPRS PDP Context
#[derive(AtatCmd)]
#[at_cmd("+CIPSHUT", NoResponse, timeout_ms = 65_000, termination = "\r")]
pub struct DeactivateGprsPdpContext;

/// 8.2.9 AT+CSTT Start Task and Set APN, USER NAME, PASSWORD
#[derive(AtatCmd)]
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
#[derive(AtatCmd)]
#[at_cmd("+CIICR", NoResponse, timeout_ms = 85_000, termination = "\r")]
pub struct BringUpWireless;

/// 8.2.11 AT+CIFSR Get Local IP Address
///
/// AT+CIFSR replies with the local IP without a terminating OK.
/// We therefore append an AT command to ensure that OK is sent.
#[derive(AtatCmd)]
#[at_cmd("+CIFSR", LocalIP, termination = "\r")]
pub struct GetLocalIP;

/// 8.2.12 AT+CIPSTATUS Query Current Connection Status
///
/// AT+CIPSTATUS replies with an OK before the actual status table.
/// The actual connection status must therefore be read using a subsequent `ReadConnectionStatus`
#[derive(AtatCmd)]
#[at_cmd("+CIPSTATUS", ConnectionStatus, termination = "\r")]
pub struct GetConnectionStatus {
    pub id: usize,
}

/// 8.2.13 AT+CDNSCFG Configure Domain Name Server
#[derive(AtatCmd)]
#[at_cmd("+CDNSCFG", NoResponse, termination = "\r")]
pub struct ConfigureDomainNameServer<'a> {
    #[at_arg(len = 15)]
    pub pri_dns: &'a str,
    #[at_arg(len = 15)]
    pub sec_dns: Option<&'a str>,
}

/// 8.2.14 AT+CDNSGIP Query the IP Address of Given Domain Name
#[derive(AtatCmd)]
#[at_cmd("+CDNSGIP", NoResponse, termination = "\r")]
pub struct ResolveHostIp<'a> {
    #[at_arg(len = 128)]
    pub host: &'a str,
}

/// 8.2.26 AT+CIPRXGET Get Data from Network Manually
#[derive(AtatCmd)]
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
///
/// No timeout is given for the command in the command reference,
/// but times much longer than the default 1s has been seen in the wild.
///
/// A typical simcard in speeddrop will provide 64 kbit/s
/// The simcom max read is 1460 bytes.
/// This should therefore not have any significant impact on the timeout.
#[derive(AtatCmd)]
#[at_cmd(
    "+CIPRXGET=2,",
    NoResponse,
    timeout_ms = 5_000,
    value_sep = false,
    termination = "\r"
)]
pub struct ReadData {
    pub id: usize,
    // The maximum number of bytes to read in the range 0..1460
    pub max_len: usize,
}

#[cfg(test)]
mod tests {
    use assert_hex::assert_eq_hex;
    use atat::{
        AtatCmd, AtatIngress, AtatUrcChannel, DigestResult, Digester, Ingress, Response,
        ResponseChannel, UrcChannel,
    };

    use crate::{commands::urc::Urc, SimcomDigester};

    use super::*;

    macro_rules! setup_atat {
        () => {{
            static RES_CHANNEL: ResponseChannel<100> = ResponseChannel::new();
            static URC_CHANNEL: UrcChannel<Urc, 1, 1> = UrcChannel::new();
            let ingress = Ingress::<SimcomDigester, Urc, 100, 1, 1>::new(
                SimcomDigester::new(),
                RES_CHANNEL.publisher().unwrap(),
                URC_CHANNEL.publisher(),
            );

            let res_sub = RES_CHANNEL.subscriber().unwrap();
            let urc_sub = URC_CHANNEL.subscribe().unwrap();

            (ingress, res_sub, urc_sub)
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

    #[test]
    fn can_get_local_ip() {
        let cmd = GetLocalIP;
        assert_eq_hex!(b"AT+CIFSR\r", cmd.as_bytes());

        let (mut ingress, mut res_sub, _) = setup_atat!();
        ingress.try_write(b"\r\n10.0.109.44\r\n").unwrap();

        if let Response::Ok(message) = res_sub.try_next_message_pure().unwrap() {
            let response = cmd.parse(Ok(&message)).unwrap();
            assert_eq!(b"10.0.109.44", response.ip.as_ref());
        } else {
            panic!("Invalid response");
        }
    }

    #[test]
    fn can_get_connection_status_initial() {
        let cmd = GetConnectionStatus { id: 2 };
        assert_eq_hex!(b"AT+CIPSTATUS=2\r", cmd.as_bytes());

        let (mut ingress, mut res_sub, _) = setup_atat!();
        ingress
            .try_write(b"\r\n+CIPSTATUS: 2,,\"\",\"\",\"\",\"INITIAL\"\r\n\r\nOK\r\n")
            .unwrap();

        if let Response::Ok(message) = res_sub.try_next_message_pure().unwrap() {
            let response = cmd.parse(Ok(&message)).unwrap();
            assert_eq!(2, response.id);
            assert_eq!("", response.mode);
            assert_eq!("", response.ip);
            assert_eq!("", response.port);
            assert_eq!(ClientState::Initial, response.state);
        } else {
            panic!("Invalid response");
        }
    }

    #[test]
    fn can_get_connection_status_connected() {
        let cmd = GetConnectionStatus { id: 2 };
        assert_eq_hex!(b"AT+CIPSTATUS=2\r", cmd.as_bytes());

        let (mut ingress, mut res_sub, _) = setup_atat!();
        ingress.try_write(
            b"\r\n+CIPSTATUS: 2,0,\"TCP\",\"123.123.123.123\",\"80\",\"CONNECTED\"\r\n\r\nOK\r\n",
        ).unwrap();

        if let Response::Ok(message) = res_sub.try_next_message_pure().unwrap() {
            let response = cmd.parse(Ok(&message)).unwrap();
            assert_eq!(2, response.id);
            assert_eq!("TCP", response.mode);
            assert_eq!("123.123.123.123", response.ip);
            assert_eq!("80", response.port);
            assert_eq!(ClientState::Connected, response.state);
        } else {
            panic!("Invalid response");
        }
    }

    #[test]
    fn can_configure_domain_name_server() {
        let cmd = ConfigureDomainNameServer {
            pri_dns: "111.222.333.444",
            sec_dns: Some("555.666.777.888"),
        };
        assert_eq_hex!(
            b"AT+CDNSCFG=\"111.222.333.444\",\"555.666.777.888\"\r",
            cmd.as_bytes()
        );
    }

    #[test]
    fn can_resolve_host_ip() {
        let cmd = ResolveHostIp {
            host: "utiliread.dk",
        };
        assert_eq_hex!(b"AT+CDNSGIP=\"utiliread.dk\"\r", cmd.as_bytes());

        let (mut ingress, mut res_sub, mut urc_sub) = setup_atat!();
        ingress.try_write(b"\r\nOK\r\n").unwrap();
        ingress
            .try_write(b"\r\n+CDNSGIP: 1,\"utiliread.dk\",\"1.2.3.4\"\r\n")
            .unwrap();

        if let Response::Ok(message) = res_sub.try_next_message_pure().unwrap() {
            assert!(message.is_empty());
        } else {
            panic!("Invalid response");
        }

        if let Urc::DnsResult(Ok(res)) = urc_sub.try_next_message_pure().unwrap() {
            assert_eq!("utiliread.dk", res.host);
            assert_eq!("1.2.3.4", res.ip);
        } else {
            panic!("Invalid URC");
        }

        assert_eq!(0, urc_sub.available());
    }

    #[test]
    fn can_handle_resolve_host_ip_urc_error() {
        let cmd = ResolveHostIp {
            host: "utiliread.dk",
        };
        assert_eq_hex!(b"AT+CDNSGIP=\"utiliread.dk\"\r", cmd.as_bytes());

        let (mut ingress, mut res_sub, mut urc_sub) = setup_atat!();
        ingress.try_write(b"\r\nOK\r\n").unwrap();
        ingress.try_write(b"\r\n+CDNSGIP: 0,8\r\n").unwrap();

        if let Response::Ok(message) = res_sub.try_next_message_pure().unwrap() {
            assert!(message.is_empty());
        } else {
            panic!("Invalid response");
        }

        if let Urc::DnsResult(Err(kind)) = urc_sub.try_next_message_pure().unwrap() {
            assert_eq!(8, kind);
        } else {
            panic!("Invalid URC");
        }

        assert_eq!(0, urc_sub.available());
    }

    #[test]
    fn can_handle_resolve_host_ip_immediate_error() {
        // This error is echo'ed
        let (mut ingress, mut res_sub, urc_sub) = setup_atat!();
        ingress
            .try_write(b"AT+CDNSGIP=\"utiliread.com\"\r\r\nERROR\r\n")
            .unwrap();

        assert_eq!(
            Response::OtherError,
            res_sub.try_next_message_pure().unwrap()
        );

        assert_eq!(0, urc_sub.available());
    }

    #[test]
    fn can_set_manual_rx_get_mode() {
        let cmd = SetManualRxGetMode;
        assert_eq_hex!(b"AT+CIPRXGET=1\r", cmd.as_bytes());
    }

    #[test]
    fn can_read_data() {
        let cmd = ReadData { id: 5, max_len: 16 };
        assert_eq_hex!(b"AT+CIPRXGET=2,5,16\r", cmd.as_bytes());

        let (mut ingress, mut res_sub, mut urc_sub) = setup_atat!();
        ingress
            .try_write(b"\r\n+CIPRXGET: 2,5,8,0\r\nHTTP\r\n\r\n")
            .unwrap();
        ingress.try_write(b"\r\nOK\r\n").unwrap();

        if let Response::Ok(message) = res_sub.try_next_message_pure().unwrap() {
            assert!(message.is_empty());
        } else {
            panic!("Invalid response");
        }

        if let Urc::ReadData(data) = urc_sub.try_next_message_pure().unwrap() {
            assert_eq!(5, data.id);
            assert_eq!(8, data.data_len);
            assert_eq!(0, data.pending_len);
            assert_eq!(b"HTTP\r\n\r\n", data.data.take().unwrap().as_slice());
        } else {
            panic!("Invalid URC");
        }

        assert_eq!(0, urc_sub.available());
    }

    #[test]
    fn can_read_data_after_prompt() {
        let (mut ingress, mut res_sub, mut urc_sub) = setup_atat!();

        ingress.try_write(b"\r\n>").unwrap();

        if let Response::Prompt(prompt) = res_sub.try_next_message_pure().unwrap() {
            assert_eq!(b'>', prompt);
        } else {
            panic!("Invalid prompt");
        }

        ingress.try_write(b" ").unwrap();
        ingress
            .try_write(b"\r\n+CIPRXGET: 2,0,0,0\r\n\r\nOK\r\n")
            .unwrap();

        if let Urc::ReadData(data) = urc_sub.try_next_message_pure().unwrap() {
            assert_eq!(0, data.id);
            assert_eq!(0, data.data_len);
            assert_eq!(0, data.pending_len);
        } else {
            panic!("Invalid URC");
        }

        assert_eq!(0, urc_sub.available());
    }
}
