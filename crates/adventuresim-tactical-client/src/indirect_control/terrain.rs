//! Local picking shapes mirror terrain without adding client physics bodies.

use adventuresim_tactical_core::prelude::*;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub(super) struct TerrainPicking {
    scene: Option<Entity>,
    colliders: Vec<Collider>,
    transform: Transform,
}

type TerrainScenes<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        Ref<'static, SceneTerrain>,
        Option<Ref<'static, TerrainLandformRecipe>>,
        &'static Transform,
    ),
>;

impl TerrainPicking {
    pub(super) fn distance(&self, origin: Vec3, direction: Dir3, maximum: f32) -> Option<f32> {
        self.colliders
            .iter()
            .filter_map(|collider| {
                collider
                    .cast_ray(
                        self.transform.translation,
                        Rotation(self.transform.rotation),
                        origin,
                        *direction,
                        maximum,
                        true,
                    )
                    .map(|(distance, _)| distance)
            })
            .min_by(f32::total_cmp)
    }
}

pub(super) fn update_terrain_picking(mut picking: ResMut<TerrainPicking>, scenes: TerrainScenes) {
    let Some((entity, terrain, recipe, transform)) = scenes.iter().next() else {
        *picking = TerrainPicking::default();
        return;
    };
    if picking.scene == Some(entity)
        && !terrain.is_changed()
        && !recipe.as_ref().is_some_and(|recipe| recipe.is_changed())
    {
        picking.transform = *transform;
        return;
    }
    let colliders = match recipe {
        Some(recipe) => terrain_landform_patch(&terrain, *recipe)
            .ok()
            .and_then(|patch| patch.colliders_with_terrain(&terrain).ok()),
        None => terrain.colliders().ok(),
    };
    if let Some(colliders) = colliders {
        *picking = TerrainPicking {
            scene: Some(entity),
            colliders,
            transform: *transform,
        };
    } else {
        *picking = TerrainPicking::default();
        warn!(?entity, "cannot prepare terrain for order picking");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource, Default)]
    struct PickedDistance(Option<f32>);

    fn pick_ground(picking: Res<TerrainPicking>, mut result: ResMut<PickedDistance>) {
        result.0 = picking.distance(Vec3::new(0.0, 10.0, 0.0), Dir3::NEG_Y, 20.0);
    }

    #[test]
    fn terrain_orders_can_pick_ground_without_a_client_physics_body() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<TerrainPicking>()
            .init_resource::<PickedDistance>()
            .add_systems(Update, (update_terrain_picking, pick_ground).chain());
        app.world_mut().spawn((
            SceneTerrain::new(8, 8, 1.0, |_| 0.0).unwrap(),
            Transform::default(),
        ));
        app.update();
        let distance = app.world().resource::<PickedDistance>().0.unwrap();
        assert!(
            (distance - 10.0).abs() < 0.001,
            "ground hit differs by more than a millimetre"
        );
        assert_eq!(
            app.world_mut()
                .query::<&RigidBody>()
                .iter(app.world())
                .count(),
            0
        );
    }
}
