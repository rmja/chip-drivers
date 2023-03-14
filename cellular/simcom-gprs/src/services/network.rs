use atat::{asynch::AtatClient, AtatUrcChannel};
use embassy_time::{Duration, Timer};

use crate::{
    commands::{
        gprs::{
            GPRSAttachedState, GPRSNetworkRegistrationStat, GetGPRSAttached,
            GetGPRSNetworkRegistrationStatus, SetGPRSAttached,
        },
        gsm::{
            self, EnterPin, GetNetworkRegistrationStatus, GetPinStatus, NetworkRegistrationStat,
            PinStatusCode,
        },
        simcom::{CallReady, GetCallReady},
        urc::Urc,
    },
    device::Handle,
    Device,
};

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum NetworkError {
    Atat(atat::Error),
    NotReady,
    NotRegistered,
    NotAttached,
    PinRequired,
    InvalidRssi,
    UnexpectedPinStatus(PinStatusCode),
}

impl From<atat::Error> for NetworkError {
    fn from(value: atat::Error) -> Self {
        Self::Atat(value)
    }
}

pub struct Network<'dev, 'sub, AtCl: AtatClient> {
    handle: &'dev Handle<'sub, AtCl>,
    gsm_status: NetworkRegistrationStat,
    gprs_status: GPRSNetworkRegistrationStat,
}

impl<'dev, 'sub, AtCl: AtatClient, AtUrcCh: AtatUrcChannel<Urc>> Device<'dev, 'sub, AtCl, AtUrcCh> {
    pub fn network(&'dev self) -> Network<'dev, 'sub, AtCl> {
        Network {
            handle: &self.handle,
            gsm_status: NetworkRegistrationStat::NotRegistered,
            gprs_status: GPRSNetworkRegistrationStat::NotRegistered,
        }
    }
}

impl<AtCl: AtatClient> Network<'_, '_, AtCl> {
    /// Attach the modem to the network
    pub async fn attach(&mut self, pin: Option<&str>) -> Result<(), NetworkError> {
        let mut client = self.handle.client.lock().await;

        async {
            for _ in 0..20 {
                let response = client.send(&GetCallReady).await?;
                if response.ready == CallReady::Ready {
                    return Ok(());
                }

                Timer::after(Duration::from_millis(1_000)).await;
            }
            Err(NetworkError::NotReady)
        }
        .await?;

        let status = client.send(&GetPinStatus).await?;
        match status.code {
            PinStatusCode::Ready => {}
            PinStatusCode::SimPin => {
                let pin = pin.ok_or(NetworkError::PinRequired)?;
                client.send(&EnterPin { pin }).await?;
            }
            _ => return Err(NetworkError::UnexpectedPinStatus(status.code)),
        }

        for _ in 0..60 {
            self.update_registration(&mut *client).await?;

            if self.is_gsm_registered() {
                break;
            }
        }
        if !self.is_gsm_registered() {
            return Err(NetworkError::NotRegistered);
        }

        if client.send(&GetGPRSAttached).await?.state == GPRSAttachedState::Detached {
            async {
                for _ in 0..10 {
                    match client
                        .send(&SetGPRSAttached {
                            state: GPRSAttachedState::Attached,
                        })
                        .await
                    {
                        Ok(_) => return Ok(()),
                        // sim800 (not sim900) reports CME ERROR 100 if it was unable to attach
                        Err(atat::Error::CmeError(err)) if err as u16 == 100 => {}
                        Err(err) => return Err(err.into()),
                    }

                    Timer::after(Duration::from_millis(1_000)).await;
                }

                Err(NetworkError::NotAttached)
            }
            .await?;
        }

        for _ in 0..60 {
            self.update_registration(&mut *client).await?;

            if self.is_gprs_registered() {
                break;
            }
        }
        if !self.is_gprs_registered() {
            return Err(NetworkError::NotRegistered);
        }

        Ok(())
    }

    /// Get the current signal quality from modem
    pub async fn get_signal_quality(&mut self) -> Result<i8, NetworkError> {
        let mut client = self.handle.client.lock().await;
        client
            .send(&gsm::GetSignalQuality)
            .await?
            .rssi()
            .ok_or(NetworkError::InvalidRssi)
    }

    fn is_gsm_registered(&self) -> bool {
        [
            NetworkRegistrationStat::Registered,
            NetworkRegistrationStat::RegisteredRoaming,
        ]
        .contains(&self.gsm_status)
    }

    fn is_gprs_registered(&self) -> bool {
        [
            GPRSNetworkRegistrationStat::Registered,
            GPRSNetworkRegistrationStat::RegisteredRoaming,
        ]
        .contains(&self.gprs_status)
    }

    async fn update_registration(&mut self, client: &mut AtCl) -> Result<(), NetworkError> {
        let response = client.send(&GetNetworkRegistrationStatus).await?;
        self.gsm_status = response.stat;

        let response = client.send(&GetGPRSNetworkRegistrationStatus).await?;
        self.gprs_status = response.stat;

        Ok(())
    }
}
