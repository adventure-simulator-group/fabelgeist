//! Translate private mission listeners into the deployment's HTTPS origin.

use reqwest::Url;
use std::{net::SocketAddr, str::FromStr};

const SECURE_HTTP_SCHEME: &str = "https";
const ORIGIN_ROOT_PATH: &str = "/";

pub(crate) const TACTICAL_PROXY_FIRST_PORT: u16 = 6001;
pub(crate) const TACTICAL_PROXY_LAST_PORT: u16 = 6999;

#[derive(Clone, Debug)]
pub(crate) struct TacticalProxyOrigin(Url);

impl FromStr for TacticalProxyOrigin {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut origin =
            Url::parse(value).map_err(|_| "tactical proxy origin must be an HTTPS origin")?;
        if origin.scheme() != SECURE_HTTP_SCHEME
            || origin.host_str().is_none()
            || !origin.username().is_empty()
            || origin.password().is_some()
            || origin.path() != ORIGIN_ROOT_PATH
            || origin.query().is_some()
            || origin.fragment().is_some()
        {
            return Err(
                "tactical proxy origin must contain only an HTTPS scheme, host and optional port",
            );
        }
        origin
            .set_scheme("wss")
            .map_err(|_| "invalid tactical proxy scheme")?;
        Ok(Self(origin))
    }
}

impl TacticalProxyOrigin {
    pub(crate) fn mission_url(&self, address: &str) -> Result<String, &'static str> {
        let listener: SocketAddr = address
            .parse()
            .map_err(|_| "invalid tactical listener address")?;
        if !listener.ip().is_loopback()
            || !(TACTICAL_PROXY_FIRST_PORT..=TACTICAL_PROXY_LAST_PORT).contains(&listener.port())
        {
            return Err("tactical listener is outside the private deployment port range");
        }
        let mut url = self.0.clone();
        url.set_path(&format!("/t{}", listener.port()));
        Ok(url.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_listeners_use_the_configured_secure_origin() {
        let proxy: TacticalProxyOrigin = "https://test.fabelgeist.com".parse().unwrap();
        assert_eq!(
            proxy.mission_url("127.0.0.1:6001").unwrap(),
            "wss://test.fabelgeist.com/t6001"
        );
        assert_eq!(
            proxy.mission_url("[::1]:6999").unwrap(),
            "wss://test.fabelgeist.com/t6999"
        );
    }

    #[test]
    fn database_and_public_listeners_cannot_be_proxied() {
        let proxy: TacticalProxyOrigin = "https://test.fabelgeist.com".parse().unwrap();
        for address in [
            "127.0.0.1:3000",
            "127.0.0.1:6000",
            "127.0.0.1:7000",
            "203.0.113.1:6001",
            "garbage",
        ] {
            assert!(proxy.mission_url(address).is_err(), "{address}");
        }
    }

    #[test]
    fn proxy_configuration_rejects_credentials_and_non_origins() {
        for origin in [
            "http://example.com",
            "https://a:b@example.com",
            "https://example.com/path",
            "https://example.com?q=1",
            "https://example.com/#x",
        ] {
            assert!(origin.parse::<TacticalProxyOrigin>().is_err(), "{origin}");
        }
    }
}
