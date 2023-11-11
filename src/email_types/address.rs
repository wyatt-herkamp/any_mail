use std::{
    fmt::{Display, Formatter},
    ops::Deref,
};

use serde::Serialize;
use thiserror::Error;
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Error)]
pub enum AddressError {
    #[error("Missing @")]
    MissingAt,
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Address {
    pub(crate) serialized: String,
    pub(crate) at_start: usize,
}

impl Address {
    pub unsafe fn new_unchecked(serialized: impl Into<String>) -> Self {
        let serialized = serialized.into();
        let at_start = serialized.find('@').unwrap();
        Self {
            serialized,
            at_start,
        }
    }
}
impl Display for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.serialized)
    }
}
impl Deref for Address {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.serialized
    }
}
impl AsRef<str> for Address {
    fn as_ref(&self) -> &str {
        &self.serialized
    }
}

impl Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.serialized)
    }
}

impl<'de> serde::Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Address, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Address::try_from(s.as_str()).map_err(serde::de::Error::custom)?)
    }
}
impl TryFrom<String> for Address {
    type Error = AddressError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let at_start = value.find('@').ok_or(AddressError::MissingAt)?;
        return Ok(Self {
            serialized: value.to_string(),
            at_start,
        });
    }
}
impl TryFrom<&str> for Address {
    type Error = AddressError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        return value.to_owned().try_into();
    }
}
impl TryFrom<(&str, &str)> for Address {
    type Error = AddressError;
    fn try_from((user, domain): (&str, &str)) -> Result<Self, Self::Error> {
        let mut serialized = String::with_capacity(user.len() + domain.len() + 1);
        serialized.push_str(user);
        serialized.push('@');
        serialized.push_str(domain);
        let at_start = user.len();
        Ok(Self {
            serialized,
            at_start,
        })
    }
}
