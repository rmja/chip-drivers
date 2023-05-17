use atat::atat_derive::AtatEnum;

#[derive(PartialEq)]
pub enum Facility {
    /// BAOC (Barr All Outgoing Calls)
    AO,
    /// BOIC (Barr Outgoing International Calls)
    OI,
    /// BOIC-exHC (Barr Outgoing International Calls except to Home Country)
    OX,
    /// BAIC (Barr All Incoming Calls)
    AI,
    /// BIC-Roam (Barr Incoming Calls when Roaming outside the home country)
    IR,
    /// SIM card or active application in the UICC (GSM or USIM) fixed dialling memory feature (if PIN2 authentication has not been done during the current session, PIN2 is required as <passwd>)
    FD,
    /// SIM (lock SIM/UICC card) (SIM/UICC asks password in MT power-up and when this lock command issued) Correspond to PIN1 code.
    SC,
    /// Network Personalization, Correspond to NCK code
    PN,
    /// Network subset Personalization Correspond to NSCK code
    PU,
    /// Service Provider Personalization Correspond to SPCK code
    PP,
}

#[derive(AtatEnum, PartialEq)]
#[at_enum(u8)]
pub enum FacilityMode {
    #[at_arg(value = 0)]
    Unlock,
    #[at_arg(value = 1)]
    Lock,
    #[at_arg(value = 2)]
    QueryStatus,
}

#[derive(AtatEnum, PartialEq)]
#[at_enum(u8)]
pub enum MobileEquipmentError {
    #[at_arg(value = 0)]
    Disable,
    #[at_arg(value = 1)]
    EnableNumeric,
    #[at_arg(value = 2)]
    EnableVerbose,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PinStatusCode {
    /// READY: MT is not pending for any password
    Ready,
    /// SIM PIN: MT is waiting SIM PIN to be given
    SimPin,
    /// SIM PUK: MT is waiting SIM PUK to be given
    SimPuk,
    /// PH-SIM PIN: ME is waiting for phone to SIM card (antitheft)
    PhSimPin,
    /// PH-SIM PIN: ME is waiting for SIM PUK (antitheft)
    PhSimPuk,
    /// SIM PIN2: MT is waiting SIM PIN2 to be given
    SimPin2,
    /// SIM PUK2: MT is waiting SIM PUK2 to be given
    SimPuk2,
}

#[derive(AtatEnum, Debug, PartialEq)]
pub enum NetworkRegistrationUrcConfig {
    /// Disable network registration unsolicited result code (default)
    Disabled = 0,
    /// Enable network registration unsolicited result code
    Enabled = 1,
    /// Enable network registration unsolicited result code with location information
    EnabledWithLocation = 2,
}

#[derive(AtatEnum, Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum NetworkRegistrationStat {
    /// Not registered, the MT is not currently searching a new operator to register to
    NotRegistered = 0,
    /// Registered, home network
    Registered = 1,
    /// Not registered, but the MT is currently searching a new operator to register to
    NotRegisteredSearching = 2,
    /// Registration denied
    RegistrationDenied = 3,
    /// Unknown (e.g. out of GERAN/UTRAN/E-UTRAN coverage)
    Unknown = 4,
    /// Registered, roaming
    RegisteredRoaming = 5,
}

impl NetworkRegistrationStat {
    pub fn is_registered(self) -> bool {
        self == NetworkRegistrationStat::Registered
            || self == NetworkRegistrationStat::RegisteredRoaming
    }
}
