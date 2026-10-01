use super::Character;
use spacetimedb::Identity;

impl Character {
    /// Assignment is owned by the indexed server identity. A stale assignment
    /// is reclaimed only after checking that the previous server is absent.
    pub(crate) fn has_tactical_server_assignment(&self) -> bool {
        self.server != Identity::ZERO
    }
}
