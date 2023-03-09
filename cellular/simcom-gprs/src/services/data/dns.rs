use atat::{asynch::AtatClient, AtatUrcChannel};
use embassy_time::{with_timeout, Duration, Instant};
use embedded_nal_async::{AddrType, Dns};

use crate::commands::{tcpip::ResolveHostIp, urc::Urc};

use super::{DataService, SocketError};

impl<'a, AtCl: AtatClient, AtUrcCh: AtatUrcChannel<Urc>> Dns for DataService<'a, AtCl, AtUrcCh> {
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

        self.handle.drain_background_urcs();

        // For now we can only have one lookup going at a time,
        // as having more would require that we have multiple dns subscriptions
        self.dns_lock.lock().await;
        let mut subscription = {
            let mut client = self.handle.client.lock().await;
            let subscription = self.urc_channel.subscribe().unwrap();

            // Start resolving the host ip
            client.send(&ResolveHostIp { host }).await?;

            subscription
        };

        // Wait for the URC reporting the resolved ip
        let timeout_instant = Instant::now() + Duration::from_secs(10);
        while let Some(remaining) = timeout_instant.checked_duration_since(Instant::now()) {
            let urc = with_timeout(remaining, subscription.next_message_pure())
                .await
                .map_err(|_| SocketError::DnsTimeout)?;
            self.handle.drain_background_urcs();

            if let Urc::IpLookup(result) = urc && result.host == host {
                return Ok(result.ip.parse().unwrap());
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
