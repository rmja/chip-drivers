use atat::{AtatIngress, Ingress, IngressError};

use crate::{
    commands::urc::Urc,
    device::{URC_CAPACITY, URC_SUBSCRIBERS},
    SimcomDigester, SimcomResponseChannel, SimcomUrcChannel,
};

pub struct SimcomIngress<'a, const INGRESS_BUF_SIZE: usize>(
    Ingress<'a, SimcomDigester, Urc, INGRESS_BUF_SIZE, URC_CAPACITY, URC_SUBSCRIBERS>,
);

impl<'a, const INGRESS_BUF_SIZE: usize> SimcomIngress<'a, INGRESS_BUF_SIZE> {
    pub fn new(
        res_channel: &'a SimcomResponseChannel<INGRESS_BUF_SIZE>,
        urc_channel: &'a SimcomUrcChannel,
    ) -> Self {
        Self(Ingress::new(
            SimcomDigester::new(),
            res_channel,
            urc_channel,
        ))
    }
}

impl<const INGRESS_BUF_SIZE: usize> AtatIngress for SimcomIngress<'_, INGRESS_BUF_SIZE> {
    fn write_buf(&mut self) -> &mut [u8] {
        self.0.write_buf()
    }

    fn try_advance(&mut self, commit: usize) -> Result<(), IngressError> {
        self.0.try_advance(commit)
    }

    async fn advance(&mut self, commit: usize) {
        self.0.advance(commit).await
    }

    fn clear(&mut self) {
        self.0.clear()
    }
}
