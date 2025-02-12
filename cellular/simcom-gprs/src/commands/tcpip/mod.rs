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
pub struct QuerySendBufferSize;

#[derive(AtatCmd)]
#[at_cmd("+CIPSEND", NoResponse, termination = "\r")]
pub struct SendData {
    pub id: usize,
    pub len: Option<usize>,
}

/// The maximum write size
/// This is smaller than the value reported by AT+CIPSEND?
/// but it seems by observing the TCP packets that the modem
/// prefers to write packets with TCP payload of size 1024
pub const MAX_WRITE: usize = 1024;
pub const WRITE_DATA_MAX_LEN: usize = MAX_WRITE;

pub struct WriteData<'a> {
    pub buf: &'a [u8],
}

/// 8.2.4 AT+CIPQSEND Select Data Transmitting Mode
#[derive(AtatCmd)]
#[at_cmd("+CIPQSEND", NoResponse, termination = "\r")]
pub struct SelectDataTransmittingMode {
    pub mode: DataTransmittingMode,
}

/// 8.2.5 AT+CIPACK Query Previous Connection Data Transmitting State
#[derive(AtatCmd)]
#[at_cmd("+CIPACK", DataTransmittingState, termination = "\r")]
pub struct QueryPreviousConnectionDataTransmittingState {
    pub id: usize,
}

/// 8.2.6 AT+CIPCLOSE Close TCP or UDP Connection
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
    timeout_ms = 10_000,
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
    use atat::{AtatCmd, AtatIngress, DigestResult, Digester, Response};
    use static_cell::StaticCell;

    use crate::{
        commands::{
            gprs::{GetPDPContextStates, PdpState},
            urc::Urc,
            AtatCmdEx,
        },
        ContextId, SimcomDigester, SimcomIngress, SimcomResponseSlot, SimcomUrcChannel,
    };

    use super::*;

    macro_rules! setup_atat {
        () => {{
            static BUF: StaticCell<[u8; 256]> = StaticCell::new();
            let buf = BUF.init([0; 256]);
            static RES_SLOT: SimcomResponseSlot<200> = SimcomResponseSlot::new();
            static URC_CHANNEL: SimcomUrcChannel = SimcomUrcChannel::new();
            let ingress = SimcomIngress::<200>::new(buf, &RES_SLOT, &URC_CHANNEL);

            let urc_sub = URC_CHANNEL.subscribe().unwrap();

            (ingress, &RES_SLOT, urc_sub)
        }};
    }

    #[test]
    fn can_start_multi_ip_connection() {
        let cmd = StartMultiIpConnection {
            n: MultiIpValue::MultiIpConnection,
        };
        assert_eq_hex!(b"AT+CIPMUX=1\r", cmd.to_vec().as_slice());
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
            cmd.to_vec().as_slice()
        );
    }

    #[test]
    fn can_query_send_buffer_size_sim800() {
        let cmd = QuerySendBufferSize;
        assert_eq_hex!(b"AT+CIPSEND?\r", cmd.to_vec().as_slice());

        let (mut ingress, res_sub, _) = setup_atat!();
        ingress.try_write(b"\r\n+CIPSEND: 0,1460\r\n+CIPSEND: 1,0\r\n+CIPSEND: 2,0\r\n+CIPSEND: 3,0\r\n+CIPSEND: 4,0\r\n+CIPSEND: 5,0\r\n\r\nOK\r\n").unwrap();

        let response = res_sub.try_get().unwrap();
        let response: &Response<200> = &response.borrow();
        if let Response::Ok(message) = response {
            let response = cmd.parse(Ok(&message)).unwrap();
            assert_eq!(1460, response.size[0]);
            assert_eq!(0, response.size[1]);
            assert_eq!(0, response.size[2]);
            assert_eq!(0, response.size[3]);
            assert_eq!(0, response.size[4]);
            assert_eq!(0, response.size[5]);
        } else {
            panic!("Invalid response");
        }
    }

    #[cfg(feature = "sim900")]
    #[test]
    fn can_query_send_buffer_size_sim900() {
        let cmd = QuerySendBufferSize;
        assert_eq_hex!(b"AT+CIPSEND?\r", cmd.to_vec().as_slice());

        let (mut ingress, res_sub, _) = setup_atat!();
        ingress.try_write(b"\r\n+CIPSEND: 0,1460\r\n+CIPSEND: 1,0\r\n+CIPSEND: 2,0\r\n+CIPSEND: 3,0\r\n+CIPSEND: 4,0\r\n+CIPSEND: 5,0\r\n+CIPSEND: 6,0\r\n+CIPSEND: 7,0\r\n\r\nOK\r\n").unwrap();

        let response = res_sub.try_get().unwrap();
        let response: &Response<200> = &response.borrow();
        if let Response::Ok(message) = response {
            let response = cmd.parse(Ok(&message)).unwrap();
            assert_eq!(1460, response.size[0]);
            assert_eq!(0, response.size[1]);
            assert_eq!(0, response.size[2]);
            assert_eq!(0, response.size[3]);
            assert_eq!(0, response.size[4]);
            assert_eq!(0, response.size[5]);
            assert_eq!(0, response.size[6]);
            assert_eq!(0, response.size[7]);
        } else {
            panic!("Invalid response");
        }
    }

    #[test]
    fn can_send_data() {
        let cmd = SendData {
            id: 2,
            len: Some(10),
        };
        assert_eq_hex!(b"AT+CIPSEND=2,10\r", cmd.to_vec().as_slice());
    }

    #[test]
    fn can_write_data() {
        let cmd = WriteData { buf: b"HELLO" };
        assert_eq_hex!(b"HELLO", cmd.to_vec().as_slice());

        let (mut ingress, res_sub, _) = setup_atat!();
        ingress.try_write(b"\r\nDATA ACCEPT:1,2\r\n").unwrap();

        let response = res_sub.try_get().unwrap();
        let response: &Response<200> = &response.borrow();
        if let Response::Ok(message) = response {
            let response = cmd.parse(Ok(&message)).unwrap();
            assert_eq!(1, response.id);
            assert_eq!(2, response.accepted);
        } else {
            panic!("Invalid response");
        }
    }

    #[test]
    fn can_select_data_transmitting_mode() {
        let cmd = SelectDataTransmittingMode {
            mode: DataTransmittingMode::QuickSendMode,
        };
        assert_eq_hex!(b"AT+CIPQSEND=1\r", cmd.to_vec().as_slice());
    }

    #[test]
    fn can_query_connection_transmitting_state() {
        let cmd = QueryPreviousConnectionDataTransmittingState { id: 2 };
        assert_eq_hex!(b"AT+CIPACK=2\r", cmd.to_vec().as_slice());

        let (mut ingress, res_sub, _) = setup_atat!();
        ingress
            .try_write(b"\r\n+CIPACK: 3,2,1\r\n\r\nOK\r\n")
            .unwrap();

        let response = res_sub.try_get().unwrap();
        let response: &Response<200> = &response.borrow();
        if let Response::Ok(message) = response {
            let response = cmd.parse(Ok(&message)).unwrap();
            assert_eq!(3, response.txlen);
            assert_eq!(2, response.acklen);
            assert_eq!(1, response.nacklen);
        } else {
            panic!("Invalid response");
        }
    }

    #[test]
    fn can_close_connection() {
        let cmd = CloseConnection { id: 2 };
        assert_eq_hex!(b"AT+CIPCLOSE=2\r", cmd.to_vec().as_slice());

        let mut digester = SimcomDigester::new();
        assert_eq!(
            (DigestResult::Response(Ok(b"2, CLOSE OK")), 15),
            digester.digest(b"\r\n2, CLOSE OK\r\n")
        );
    }

    #[test]
    fn can_deactivate_gprs_pdp_context() {
        let cmd = DeactivateGprsPdpContext;
        assert_eq_hex!(b"AT+CIPSHUT\r", cmd.to_vec().as_slice());

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
        assert_eq_hex!(b"AT+CSTT=\"internet\",\"\",\"\"\r", cmd.to_vec().as_slice());
    }

    #[test]
    fn can_bring_up_wireless() {
        let cmd = BringUpWireless;
        assert_eq_hex!(b"AT+CIICR\r", cmd.to_vec().as_slice());
    }

    #[test]
    fn can_get_local_ip() {
        let cmd = GetLocalIP;
        assert_eq_hex!(b"AT+CIFSR\r", cmd.to_vec().as_slice());

        let (mut ingress, res_sub, _) = setup_atat!();
        ingress.try_write(b"\r\n10.0.109.44\r\n").unwrap();

        let response = res_sub.try_get().unwrap();
        let response: &Response<200> = &response.borrow();
        if let Response::Ok(message) = response {
            let response = cmd.parse(Ok(&message)).unwrap();
            assert_eq!(b"10.0.109.44", response.ip.as_ref());
        } else {
            panic!("Invalid response");
        }
    }

    #[test]
    fn can_get_connection_status_initial() {
        let cmd = GetConnectionStatus { id: 2 };
        assert_eq_hex!(b"AT+CIPSTATUS=2\r", cmd.to_vec().as_slice());

        let (mut ingress, res_sub, _) = setup_atat!();
        ingress
            .try_write(b"\r\n+CIPSTATUS: 2,,\"\",\"\",\"\",\"INITIAL\"\r\n\r\nOK\r\n")
            .unwrap();

        let response = res_sub.try_get().unwrap();
        let response: &Response<200> = &response.borrow();
        if let Response::Ok(message) = response {
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
        assert_eq_hex!(b"AT+CIPSTATUS=2\r", cmd.to_vec().as_slice());

        let (mut ingress, res_sub, _) = setup_atat!();
        ingress.try_write(
            b"\r\n+CIPSTATUS: 2,0,\"TCP\",\"123.123.123.123\",\"80\",\"CONNECTED\"\r\n\r\nOK\r\n",
        ).unwrap();

        let response = res_sub.try_get().unwrap();
        let response: &Response<200> = &response.borrow();
        if let Response::Ok(message) = response {
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
    fn can_get_pdp_context_states() {
        let cmd = GetPDPContextStates;
        assert_eq_hex!(b"AT+CGACT?\r", cmd.to_vec().as_slice());

        // Note that we consider e.g. "\r\n+CGACT: 1,0" to be a URC without terminating \r\n
        // This effectively gives us a final "\r\n\r\nOK\r\n" (with an additional leading "\r\n")
        // This however is discarded whitespace by the digester so it does not really matter.
        let (mut ingress, res_sub, mut urc_sub) = setup_atat!();
        ingress
            .try_write(b"\r\n+CGACT: 1,0\r\n+CGACT: 2,0\r\n+CGACT: 3,0\r\n\r\nOK\r\n")
            .unwrap();

        let response = res_sub.try_get().unwrap();
        let response: &Response<200> = &response.borrow();
        if let Response::Ok(message) = response {
            assert!(message.is_empty());
        } else {
            panic!("Invalid response");
        }

        for cid in 1..=3 {
            if let Urc::PdbState(res) = urc_sub.try_next_message_pure().unwrap() {
                assert_eq!(ContextId(cid), res.cid);
                assert_eq!(PdpState::Deactivated, res.state);
            } else {
                panic!("Invalid URC");
            }
        }

        assert_eq!(0, urc_sub.available());
    }

    #[test]
    fn can_configure_domain_name_server() {
        let cmd = ConfigureDomainNameServer {
            pri_dns: "111.222.333.444",
            sec_dns: Some("555.666.777.888"),
        };
        assert_eq_hex!(
            b"AT+CDNSCFG=\"111.222.333.444\",\"555.666.777.888\"\r",
            cmd.to_vec().as_slice()
        );
    }

    #[test]
    fn can_resolve_host_ip() {
        let cmd = ResolveHostIp {
            host: "utiliread.dk",
        };
        assert_eq_hex!(b"AT+CDNSGIP=\"utiliread.dk\"\r", cmd.to_vec().as_slice());

        let (mut ingress, res_sub, mut urc_sub) = setup_atat!();
        ingress.try_write(b"\r\nOK\r\n").unwrap();
        ingress
            .try_write(b"\r\n+CDNSGIP: 1,\"utiliread.dk\",\"1.2.3.4\"\r\n")
            .unwrap();

        let response = res_sub.try_get().unwrap();
        let response: &Response<200> = &response.borrow();
        if let Response::Ok(message) = response {
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
        assert_eq_hex!(b"AT+CDNSGIP=\"utiliread.dk\"\r", cmd.to_vec().as_slice());

        let (mut ingress, res_sub, mut urc_sub) = setup_atat!();
        ingress.try_write(b"\r\nOK\r\n").unwrap();
        ingress.try_write(b"\r\n+CDNSGIP: 0,8\r\n").unwrap();

        let response = res_sub.try_get().unwrap();
        let response: &Response<200> = &response.borrow();
        if let Response::Ok(message) = response {
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
        let (mut ingress, res_sub, urc_sub) = setup_atat!();
        ingress
            .try_write(b"AT+CDNSGIP=\"utiliread.com\"\r\r\nERROR\r\n")
            .unwrap();

        let response = res_sub.try_get().unwrap();
        let response: &Response<200> = &response.borrow();
        assert_eq!(&Response::OtherError, response);

        assert_eq!(0, urc_sub.available());
    }

    #[test]
    fn can_set_manual_rx_get_mode() {
        let cmd = SetManualRxGetMode;
        assert_eq_hex!(b"AT+CIPRXGET=1\r", cmd.to_vec().as_slice());
    }

    #[test]
    fn can_read_data() {
        let cmd = ReadData { id: 5, max_len: 16 };
        assert_eq_hex!(b"AT+CIPRXGET=2,5,16\r", cmd.to_vec().as_slice());

        let (mut ingress, res_sub, mut urc_sub) = setup_atat!();
        ingress
            .try_write(b"\r\n+CIPRXGET: 2,5,8,0\r\nHTTP\r\n\r\n")
            .unwrap();
        ingress.try_write(b"\r\nOK\r\n").unwrap();

        let response = res_sub.try_get().unwrap();
        let response: &Response<200> = &response.borrow();
        if let Response::Ok(message) = response {
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
        let (mut ingress, res_sub, mut urc_sub) = setup_atat!();

        ingress.try_write(b"\r\n>").unwrap();

        let response = res_sub.try_get().unwrap();
        let response: &Response<200> = &response.borrow();
        if let Response::Prompt(prompt) = response {
            assert_eq!(b'>', *prompt);
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
