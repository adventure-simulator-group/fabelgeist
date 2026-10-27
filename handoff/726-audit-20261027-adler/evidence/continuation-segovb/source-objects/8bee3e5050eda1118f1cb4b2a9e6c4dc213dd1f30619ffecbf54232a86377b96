//! Authored use and upkeep variation, applied after room access is established.
use super::InteriorPlacement;
use crate::furniture::{FinishableFurnitureKind, FurnitureKey, FurnitureWoodState};
use crate::{BuildingPlan, BuildingProgram, RoomKind};

const FINISH_DOMAIN: fabelgeist_determinism::StreamId =
    fabelgeist_determinism::StreamId::new("building.furniture-finish");

pub(super) fn assign(
    plan: &BuildingPlan,
    program: &BuildingProgram,
    placements: &mut [InteriorPlacement],
) -> Result<(), super::InteriorLayoutError> {
    for placement in placements {
        let Some(kind) = FinishableFurnitureKind::from_kind(placement.key.kind()) else {
            continue;
        };
        // Spatial placement identifies furniture independently of traversal order.
        let mut random = FINISH_DOMAIN.rng(
            program.seed,
            &[
                placement.storey.index() as u64,
                u64::from(placement.room_id.serialized_ordinal()),
                placement.key.kind() as u64,
                placement.key.variant() as u64,
                u64::from(placement.centre_metres.metres().x.to_bits()),
                u64::from(placement.centre_metres.metres().y.to_bits()),
                placement.facing as u64,
            ],
        );
        let room = plan
            .storeys
            .iter()
            .find(|storey| crate::StoreyIndex::from_serialized(storey.level) == placement.storey)
            .and_then(|storey| {
                storey
                    .rooms
                    .iter()
                    .find(|room| crate::RoomIndex::from_serialized(room.id) == placement.room_id)
            })
            .ok_or(super::InteriorLayoutError::MissingRoom {
                storey: placement.storey,
                room: placement.room_id,
            })?;
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
            states[random.index(states.len())],
        );
    }
    Ok(())
}
