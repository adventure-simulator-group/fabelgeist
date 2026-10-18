//! Imported route acceptance uses the real controller and retained solid geometry.
use super::*;
use crate::scene_input::{GeneratedBuildingRecipes, TacticalSceneInput};

#[test]
#[ignore = "requires a frozen production export in FABELGEIST_TERRAIN_SCENE"]
fn production_garden_routes_allow_entry_tending_and_return() {
    let path = std::env::var("FABELGEIST_TERRAIN_SCENE").unwrap();
    let input = TacticalSceneInput::load(std::path::Path::new(&path)).unwrap();
    let selected = std::env::var("FABELGEIST_GARDEN_PROPERTY")
        .ok()
        .map(|id| id.parse::<u64>().unwrap());
    let scene = input
        .generate_unfurnished(GeneratedBuildingRecipes::default())
        .unwrap();
    let outdoor = outdoor::OutdoorCollision::compile(
        &input,
        &scene.terrain,
        &scene.obstacles,
        &scene.furniture,
    );
    let terrain = scene.terrain.colliders().unwrap();
    let mut buildings = scene.buildings;
    for placement in &input.distant_buildings {
        let placement = crate::scene_input::TacticalBuildingPlacement::from(*placement);
        let recipe = GeneratedBuildingRecipe::generate(placement.program.clone()).unwrap();
        buildings.push(crate::scene_input::GeneratedBuilding {
            placement,
            plan: recipe.plan,
            collision: recipe.collision,
        });
    }
    let buildings: Vec<_> = buildings.into_iter().map(OccupiedBuilding::from).collect();
    let mut gardens: Vec<_> = input.gardens.iter().collect();
    gardens.sort_by_key(|garden| garden.owner);
    let mut observations = Vec::new();
    for garden in gardens {
        if selected.is_some_and(|id| id != garden.owner.0) {
            continue;
        }
        garden.validate_geometry(&input.streets).unwrap();
        let mut walker = garden_walker(
            garden,
            &scene.terrain,
            &terrain,
            &buildings,
            &outdoor,
            &input,
        );
        let targets = [
            garden.access[0].end_metres(),
            garden.access[1].end_metres(),
            garden.access[2].start_metres(),
            garden.access[2].end_metres(),
            garden.access[2].start_metres(),
            garden.access[1].end_metres(),
            garden.access[0].end_metres(),
            garden.access[0].start_metres(),
        ];
        let mut visits = Vec::new();
        for (index, target) in targets.into_iter().enumerate() {
            let arrived = walker.attempt_walk_to(target);
            let distance = arrived.xz().distance(target);
            let passed = distance < 0.08;
            visits.push(
                serde_json::json!({"target_index":index,"target_metres":target,
                "arrived_metres":arrived,"horizontal_shortfall_metres":distance,
                "permitted_metres":0.08,"pass":passed}),
            );
            if !passed {
                walker.capture_step_casts(arrived);
                break;
            }
        }
        let passed =
            visits.len() == targets.len() && visits.iter().all(|visit| visit["pass"] == true);
        observations.push(serde_json::json!({"property_id":garden.owner,
            "member_building_ids":[garden.front_building_id],"pass":passed,"visits":visits}));
        println!(
            "garden {}: {}",
            garden.owner.0,
            if passed { "pass" } else { "FAIL" }
        );
        if !passed {
            break;
        }
    }
    assert!(
        !observations.is_empty(),
        "no exact requested garden present"
    );
    if let Ok(output) = std::env::var("FABELGEIST_GARDEN_REPORT") {
        std::fs::write(output, serde_json::to_vec_pretty(&serde_json::json!({
            "input_digest":input.digest().unwrap(),"observations":observations,
            "scope":"Production KCC, exact terrain, nearby fixed buildings/enclosures and production tactical obstacles and outdoor furniture, closed gates. Unfurnished interiors; vista furniture has no physics. Entry, tending centreline and return. Full-width route discontinuities and garden soil remain separate checks."
        })).unwrap()).unwrap();
    }
    assert!(
        observations.iter().all(|row| row["pass"] == true),
        "{:?}",
        observations.iter().find(|row| row["pass"] == false)
    );
}

fn garden_walker(
    garden: &crate::city_layout::CityGarden,
    ground: &crate::scene::SceneTerrain,
    terrain: &[Collider],
    buildings: &[OccupiedBuilding],
    outdoor: &outdoor::OutdoorCollision,
    input: &TacticalSceneInput,
) -> Walker {
    occupied_walker(
        garden.access[0].start_metres(),
        garden.plot.centre_metres(),
        garden.plot.dimensions_metres().length() + 6.0,
        ground,
        terrain,
        buildings,
        outdoor,
        input,
        None,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "test world retains explicit route bounds and production geometry"
)]
pub(super) fn occupied_walker(
    start: Vec2,
    centre: Vec2,
    radius: f32,
    ground: &crate::scene::SceneTerrain,
    terrain: &[Collider],
    buildings: &[OccupiedBuilding],
    outdoor: &outdoor::OutdoorCollision,
    input: &TacticalSceneInput,
    open_property: Option<crate::city_layout::CityPropertyId>,
) -> Walker {
    let elevation = ground.height_at(start).unwrap();
    // Bounds selection drops no geometry near an access lane. The natural
    // source body intersects this box and retains its complete original BVH.
    let inside = |position: Vec3, shape: &Collider, rotation: Quat| {
        let bounds = shape.aabb(position, rotation);
        let half = Vec2::splat(radius);
        !bounds.max.xz().cmplt(centre - half).any() && !bounds.min.xz().cmpgt(centre + half).any()
    };
    let mut walker = Walker::on_colliders(
        terrain
            .iter()
            .filter(|shape| inside(Vec3::ZERO, shape, Quat::IDENTITY))
            .cloned(),
        start,
        elevation,
    );
    for building in buildings {
        let shape = &building.collider;
        let transform = building.building.transform().unwrap();
        if inside(transform.translation, shape, transform.rotation) {
            walker
                .app
                .world_mut()
                .spawn((RigidBody::Static, shape.clone(), transform));
        }
    }
    outdoor.install(&mut walker.app, inside);
    for compound in &input.compounds {
        if compound.plot.centre_metres().distance(centre)
            > radius + compound.plot.dimensions_metres().length()
        {
            continue;
        }
        let boundary = crate::scene_input::GeneratedBoundary::project(compound, ground).unwrap();
        walker.app.world_mut().spawn((
            RigidBody::Static,
            boundary
                .scene()
                .fixed_support()
                .collider()
                .unwrap()
                .into_solid()
                .unwrap(),
            Transform::from_xyz(0.0, boundary.elevation_metres().metres(), 0.0),
        ));
        let door = compound.boundary.gate.door(compound.id).unwrap();
        let rotation = Quat::from_rotation_y(if open_property == Some(compound.id) {
            door.open_angle_radians.radians()
        } else {
            0.0
        });
        let leaf = door.hinge_centre.metres()
            + rotation * (door.closed_centre.metres() - door.hinge_centre.metres())
            + Vec3::Y * boundary.elevation_metres().metres();
        walker.app.world_mut().spawn((
            RigidBody::Static,
            Collider::cuboid(
                door.size_metres.metres().x,
                door.size_metres.metres().y,
                door.size_metres.metres().z,
            ),
            Transform::from_translation(leaf)
                .with_rotation(rotation * Quat::from_rotation_y(door.closed_yaw_radians.radians())),
        ));
    }
    for _ in 0..64 {
        walker.app.update();
    }
    walker
}

/// Compile immutable building collision once for repeated route visits.
pub(super) struct OccupiedBuilding {
    pub(super) building: crate::scene_input::GeneratedBuilding,
    collider: Collider,
}

impl From<crate::scene_input::GeneratedBuilding> for OccupiedBuilding {
    fn from(building: crate::scene_input::GeneratedBuilding) -> Self {
        let collider =
            crate::scene_input::compile_tactical_building_collider(&building.collision).unwrap();
        Self { building, collider }
    }
}
