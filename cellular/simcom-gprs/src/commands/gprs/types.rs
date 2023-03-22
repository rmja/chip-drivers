use atat::atat_derive::AtatEnum;

#[derive(AtatEnum, Debug, PartialEq)]
pub enum GPRSAttachedState {
    Detached = 0,
    Attached = 1,
}
