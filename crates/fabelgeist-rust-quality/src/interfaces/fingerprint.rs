//! Stable token fingerprints force another review when an opaque API changes.
use std::fmt;

use quote::ToTokens;
use sha2::{Digest, Sha256};

pub(super) struct OpaqueInterfaceFingerprint([u8; 32]);

impl OpaqueInterfaceFingerprint {
    pub(super) fn from_tokens(tokens: &impl ToTokens) -> Self {
        Self(Sha256::digest(tokens.to_token_stream().to_string().as_bytes()).into())
    }
}

impl fmt::Display for OpaqueInterfaceFingerprint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}
