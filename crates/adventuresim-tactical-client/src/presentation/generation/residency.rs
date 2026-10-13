//! Mutable bindings belong to the current preparation ticket.
use super::*;

impl PreparedProducts {
    pub(super) fn begin_generation(&mut self) {
        self.facades.clear();
        self.placements.clear();
    }
}
