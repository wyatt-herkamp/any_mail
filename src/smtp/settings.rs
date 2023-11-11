use std::{
    convert::Infallible,
    fmt::Display,
    net::{Ipv4Addr, Ipv6Addr},
    str::FromStr,
    time::Duration,
};

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIs, EnumString, EnumVariantNames};

use crate::{
    email_types::{Address, Mailbox},
    EmailSettingsType,
};
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ClientId {
    Domain(String),
    IPv4(Ipv4Addr),
    IPv6(Ipv6Addr),
}
impl From<ClientId> for lettre::transport::smtp::extension::ClientId {
    fn from(value: ClientId) -> Self {
        match value {
            ClientId::Domain(v) => Self::Domain(v),
            ClientId::IPv4(v) => Self::Ipv4(v),
            ClientId::IPv6(v) => Self::Ipv6(v),
        }
    }
}
impl Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientId::Domain(v) => write!(f, "{}", v),
            ClientId::IPv4(v) => write!(f, "{}", v),
            ClientId::IPv6(v) => write!(f, "{}", v),
        }
    }
}
impl Default for ClientId {
    fn default() -> Self {
        hostname::get()
            .ok()
            .and_then(|v| v.to_str().map(ToOwned::to_owned))
            .map(|v| Self::Domain(v))
            .unwrap_or(Self::IPv4(Ipv4Addr::LOCALHOST))
    }
}
impl From<String> for ClientId {
    fn from(s: String) -> Self {
        if let Ok(ip) = s.parse::<Ipv4Addr>() {
            return Self::IPv4(ip);
        }
        if let Ok(ip) = s.parse::<Ipv6Addr>() {
            return Self::IPv6(ip);
        }
        Self::Domain(s)
    }
}
impl From<&str> for ClientId {
    fn from(s: &str) -> Self {
        if let Ok(ip) = s.parse::<Ipv4Addr>() {
            return Self::IPv4(ip);
        }
        if let Ok(ip) = s.parse::<Ipv6Addr>() {
            return Self::IPv6(ip);
        }
        Self::Domain(s.to_owned())
    }
}
impl FromStr for ClientId {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

impl Serialize for ClientId {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ClientId::Domain(v) => serializer.serialize_str(v),
            ClientId::IPv4(v) => serializer.serialize_str(&v.to_string()),
            ClientId::IPv6(v) => serializer.serialize_str(&v.to_string()),
        }
    }
}
impl<'de> Deserialize<'de> for ClientId {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer).map(Self::from)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_client_id() {
        let domain = "example.com";
        let ipv4 = "127.0.0.1";
        let ipv6 = "::1";
        let domain_parsed = ClientId::from_str(domain).unwrap();
        let ipv4_parsed = ClientId::from_str(ipv4).unwrap();
        let ipv6_parsed = ClientId::from_str(ipv6).unwrap();
        assert_eq!(domain_parsed, ClientId::Domain(domain.to_string()));
        assert_eq!(ipv4_parsed, ClientId::IPv4(ipv4.parse().unwrap()));
        assert_eq!(ipv6_parsed, ClientId::IPv6(ipv6.parse().unwrap()));
    }
}
#[derive(
    Debug,
    Deserialize,
    Serialize,
    Clone,
    Default,
    PartialEq,
    Eq,
    Hash,
    Display,
    EnumString,
    EnumIs,
    EnumVariantNames,
)]
pub enum SMTPServiceEncryption {
    /// Default Port is 25
    #[default]
    NONE,
    /// Default Port is 587
    StartTLS,
    /// Default Port is 465
    TLS,
}
impl SMTPServiceEncryption {
    pub fn default_port(&self) -> u16 {
        match self {
            SMTPServiceEncryption::NONE => 25,
            SMTPServiceEncryption::StartTLS => 587,
            SMTPServiceEncryption::TLS => 465,
        }
    }
}
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
pub struct SMTPServiceSettings {
    pub username: String,
    pub password: String,
    pub host: String,
    /// If None, the port will be determined by the encryption type
    ///
    /// - NONE: 25
    /// - StartTLS: 587
    /// - TLS: 465
    pub port: Option<u16>,
    pub encryption: SMTPServiceEncryption,
    pub from: Mailbox,
    pub reply_to: Option<Mailbox>,
    pub timeout: Option<i64>,
    #[serde(default)]
    pub client_id: ClientId,
    #[serde(default)]
    pub channel_size: usize,
}
impl SMTPServiceSettings {
    pub(crate) fn get_timeout(&self) -> Option<Duration> {
        if let Some(v) = self.timeout {
            if v <= -1 {
                return None;
            }
            Some(Duration::from_millis(v as u64))
        } else {
            None
        }
    }
}
impl Default for SMTPServiceSettings {
    fn default() -> Self {
        Self {
            username: Default::default(),
            password: Default::default(),
            host: "127.0.0.1".to_string(),
            port: None,
            encryption: SMTPServiceEncryption::TLS,
            from: Mailbox::new(None, unsafe {
                Address::new_unchecked("no-reply@example.com")
            }),
            reply_to: Some(Mailbox::new(None, unsafe {
                Address::new_unchecked("admin@edxample.com")
            })),
            channel_size: 0,
            timeout: Some(60000),
            client_id: ClientId::default(),
        }
    }
}

impl EmailSettingsType for SMTPServiceSettings {
    fn from(&self) -> &Mailbox {
        &self.from
    }

    fn reply_to(&self) -> Option<&Mailbox> {
        self.reply_to.as_ref()
    }
}
