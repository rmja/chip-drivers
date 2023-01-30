use atat::atat_derive::{AtatEnum, AtatResp};

/// 6.2.44 Call Ready Query
#[derive(Clone, AtatResp)]
pub struct CallReadyResponse {
    pub ready: CallReady,
}

#[derive(Clone, PartialEq, AtatEnum)]
#[at_enum(u8)]
pub enum CallReady {
    #[at_arg(value = 0)]
    NotReady,
    #[at_arg(value = 1)]
    Ready,
}
