//! The scene host reads only its immutable selected-area resident projection.

use super::*;
use adventuresim_tactical_core::player::CharacterId;

impl SpacetimeDb {
    pub(crate) fn resident_candidates(&self) -> Vec<TacticalResidentCandidate> {
        let mut residents: Vec<_> = self.conn.db.scene_resident_candidates().iter().collect();
        residents.sort_by_key(|resident| resident.character_id);
        residents
    }

    pub(crate) fn configure_residents(
        &self,
        area: TacticalResidentArea,
    ) -> spacetimedb_sdk::Result<()> {
        info!(operators = ?area.operator_character_ids, venues = ?area.common_venues, "Requesting selected-area resident roster");
        let mailbox = self.scene_operations.clone();
        self.reducers()
            .configure_settlement_residents_then(area, move |_, result| {
                scene_operations::record(
                    &mailbox,
                    scene_operations::SceneOperation::ResidentConfiguration,
                    result,
                );
            })
    }

    pub(crate) fn request_resident(&self, character: CharacterId) -> spacetimedb_sdk::Result<()> {
        let mailbox = self.scene_operations.clone();
        self.reducers()
            .enter_settlement_resident_then(character.0, move |_, result| {
                scene_operations::record(
                    &mailbox,
                    scene_operations::SceneOperation::ResidentAdmission { character },
                    result,
                );
            })
    }
}
