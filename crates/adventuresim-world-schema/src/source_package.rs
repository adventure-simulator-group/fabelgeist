//! Checked identity of an immutable imported source package.
//!
//! Shared geographic producers and consumers use this authority without
//! depending on tactical scene generation. Geometry content and placement
//! bindings have separate identities.
use serde::{Deserialize, Serialize};
use std::fmt;

const SHA256_HEX_BYTES: usize = 64;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct SourcePackageDigest(String);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourcePackageDigestError;

impl SourcePackageDigest {
    pub fn from_hex(value: &str) -> Result<Self, SourcePackageDigestError> {
        if value.len() != SHA256_HEX_BYTES
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(SourcePackageDigestError);
        }
        Ok(Self(value.into()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SourcePackageDigest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl<'de> Deserialize<'de> for SourcePackageDigest {
    fn deserialize<D: serde::Deserializer<'de>>(decoder: D) -> Result<Self, D::Error> {
        Self::from_hex(&String::deserialize(decoder)?).map_err(serde::de::Error::custom)
    }
}

impl fmt::Display for SourcePackageDigestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("imported source identity must be a lowercase SHA-256 digest")
    }
}

impl std::error::Error for SourcePackageDigestError {}
