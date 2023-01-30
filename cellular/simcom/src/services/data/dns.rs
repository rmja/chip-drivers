use embedded_hal_async::delay::DelayUs;
use embedded_nal::AddrType;
use embedded_nal_async::Dns;

use crate::{
    atat_async::AtatClient,
    commands::{tcpip::ResolveHostIp, urc::Urc},
};

use super::{DataService, SocketError};

impl<'a, At: AtatClient, Delay: DelayUs + Clone> Dns for DataService<'a, At, Delay> {
    type Error = SocketError;

    async fn get_host_by_name<'m>(
        &self,
        host: &str,
        addr_type: AddrType,
    ) -> Result<embedded_nal_async::IpAddr, Self::Error> {
        if addr_type == AddrType::IPv6 {
            return Err(SocketError::UnsupportedIpVersion);
        }
        assert!(addr_type == AddrType::IPv4 || addr_type == AddrType::Either);

        {
            let mut client = self.handle.client.lock().await;

            // Start resolving the host ip
            client.send(&ResolveHostIp { host }).await?;
        }

        // Wait for the URC reporting the resolved ip
        let mut ip = None;
        let mut delay = self.delay.clone();
        for _ in 0..50 {
            {
                let mut client = self.handle.client.lock().await;
                client.try_read_urc_with::<Urc, _>(|f| match f {
                    Urc::IpLookup(urc) if urc.host == host => {
                        ip = Some(urc.ip.parse().unwrap());
                        true
                    }
                    _ => false,
                });
            }

            if ip.is_some() {
                break;
            }

            delay.delay_ms(200).await.unwrap();
        }

        ip.ok_or(SocketError::DnsTimeout)
    }

    async fn get_host_by_address(
        &self,
        _addr: embedded_nal_async::IpAddr,
    ) -> Result<heapless::String<256>, Self::Error> {
        todo!()
    }
}
