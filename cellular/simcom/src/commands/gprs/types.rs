use atat::atat_derive::AtatEnum;

#[derive(Debug, Clone, PartialEq, AtatEnum)]
pub enum GPRSAttachedState {
    Detached = 0,
    Attached = 1,
}
