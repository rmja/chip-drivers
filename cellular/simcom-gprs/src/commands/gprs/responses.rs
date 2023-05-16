use atat::atat_derive::{AtatEnum, AtatResp};
use heapless::String;

use super::GPRSAttachedState;

/// 7.2.1 AT+CGATT Attach or Detach from GPRS Service
#[derive(AtatResp)]
pub struct GPRSAttached {
    #[at_arg(position = 0)]
    pub state: GPRSAttachedState,
}

/// 7.2.10 AT+CGREG Network Registration Status
#[derive(AtatResp)]
pub struct GPRSNetworkRegistrationStatus {
    #[at_arg(position = 0)]
    pub n: GPRSNetworkRegistrationUrcConfig,
    #[at_arg(position = 1)]
    pub stat: GPRSNetworkRegistrationStat,
    #[at_arg(position = 2)]
    pub lac: Option<String<4>>,
    #[at_arg(position = 3)]
    pub ci: Option<String<8>>,
}

#[derive(AtatEnum, Debug, PartialEq)]
pub enum GPRSNetworkRegistrationUrcConfig {
    /// Disable network registration unsolicited result code (default)
    Disabled = 0,
    /// Enable network registration unsolicited result code
    Enabled = 1,
    /// Enable network registration unsolicited result code with location information
    EnabledWithLocation = 2,
}

#[derive(AtatEnum, Debug, PartialEq)]
pub enum GPRSNetworkRegistrationStat {
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

impl GPRSNetworkRegistrationStat {
    pub fn is_registered(self) -> bool {
        self == GPRSNetworkRegistrationStat::Registered
            || self == GPRSNetworkRegistrationStat::RegisteredRoaming
    }
}
