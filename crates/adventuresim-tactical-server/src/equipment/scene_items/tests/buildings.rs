//! Drops against the generated compound review scene: building walls, closed
//! doors and windows, property enclosures and furniture, all spawned through
//! the production scene observers.
use super::*;
use adventuresim_tactical_core::physics::TACTICAL_MAXIMUM_STEP_HEIGHT_METRES;
use adventuresim_tactical_core::scene_input::{
    SceneBoundary, SceneBuilding, SceneDoor, SceneWindow,
};

const PROBE_SPACING_M: f32 = 0.5;
/// Every floor, roof and ground surface is found by short downward rays
/// stacked this far apart, spanning the building transforms plus a margin.
const FLOOR_LATTICE_STEP_M: f32 = 0.9;
const FLOOR_LATTICE_MARGIN_M: f32 = 8.0;
const FLOOR_PROBE_DEPTH_M: f32 = 1.0;
/// Feet heights at which a facing is probed for a blocker.
const BLOCKER_PROBE_HEIGHTS_M: [f32; 3] = [0.4, 1.0, 1.6];
/// Blockers nearer than this to the actor's centre sit in the drop candidates.
const BLOCKER_REACH_M: f32 = 1.6;
const STANDING_CLEARANCE_M: f32 = 0.45;
const SITE_SPREAD_M: f32 = 2.0;
const SITES_PER_BLOCKER: usize = 6;
/// A resting item may touch its support, so only a shrunken box must be clear.
const RESTING_BOX_SCALE: f32 = 0.8;
/// Distances from a hidden item at which the actor looks for somewhere to
/// stand, each tried in `RECOVERY_DIRECTIONS` directions.
const RECOVERY_REACHES_M: [f32; 4] = [0.5, 0.7, 0.9, 1.2];
const RECOVERY_DIRECTIONS: usize = 16;

/// Building, opening, enclosure and furniture bodies, but not terrain.
type SceneSolid = Or<(
    With<SceneBuilding>,
    With<SceneDoor>,
    With<SceneWindow>,
    With<SceneBoundary>,
    With<SceneFurniture>,
)>;

/// Ordered from the most specific blocker to the least.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Blocker {
    Door,
    Window,
    Furniture,
    Enclosure,
    Wall,
}

impl Blocker {
    const ALL: [Self; 5] = [
        Self::Door,
        Self::Window,
        Self::Furniture,
        Self::Enclosure,
        Self::Wall,
    ];
}

struct BlockedSite {
    feet: Vec3,
    facing: Vec3,
    blocker: Blocker,
    distance_m: f32,
}

fn populate(app: &mut App) {
    let input: TacticalSceneInput = serde_json::from_str(include_str!(
        "../../../../../../assets/tactical-scenes/compound-review.json"
    ))
    .unwrap();
    let generated = input.generate().unwrap();
    app.add_observer(crate::openings::on_scene_building_added)
        .add_observer(crate::openings::on_scene_boundary_added)
        .add_observer(crate::furniture::on_furniture_added);
    let terrain = match &generated.terrain_patch {
        Some(patch) => patch.colliders_with_terrain(&generated.terrain).unwrap(),
        None => generated.terrain.colliders().unwrap(),
    };
    spawn_colliders(app.world_mut(), terrain);
    let mut commands = app.world_mut().commands();
    crate::openings::spawn_generated_buildings(
        &mut commands,
        generated.buildings,
        &input.establishments,
    )
    .unwrap();
    crate::openings::spawn_generated_boundaries(&mut commands, generated.boundaries);
    for instance in generated.furniture.instances {
        commands.spawn((
            instance.scene,
            Transform::from_translation(instance.position_metres.metres())
                .with_rotation(Quat::from_rotation_y(instance.orientation.yaw_radians())),
        ));
    }
    app.world_mut().flush();
}

fn scene_filter(scene: &Scene) -> SpatialQueryFilter {
    SpatialQueryFilter::from_excluded_entities([scene.actor, scene.item])
}

fn classify(world: &World, entity: Entity) -> Option<Blocker> {
    let entity = world.entity(entity);
    if entity.contains::<SceneBuilding>() {
        Some(Blocker::Wall)
    } else if entity.contains::<SceneDoor>() {
        Some(Blocker::Door)
    } else if entity.contains::<SceneWindow>() {
        Some(Blocker::Window)
    } else if entity.contains::<SceneBoundary>() {
        Some(Blocker::Enclosure)
    } else if entity.contains::<SceneFurniture>() {
        Some(Blocker::Furniture)
    } else {
        None
    }
}

/// The first surface within `depth` below `start` at `position`.
fn floor_below(
    spatial: &SpatialQuery,
    filter: &SpatialQueryFilter,
    position: Vec2,
    start: f32,
    depth: f32,
) -> Option<f32> {
    spatial
        .cast_ray(
            Vec3::new(position.x, start, position.y),
            Dir3::NEG_Y,
            depth,
            true,
            filter,
        )
        .map(|hit| start - hit.distance)
}

/// The actor's body fits above `feet` and the floor stays within a step of
/// it all around the cylinder.
fn standable(spatial: &SpatialQuery, filter: &SpatialQueryFilter, feet: Vec3) -> bool {
    let supported = [Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y]
        .into_iter()
        .all(|side| {
            floor_below(
                spatial,
                filter,
                feet.xz() + side * STANDING_CLEARANCE_M,
                feet.y + 0.5,
                FLOOR_PROBE_DEPTH_M,
            )
            .is_some_and(|ground| (ground - feet.y).abs() <= TACTICAL_MAXIMUM_STEP_HEIGHT_METRES)
        });
    let body = Collider::cylinder(0.4, 1.9 - 2.0 * TACTICAL_MAXIMUM_STEP_HEIGHT_METRES);
    let body_centre = feet + Vec3::Y * (TACTICAL_MAXIMUM_STEP_HEIGHT_METRES + 0.6);
    supported
        && spatial
            .shape_intersections(&body, body_centre, Quat::IDENTITY, filter)
            .is_empty()
}

/// Places where the actor could stand on a floor or the ground while facing a
/// nearby wall, opening, enclosure or furnishing, spread across the scene.
fn blocked_sites(scene: &mut Scene) -> Vec<BlockedSite> {
    let filter = scene_filter(scene);
    let world = scene.app.world_mut();
    let bases: Vec<f32> = world
        .query_filtered::<&Transform, With<SceneBuilding>>()
        .iter(world)
        .map(|transform| transform.translation.y)
        .collect();
    let (low, high) = world
        .query_filtered::<&Transform, Or<(With<SceneBuilding>, With<SceneFurniture>)>>()
        .iter(world)
        .fold((Vec2::MAX, Vec2::MIN), |(low, high), transform| {
            let position = transform.translation.xz();
            (low.min(position), high.max(position))
        });
    let probes = world
        .run_system_once(move |spatial: SpatialQuery| {
            let lowest = bases.iter().copied().fold(f32::MAX, f32::min) - FLOOR_LATTICE_MARGIN_M;
            let highest = bases.iter().copied().fold(f32::MIN, f32::max) + FLOOR_LATTICE_MARGIN_M;
            let mut sites = Vec::new();
            let margin = Vec2::splat(6.0);
            let (low, high) = (low - margin, high + margin);
            let mut x = low.x;
            while x <= high.x {
                let mut z = low.y;
                while z <= high.y {
                    let position = Vec2::new(x, z);
                    let mut floors: Vec<f32> = Vec::new();
                    let mut start = highest;
                    while start >= lowest {
                        if let Some(height) =
                            floor_below(&spatial, &filter, position, start, FLOOR_LATTICE_STEP_M)
                            && floors.iter().all(|kept| (kept - height).abs() > 0.1)
                        {
                            floors.push(height);
                        }
                        start -= FLOOR_LATTICE_STEP_M;
                    }
                    for height in floors {
                        let feet = Vec3::new(x, height, z);
                        if !standable(&spatial, &filter, feet) {
                            continue;
                        }
                        for step in 0..8 {
                            let facing =
                                Quat::from_rotation_y(step as f32 * std::f32::consts::FRAC_PI_4)
                                    * Vec3::NEG_Z;
                            let hits: Vec<_> = BLOCKER_PROBE_HEIGHTS_M
                                .into_iter()
                                .filter_map(|lift| {
                                    spatial.cast_ray(
                                        feet + Vec3::Y * lift,
                                        Dir3::new(facing).unwrap(),
                                        BLOCKER_REACH_M,
                                        true,
                                        &filter,
                                    )
                                })
                                .collect();
                            if !hits.is_empty() {
                                sites.push((feet, facing, hits));
                            }
                        }
                    }
                    z += PROBE_SPACING_M;
                }
                x += PROBE_SPACING_M;
            }
            sites
        })
        .unwrap();
    let mut spread: Vec<BlockedSite> = Vec::new();
    for (feet, facing, hits) in probes {
        // A recessed casement or leaf sits behind the wall around it, so an
        // opening hit at any probe height names the site.
        let Some((blocker, distance_m)) = hits
            .iter()
            .filter_map(|hit| Some((classify(world, hit.entity)?, hit.distance)))
            .min_by_key(|(blocker, _)| *blocker)
        else {
            continue;
        };
        let crowded = spread
            .iter()
            .filter(|kept| kept.blocker == blocker)
            .any(|kept| kept.feet.distance(feet) <= SITE_SPREAD_M);
        let full =
            spread.iter().filter(|kept| kept.blocker == blocker).count() == SITES_PER_BLOCKER;
        if !crowded && !full {
            spread.push(BlockedSite {
                feet,
                facing,
                blocker,
                distance_m,
            });
        }
    }
    spread
}

impl Scene {
    /// Walks to a clear, supported spot beside the resting item on the
    /// floor at `floor_height` and picks it up from there.
    fn pick_up_from_nearby(&mut self, floor_height: f32) -> bool {
        let filter = scene_filter(self);
        let rest = self.item_translation().xz();
        let spots = self
            .app
            .world_mut()
            .run_system_once(move |spatial: SpatialQuery| {
                RECOVERY_REACHES_M
                    .into_iter()
                    .flat_map(|reach| (0..RECOVERY_DIRECTIONS).map(move |step| (reach, step)))
                    .filter_map(|(reach, step)| {
                        let turn = std::f32::consts::TAU / RECOVERY_DIRECTIONS as f32;
                        let position = rest + Vec2::from_angle(step as f32 * turn) * reach;
                        let height = floor_below(
                            &spatial,
                            &filter,
                            position,
                            floor_height + 0.5,
                            FLOOR_PROBE_DEPTH_M,
                        )?;
                        let feet = position.extend(height).xzy();
                        standable(&spatial, &filter, feet).then_some(feet)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap();
        spots.into_iter().any(|feet| {
            self.stand(feet, Vec3::NEG_Z);
            self.pick_up_item()
        })
    }

    fn stand(&mut self, feet: Vec3, facing: Vec3) {
        let actor = self.actor;
        *self.app.world_mut().get_mut::<Transform>(actor).unwrap() =
            Transform::from_translation(feet + Vec3::Y * 0.95).looking_to(facing, Vec3::Y);
        self.settle(2);
    }

    /// The resting item box is on the actor's side of every wall, door and
    /// window and not inside anything. Furniture may hide it: an item can
    /// slide off a chest, and the player then walks round to reach it.
    fn item_placement_fault(&mut self) -> Option<String> {
        let filter = scene_filter(self);
        let (actor, item) = (self.actor, self.item);
        self.app
            .world_mut()
            .run_system_once(
                move |spatial: SpatialQuery,
                      transforms: Query<&Transform>,
                      physical: Query<&TacticalEquipmentPhysical>,
                      names: Query<&Name>,
                      solids: Query<(), SceneSolid>,
                      furniture: Query<Entity, With<SceneFurniture>>| {
                    let name = |entity: Entity| {
                        names
                            .get(entity)
                            .map_or_else(|_| format!("{entity:?}"), |name| name.to_string())
                    };
                    let actor = transforms.get(actor).unwrap().translation;
                    let pose = transforms.get(item).unwrap();
                    let physical = physical.get(item).unwrap();
                    let centre = pose.translation - pose.rotation * physical.anchor_offset_m;
                    let origin = actor + Vec3::Y * ITEM_SIGHT_HEIGHT_M;
                    let sight = centre - origin;
                    // `with_excluded_entities` replaces the set, so the
                    // actor and item are named again alongside furniture.
                    let separating = SpatialQueryFilter::from_excluded_entities(
                        filter.excluded_entities.iter().copied().chain(&furniture),
                    );
                    if let Some(hit) = Dir3::new(sight).ok().and_then(|direction| {
                        spatial.cast_ray(origin, direction, sight.length(), true, &separating)
                    }) {
                        return Some(format!(
                            "item box at {centre} is behind {} {:.2} m from the actor's eyes",
                            name(hit.entity),
                            hit.distance
                        ));
                    }
                    let core = physical.dimensions_m * RESTING_BOX_SCALE;
                    let inside = spatial.shape_intersections(
                        &Collider::cuboid(core.x, core.y, core.z),
                        centre,
                        pose.rotation,
                        &filter,
                    );
                    // Terrain surfaces are excluded: a thin blade lying on
                    // the ground sits within its contact margin.
                    let inside: Vec<_> = inside
                        .into_iter()
                        .filter(|entity| solids.contains(*entity))
                        .map(name)
                        .collect();
                    (!inside.is_empty())
                        .then(|| format!("item box at {centre} is inside {inside:?}"))
                },
            )
            .unwrap()
    }
}

#[test]
fn drops_against_walls_openings_and_furniture_stay_on_the_players_side() {
    let mut faults = Vec::new();
    for item_id in CATALOG_ITEMS {
        let mut scene = Scene::assemble(
            populate,
            Box::new(|_, _| 0.0),
            item_id,
            Vec2::ZERO,
            Vec3::NEG_Z,
        );
        scene.settle(8);
        let sites = blocked_sites(&mut scene);
        for blocker in Blocker::ALL {
            assert!(
                sites.iter().any(|site| site.blocker == blocker),
                "the scene offers no place to stand facing a {blocker:?}"
            );
        }
        for site in &sites {
            let context = format!(
                "{item_id} at {} facing {:?} {:.2} m away toward {}",
                site.feet, site.blocker, site.distance_m, site.facing
            );
            scene.stand(site.feet, site.facing);
            let actor = scene.actor_translation();
            if !scene.drop_item() {
                faults.push(format!("{context}: drop rejected"));
                continue;
            }
            scene.settle(1);
            let released = scene.item_translation();
            if released.xz().distance(actor.xz()) > 1.6 {
                faults.push(format!("{context}: released at {released}"));
            }
            scene.settle(SETTLE_TICKS);
            if let Some(fault) = scene.item_placement_fault() {
                faults.push(format!("{context}: released at {released}, {fault}"));
            }
            if !scene.pick_up_item() && !scene.pick_up_from_nearby(site.feet.y) {
                let rest = scene.item_translation();
                faults.push(format!("{context}: item at {rest} cannot be picked up"));
                // Recover the item so the remaining sites still run.
                let actor = scene.actor;
                let item = scene.item;
                let world = scene.app.world_mut();
                world
                    .entity_mut(item)
                    .remove::<(TacticalSceneItem, RigidBody, Collider, CollisionLayers)>()
                    .insert((
                        ItemOf(actor),
                        hand_topology(EquipmentHand::Right),
                        EquipmentHand::Right.slot(),
                        Transform::default(),
                    ));
                scene.settle(2);
            }
        }
    }
    assert!(faults.is_empty(), "{}", faults.join("\n"));
}
