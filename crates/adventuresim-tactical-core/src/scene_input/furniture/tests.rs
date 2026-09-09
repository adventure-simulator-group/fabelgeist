use super::*;
use crate::{
    city_layout::{CityStreetPatch, CityStreetSurface},
    scene::{GroundCover, GroundSurface},
    scene_input::{EnvironmentalSample, TacticalBuildingPlacement, TerrainSampleGrid},
};
use adventuresim_building_generator::{
    BuildingProgram, ServiceBuildingSize,
    furniture::{FurnitureKind, FurnitureVariant},
    settlement_archetype,
};
use adventuresim_world_schema::settlement_buildings::BuildingUse;
use avian3d::prelude::Rotation;

mod city_acceptance;

fn fixture() -> (
    TacticalSceneInput,
    Vec<GeneratedBuilding>,
    SceneTerrain,
    SceneGround,
) {
    let mut input = TacticalSceneInput::load(std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/tactical-scenes/flat-dry-grassland.json"
    )))
    .unwrap();
    input.playable = TerrainSampleGrid {
        width: 81,
        depth: 81,
        spacing_metres: 2.0,
        heights_metres: vec![0.0; 81 * 81],
        environment: vec![EnvironmentalSample::default(); 81 * 81],
    };
    input.streets = vec![CityStreetPatch::Market {
        corners_metres: [
            Vec2::splat(-25.0),
            Vec2::new(25.0, -25.0),
            Vec2::splat(25.0),
            Vec2::new(-25.0, 25.0),
        ],
        surface: CityStreetSurface::Fieldstone,
    }];
    input.buildings = [
        (BuildingUse::Warehouse, Vec2::new(-53.0, -22.0)),
        (BuildingUse::Stable, Vec2::new(52.0, 18.0)),
    ]
    .into_iter()
    .enumerate()
    .map(
        |(index, (usage, centre_metres))| TacticalBuildingPlacement {
            id: index as u64 + 1,
            program: BuildingProgram::validated_settlement(
                settlement_archetype(usage),
                usage,
                42,
                Some(ServiceBuildingSize::Small),
            )
            .unwrap(),
            centre_metres,
            orientation: BuildingOrientation::IDENTITY,
        },
    )
    .collect();
    let buildings = super::super::buildings::prepare_buildings(&input.buildings).unwrap();
    let terrain = SceneTerrain::from_heightmap(81, 81, 2.0, vec![0.0; 81 * 81]).unwrap();
    let ground = SceneGround::uniform_for_terrain(
        &terrain,
        GroundSurface {
            cover: GroundCover::Bare,
            ..Default::default()
        },
    );
    (input, buildings, terrain, ground)
}

#[test]
fn production_review_input_places_every_furniture_family() {
    let input = TacticalSceneInput::load(std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/tactical-scenes/furniture-review.json"
    )))
    .unwrap();
    let scene = input.generate().unwrap();
    city_acceptance::report_terrain_delta("furniture-review", &input, &scene.terrain);
    city_acceptance::assert_vendor_stock_variety(&scene.furniture);
    for kind in FurnitureKind::ALL {
        let count = scene
            .furniture
            .instances
            .iter()
            .filter(|instance| instance.scene.key.kind == kind)
            .count();
        println!("{kind:?}: {count} accepted instances");
        assert!(count > 0, "production review has no {kind:?}");
    }
}

#[test]
fn ordered_candidate_identity_separates_swapped_fields_and_anchor_kinds() {
    let id =
        |anchor, slot| candidates::Candidate::new(42, slot, FurnitureGroupKind::Vendor, anchor).id;
    assert_ne!(
        id(FurnitureAnchor::Building { id: 1 }, 2),
        id(FurnitureAnchor::Building { id: 2 }, 1)
    );
    assert_ne!(
        id(FurnitureAnchor::Market { patch_index: 1 }, 2),
        id(FurnitureAnchor::Building { id: 1 }, 2)
    );
    let mut unique = std::collections::BTreeSet::new();
    for anchor in 0..64 {
        for slot in 0..48 {
            assert!(unique.insert(id(FurnitureAnchor::Building { id: anchor }, slot)));
            assert!(unique.insert(id(
                FurnitureAnchor::Market {
                    patch_index: anchor as u32
                },
                slot
            )));
        }
    }
}

#[test]
fn a_wet_gentle_grade_keeps_supported_examples_of_every_family() {
    let mut input = TacticalSceneInput::load(std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/tactical-scenes/furniture-review.json"
    )))
    .unwrap();
    input.seed += 1;
    input.weather.ground_moisture_bps = 8500;
    let width = usize::from(input.playable.width);
    for (index, height) in input.playable.heights_metres.iter_mut().enumerate() {
        let x = index % width;
        *height += (x as f32 - (width - 1) as f32 * 0.5) * input.playable.spacing_metres * 0.004;
    }
    let scene = input.generate().unwrap();
    for kind in FurnitureKind::ALL {
        assert!(
            scene
                .furniture
                .instances
                .iter()
                .any(|instance| instance.scene.key.kind == kind),
            "wet gentle slope lost {kind:?}"
        );
    }
}

#[test]
fn furniture_groups_are_deterministic_supported_and_leave_routes_clear() {
    let (input, buildings, terrain, ground) = fixture();
    let first = generate(&input, &buildings, &terrain, &ground, &[]);
    let second = generate(&input, &buildings, &terrain, &ground, &[]);
    assert_eq!(first, second);
    for kind in FurnitureKind::ALL {
        assert!(
            first
                .instances
                .iter()
                .any(|instance| instance.scene.key.kind == kind),
            "missing {kind:?}"
        );
    }
    assert!(
        first
            .groups
            .iter()
            .filter(|group| group.kind == FurnitureGroupKind::Vendor)
            .count()
            >= 4
    );
    for (index, group) in first.groups.iter().enumerate() {
        assert!(
            first
                .reserved_routes
                .iter()
                .all(|route| !route.intersects(group.footprint))
        );
        assert!(
            first.groups[index + 1..]
                .iter()
                .all(|other| !other.footprint.intersects(group.footprint))
        );
        if let FurnitureAnchor::Market { patch_index } = group.anchor {
            assert!(
                group
                    .footprint
                    .corners()
                    .into_iter()
                    .all(|point| input.streets[patch_index as usize].contains(point))
            );
        }
    }
    for instance in first.instances {
        for support in &instance.scene.key.recipe().support_points_metres {
            let point = Vec2::new(instance.position_metres.x, instance.position_metres.z)
                + instance
                    .orientation
                    .local_to_world(Vec2::new(support.x, support.z));
            assert!(
                (terrain.height_at(point).unwrap() - instance.position_metres.y - support.y).abs()
                    < 0.001
            );
        }
    }
}

#[test]
fn inserted_street_obstruction_removes_every_conflicting_group() {
    let (mut input, buildings, terrain, ground) = fixture();
    let original = generate(&input, &buildings, &terrain, &ground, &[]);
    let group = original
        .groups
        .iter()
        .find(|group| group.kind == FurnitureGroupKind::Vendor)
        .unwrap();
    let centre = group.footprint.centre_metres;
    input.streets.push(CityStreetPatch::Corridor {
        start_metres: centre - Vec2::X * 12.0,
        end_metres: centre + Vec2::X * 12.0,
        half_width_metres: 3.0,
        surface: CityStreetSurface::CompactedEarth,
    });
    let changed = generate(&input, &buildings, &terrain, &ground, &[]);
    assert!(
        !changed
            .groups
            .iter()
            .any(|candidate| candidate.footprint == group.footprint)
    );
    assert!(changed.groups.iter().all(|candidate| {
        changed
            .reserved_routes
            .iter()
            .all(|route| !route.intersects(candidate.footprint))
    }));
    assert!(
        !changed.groups.is_empty(),
        "a local obstruction must not disable unrelated placement"
    );
}

#[test]
fn unsupported_or_submerged_candidates_are_rejected_without_moving_terrain() {
    let (input, buildings, mut terrain, ground) = fixture();
    assert!(terrain.rewrite_heights(|point, _| point.x * 0.3));
    assert!(
        generate(&input, &buildings, &terrain, &ground, &[])
            .instances
            .is_empty()
    );
    let (input, buildings, terrain, _) = fixture();
    let water = SceneGround::uniform_for_terrain(
        &terrain,
        GroundSurface {
            substrate: crate::scene::GroundSubstrate::Water,
            ..Default::default()
        },
    );
    assert!(
        generate(&input, &buildings, &terrain, &water, &[])
            .instances
            .is_empty()
    );
}

#[test]
fn furniture_physics_blocks_real_members_and_keeps_stall_approach_open() {
    let barrel = FurnitureKey {
        kind: FurnitureKind::Barrel,
        variant: FurnitureVariant::Compact,
    };
    let bounds = barrel.recipe().bounds;
    let collider = furniture_collider(barrel);
    assert!(
        collider
            .cast_ray(
                Vec3::ZERO,
                Rotation::default(),
                Vec3::new(0.0, bounds.max.y * 0.5, -4.0),
                Vec3::Z,
                8.0,
                false
            )
            .is_some()
    );
    let stall = FurnitureKey {
        kind: FurnitureKind::CanvasStall,
        variant: FurnitureVariant::Compact,
    };
    let collider = furniture_collider(stall);
    for member in &stall.recipe().colliders {
        let ray = member.centre + Vec3::Y * 5.0;
        assert!(
            collider
                .cast_ray(
                    Vec3::ZERO,
                    Rotation::default(),
                    ray,
                    Vec3::NEG_Y,
                    10.0,
                    false
                )
                .is_some()
        );
    }
    let recipe = stall.recipe();
    let clearance = recipe
        .clearances
        .first()
        .expect("stall access is explicitly reserved")
        .bounds;
    let probe = clearance.centre();
    assert!(
        collider
            .cast_ray(
                Vec3::ZERO,
                Rotation::default(),
                probe + Vec3::Y * 3.0,
                Vec3::NEG_Y,
                3.0,
                false
            )
            .is_none()
    );
}
