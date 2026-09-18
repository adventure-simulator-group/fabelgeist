//! Opaque strategic encounter identities and retry receipt validation.
use super::*;

/// Observer-safe, deterministic identity for a strategic encounter. The two
/// independently domain-separated words retain no readable journey shape and
/// provide 128 bits for durable action-receipt identity.
pub fn opaque_strategic_encounter_id(seed: u64, roll_index: u64) -> String {
    let high = StreamId::new("encounter.identity-high")
        .rng(seed, &[roll_index])
        .next_u64();
    let low = StreamId::new("encounter.identity-low")
        .rng(seed, &[roll_index])
        .next_u64();
    format!("enc:{high:016x}{low:016x}")
}

#[expect(
    clippy::too_many_arguments,
    reason = "retry validation compares every supplied receipt field explicitly"
)]
pub fn strategic_encounter_retry_matches(
    receipt_encounter_id: &str,
    receipt_character_id: u64,
    receipt_choice: &str,
    receipt_expected_revision: u32,
    encounter_id: &str,
    character_id: u64,
    choice: &str,
    expected_revision: u32,
) -> bool {
    receipt_encounter_id == encounter_id
        && receipt_character_id == character_id
        && receipt_choice == choice
        && receipt_expected_revision == expected_revision
}
