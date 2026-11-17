use super::Character;
use spacetimedb::Identity;

impl Character {
    pub(crate) fn assign_tactical_server(&mut self, server: Identity) -> Result<(), String> {
        if server == Identity::ZERO {
            return Err("Tactical enrollment requires a serving identity".into());
        }
        if self.server != server {
            self.tactical_admission = self
                .tactical_admission
                .successor()
                .map_err(|error| error.to_string())?;
        }
        self.server = server;
        Ok(())
    }

    pub(crate) fn clear_tactical_server(&mut self) {
        self.server = Identity::ZERO;
    }

    /// Assignment is owned by the indexed server identity. A stale assignment
    /// is reclaimed only after checking that the previous server is absent.
    pub(crate) fn has_tactical_server_assignment(&self) -> bool {
        self.server != Identity::ZERO
    }
}
