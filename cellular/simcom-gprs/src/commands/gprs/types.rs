use atat::atat_derive::AtatEnum;

#[derive(AtatEnum, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GPRSAttachedState {
    Detached = 0,
    Attached = 1,
}

#[derive(AtatEnum, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PdpState {
    Deactivated = 0,
    Activated = 1,
}
