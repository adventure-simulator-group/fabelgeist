//! Authored use and upkeep variation, applied after room access is established.
use super::InteriorPlacement;
use crate::furniture::{FinishableFurnitureKind, FurnitureKey, FurnitureWoodState};
use crate::{BuildingPlan, BuildingProgram, RoomKind};
use std::collections::BTreeMap;

const FINISH_DOMAIN: u64 = 0x6675_726e_6973_6801;

pub(super) fn assign(
    plan: &BuildingPlan,
    program: &BuildingProgram,
    placements: &mut [InteriorPlacement],
) {
    let mut occurrences = BTreeMap::<_, u64>::new();
    for placement in placements {
        let Some(kind) = FinishableFurnitureKind::from_kind(placement.key.kind()) else {
            continue;
        };
        let occurrence = occurrences
            .entry((placement.storey, placement.room_id, placement.key.kind()))
            .or_default();
        let seed = fabelgeist_determinism::mix64(
            program.seed
                ^ FINISH_DOMAIN
                ^ (u64::from(placement.storey) << 48)
                ^ (u64::from(placement.room_id) << 32)
                ^ ((placement.key.kind() as u64) << 16)
                ^ *occurrence,
        );
        *occurrence += 1;
        let room = plan
            .storeys
            .iter()
            .find(|storey| storey.level == placement.storey)
            .and_then(|storey| {
                storey
                    .rooms
                    .iter()
                    .find(|room| room.id == placement.room_id)
            })
            .expect("accepted placement belongs to a room");
        let states = if room.kind == RoomKind::Workshop {
            [
                FurnitureWoodState::Natural,
                FurnitureWoodState::Handled,
                FurnitureWoodState::Handled,
                FurnitureWoodState::Repaired,
            ]
        } else if matches!(room.kind, RoomKind::Storage | RoomKind::Guardroom) {
            [
                FurnitureWoodState::Natural,
                FurnitureWoodState::Natural,
                FurnitureWoodState::Handled,
                FurnitureWoodState::Repaired,
            ]
        } else {
            FurnitureWoodState::ALL
        };
        placement.key = FurnitureKey::wood(
            kind,
            placement.key.variant(),
            states[seed as usize % states.len()],
        );
    }
}
