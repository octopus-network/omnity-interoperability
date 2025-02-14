use anyhow::{anyhow, Result};
use candid::CandidType;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use url::Url;

#[derive(
    CandidType, Serialize, Deserialize, Default, Clone, Debug, Eq, PartialEq, Ord, PartialOrd,
)]
pub enum Provider {
    Localnet,
    Devnet,
    #[default]
    Testnet,
    Mainnet,
    Custom(String, String),
}

impl FromStr for Provider {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Provider> {
        match s.to_lowercase().as_str() {
            "l" | "localnet" => Ok(Provider::Localnet),
            "d" | "devnet" => Ok(Provider::Devnet),
            "t" | "testnet" => Ok(Provider::Testnet),
            "m" | "mainnet" => Ok(Provider::Mainnet),
            _ if s.starts_with("http") => {
                let http_url = s;

                let mut ws_url = Url::parse(http_url)?;
                if let Some(port) = ws_url.port() {
                    ws_url.set_port(Some(port + 1))
                        .map_err(|_| anyhow!("Unable to set port"))?;
                }
                if ws_url.scheme() == "https" {
                    ws_url.set_scheme("wss")
                        .map_err(|_| anyhow!("Unable to set scheme"))?;
                } else {
                    ws_url.set_scheme("ws")
                        .map_err(|_| anyhow!("Unable to set scheme"))?;
                }

                Ok(Provider::Custom(http_url.to_string(), ws_url.to_string()))
            }
            _ => Err(anyhow::Error::msg(
                "Cluster must be one of [localnet, testnet, mainnet, devnet] or be an http or https url\n",
            )),
        }
    }
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let cluster_str = match self {
            Provider::Localnet => "localnet",
            Provider::Devnet => "devnet",
            Provider::Testnet => "testnet",
            Provider::Mainnet => "mainnet",
            Provider::Custom(url, _ws_url) => url,
        };
        write!(f, "{cluster_str}")
    }
}

impl Provider {
    pub fn url(&self) -> &str {
        match self {
            Provider::Localnet => "http://127.0.0.1:9000",
            Provider::Devnet => "https://fullnode.devnet.sui.io:443",
            Provider::Testnet => "https://fullnode.testnet.sui.io:443",
            Provider::Mainnet => "https://fullnode.mainnet.sui.io:443",
            Provider::Custom(url, _ws_url) => url,
        }
    }
    pub fn ws_url(&self) -> &str {
        match self {
            Provider::Localnet => "ws://127.0.0.1:9000",
            Provider::Devnet => "wss://fullnode.devnet.sui.io:443",
            Provider::Testnet => "wss://fullnode.testnet.sui.io:443",
            Provider::Mainnet => "wss://fullnode.mainnet.sui.io:443",
            Provider::Custom(_url, ws_url) => ws_url,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_cluster(name: &str, cluster: Provider) {
        assert_eq!(Provider::from_str(name).unwrap(), cluster);
    }

    #[test]
    fn test_cluster_parse() {
        test_cluster("testnet", Provider::Testnet);
        test_cluster("mainnet", Provider::Mainnet);
        test_cluster("devnet", Provider::Devnet);
        test_cluster("localnet", Provider::Localnet);
    }

    #[test]
    #[should_panic]
    fn test_cluster_bad_parse() {
        let bad_url = "httq://my_custom_url.test.net";
        Provider::from_str(bad_url).unwrap();
    }

    #[test]
    fn test_http_port() {
        let url = "http://my-url.com:7000/";
        let cluster = Provider::from_str(url).unwrap();
        assert_eq!(
            Provider::Custom(url.to_string(), "ws://my-url.com:7001/".to_string()),
            cluster
        );
    }

    #[test]
    fn test_http_no_port() {
        let url = "http://my-url.com/";
        let cluster = Provider::from_str(url).unwrap();
        assert_eq!(
            Provider::Custom(url.to_string(), "ws://my-url.com/".to_string()),
            cluster
        );
    }

    #[test]
    fn test_https_port() {
        let url = "https://my-url.com:7000/";
        let cluster = Provider::from_str(url).unwrap();
        assert_eq!(
            Provider::Custom(url.to_string(), "wss://my-url.com:7001/".to_string()),
            cluster
        );
    }
    #[test]
    fn test_https_no_port() {
        let url = "https://my-url.com/";
        let cluster = Provider::from_str(url).unwrap();
        assert_eq!(
            Provider::Custom(url.to_string(), "wss://my-url.com/".to_string()),
            cluster
        );
    }

    #[test]
    fn test_upper_case() {
        let url = "http://my-url.com/FooBar";
        let cluster = Provider::from_str(url).unwrap();
        assert_eq!(
            Provider::Custom(url.to_string(), "ws://my-url.com/FooBar".to_string()),
            cluster
        );
    }
}
