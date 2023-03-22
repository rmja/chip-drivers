mod responses;

use atat::atat_derive::AtatCmd;
pub use responses::*;

/// 6.2.38 AT+CCALR Call Ready Query
#[derive(AtatCmd)]
#[at_cmd("+CCALR?", CallReadyResponse, termination = "\r")]
pub struct GetCallReady;

#[cfg(test)]
mod tests {
    use assert_hex::assert_eq_hex;
    use atat::AtatCmd;

    use super::*;

    #[test]
    fn can_get_call_ready() {
        let cmd = GetCallReady {};
        assert_eq_hex!(b"AT+CCALR?\r", cmd.as_bytes());
    }
}
