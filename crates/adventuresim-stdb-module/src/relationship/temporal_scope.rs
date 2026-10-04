//! Chronology policy with the participants required by each scope.
//!
//! ```compile_fail
//! use adventuresim_core::identity::CharacterId;
//! use adventuresim_stdb_module::relationship::TemporalScope;
//! let scope = TemporalScope::ExclusiveShared { actor: CharacterId::from(7) };
//! ```

use super::*;
use adventuresim_core::identity::CharacterId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TemporalScope {
    ActorLocal {
        actor: CharacterId,
    },
    PairwiseSoft {
        actor: CharacterId,
    },
    Institutional {
        actor: CharacterId,
    },
    NpcCanonical {
        actor: CharacterId,
        target: CharacterId,
    },
    ExclusiveShared {
        actor: CharacterId,
        participant: CharacterId,
    },
}

impl TemporalScope {
    fn actor(self) -> CharacterId {
        match self {
            Self::ActorLocal { actor }
            | Self::PairwiseSoft { actor }
            | Self::Institutional { actor }
            | Self::NpcCanonical { actor, .. }
            | Self::ExclusiveShared { actor, .. } => actor,
        }
    }

    /// Soft scopes inspect only the actor's frontier. Exclusive commitments
    /// use canonical world time after admitting the other participant's row.
    pub(crate) fn enforce(
        self,
        ctx: &ReducerContext,
    ) -> Result<StrategicMinute, TemporalScopeError> {
        let actor_minute = canonical_now(ctx, self.actor())?;
        match self {
            Self::ActorLocal { .. } | Self::PairwiseSoft { .. } | Self::Institutional { .. } => {
                Ok(actor_minute)
            }
            Self::NpcCanonical { actor, target } => {
                if ctx
                    .db
                    .npc_policy()
                    .character_id()
                    .find(u64::from(target))
                    .is_none()
                {
                    return Err(TemporalScopeError::TargetNotNpcPolicy { actor, target });
                }
                Ok(actor_minute)
            }
            Self::ExclusiveShared { actor, participant } => {
                if ctx
                    .db
                    .character()
                    .id()
                    .find(u64::from(participant))
                    .is_none()
                {
                    return Err(TemporalScopeError::MissingParticipant { actor, participant });
                }
                Ok(crate::time::refresh_clock(ctx)?)
            }
        }
    }
}
