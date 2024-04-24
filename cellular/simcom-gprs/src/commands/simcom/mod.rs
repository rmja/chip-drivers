mod responses;

use atat::atat_derive::AtatCmd;
pub use responses::*;

/// 6.2.23 AT+CCID Show ICCID
#[derive(AtatCmd)]
#[at_cmd("+CCID", GetCcidResponse, termination = "\r")]
pub struct GetCcid;

/// 6.2.38 AT+CCALR Call Ready Query
#[derive(AtatCmd)]
#[at_cmd("+CCALR?", CallReadyResponse, termination = "\r")]
pub struct GetCallReady;

#[cfg(test)]
mod tests {
    use assert_hex::assert_eq_hex;
    use atat::AtatCmd;

    use crate::commands::AtatCmdEx;

    use super::*;

    #[test]
    fn can_show_ccid() {
        let cmd = GetCcid {};
        assert_eq_hex!(b"AT+CCID\r", cmd.to_vec().as_slice());

        let response = cmd.parse(Ok(b"89457387300008689393\r\n")).unwrap();
        assert_eq!(b"89457387300008689393", response.iccid.as_slice());
        let iccid = core::str::from_utf8(response.iccid.as_slice()).unwrap();
        let iccid = iccid.parse::<u128>().unwrap();
        assert_eq!(89457387300008689393u128, iccid);
    }

    #[test]
    fn can_get_call_ready() {
        let cmd = GetCallReady {};
        assert_eq_hex!(b"AT+CCALR?\r", cmd.to_vec().as_slice());
    }
}
