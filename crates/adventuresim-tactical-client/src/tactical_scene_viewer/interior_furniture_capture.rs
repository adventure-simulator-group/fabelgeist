//! Catalog staging and route-based cameras; all furniture uses production ECS presentation.
use adventuresim_building_generator::furniture::{FurnitureKey, FurnitureKind, FurnitureVariant};
use adventuresim_tactical_core::prelude::*;
use adventuresim_world_schema::settlement_buildings::BuildingUse;
use bevy::prelude::*;

use super::{
    capture_state::BuildingReviewCamera,
    view_specs::{CapturePose, CaptureViewSpec},
};

mod rooms;

pub(super) const PROFILE: &str = "interior-furniture-catalog";
pub(super) const ROOMS_PROFILE: &str = "furnished-room-review";
const CATALOG_COLUMNS: usize = 5;
const CATALOG_BAY_SPACING_METRES: f32 = 20.0;
const CATALOG_PAIR_GAP_METRES: f32 = 0.8;
const CATALOG_INSTANCE_ID_BASE: u64 = 0x6361_7461_6c6f_6700;
const CAMERA_EYE_HEIGHT_METRES: f32 = 1.55;
const CATALOG_FOV_DEGREES: f32 = 48.0;
const ROOM_FOV_DEGREES: f32 = 78.0;
const CATALOG_FRAMING_MARGIN: f32 = 1.1;

const CATALOG: [(&str, &str); 34] = [
    ("dining-table", "Trestle dining tables"),
    ("bench", "Joined benches"),
    ("chair", "Joined chairs"),
    ("stool", "Stools"),
    ("bed", "Domestic beds"),
    ("bunk-bed", "Bunk beds and integral ladders"),
    ("storage-chest", "Iron-bound chests"),
    ("cupboard", "Panelled cupboards"),
    ("shelving", "Empty shelving"),
    ("writing-desk", "Sloped writing desks"),
    ("lectern", "Lecterns"),
    ("church-bench", "Church benches"),
    ("altar", "Plain altars"),
    ("ward-bed", "Open-frame ward beds"),
    ("bath-tub", "Open stave bathing tubs"),
    ("wash-stand", "Wash stands"),
    ("workbench", "Workbenches"),
    ("cutting-table", "Cutting tables"),
    ("tool-rack", "Empty tool racks"),
    ("weapon-rack", "Empty weapon racks"),
    ("armour-stand", "Armour stands"),
    ("grain-bin", "Empty grain bins"),
    ("storage-crate", "Storage crates"),
    ("counter", "Modular counters"),
    ("counter-left-end", "Left counter endcaps"),
    ("counter-right-end", "Right counter endcaps"),
    ("counter-corner", "Corner counters"),
    ("display-counter", "Empty display counters"),
    ("drying-rack", "Drying racks"),
    ("kneading-trough", "Kneading troughs"),
    ("butchers-block", "Butchers' blocks"),
    ("cask-rack", "Empty cask racks"),
    ("hay-rack", "Empty hay racks"),
    ("feed-trough", "Empty feed troughs"),
];

const ROOM_USES: [BuildingUse; 11] = [
    BuildingUse::Dwelling,
    BuildingUse::Inn,
    BuildingUse::GeneralShop,
    BuildingUse::Smithy,
    BuildingUse::ParishChurch,
    BuildingUse::Hospital,
    BuildingUse::Guardhouse,
    BuildingUse::Warehouse,
    BuildingUse::Castle,
    BuildingUse::Dwelling,
    BuildingUse::Cathedral,
];
const ROOM_LABELS: [(&str, &str); 11] = [
    ("house", "Furnished dwelling"),
    ("inn", "Furnished inn"),
    ("shop", "Furnished shop"),
    ("workshop", "Furnished workshop"),
    ("church", "Furnished parish church"),
    ("hospital", "Furnished hospital ward"),
    ("barracks", "Furnished guardhouse"),
    ("warehouse", "Furnished warehouse"),
    ("keep-upper", "Keep upper floor and spiral landing"),
    ("bedroom", "Dwelling upper bedchamber"),
    ("cathedral", "Furnished cathedral nave"),
];

pub(super) const VIEWS: [CaptureViewSpec; 36] = catalog_views();
pub(super) const ROOM_VIEWS: [CaptureViewSpec; 12] = room_views();

const fn catalog_views() -> [CaptureViewSpec; 36] {
    let warmup = CaptureViewSpec::new(
        "warmup",
        "Production furniture catalog warmup",
        CapturePose::CityExterior { camera: 0 },
        CATALOG_FOV_DEGREES,
        100,
    )
    .warmup()
    .vista();
    let mut views = [warmup; 36];
    let mut index = 0;
    while index < CATALOG.len() {
        views[index + 1] = CaptureViewSpec::new(
            CATALOG[index].0,
            CATALOG[index].1,
            CapturePose::CityExterior {
                camera: index as u8,
            },
            CATALOG_FOV_DEGREES,
            100,
        )
        .vista()
        .settled_readback_pair();
        index += 1;
    }
    views[35] = CaptureViewSpec::new(
        "joined-counters",
        "Contiguous counter runs with corner and endcaps",
        CapturePose::CityExterior { camera: 34 },
        CATALOG_FOV_DEGREES,
        100,
    )
    .vista()
    .settled_readback_pair();
    views
}

const fn room_views() -> [CaptureViewSpec; 12] {
    let warmup = CaptureViewSpec::new(
        "warmup",
        "Production furnished room warmup",
        CapturePose::BuildingInterior { camera: 0 },
        ROOM_FOV_DEGREES,
        100,
    )
    .warmup();
    let mut views = [warmup; 12];
    let mut index = 0;
    while index < ROOM_LABELS.len() {
        views[index + 1] = CaptureViewSpec::new(
            ROOM_LABELS[index].0,
            ROOM_LABELS[index].1,
            CapturePose::BuildingInterior {
                camera: index as u8,
            },
            ROOM_FOV_DEGREES,
            100,
        )
        .settled_readback_pair();
        index += 1;
    }
    views
}

pub(super) fn is_profile(profile: &str) -> bool {
    matches!(profile, PROFILE | ROOMS_PROFILE)
}

pub(super) fn setup_catalog(
    commands: &mut Commands,
    layout: &mut FurnitureLayout,
    terrain: &SceneTerrain,
    profile: &str,
    output: &std::path::Path,
) -> Option<Vec<BuildingReviewCamera>> {
    if profile != PROFILE {
        return None;
    }
    assert!(
        layout.instances.is_empty(),
        "catalog fixture must have no placed furniture"
    );
    commands.insert_resource(crate::presentation::InteriorFurnitureExhibition);
    let mut cameras = Vec::new();
    for (index, kind) in FurnitureKind::INTERIOR.into_iter().enumerate() {
        let bay = Vec2::new(
            (index % CATALOG_COLUMNS) as f32 - 2.0,
            (index / CATALOG_COLUMNS) as f32 - 3.0,
        ) * CATALOG_BAY_SPACING_METRES;
        let keys = FurnitureVariant::ALL.map(|variant| FurnitureKey { kind, variant });
        let sizes = keys.map(|key| key.interior_spec().unwrap().size_metres);
        let pair_width = sizes[0].x + sizes[1].x + catalog_pair_gap(kind);
        for (variant, key) in keys.into_iter().enumerate() {
            let x = if variant == 0 {
                -pair_width * 0.5 + sizes[0].x * 0.5
            } else {
                pair_width * 0.5 - sizes[1].x * 0.5
            };
            let point = bay + Vec2::X * x;
            layout.instances.push(GeneratedFurniture {
                scene: SceneFurniture {
                    id: FurnitureInstanceId(
                        CATALOG_INSTANCE_ID_BASE + (index * 2 + variant) as u64,
                    ),
                    key,
                    // Specimens have no real building, but use the same interior location payload.
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
        let envelope = Vec3::new(
            pair_width,
            sizes[0].y.max(sizes[1].y),
            sizes[0].z.max(sizes[1].z),
        );
        let ground = Vec3::new(bay.x, terrain.height_at(bay).unwrap(), bay.y);
        cameras.push(catalog_camera(kind, envelope, ground));
    }
    cameras.push(joined_counters(layout, terrain));
    expect_furniture(commands, layout);
    std::fs::write(output.join("interior-catalog.json"), serde_json::to_vec_pretty(
            &serde_json::json!({ "left_variant": "Compact", "right_variant": "Broad",
            "instances": layout.instances, "note": "Production furniture models staged as catalog specimens; building_id zero is reserved for these captures." })).unwrap())
        .expect("write catalog specimen evidence");
    Some(cameras)
}

fn catalog_camera(kind: FurnitureKind, size: Vec3, ground: Vec3) -> BuildingReviewCamera {
    let side = if kind == FurnitureKind::BunkBed {
        1.1
    } else {
        0.24
    };
    let outward = Vec3::new(side, 0.5, 1.0).normalize();
    let right = Vec3::Y.cross(outward).normalize();
    let up = outward.cross(right);
    let tangent_y = (CATALOG_FOV_DEGREES.to_radians() * 0.5).tan();
    let tangent_x = tangent_y * super::VIEW_WIDTH as f32 / super::VIEW_HEIGHT as f32;
    let target = ground + Vec3::Y * size.y * 0.5;
    // Fit every envelope corner in the actual perspective frustum, including tall beds' feet.
    let distance = envelope_corners(size)
        .into_iter()
        .map(|corner| {
            let point = corner - Vec3::Y * size.y * 0.5;
            point.dot(outward)
                + CATALOG_FRAMING_MARGIN
                    * (point.dot(right).abs() / tangent_x).max(point.dot(up).abs() / tangent_y)
        })
        .fold(0.0_f32, f32::max);
    BuildingReviewCamera {
        position: target + outward * distance,
        target,
        plaster_raking_light: None,
    }
}

fn catalog_pair_gap(kind: FurnitureKind) -> f32 {
    // The side-on ladder proof needs room between the deep bed frames in projection.
    if kind == FurnitureKind::BunkBed {
        2.4
    } else {
        CATALOG_PAIR_GAP_METRES
    }
}

fn envelope_corners(size: Vec3) -> [Vec3; 8] {
    std::array::from_fn(|index| {
        Vec3::new(
            if index & 1 == 0 {
                -size.x * 0.5
            } else {
                size.x * 0.5
            },
            if index & 2 == 0 { 0.0 } else { size.y },
            if index & 4 == 0 {
                -size.z * 0.5
            } else {
                size.z * 0.5
            },
        )
    })
}

fn joined_counters(layout: &mut FurnitureLayout, terrain: &SceneTerrain) -> BuildingReviewCamera {
    use FurnitureKind::*;
    let variant = FurnitureVariant::Broad;
    let size = FurnitureKey {
        kind: Counter,
        variant,
    }
    .interior_spec()
    .unwrap()
    .size_metres;
    let corner = FurnitureKey {
        kind: CounterCorner,
        variant,
    }
    .interior_spec()
    .unwrap()
    .size_metres;
    let first = (corner.x + size.x) * 0.5;
    let second = first + size.x;
    let bay = Vec2::new(80.0, 60.0);
    let assembly = [
        (CounterCorner, Vec2::ZERO, 0.0),
        (Counter, Vec2::new(first, 0.0), 0.0),
        (CounterRightEnd, Vec2::new(second, 0.0), 0.0),
        (Counter, Vec2::new(0.0, first), std::f32::consts::FRAC_PI_2),
        (
            CounterLeftEnd,
            Vec2::new(0.0, second),
            std::f32::consts::FRAC_PI_2,
        ),
    ];
    let ground = terrain.height_at(bay).unwrap();
    for (index, (kind, local, yaw)) in assembly.into_iter().enumerate() {
        let point = bay - local;
        layout.instances.push(GeneratedFurniture {
            scene: SceneFurniture {
                id: FurnitureInstanceId(CATALOG_INSTANCE_ID_BASE + 68 + index as u64),
                key: FurnitureKey { kind, variant },
                location: FurnitureLocation::Interior {
                    building_id: 0,
                    room_id: 34,
                    storey: 0,
                },
            },
            position_metres: Vec3::new(point.x, ground, point.y),
            orientation: BuildingOrientation::from_radians(yaw + std::f32::consts::PI).unwrap(),
        });
    }
    let target = Vec3::new(bay.x - first, ground + size.y * 0.5, bay.y - first);
    BuildingReviewCamera {
        position: target + Vec3::new(5.5, 4.5, 6.0),
        target,
        plaster_raking_light: None,
    }
}

pub(super) fn setup_rooms(
    commands: &mut Commands,
    buildings: &[GeneratedBuilding],
    layout: &FurnitureLayout,
    profile: &str,
    output: &std::path::Path,
) -> Option<Vec<BuildingReviewCamera>> {
    if profile != ROOMS_PROFILE {
        return None;
    }
    super::building_review::setup_geometry_requirements(commands, buildings, output);
    let cameras = ROOM_USES
        .into_iter()
        .enumerate()
        .map(|(index, usage)| {
            let building = buildings
                .iter()
                .find(|building| building.placement.program.usage == Some(usage))
                .unwrap_or_else(|| panic!("furnished room fixture lacks {usage:?}"));
            use adventuresim_building_generator::RoomKind;
            let selection = match index {
                0 | 1 => rooms::RoomSelection::Role(RoomKind::CommonRoom),
                2 => rooms::RoomSelection::Role(RoomKind::Shop),
                3 => rooms::RoomSelection::Role(RoomKind::Workshop),
                4 | 10 => rooms::RoomSelection::Role(RoomKind::Nave),
                5 => rooms::RoomSelection::Role(RoomKind::Ward),
                6 => rooms::RoomSelection::Role(RoomKind::Guardroom),
                7 => rooms::RoomSelection::Role(RoomKind::Storage),
                8 => rooms::RoomSelection::UpperKeep,
                9 => rooms::RoomSelection::Bedroom,
                _ => unreachable!("every authored room view has a semantic selection"),
            };
            rooms::camera(building, layout, selection)
        })
        .collect();
    expect_furniture(commands, layout);
    std::fs::write(
        output.join("interior-access-proofs.json"),
        serde_json::to_vec_pretty(&layout.interiors).unwrap(),
    )
    .expect("write room access proofs");
    Some(cameras)
}

fn expect_furniture(commands: &mut Commands, layout: &FurnitureLayout) {
    commands.insert_resource(super::furniture_readiness::ExpectedFurniture {
        instances: layout.instances.len() + layout.distant_instances.len(),
        batches: layout
            .instances
            .iter()
            .chain(&layout.distant_instances)
            .map(|instance| instance.scene.key.recipe().meshes.len())
            .sum(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_framing_contains_every_variant_envelope_corner() {
        for kind in FurnitureKind::INTERIOR {
            let sizes = FurnitureVariant::ALL.map(|variant| {
                FurnitureKey { kind, variant }
                    .interior_spec()
                    .unwrap()
                    .size_metres
            });
            let size = Vec3::new(
                sizes[0].x + sizes[1].x + catalog_pair_gap(kind),
                sizes[0].y.max(sizes[1].y),
                sizes[0].z.max(sizes[1].z),
            );
            let camera = catalog_camera(kind, size, Vec3::ZERO);
            let view =
                Transform::from_translation(camera.position).looking_at(camera.target, Vec3::Y);
            let tangent = (CATALOG_FOV_DEGREES.to_radians() * 0.5).tan();
            for corner in envelope_corners(size) {
                let local = view.rotation.inverse() * (corner - camera.position);
                assert!(local.z < 0.0, "{kind:?} behind camera");
                assert!(
                    local.y.abs() < -local.z * tangent,
                    "{kind:?} vertical clipping"
                );
                assert!(
                    local.x.abs()
                        < -local.z * tangent * super::super::VIEW_WIDTH as f32
                            / super::super::VIEW_HEIGHT as f32,
                    "{kind:?} horizontal clipping"
                );
            }
        }
    }

    #[test]
    fn every_interior_recipe_family_has_a_unique_catalog_plate() {
        assert_eq!(CATALOG.len(), FurnitureKind::INTERIOR.len());
        let slugs = VIEWS
            .iter()
            .map(|view| view.slug)
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(slugs.len(), VIEWS.len());
        for (index, view) in VIEWS.iter().skip(1).enumerate() {
            assert_eq!(
                view.pose,
                CapturePose::CityExterior {
                    camera: index as u8
                }
            );
            assert!(!view.warmup);
        }
        assert!(VIEWS[0].warmup && ROOM_VIEWS[0].warmup);
        assert!(
            VIEWS
                .iter()
                .chain(ROOM_VIEWS.iter())
                .filter(|view| !view.warmup)
                .all(|view| view.verify_settled_readbacks),
            "every recorded furniture review must verify its final settled pixel pair"
        );
    }
}
