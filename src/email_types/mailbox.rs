use std::fmt::Display;

use serde::{Deserialize, Serialize};

use super::{Address, AddressError};
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Mailbox {
    pub name: Option<String>,
    pub email: Address,
}
impl Display for Mailbox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.name {
            Some(name) => write!(f, "{} <{}>", name, self.email),
            None => write!(f, "{}", self.email.as_ref()),
        }
    }
}
impl Serialize for Mailbox {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self.name {
            Some(name) => serializer.serialize_str(&format!("{} <{}>", name, self.email.as_ref())),
            None => serializer.serialize_str(self.email.as_ref()),
        }
    }
}
impl<'de> Deserialize<'de> for Mailbox {
    fn deserialize<D>(deserializer: D) -> Result<Mailbox, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Mailbox::try_from(s.as_str()).map_err(serde::de::Error::custom)?)
    }
}
impl Mailbox {
    pub fn new(name: Option<String>, email: Address) -> Self {
        Self { name, email }
    }
}
impl TryFrom<&str> for Mailbox {
    type Error = AddressError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // TODO Support Name <email> format
        Ok(Self {
            name: None,
            email: value.try_into()?,
        })
    }
}
