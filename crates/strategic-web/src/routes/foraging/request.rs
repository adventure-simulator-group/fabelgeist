//! Web-generated forage correlation and admission of exact receipt references.

use adventuresim_core::identity::CharacterId;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::spacetimedb::{SqlQuery, sql_string_literal};

const FORAGE_RECEIPT_REFERENCE_BYTES: usize = 64;

static NEXT_FORAGE_REQUEST: AtomicU64 = AtomicU64::new(1);

/// Exact hexadecimal request spelling used by the web receipt workflow.
///
/// This is correlation, not authorization: receipt reads still require the
/// selected character. Accepted uppercase spelling remains unchanged.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub(super) struct ForageReceiptReference(String);

impl ForageReceiptReference {
    pub(super) fn issue(character: CharacterId) -> Self {
        Self::from_entropy(
            character,
            ForageRequestSequence::next(),
            ForageRequestInstant::now(),
        )
    }

    fn from_entropy(
        character: CharacterId,
        sequence: ForageRequestSequence,
        instant: ForageRequestInstant,
    ) -> Self {
        Self(format!(
            "{:x}",
            Sha256::digest(
                [
                    b"forage-request-v1".as_slice(),
                    &u64::from(character).to_le_bytes(),
                    &sequence.0.to_le_bytes(),
                    &instant.0.to_le_bytes(),
                ]
                .concat()
            )
        ))
    }

    pub(super) fn query(&self, character: CharacterId) -> SqlQuery {
        SqlQuery::from(format!(
            "SELECT * FROM backend_forage_receipts WHERE character_id = {character} AND request_id = {}",
            sql_string_literal(&self.0)
        ))
    }
}

impl TryFrom<&str> for ForageReceiptReference {
    type Error = ForageReceiptReferenceError;

    fn try_from(reference: &str) -> std::result::Result<Self, Self::Error> {
        if reference.len() != FORAGE_RECEIPT_REFERENCE_BYTES {
            return Err(ForageReceiptReferenceError::IncorrectLength);
        }
        if !reference.as_bytes().iter().all(u8::is_ascii_hexdigit) {
            return Err(ForageReceiptReferenceError::NonHexadecimal);
        }
        Ok(Self(reference.to_owned()))
    }
}

impl std::fmt::Display for ForageReceiptReference {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ForageReceiptReferenceError {
    IncorrectLength,
    NonHexadecimal,
}

impl std::fmt::Display for ForageReceiptReferenceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::IncorrectLength => "A forage receipt reference must contain 64 bytes",
            Self::NonHexadecimal => {
                "A forage receipt reference must contain only hexadecimal digits"
            }
        })
    }
}
impl std::error::Error for ForageReceiptReferenceError {}

/// Process-local wrapping nonce; it has no ordering or elapsed-time meaning.
struct ForageRequestSequence(u64);
impl ForageRequestSequence {
    fn next() -> Self {
        Self(NEXT_FORAGE_REQUEST.fetch_add(1, Ordering::Relaxed))
    }
}

/// Exact native UNIX nanoseconds used only as request entropy.
struct ForageRequestInstant(u128);
impl ForageRequestInstant {
    fn now() -> Self {
        Self(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_entropy_keeps_exact_framing_and_native_widths() {
        for (character, sequence, instant, expected) in [
            (
                17,
                1,
                0,
                "b35626ab4d9733bac190dbc8bb129f6e2b470f1f85e49d466bad575e74c7fc86",
            ),
            (
                17,
                u64::MAX,
                u128::MAX,
                "4a672268d60560c02f3803678e4d2780b613f8a4e5469564408e8b7ba0f861d4",
            ),
            (
                0,
                0,
                42,
                "281d50810baedc7cacb0e816a9a43aa0a350fbee5de84098afcb312eabd99754",
            ),
        ] {
            let reference = ForageReceiptReference::from_entropy(
                character.into(),
                ForageRequestSequence(sequence),
                ForageRequestInstant(instant),
            );
            assert_eq!(reference.to_string(), expected);
            assert_eq!(
                serde_json::to_value(&reference).unwrap(),
                serde_json::json!(expected)
            );
            assert_eq!(ForageReceiptReference::try_from(expected), Ok(reference));
        }
    }

    #[test]
    fn receipt_admission_classifies_failures_and_preserves_spelling() {
        assert_eq!(
            ForageReceiptReference::try_from("../client-feedback"),
            Err(ForageReceiptReferenceError::IncorrectLength)
        );
        assert_eq!(
            ForageReceiptReference::try_from("g".repeat(64).as_str()),
            Err(ForageReceiptReferenceError::NonHexadecimal)
        );
        assert_eq!(
            ForageReceiptReference::try_from("é".repeat(32).as_str()),
            Err(ForageReceiptReferenceError::NonHexadecimal)
        );
        let spelling = "ABCDEF0123456789".repeat(4);
        let reference = ForageReceiptReference::try_from(spelling.as_str()).unwrap();
        assert_eq!(reference.to_string(), spelling);
        assert_eq!(
            reference.query(17.into()).to_string(),
            format!(
                "SELECT * FROM backend_forage_receipts WHERE character_id = 17 AND request_id = '{spelling}'"
            )
        );
    }
}
