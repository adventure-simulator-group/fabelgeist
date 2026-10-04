//! Per-actor identity of an independent forage attempt.

use crate::strategic_action::SnapshotRevision;

/// A forage cursor advances once after a new attempt is committed. It never
/// wraps; replaying an immutable receipt does not advance it.
///
/// Snapshot revisions and attempt generations have separate roles:
///
/// ```compile_fail
/// use adventuresim_core::{
///     foraging::ForageAttemptGeneration, strategic_action::SnapshotRevision,
/// };
/// ForageAttemptGeneration::INITIAL.advance_from(SnapshotRevision(0));
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct ForageAttemptGeneration(u64);

impl ForageAttemptGeneration {
    pub const INITIAL: Self = Self(0);

    /// Admit the expected cursor before computing the successor. Stale input
    /// takes precedence over exhaustion, including at the maximum encoding.
    pub fn advance_from(self, expected: Self) -> Result<Self, ForageGenerationError> {
        if self != expected {
            return Err(ForageGenerationError::Stale {
                submitted: self,
                expected,
            });
        }
        self.0
            .checked_add(1)
            .map(Self)
            .ok_or(ForageGenerationError::Exhausted { generation: self })
    }

    /// The attempt identity is also the forage plan's snapshot revision.
    pub const fn snapshot_revision(self) -> SnapshotRevision {
        SnapshotRevision(self.0)
    }

    /// Exact fixed-width representation used by the authority digest.
    pub const fn to_le_bytes(self) -> [u8; 8] {
        self.0.to_le_bytes()
    }
}

impl From<u64> for ForageAttemptGeneration {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<ForageAttemptGeneration> for u64 {
    fn from(value: ForageAttemptGeneration) -> Self {
        value.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ForageGenerationError {
    Stale {
        submitted: ForageAttemptGeneration,
        expected: ForageAttemptGeneration,
    },
    Exhausted {
        generation: ForageAttemptGeneration,
    },
}

impl std::fmt::Display for ForageGenerationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Stale { .. } => "Forage attempt generation is stale",
            Self::Exhausted { .. } => "Forage attempt generation is exhausted",
        })
    }
}

impl std::error::Error for ForageGenerationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn successor_never_wraps_and_stale_input_precedes_exhaustion() {
        assert_eq!(
            ForageAttemptGeneration::INITIAL
                .advance_from(ForageAttemptGeneration::INITIAL)
                .unwrap(),
            ForageAttemptGeneration::from(1),
        );
        let last = ForageAttemptGeneration::from(u64::MAX);
        assert_eq!(
            ForageAttemptGeneration::from(u64::MAX - 1)
                .advance_from(ForageAttemptGeneration::from(u64::MAX - 1))
                .unwrap(),
            last,
        );
        assert_eq!(
            last.advance_from(last),
            Err(ForageGenerationError::Exhausted { generation: last }),
        );
        assert_eq!(
            last.advance_from(ForageAttemptGeneration::INITIAL),
            Err(ForageGenerationError::Stale {
                submitted: last,
                expected: ForageAttemptGeneration::INITIAL,
            }),
        );
        assert_eq!(
            ForageGenerationError::Exhausted { generation: last }.to_string(),
            "Forage attempt generation is exhausted",
        );
        assert_eq!(
            ForageGenerationError::Stale {
                submitted: last,
                expected: ForageAttemptGeneration::INITIAL,
            }
            .to_string(),
            "Forage attempt generation is stale",
        );
    }

    #[test]
    fn cursor_wire_words_digest_bytes_and_snapshot_identity_are_preserved() {
        for encoded in [0, 1, u64::MAX - 1, u64::MAX] {
            let generation = ForageAttemptGeneration::from(encoded);
            assert_eq!(u64::from(generation), encoded);
            assert_eq!(generation.snapshot_revision(), SnapshotRevision(encoded));
            let serialized = serde_json::to_string(&generation).unwrap();
            assert_eq!(serialized, encoded.to_string());
            assert_eq!(
                serde_json::from_str::<ForageAttemptGeneration>(&serialized).unwrap(),
                generation,
            );
        }
        assert_eq!(
            ForageAttemptGeneration::from(0x0123_4567_89ab_cdef).to_le_bytes(),
            [0xef, 0xcd, 0xab, 0x89, 0x67, 0x45, 0x23, 0x01],
        );
        assert!(serde_json::from_str::<ForageAttemptGeneration>("-1").is_err());
        assert!(serde_json::from_str::<ForageAttemptGeneration>("18446744073709551616").is_err());
    }
}
