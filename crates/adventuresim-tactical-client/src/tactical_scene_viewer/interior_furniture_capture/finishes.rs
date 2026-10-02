//! Matched production specimens for the four authored wood treatments.
use super::*;
use adventuresim_building_generator::furniture::{FinishableFurnitureKind, FurnitureWoodState};

pub(super) const LABELS: [(&str, &str); FinishableFurnitureKind::ALL.len()] = [
    (
        "finish-dining-table",
        "Dining table: natural, handled, repaired, painted",
    ),
    ("finish-bench", "Bench: natural, handled, repaired, painted"),
    ("finish-chair", "Chair: natural, handled, repaired, painted"),
    (
        "finish-storage-chest",
        "Chest: natural, handled, repaired, painted",
    ),
    (
        "finish-workbench",
        "Workbench: natural, handled, repaired, painted",
    ),
];
const FINISH_INSTANCE_ID_BASE: u64 = 0x6669_6e69_7368_0000;

pub(super) fn stage(
    layout: &mut FurnitureLayout,
    terrain: &SceneTerrain,
) -> Vec<BuildingReviewCamera> {
    FinishableFurnitureKind::ALL
        .into_iter()
        .enumerate()
        .map(|(index, kind)| {
            let bay = Vec2::new(100.0, (index as f32 - 2.0) * CATALOG_BAY_SPACING_METRES);
            let key = FurnitureKey::natural(kind.kind(), FurnitureVariant::Compact);
            let size = key.interior_spec().unwrap().size_metres;
            let width = size.x * FurnitureWoodState::ALL.len() as f32
                + CATALOG_PAIR_GAP_METRES * (FurnitureWoodState::ALL.len() - 1) as f32;
            for (state_index, state) in FurnitureWoodState::ALL.into_iter().enumerate() {
                let key = FurnitureKey::wood(kind, FurnitureVariant::Compact, state);
                let ordinal = FurnitureKey::ALL
                    .iter()
                    .position(|entry| *entry == key)
                    .unwrap();
                let point = bay
                    + Vec2::X
                        * (-width * 0.5
                            + size.x * 0.5
                            + state_index as f32 * (size.x + CATALOG_PAIR_GAP_METRES));
                layout.instances.push(GeneratedFurniture {
                    scene: SceneFurniture {
                        id: FurnitureInstanceId(FINISH_INSTANCE_ID_BASE + ordinal as u64),
                        key,
                        location: FurnitureLocation::Interior {
                            building_id: 0,
                            room_id: index as u16,
                            storey: 0,
                        },
                    },
                    position_metres: Vec3::new(point.x, terrain.height_at(point).unwrap(), point.y),
                    orientation: BuildingOrientation::from_radians(std::f32::consts::PI).unwrap(),
                });
            }
            catalog_camera(
                kind.kind(),
                Vec3::new(width, size.y, size.z),
                Vec3::new(bay.x, terrain.height_at(bay).unwrap(), bay.y),
            )
        })
        .collect()
}
