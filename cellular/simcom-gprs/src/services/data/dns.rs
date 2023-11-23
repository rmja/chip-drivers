use atat::{asynch::AtatClient, AtatUrcChannel};
use embassy_time::{with_timeout, Duration, Instant};
use embedded_nal_async::{AddrType, Dns};

use crate::{
    commands::{tcpip::ResolveHostIp, urc::Urc},
    device::{URC_CAPACITY, URC_SUBSCRIBERS},
};

use super::{DataService, SocketError};

impl<AtCl: AtatClient + 'static, AtUrcCh: AtatUrcChannel<Urc, URC_CAPACITY, URC_SUBSCRIBERS>> Dns
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
            client.send(&ResolveHostIp { host }).await?;

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
        _result: &mut [u8],
    ) -> Result<usize, Self::Error> {
        unimplemented!()
    }
}
