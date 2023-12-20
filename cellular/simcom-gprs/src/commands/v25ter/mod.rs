use atat::atat_derive::{AtatCmd, AtatEnum};

use super::NoResponse;

/// 2.2.7 ATE Set Command Echo Mode
#[derive(AtatCmd)]
#[at_cmd("E", NoResponse, value_sep = false, termination = "\r")]
pub struct SetCommandEchoMode {
    pub mode: CommandEchoMode,
}

#[derive(PartialEq, AtatEnum)]
#[at_enum(u8)]
pub enum CommandEchoMode {
    #[at_arg(value = 0)]
    Disable,
    #[at_arg(value = 1)]
    Enable,
}

/// 2.2.25 ATV TA Response Format
#[derive(AtatCmd)]
#[at_cmd("V", NoResponse, value_sep = false, termination = "\r")]
pub struct SetResponseFormat {
    pub format: ResponseFormat,
}

#[derive(PartialEq, AtatEnum)]
#[at_enum(u8)]
pub enum ResponseFormat {
    #[at_arg(value = 0)]
    Numeric,
    #[at_arg(value = 1)]
    Text,
}

/// 2.2.27 ATZ Reset Default Configuration
#[derive(AtatCmd)]
#[at_cmd("Z", NoResponse, termination = "\r")]
pub struct Reset;

/// 2.2.30 AT&F Factory Defined Configuration
#[derive(AtatCmd)]
#[at_cmd("&F0", NoResponse, termination = "\r")]
pub struct SetFactoryDefinedConfiguration;

/// 2.2.40 AT+IFC Set TE-TA Local Data Flow Control
#[derive(AtatCmd)]
#[at_cmd("+IFC", NoResponse, termination = "\r")]
pub struct SetFlowControl {
    /// The method used by TE (us) at receive of data from TA (modem)
    pub from_modem: FlowControl,
    /// The method used by TA (modem) at receive of data from TE (us)
    pub to_modem: Option<FlowControl>,
}

#[derive(PartialEq, AtatEnum)]
pub enum FlowControl {
    Disabled = 0,
    /// Software flow control (XON/XOFF)
    XonXoff = 1,
    /// Hardware flow control (RTS/CTS)
    RtsCts = 2,
}

#[cfg(test)]
mod tests {
    use assert_hex::assert_eq_hex;

    use crate::commands::AtatCmdEx;

    use super::*;

    #[test]
    fn can_set_command_echo_mode() {
        let cmd = SetCommandEchoMode {
            mode: CommandEchoMode::Disable,
        };
        assert_eq_hex!(b"ATE0\r", cmd.to_vec().as_slice());
    }

    #[test]
    fn can_reset() {
        let cmd = Reset {};
        assert_eq_hex!(b"ATZ\r", cmd.to_vec().as_slice());
    }

    #[test]
    fn can_set_factory_default_configuration() {
        let cmd = SetFactoryDefinedConfiguration {};
        assert_eq_hex!(b"AT&F0\r", cmd.to_vec().as_slice());
    }

    #[test]
    fn can_set_flow_control() {
        let cmd = SetFlowControl {
            from_modem: FlowControl::Disabled,
            to_modem: None,
        };
        assert_eq_hex!(b"AT+IFC=0\r", cmd.to_vec().as_slice());

        let cmd = SetFlowControl {
            from_modem: FlowControl::Disabled,
            to_modem: Some(FlowControl::Disabled),
        };
        assert_eq_hex!(b"AT+IFC=0,0\r", cmd.to_vec().as_slice());
    }
}
