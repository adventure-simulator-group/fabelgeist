//! Stable terminal-shoot identity derived before native geometry evaluation.
use fabelgeist_determinism::{Seed, StreamId};

const SHOOT_IDENTITY: StreamId = StreamId::new("visual.tree.shoot-identity");

/// Identifies one source shoot across leaf ordering and native float rounding.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(in crate::presentation) struct TreeShootId(u64);

impl TreeShootId {
    pub(in crate::presentation) fn from_seed(source: Seed) -> Self {
        Self(SHOOT_IDENTITY.seed(source, &[]).to_u64())
    }

    /// Adapt the identity to the deterministic sampler's native seed boundary.
    pub(in crate::presentation) fn seed(self) -> Seed {
        Seed::from_u64(self.0)
    }
}
