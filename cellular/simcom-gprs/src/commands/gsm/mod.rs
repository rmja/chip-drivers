//! Commands according to 3GPP TS27.007
mod impls;
mod responses;
mod types;
pub mod urcs;

use super::NoResponse;
use atat::atat_derive::AtatCmd;
pub use responses::*;
pub use types::*;

/// 3.2.8 Request Manufacturer Identification
#[derive(AtatCmd)]
#[at_cmd("+CGMI", ManufacturerIdResponse, termination = "\r")]
pub struct GetManufacturerId;

/// 3.2.9 Request Manufacturer Model
#[derive(AtatCmd)]
#[at_cmd("+CGMM", ModelIdResponse, termination = "\r")]
pub struct GetModelId;

/// 3.2.10 Request Revision Identification of Software Release
#[derive(AtatCmd)]
#[at_cmd("+CGMR", SoftwareVersionResponse, termination = "\r")]
pub struct GetSoftwareVersion;

/// 3.2.17 AT+CLCK Facility Lock
#[derive(AtatCmd)]
#[at_cmd("+CLCK", NoResponse, timeout_ms = 15_000, termination = "\r")]
pub struct SetFacilityLock<'a> {
    #[at_arg(position = 0)]
    pub facility: Facility,
    #[at_arg(position = 1)]
    pub mode: FacilityMode,
    #[at_arg(position = 2, len = 8)]
    pub password: Option<&'a str>,
}

/// 3.2.20 Report Mobile Equipment Error
#[derive(AtatCmd)]
#[at_cmd("+CMEE", NoResponse, termination = "\r")]
pub struct SetMobileEquipmentError {
    pub value: MobileEquipmentError,
}

/// 3.2.28 AT+CPIN Enter PIN
#[derive(AtatCmd)]
#[at_cmd("+CPIN?", NoResponse, timeout_ms = 5_000, termination = "\r")]
pub struct GetPinStatus;

#[derive(AtatCmd)]
#[at_cmd("+CPIN", NoResponse, timeout_ms = 5_000, termination = "\r")]
pub struct EnterPin<'a> {
    #[at_arg(len = 4)]
    pub pin: &'a str,
}

#[derive(AtatCmd)]
#[at_cmd("+CPIN", NoResponse, timeout_ms = 5_000, termination = "\r")]
pub struct ChangePin<'a> {
    #[at_arg(len = 8)]
    pub password: &'a str,
    #[at_arg(len = 4)]
    pub new_pin: &'a str,
}

// 3.2.29 AT+CPWD Change Password
#[derive(AtatCmd)]
#[at_cmd("+CPWD", NoResponse, timeout_ms = 15_000, termination = "\r")]
pub struct ChangePassword<'a> {
    #[at_arg(position = 0)]
    pub facility: Facility,
    #[at_arg(position = 1, len = 8)]
    pub old_password: &'a str,
    #[at_arg(position = 2, len = 8)]
    pub new_password: &'a str,
}

// 3.2.32 AT+CREG Network Registration
#[derive(AtatCmd)]
#[at_cmd("+CREG?", NetworkRegistrationStatus, termination = "\r")]
pub struct GetNetworkRegistrationStatus;

// 3.2.35 AT+CSQ Signal Quality Report
#[derive(AtatCmd)]
#[at_cmd("+CSQ?", SignalQuality, termination = "\r")]
pub struct GetSignalQuality;

#[cfg(test)]
mod tests {
    use assert_hex::assert_eq_hex;
    use atat::{nom::AsBytes, AtatCmd, DigestResult, Digester};

    use crate::{commands::AtatCmdEx, SimcomDigester};

    use super::*;

    #[test]
    fn can_get_manufacturer_id() {
        let cmd = GetManufacturerId;
        assert_eq_hex!(b"AT+CGMI\r", cmd.to_vec().as_bytes());

        let response = cmd.parse(Ok(b"SIMCOM_Ltd\r\n")).unwrap();
        assert_eq!(b"SIMCOM_Ltd", response.manufacturer.as_ref());

        let mut digester = SimcomDigester::new();
        assert_eq!(
            (DigestResult::Response(Ok(b"SIMCOM_Ltd")), 28),
            digester.digest(b"AT+CGMI\r\r\nSIMCOM_Ltd\r\n\r\nOK\r\n")
        );
        assert_eq!(
            (DigestResult::Response(Ok(b"SIMCOM_Ltd")), 20),
            digester.digest(b"\r\nSIMCOM_Ltd\r\n\r\nOK\r\n")
        );
    }

    #[test]
    fn can_get_model_id() {
        let cmd = GetModelId;
        assert_eq_hex!(b"AT+CGMM\r", cmd.to_vec().as_bytes());

        let response = cmd.parse(Ok(b"SIMCOM_SIM800\r\n")).unwrap();
        assert_eq!(b"SIMCOM_SIM800", response.model.as_ref());
    }

    #[test]
    fn can_get_software_version() {
        let cmd = GetSoftwareVersion;
        assert_eq_hex!(b"AT+CGMR\r", cmd.to_vec().as_bytes());

        let response = cmd.parse(Ok(b"Revision:1308B04SIM800M32\r\n")).unwrap();
        assert_eq!(b"Revision:1308B04SIM800M32", response.version.as_ref());
    }

    #[test]
    fn can_set_facility_lock_disable_pin() {
        let cmd = SetFacilityLock {
            facility: Facility::SC,
            mode: FacilityMode::Unlock,
            password: Some("1234"),
        };
        assert_eq_hex!(b"AT+CLCK=\"SC\",0,\"1234\"\r", cmd.to_vec().as_bytes());
    }

    #[test]
    fn can_set_facility_lock_enable_pin() {
        let cmd = SetFacilityLock {
            facility: Facility::SC,
            mode: FacilityMode::Lock,
            password: None,
        };
        assert_eq_hex!(b"AT+CLCK=\"SC\",1\r", cmd.to_vec().as_bytes());
    }

    #[test]
    fn can_set_mobile_equipment_error() {
        let cmd = SetMobileEquipmentError {
            value: MobileEquipmentError::EnableNumeric,
        };
        assert_eq_hex!(b"AT+CMEE=1\r", cmd.to_vec().as_bytes());
    }

    #[test]
    fn can_get_pin_status() {
        let cmd = GetPinStatus;
        assert_eq_hex!(b"AT+CPIN?\r", cmd.to_vec().as_bytes());
    }

    #[test]
    fn can_enter_pin() {
        let cmd = EnterPin { pin: "1234" };
        assert_eq_hex!(b"AT+CPIN=\"1234\"\r", cmd.to_vec().as_bytes());
    }

    #[test]
    fn can_change_pin() {
        let cmd = ChangePin {
            password: "11223344",
            new_pin: "1234",
        };
        assert_eq_hex!(b"AT+CPIN=\"11223344\",\"1234\"\r", cmd.to_vec().as_bytes());
    }

    #[test]
    fn can_change_password() {
        let cmd = ChangePassword {
            facility: Facility::SC,
            old_password: "1234",
            new_password: "4321",
        };
        assert_eq_hex!(
            b"AT+CPWD=\"SC\",\"1234\",\"4321\"\r",
            cmd.to_vec().as_bytes()
        );
    }

    #[test]
    fn can_get_network_registration_status() {
        let cmd = GetNetworkRegistrationStatus;
        assert_eq_hex!(b"AT+CREG?\r", cmd.to_vec().as_bytes());

        let response = cmd.parse(Ok(b"+CREG: 0,0")).unwrap();
        assert_eq!(NetworkRegistrationUrcConfig::Disabled, response.n);
        assert_eq!(NetworkRegistrationStat::NotRegistered, response.stat);
    }

    #[test]
    fn can_get_signal_quality() {
        let cmd = GetSignalQuality;
        assert_eq_hex!(b"AT+CSQ?\r", cmd.to_vec().as_bytes());

        let response = cmd.parse(Ok(b"+CSQ: 20,0")).unwrap();
        assert_eq!(-74, response.rssi().unwrap());
    }
}
