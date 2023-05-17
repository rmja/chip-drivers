use atat::{asynch::AtatClient, AtatUrcChannel};
use embassy_time::{with_timeout, Duration, Instant, Timer};
use embedded_nal_async::{AddrType, Dns};

use crate::commands::{tcpip::ResolveHostIp, urc::Urc};

use super::{DataService, SocketError};

impl<AtCl: AtatClient, AtUrcCh: AtatUrcChannel<Urc>> Dns
    for DataService<'_, '_, '_, AtCl, AtUrcCh>
{
    type Error = SocketError;

    async fn get_host_by_name(
        &self,
        host: &str,
        addr_type: AddrType,
    ) -> Result<embedded_nal_async::IpAddr, Self::Error> {
        if addr_type == AddrType::IPv6 {
            return Err(SocketError::UnsupportedIpVersion);
        }
        assert!(addr_type == AddrType::IPv4 || addr_type == AddrType::Either);

        // The modem can only handle one dns lookup at a time
        // TODO: Maybe let the mutex protect the handle instead of having a Mutex<()>
        let _guard = self.dns_lock.lock().await;

        self.handle.drain_background_urcs();

        let mut urc_subscription = {
            let mut client = self.handle.client.lock().await;
            let subscription = self.urc_channel.subscribe().unwrap();

            // Start resolving the host ip
            // It can happen that the modem returns transient ERROR's, so we do this with retries
            'retries: for attempt in 1..=5 {
                if attempt > 1 {
                    debug!("Attempt {}:", attempt);
                }

                match client.send(&ResolveHostIp { host }).await {
                    Ok(_) => {
                        break 'retries;
                    }
                    Err(e) => {
                        warn!("DNS lookup error: {:?}", e);
                    }
                }

                Timer::after(Duration::from_millis(1000)).await;
            }

            subscription
        };

        // Wait for the URC reporting the resolved ip
        let timeout_instant = Instant::now() + Duration::from_secs(20);
        while let Some(remaining) = timeout_instant.checked_duration_since(Instant::now()) {
            let urc = with_timeout(remaining, urc_subscription.next_message_pure())
                .await
                .map_err(|_| SocketError::DnsTimeout)?;
            self.handle.drain_background_urcs();

            if let Urc::DnsResult(result) = urc {
                if let Ok(result) = result {
                    if result.host == host {
                        return Ok(result.ip.parse().unwrap());
                    }
                } else {
                    return Err(SocketError::DnsError);
                }
            }
        }

        Err(SocketError::DnsTimeout)
    }

    async fn get_host_by_address(
        &self,
        _addr: embedded_nal_async::IpAddr,
    ) -> Result<heapless::String<256>, Self::Error> {
        unimplemented!()
    }
}
