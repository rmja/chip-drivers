mod responses;
mod types;

use atat::atat_derive::AtatCmd;
pub use responses::*;
pub use types::*;

use crate::ContextId;

use super::NoResponse;

/// 7.2.1 AT+CGATT Attach or Detach from GPRS Service
#[derive(AtatCmd)]
#[at_cmd(
    "+CGATT?",
    GPRSAttached,
    timeout_ms = 10_000,
    abortable = true,
    termination = "\r"
)]
pub struct GetGPRSAttached;

/// 7.2.1 AT+CGATT Attach or Detach from GPRS Service
#[derive(AtatCmd)]
#[at_cmd(
    "+CGATT",
    NoResponse,
    timeout_ms = 10_000,
    abortable = true,
    termination = "\r"
)]
pub struct SetGPRSAttached {
    pub state: GPRSAttachedState,
}

/// 7.2.2 AT+CGDCONT Define PDP Context
#[derive(AtatCmd)]
#[at_cmd("+CGDCONT", NoResponse, termination = "\r")]
pub struct SetPDPContextDefinition<'a> {
    #[at_arg(position = 0)]
    pub cid: ContextId,
    #[at_arg(position = 1, len = 6)]
    pub pdp_type: &'a str,
    #[at_arg(position = 2, len = 99)]
    pub apn: &'a str,
}

/// 7.2.10 AT+CGREG Network Registration Status
#[derive(AtatCmd)]
#[at_cmd("+CGREG?", GPRSNetworkRegistrationStatus, termination = "\r")]
pub struct GetGPRSNetworkRegistrationStatus;

#[cfg(test)]
mod tests {
    use assert_hex::assert_eq_hex;
    use atat::{AtatCmd, DigestResult, Digester, InternalError};

    use crate::SimcomDigester;

    use super::*;

    #[test]
    fn can_get_gprs_attached() {
        let cmd = GetGPRSAttached {};
        assert_eq_hex!(b"AT+CGATT?\r", cmd.as_bytes());

        let response = cmd.parse(Ok(b"+CGATT: 0")).unwrap();
        assert_eq!(GPRSAttachedState::Detached, response.state);
    }

    #[test]
    fn can_set_gprs_attached() {
        let cmd = SetGPRSAttached {
            state: GPRSAttachedState::Attached,
        };
        assert_eq_hex!(b"AT+CGATT=1\r", cmd.as_bytes());

        // sim800 timeout response
        // sim900 simply times out and does not send this error
        let mut digester = SimcomDigester::new();
        assert_eq!(
            (
                DigestResult::Response(Err(InternalError::CmeError(100.into()))),
                19
            ),
            digester.digest(b"\r\n+CME ERROR: 100\r\n")
        );
    }

    #[test]
    fn can_set_pdp_context_definition() {
        let cmd = SetPDPContextDefinition {
            cid: ContextId(1),
            pdp_type: "IP",
            apn: "internet",
        };

        assert_eq_hex!(b"AT+CGDCONT=1,\"IP\",\"internet\"\r", cmd.as_bytes());
    }

    #[test]
    fn can_get_gprs_network_registration_status() {
        let cmd = GetGPRSNetworkRegistrationStatus;
        assert_eq_hex!(b"AT+CGREG?\r", cmd.as_bytes());

        let response = cmd.parse(Ok(b"+CGREG: 0,2")).unwrap();
        assert_eq!(GPRSNetworkRegistrationUrcConfig::Disabled, response.n);
        assert_eq!(
            GPRSNetworkRegistrationStat::NotRegisteredSearching,
            response.stat
        );
    }
}
