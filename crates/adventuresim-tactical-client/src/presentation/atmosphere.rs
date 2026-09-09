//! Freeze Bevy's physically based atmosphere into one generated cubemap.

use super::*;
use bevy::light::{GeneratedEnvironmentMapLight, LightProbe, Skybox};
use bevy::{
    pbr::{ExtractedAtmosphere, GpuAtmosphereSettings, extract_atmosphere},
    render::{
        Extract, ExtractSchedule, RenderApp, extract_component::DynamicUniformIndex,
        sync_world::RenderEntity,
    },
};

mod gpu_bake;
use gpu_bake::AtmosphereBakeGpu;

// Bevy #24884 contains the native fix. Delete this backport when 0.20 is released
// and the project upgrades: https://github.com/bevyengine/bevy/pull/24884
todo_or_die::crates_io!("bevy", ">=0.20.0");

/// 64 * 64 * 6 RGBA16F texels = 192 KiB.
const FROZEN_SKY_CUBEMAP_SIZE: u32 = 64;

#[derive(Resource, Debug, Default)]
pub(crate) struct FrozenAtmosphereStatus {
    phase: FrozenAtmospherePhase,
    pub(crate) completed_bakes: u32,
}

impl FrozenAtmosphereStatus {
    pub(crate) fn is_frozen(&self) -> bool {
        matches!(self.phase, FrozenAtmospherePhase::Frozen { .. })
    }
}

#[derive(Debug, Default)]
enum FrozenAtmospherePhase {
    #[default]
    WaitingForScene,
    Baking {
        scene: Entity,
    },
    Frozen {
        scene: Entity,
    },
}

#[derive(Component)]
pub(in crate::presentation) struct AtmosphereBakeProbe;

#[derive(Component)]
struct FrozenAtmosphereProbeAssets {
    _environment_map: Handle<Image>,
    _diffuse_map: Handle<Image>,
    _specular_map: Handle<Image>,
}

/// Backport of Bevy #24884 for Bevy 0.19.1.
///
/// Bevy's 0.19 extractor stops visiting a camera as soon as its
/// `AtmosphereSettings` is removed. That leaves `ExtractedAtmosphere` in the
/// render world, which keeps the atmospheric PBR pipeline specialization and
/// its per-fragment transmittance work alive after the sky has been frozen.
pub(in crate::presentation) fn install_atmosphere_cleanup_backport(app: &mut App) {
    AtmosphereBakeGpu::install(app);
    let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };
    render_app.add_systems(
        ExtractSchedule,
        cleanup_removed_atmosphere_settings.after(extract_atmosphere),
    );
}

#[expect(
    clippy::type_complexity,
    reason = "the render-world extraction query pairs camera entities with their optional atmosphere settings"
)]
fn cleanup_removed_atmosphere_settings(
    mut commands: Commands,
    cameras: Extract<Query<(RenderEntity, Option<&AtmosphereSettings>), With<Camera3d>>>,
    atmospheres: Extract<Query<(), With<Atmosphere>>>,
) {
    let atmosphere_exists = !atmospheres.is_empty();
    for (render_entity, settings) in &cameras {
        if settings.is_some() && atmosphere_exists {
            continue;
        }
        // These are the public, render-affecting part of Bevy #24884.
        // `ExtractedAtmosphere` clears the ATMOSPHERE mesh-pipeline key. Bevy
        // 0.19's uniform preparation does not retire its generated index when
        // the source component disappears, so remove that index explicitly as
        // well: the sky render node queries it and would otherwise keep using
        // stale bind groups and flashing corrupted aerial-atmosphere output.
        commands.entity(render_entity).remove::<(
            ExtractedAtmosphere,
            GpuAtmosphereSettings,
            DynamicUniformIndex<GpuAtmosphereSettings>,
        )>();
    }
}

/// Convert the one generated atmosphere cube into a visible skybox and
/// static IBL, then retire every public producer component. The
/// global `Atmosphere` remains as the inert owner of its scattering asset.
#[expect(
    clippy::type_complexity,
    reason = "the Bevy query observes both generated and installed environment-map consumers on the bake probe"
)]
pub(in crate::presentation) fn freeze_initialized_atmosphere(
    mut commands: Commands,
    celestial: Res<PresentedCelestialLighting>,
    mut status: ResMut<FrozenAtmosphereStatus>,
    gpu_bake: Res<AtmosphereBakeGpu>,
    settings: Option<Res<TacticalGraphicsSettings>>,
    camera: Single<(Entity, &GlobalTransform), With<TacticalGameplayCamera>>,
    probe: Query<
        (
            Entity,
            Option<&GeneratedEnvironmentMapLight>,
            Option<&EnvironmentMapLight>,
        ),
        With<AtmosphereBakeProbe>,
    >,
) {
    if settings
        .as_ref()
        .is_some_and(|settings| !settings.config.rendering.atmosphere.enabled)
    {
        return;
    }
    let environment_map_size = settings
        .as_ref()
        .map_or(FROZEN_SKY_CUBEMAP_SIZE, |settings| {
            settings.config.rendering.atmosphere.environment_map_size
        });
    let environment_light_enabled = settings
        .as_ref()
        .is_none_or(|settings| settings.config.rendering.atmosphere.environment_light);
    let Some(snapshot) = celestial.snapshot.as_ref() else {
        return;
    };
    let (camera_entity, camera_transform) = camera.into_inner();

    if let FrozenAtmospherePhase::Frozen { scene } = status.phase {
        if scene == snapshot.scene {
            return;
        }
        commands
            .entity(camera_entity)
            .remove::<(Skybox, EnvironmentMapLight)>()
            .insert(AtmosphereSettings::default());
        spawn_bake_probe(
            &mut commands,
            camera_transform.translation(),
            environment_map_size,
        );
        status.phase = FrozenAtmospherePhase::Baking {
            scene: snapshot.scene,
        };
        return;
    }

    let FrozenAtmospherePhase::Baking { scene } = status.phase else {
        spawn_bake_probe(
            &mut commands,
            camera_transform.translation(),
            environment_map_size,
        );
        status.phase = FrozenAtmospherePhase::Baking {
            scene: snapshot.scene,
        };
        return;
    };
    if scene != snapshot.scene {
        status.phase = FrozenAtmospherePhase::Baking {
            scene: snapshot.scene,
        };
        return;
    }
    let Ok((probe_entity, Some(generated), Some(filtered))) = probe.single() else {
        return;
    };
    if !gpu_bake.is_complete(scene, camera_entity, probe_entity, generated, filtered) {
        return;
    }

    let mut camera_commands = commands.entity(camera_entity);
    camera_commands
        .remove::<AtmosphereSettings>()
        .insert(Skybox {
            image: Some(generated.environment_map.clone()),
            // Skybox extraction multiplies this by view exposure; the
            // environment compute stores unexposed physical radiance.
            brightness: 1.0,
            ..default()
        });
    if environment_light_enabled {
        camera_commands.insert(filtered.clone());
    }
    camera_commands.insert(FrozenAtmosphereProbeAssets {
        _environment_map: generated.environment_map.clone(),
        _diffuse_map: filtered.diffuse_map.clone(),
        _specular_map: filtered.specular_map.clone(),
    });
    // The camera retains all three completed textures. Despawning also retires
    // Bevy's private extracted atmosphere-map producer component on the probe.
    commands.entity(probe_entity).despawn();
    status.phase = FrozenAtmospherePhase::Frozen { scene };
    status.completed_bakes += 1;
}

fn spawn_bake_probe(commands: &mut Commands, observer_translation: Vec3, size: u32) {
    commands.spawn((
        Name::new("One-shot atmosphere cubemap probe"),
        AtmosphereBakeProbe,
        LightProbe::default(),
        Transform::from_translation(observer_translation),
        AtmosphereEnvironmentMapLight {
            size: UVec2::splat(size),
            ..default()
        },
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_cube_budget_is_bounded() {
        assert_eq!(FROZEN_SKY_CUBEMAP_SIZE.pow(2) * 6 * 8, 192 * 1024);
    }

    #[test]
    fn completed_bake_installs_both_consumers_and_retires_producers() {
        let mut app = App::new();
        app.init_resource::<FrozenAtmosphereStatus>()
            .init_resource::<AtmosphereBakeGpu>()
            .init_resource::<PresentedCelestialLighting>()
            .init_resource::<ActiveTacticalScene>()
            .add_systems(
                Update,
                (
                    update_presented_celestial_lighting,
                    freeze_initialized_atmosphere,
                )
                    .chain(),
            );
        let scene = app
            .world_mut()
            .spawn(SceneEnvironmentFixture::TemperateHills.snapshot("freeze-test"))
            .id();
        app.world_mut().resource_mut::<ActiveTacticalScene>().entity = Some(scene);
        let camera = app
            .world_mut()
            .spawn((
                Camera3d::default(),
                TacticalGameplayCamera,
                AtmosphereSettings::default(),
            ))
            .id();
        app.world_mut().spawn(Atmosphere::earth(Handle::default()));

        app.update();
        let probe = app
            .world_mut()
            .query_filtered::<Entity, With<AtmosphereBakeProbe>>()
            .single(app.world())
            .unwrap();
        app.world_mut().entity_mut(probe).insert((
            GeneratedEnvironmentMapLight::default(),
            EnvironmentMapLight::default(),
        ));
        for _ in 0..120 {
            app.update();
        }

        // Allocating handles, even for twice the former timeout, is not proof
        // that an asynchronous GPU compute pipeline has ever written them.
        assert!(app.world().entity(camera).contains::<AtmosphereSettings>());
        assert!(!app.world().entity(camera).contains::<Skybox>());
        app.world()
            .resource::<AtmosphereBakeGpu>()
            .complete_for_test();
        app.update();

        let camera_ref = app.world().entity(camera);
        assert_eq!(camera_ref.get::<Skybox>().unwrap().brightness, 1.0);
        assert!(camera_ref.contains::<EnvironmentMapLight>());
        assert!(!camera_ref.contains::<AtmosphereSettings>());
        assert!(
            app.world()
                .entity(camera)
                .contains::<FrozenAtmosphereProbeAssets>()
        );

        assert!(app.world().get_entity(probe).is_err());
        assert_eq!(
            app.world()
                .resource::<FrozenAtmosphereStatus>()
                .completed_bakes,
            1
        );
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<Atmosphere>>()
                .iter(app.world())
                .count(),
            1
        );
    }
}
