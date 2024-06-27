use atat::{asynch::AtatClient, CmeError};
use embassy_time::{with_timeout, Duration, Instant, Timer};

use crate::{
    commands::{
        gprs, gsm,
        simcom::{CallReady, GetCallReady},
        urc::Urc,
    },
    device::Handle,
    SimcomConfig, SimcomDevice, SimcomUrcChannel,
};

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum NetworkError {
    Atat(atat::Error),
    PdpStateTimeout,
    NotReady,
    NotRegistered,
    GprsNotRegistered,
    NotAttached,
    PinRequired,
    PukRequired,
    PinTimeout,
    InvalidRssi,
    UnexpectedPinStatus(gsm::PinStatusCode),
}

impl From<atat::Error> for NetworkError {
    fn from(value: atat::Error) -> Self {
        Self::Atat(value)
    }
}

pub struct Network<'dev, 'sub, AtCl: AtatClient> {
    handle: &'dev Handle<'sub, AtCl>,
    urc_channel: &'dev SimcomUrcChannel,
}

impl<'dev, 'sub, AtCl: AtatClient, Config: SimcomConfig> SimcomDevice<'dev, 'sub, AtCl, Config> {
    pub fn network(&'dev self) -> Network<'dev, 'sub, AtCl> {
        Network {
            handle: &self.handle,
            urc_channel: self.urc_channel,
        }
    }
}

impl<AtCl: AtatClient + 'static> Network<'_, '_, AtCl> {
    /// Attach the modem to the network
    pub async fn attach(&mut self, pin: Option<&str>) -> Result<(), NetworkError> {
        // AT+CCALR?
        self.ensure_ready().await?;

        // AT+CPIN?
        let status = self.get_pin_status().await?;
        match status {
            gsm::PinStatusCode::Ready => {}
            gsm::PinStatusCode::SimPin => {
                let pin = pin.ok_or(NetworkError::PinRequired)?;
                self.enter_pin(pin).await?;
            }
            _ => return Err(NetworkError::UnexpectedPinStatus(status)),
        }

        let mut client = self.handle.client.lock().await;

        // AT+COPS? - Ensure that we are using automatic operator selection
        // See https://onomondo.com/blog/at-command-attach-detach-modems-tips/
        let response = client.send(&gsm::GetOperatorSelection).await?;
        if response.mode != gsm::OperatorSelectionMode::Automatic {
            client
                .send(&gsm::SetOperatorSelection {
                    mode: gsm::OperatorSelectionMode::Automatic,
                    format: None,
                    operator: None,
                })
                .await?;
        }

        // AT+CREG?
        let mut is_registered = false;
        for _ in 0..60 {
            let response = client.send(&gsm::GetNetworkRegistrationStatus).await?;
            if response.stat.is_registered() {
                is_registered = true;
                break;
            }

            Timer::after(Duration::from_millis(500)).await;
        }
        if !is_registered {
            return Err(NetworkError::NotRegistered);
        }

        // AT+CGATT
        if client.send(&gprs::GetGPRSAttached).await?.state == gprs::GPRSAttachedState::Detached {
            Self::attach_inner(&mut client).await?;
        }

        // AT+CGREG?
        let mut is_registered = false;
        for _ in 0..60 {
            let response = client.send(&gprs::GetGPRSNetworkRegistrationStatus).await?;
            if response.stat.is_registered() {
                is_registered = true;
                break;
            }
        }
        if !is_registered {
            return Err(NetworkError::GprsNotRegistered);
        }

        Ok(())
    }

    async fn attach_inner(client: &mut AtCl) -> Result<(), NetworkError> {
        for _ in 0..30 {
            match client
                .send(&gprs::SetGPRSAttached {
                    state: gprs::GPRSAttachedState::Attached,
                })
                .await
            {
                Ok(_) => break,
                // sim800 (not sim900) reports CME ERROR 100 if it was unable to attach
                Err(atat::Error::CmeError(CmeError::Unknown)) => {}
                Err(err) => return Err(err.into()),
            }

            Timer::after(Duration::from_millis(1000)).await;
        }

        if client.send(&gprs::GetGPRSAttached).await?.state == gprs::GPRSAttachedState::Attached {
            Ok(())
        } else {
            Err(NetworkError::NotAttached)
        }
    }

    async fn ensure_ready(&mut self) -> Result<(), NetworkError> {
        let mut client = self.handle.client.lock().await;
        for _ in 0..20 {
            let response = client.send(&GetCallReady).await?;
            if response.ready == CallReady::Ready {
                return Ok(());
            }

            Timer::after(Duration::from_millis(1_000)).await;
        }
        Err(NetworkError::NotReady)
    }

    /// Clear the FPLMN (forbidden network) list
    /// See e.g. https://onomondo.com/blog/how-to-clear-the-fplmn-list-on-a-sim/
    pub async fn clear_fplmn_list(&mut self) -> Result<(), NetworkError> {
        let mut client = self.handle.client.lock().await;
        client
            .send(&gsm::SetRestrictedSimAccess {
                command: gsm::RestrictedSimAccessCommand::UpdateBinary,
                file_id: 28539,
                p0: 0,
                p1: 0,
                p2: 12,
                data: "FFFFFFFFFFFFFFFFFFFFFFFF",
            })
            .await?;
        Ok(())
    }

    /// Get the current signal quality from modem
    pub async fn get_signal_quality(&self) -> Result<i8, NetworkError> {
        let mut client = self.handle.client.lock().await;
        client
            .send(&gsm::GetSignalQuality)
            .await?
            .rssi()
            .ok_or(NetworkError::InvalidRssi)
    }

    /// Get the pin status
    pub async fn get_pin_status(&mut self) -> Result<gsm::PinStatusCode, NetworkError> {
        let mut urc_subscription = {
            let mut client = self.handle.client.lock().await;
            let subscription = self.urc_channel.subscribe().unwrap();

            client.send(&gsm::GetPinStatus).await?;

            subscription
        };

        let timeout_instant = Instant::now() + Duration::from_secs(5);
        while let Some(remaining) = timeout_instant.checked_duration_since(Instant::now()) {
            let urc = with_timeout(remaining, urc_subscription.next_message_pure())
                .await
                .map_err(|_| NetworkError::PinTimeout)?;
            self.handle.drain_background_urcs();

            if let Urc::PinStatus(status) = urc {
                return Ok(status.code);
            }
        }

        Err(NetworkError::PinTimeout)
    }

    async fn enter_pin(&mut self, pin: &str) -> Result<gsm::PinStatusCode, NetworkError> {
        let mut urc_subscription = {
            let mut client = self.handle.client.lock().await;
            let subscription = self.urc_channel.subscribe().unwrap();

            client.send(&gsm::EnterPin { pin }).await?;

            subscription
        };

        let timeout_instant = Instant::now() + Duration::from_secs(5);
        while let Some(remaining) = timeout_instant.checked_duration_since(Instant::now()) {
            let urc = with_timeout(remaining, urc_subscription.next_message_pure())
                .await
                .map_err(|_| NetworkError::PinTimeout)?;
            self.handle.drain_background_urcs();

            if let Urc::PinStatus(status) = urc {
                return Ok(status.code);
            }
        }

        Err(NetworkError::PinTimeout)
    }

    pub async fn set_pin(
        &mut self,
        new_pin: &str,
        old_pin_or_puk: &str,
    ) -> Result<(), NetworkError> {
        let mut status = self.get_pin_status().await?;
        if status == gsm::PinStatusCode::SimPin {
            let old_pin = old_pin_or_puk;
            status = self.enter_pin(old_pin).await?;
        }
        match status {
            gsm::PinStatusCode::Ready => {
                let old_pin = old_pin_or_puk;

                let mut client = self.handle.client.lock().await;
                client
                    .send(&gsm::ChangePassword {
                        facility: gsm::Facility::SC,
                        old_password: old_pin,
                        new_password: new_pin,
                    })
                    .await?;
                Ok(())
            }
            gsm::PinStatusCode::SimPuk => {
                let puk = old_pin_or_puk;
                let mut client = self.handle.client.lock().await;
                client
                    .send(&gsm::ChangePin {
                        password: puk,
                        new_pin,
                    })
                    .await?;
                Ok(())
            }
            _ => Err(NetworkError::UnexpectedPinStatus(status)),
        }
    }

    pub async fn enable_pin(&mut self, pin: &str) -> Result<(), NetworkError> {
        let status = self.get_pin_status().await?;
        if status != gsm::PinStatusCode::Ready {
            return Err(NetworkError::UnexpectedPinStatus(status));
        }

        let mut client = self.handle.client.lock().await;
        client
            .send(&gsm::SetFacilityLock {
                facility: gsm::Facility::SC,
                mode: gsm::FacilityMode::Lock,
                password: Some(pin),
            })
            .await?;

        Ok(())
    }

    pub async fn disable_pin(&mut self, pin: &str) -> Result<(), NetworkError> {
        let mut status = self.get_pin_status().await?;
        if status == gsm::PinStatusCode::SimPin {
            status = self.enter_pin(pin).await?;
        }
        if status != gsm::PinStatusCode::Ready {
            return Err(NetworkError::UnexpectedPinStatus(status));
        }

        let mut client = self.handle.client.lock().await;
        client
            .send(&gsm::SetFacilityLock {
                facility: gsm::Facility::SC,
                mode: gsm::FacilityMode::Unlock,
                password: Some(pin),
            })
            .await?;

        Ok(())
    }
}
