//! Persistent per-actor forage cursor admission.

use adventuresim_core::{
    foraging::{ForageAttemptGeneration, ForageGenerationError},
    identity::CharacterId,
};
use spacetimedb::{ReducerContext, table};

/// Per-actor cursor used by the gateway to mint the next independent attempt.
#[derive(Clone, Debug)]
#[table(accessor = forage_attempt_state)]
pub struct ForageAttemptState {
    #[primary_key]
    pub character_id: u64,
    #[index(btree)]
    pub gateway_bucket: u8,
    pub next_generation: u64,
}

/// Read and admit the stored cursor without changing it. Only a committed new
/// attempt writes the successor; immutable receipt replay bypasses this read.
pub(super) fn next_attempt_generation(
    ctx: &ReducerContext,
    actor: CharacterId,
    submitted: ForageAttemptGeneration,
) -> std::result::Result<ForageAttemptGeneration, ForageGenerationError> {
    let expected = match ctx
        .db
        .forage_attempt_state()
        .character_id()
        .find(u64::from(actor))
    {
        Some(state) => ForageAttemptGeneration::from(state.next_generation),
        None => ForageAttemptGeneration::INITIAL,
    };
    submitted.advance_from(expected)
}
