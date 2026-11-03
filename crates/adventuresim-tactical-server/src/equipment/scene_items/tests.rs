use super::*;
use bevy::ecs::system::RunSystemOnce;
use bevy::time::TimeUpdateStrategy;
use std::time::Duration;

const TICK_HZ: f64 = 64.0;
const SETTLE_TICKS: usize = 300;
const TERRAIN_GRID_CELLS: usize = 121;
/// Items rest on their compound box, so the grip can sit slightly under the
/// sampled surface on a slope without the item being buried.
const BURIED_BELOW_SURFACE_M: f32 = 0.5;
const CATALOG_ITEMS: [&str; 3] = ["arming_sword", "knife", "arming_cap"];

#[derive(Clone, Copy, Debug)]
enum TerrainCollision {
    /// Sampled scenes collide with Parry's heightfield.
    Heightfield,
    /// Owned settlement terrain collides with one natural trimesh.
    Trimesh,
}

impl TerrainCollision {
    const ALL: [Self; 2] = [Self::Heightfield, Self::Trimesh];
}

struct Scene {
    app: App,
    heights: Box<dyn Fn(f32, f32) -> f32>,
    actor: Entity,
    item: Entity,
}

impl Scene {
    fn new(
        collision: TerrainCollision,
        item_id: &str,
        feet_xz: Vec2,
        facing: Vec3,
        heights: impl Fn(f32, f32) -> f32 + 'static,
    ) -> Self {
        let colliders = terrain_colliders(collision, &heights);
        Self::from_colliders(colliders, Box::new(heights), item_id, feet_xz, facing)
    }

    fn from_colliders(
        colliders: Vec<Collider>,
        heights: Box<dyn Fn(f32, f32) -> f32>,
        item_id: &str,
        feet_xz: Vec2,
        facing: Vec3,
    ) -> Self {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, PhysicsPlugins::default(), TransformPlugin));
        app.insert_resource(Time::<Fixed>::from_hz(TICK_HZ));
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1.0 / TICK_HZ,
        )));
        app.finish();
        app.cleanup();
        spawn_colliders(app.world_mut(), colliders);
        let mut scene = Self {
            app,
            heights,
            actor: Entity::PLACEHOLDER,
            item: Entity::PLACEHOLDER,
        };
        scene.settle(4);
        let origin = scene.standing_origin(feet_xz);
        scene.actor = scene
            .app
            .world_mut()
            .spawn((
                Player {
                    name: "drop regression".into(),
                },
                CharacterLook::default(),
                Transform::from_translation(origin).looking_to(facing, Vec3::Y),
                RigidBody::Kinematic,
                Collider::cylinder(0.4, 1.9),
            ))
            .id();
        scene.item = spawn_held_item(scene.app.world_mut(), scene.actor, item_id);
        scene.settle(2);
        scene
    }

    fn settle(&mut self, ticks: usize) {
        for _ in 0..ticks {
            self.app.update();
        }
    }

    fn standing_origin(&self, feet_xz: Vec2) -> Vec3 {
        let ground = (self.heights)(feet_xz.x, feet_xz.y);
        Vec3::new(feet_xz.x, ground + 0.95, feet_xz.y)
    }

    fn place_actor(&mut self, feet_xz: Vec2) {
        let origin = self.standing_origin(feet_xz);
        let actor = self.actor;
        self.app
            .world_mut()
            .get_mut::<Transform>(actor)
            .unwrap()
            .translation = origin;
        self.settle(2);
    }

    fn item_translation(&self) -> Vec3 {
        self.app
            .world()
            .get::<Transform>(self.item)
            .unwrap()
            .translation
    }

    fn actor_translation(&self) -> Vec3 {
        self.app
            .world()
            .get::<Transform>(self.actor)
            .unwrap()
            .translation
    }

    fn drop_item(&mut self) -> bool {
        let actor = self.actor;
        let accepted = self
            .app
            .world_mut()
            .run_system_once(
                move |mut commands: Commands,
                      players: Query<(&Transform, &CharacterLook), With<Player>>,
                      items: Query<ItemView<'_>>,
                      spatial: SpatialQuery| {
                    drop_hand(
                        &mut commands,
                        actor,
                        EquipmentHand::Right,
                        &players,
                        &items,
                        &spatial,
                    )
                },
            )
            .unwrap();
        self.app.world_mut().flush();
        accepted
    }

    fn pick_up_item(&mut self) -> bool {
        let (actor, item) = (self.actor, self.item);
        let accepted = self
            .app
            .world_mut()
            .run_system_once(
                move |mut commands: Commands,
                      players: Query<(&Transform, &CharacterLook), With<Player>>,
                      items: Query<ItemView<'_>>,
                      spatial: SpatialQuery| {
                    pickup(
                        &mut commands,
                        actor,
                        EquipmentHand::Right,
                        item,
                        &players,
                        &items,
                        &spatial,
                    )
                },
            )
            .unwrap();
        self.app.world_mut().flush();
        self.settle(2);
        accepted
    }

    fn assert_at_rest_above_terrain(&self, context: &str) {
        let rest = self.item_translation();
        let ground = (self.heights)(rest.x, rest.z);
        assert!(
            rest.y > ground - BURIED_BELOW_SURFACE_M,
            "{context}: item rests at {rest} under the surface at {ground}"
        );
    }
}

fn terrain_colliders(
    collision: TerrainCollision,
    heights: &impl Fn(f32, f32) -> f32,
) -> Vec<Collider> {
    let half_extent = (TERRAIN_GRID_CELLS as f32 - 1.0) / 2.0;
    let mut samples = Vec::with_capacity(TERRAIN_GRID_CELLS * TERRAIN_GRID_CELLS);
    for z in 0..TERRAIN_GRID_CELLS {
        for x in 0..TERRAIN_GRID_CELLS {
            let (x, z) = (x as f32 - half_extent, z as f32 - half_extent);
            samples.push(Vec3::new(x, heights(x, z), z));
        }
    }
    match collision {
        TerrainCollision::Heightfield => {
            let heightmap = samples.iter().map(|sample| sample.y).collect();
            SceneTerrain::from_heightmap(TERRAIN_GRID_CELLS, TERRAIN_GRID_CELLS, 1.0, heightmap)
                .unwrap()
                .colliders()
                .unwrap()
        }
        TerrainCollision::Trimesh => vec![Collider::trimesh(samples, grid_triangles())],
    }
}

fn spawn_colliders(world: &mut World, colliders: Vec<Collider>) {
    for collider in colliders {
        world.spawn((
            RigidBody::Static,
            CollisionLayers::new(TACTICAL_TERRAIN_LAYER, LayerMask::ALL),
            collider,
            Transform::IDENTITY,
        ));
    }
}

fn grid_triangles() -> Vec<[u32; 3]> {
    let cells = TERRAIN_GRID_CELLS as u32 - 1;
    let vertex = |x: u32, z: u32| x + z * TERRAIN_GRID_CELLS as u32;
    (0..cells)
        .flat_map(|z| (0..cells).map(move |x| (x, z)))
        .flat_map(|(x, z)| {
            [
                [vertex(x, z), vertex(x, z + 1), vertex(x + 1, z)],
                [vertex(x + 1, z), vertex(x, z + 1), vertex(x + 1, z + 1)],
            ]
        })
        .collect()
}

fn spawn_held_item(world: &mut World, actor: Entity, item_id: &str) -> Entity {
    let definition = item_catalog::definition(item_id).unwrap();
    let physical = definition.equipment.as_ref().unwrap().physical;
    let dimensions_m = Vec3::from_array(physical.dimensions_m);
    world
        .spawn((
            ItemProperties {
                id: item_id.into(),
                weight: definition.weight_kg,
            },
            ItemOf(actor),
            hand_topology(EquipmentHand::Right),
            EquipmentHand::Right.slot(),
            TacticalEquipmentPhysical {
                dimensions_m,
                grip_to_tip_m: physical.grip_to_tip_m,
                striking_head_length_m: dimensions_m.min_element(),
                anchor_offset_m: Vec3::from_array(physical.anchor_offset_m),
            },
            Transform::default(),
        ))
        .id()
}

fn cases<const N: usize>(values: [f32; N]) -> Vec<(TerrainCollision, &'static str, f32)> {
    TerrainCollision::ALL
        .into_iter()
        .flat_map(|collision| CATALOG_ITEMS.map(|item_id| (collision, item_id)))
        .flat_map(|(collision, item_id)| values.map(|value| (collision, item_id, value)))
        .collect()
}

fn slope(degrees: f32) -> impl Fn(f32, f32) -> f32 {
    let grade = degrees.to_radians().tan();
    move |_, z| z * grade
}

#[test]
fn pickup_box_center_accounts_for_nonzero_anchor_offset() {
    let physical = TacticalEquipmentPhysical {
        dimensions_m: Vec3::splat(0.2),
        grip_to_tip_m: 0.4,
        striking_head_length_m: 0.2,
        anchor_offset_m: Vec3::new(0.15, -0.05, 0.1),
    };
    assert_eq!(
        item_box_center(Vec3::new(2.0, 1.0, -3.0), &physical),
        Vec3::new(1.85, 1.05, -3.1)
    );
}

#[test]
fn every_repeated_drop_starts_at_the_player_and_stays_recoverable() {
    for (collision, item_id, degrees) in cases([0.0, 25.0, 40.0]) {
        let mut scene = Scene::new(
            collision,
            item_id,
            Vec2::new(10.0, 10.0),
            Vec3::NEG_X,
            slope(degrees),
        );
        for (cycle, feet_xz) in [
            Vec2::new(10.0, 10.0),
            Vec2::new(-12.0, 15.0),
            Vec2::new(-4.0, -15.0),
            Vec2::new(14.0, 3.0),
        ]
        .into_iter()
        .enumerate()
        {
            let context = format!("{item_id} on {collision:?} at {degrees} degrees, cycle {cycle}");
            scene.place_actor(feet_xz);
            let actor = scene.actor_translation();
            assert!(scene.drop_item(), "{context}: drop rejected");
            scene.settle(1);
            let released = scene.item_translation();
            assert!(
                released.xz().distance(actor.xz()) <= 1.6,
                "{context}: released at {released}, actor at {actor}"
            );
            assert!(
                released.y > actor.y,
                "{context}: released at {released}, actor at {actor}"
            );

            scene.settle(SETTLE_TICKS);
            scene.assert_at_rest_above_terrain(&context);
            let rest = scene.item_translation();
            assert!(
                rest.y < released.y,
                "{context}: never fell from {released} to {rest}"
            );
            scene.place_actor(rest.xz() + Vec2::X * 0.5);
            assert!(scene.pick_up_item(), "{context}: pickup rejected");
        }
    }
}

#[test]
fn dropping_at_the_foot_of_a_bluff_never_buries_the_item() {
    for (collision, item_id, rise) in cases([3.0, 6.0]) {
        let bluff = move |_: f32, z: f32| if z >= 1.0 { rise } else { 0.0 };
        let mut scene = Scene::new(collision, item_id, Vec2::new(0.3, 0.0), Vec3::Z, bluff);
        let context = format!("{item_id} on {collision:?} before a {rise} m bluff");
        assert!(scene.drop_item(), "{context}: drop rejected");
        scene.settle(SETTLE_TICKS);
        scene.assert_at_rest_above_terrain(&context);
        assert!(scene.pick_up_item(), "{context}: pickup rejected");
    }
}

#[test]
fn dropping_against_a_wall_keeps_the_item_on_the_players_side() {
    const WALL_Z: f32 = 0.75;
    for (collision, item_id, thickness) in cases([0.2, 0.5]) {
        let mut scene = Scene::new(collision, item_id, Vec2::ZERO, Vec3::Z, |_, _| 0.0);
        scene.app.world_mut().spawn((
            RigidBody::Static,
            CollisionLayers::new(TACTICAL_TERRAIN_LAYER, LayerMask::ALL),
            Collider::cuboid(20.0, 6.0, thickness),
            Transform::from_xyz(0.0, 3.0, WALL_Z),
        ));
        scene.settle(2);
        let context = format!("{item_id} on {collision:?} before a {thickness} m wall");
        assert!(scene.drop_item(), "{context}: drop rejected");
        scene.settle(SETTLE_TICKS);
        assert!(
            scene.item_translation().z < WALL_Z,
            "{context}: item came to rest at {}",
            scene.item_translation()
        );
        assert!(scene.pick_up_item(), "{context}: pickup rejected");
    }
}

mod settlement;
