use atat::atat_derive::AtatEnum;

#[derive(AtatEnum, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GPRSAttachedState {
    Detached = 0,
    Attached = 1,
}
