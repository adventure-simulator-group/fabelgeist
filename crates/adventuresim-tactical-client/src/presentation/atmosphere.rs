//! Cache atmosphere IBL while retaining live sky and direct-light transport.

use super::*;
#[cfg(test)]
use bevy::light::Skybox;
use bevy::light::{GeneratedEnvironmentMapLight, LightProbe};
use bevy::{
    pbr::{ExtractedAtmosphere, GpuAtmosphereSettings, extract_atmosphere},
    render::{
        Extract, ExtractSchedule, RenderApp, extract_component::DynamicUniformIndex,
        sync_world::RenderEntity,
    },
};

mod gpu_bake;
mod shader_corrections;
use gpu_bake::AtmosphereBakeGpu;
use shader_corrections::AtmosphereShaderCorrections;

// Bevy #24884 contains the native fix. Delete this backport when 0.20 is released
// and the project upgrades: https://github.com/bevyengine/bevy/pull/24884
todo_or_die::crates_io!("bevy", ">=0.20.0");

/// 64 * 64 * 6 RGBA16F texels = 192 KiB.
const ATMOSPHERE_IBL_CUBEMAP_SIZE: u32 = 64;

#[derive(Resource, Debug, Default)]
pub(crate) struct AtmosphereIblCache {
    phase: AtmosphereIblPhase,
    key: Option<AtmosphereIblKey>,
    medium_revision: u64,
    pub(crate) completed_bakes: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AtmosphereIblKey {
    scene: Entity,
    environment: SceneEnvironment,
    size: u32,
    environment_light: bool,
    atmosphere: Entity,
    atmosphere_transform: [u32; 6],
    atmosphere_tick: u32,
    medium_revision: u64,
    shader_revision: u64,
}

impl AtmosphereIblKey {
    fn new(
        scene: Entity,
        environment: &SceneEnvironment,
        settings: Option<&TacticalGraphicsSettings>,
        atmospheres: &Query<(Entity, Ref<Atmosphere>, &GlobalTransform)>,
        observer: Vec3,
        medium_revision: u64,
        shader_revision: u64,
    ) -> Option<Self> {
        // Match Bevy's nearest-atmosphere selection and its spherical transform.
        let (entity, atmosphere, transform) = atmospheres.iter().min_by(|a, b| {
            observer
                .distance(a.2.translation())
                .total_cmp(&observer.distance(b.2.translation()))
                .then_with(|| a.0.cmp(&b.0))
        })?;
        let (scale, _, translation) = transform.to_scale_rotation_translation();
        Some(Self {
            scene,
            environment: environment.clone(),
            size: settings.map_or(ATMOSPHERE_IBL_CUBEMAP_SIZE, |s| {
                s.config.rendering.atmosphere.environment_map_size
            }),
            environment_light: settings
                .is_none_or(|s| s.config.rendering.atmosphere.environment_light),
            atmosphere: entity,
            atmosphere_transform: [
                scale.x,
                scale.y,
                scale.z,
                translation.x,
                translation.y,
                translation.z,
            ]
            .map(f32::to_bits),
            atmosphere_tick: atmosphere.last_changed().get(),
            medium_revision,
            shader_revision,
        })
    }
}

#[derive(Debug, Default)]
enum AtmosphereIblPhase {
    #[default]
    WaitingForScene,
    Baking {
        scene: Entity,
    },
    Cached,
}

#[derive(Component)]
pub(in crate::presentation) struct AtmosphereBakeProbe;

#[derive(Component)]
struct CachedAtmosphereProbeAssets {
    _environment_map: Handle<Image>,
    _diffuse_map: Handle<Image>,
    _specular_map: Handle<Image>,
}

/// Backport of Bevy #24884 for Bevy 0.19.1.
///
/// Bevy's 0.19 extractor stops visiting a camera as soon as its
/// `AtmosphereSettings` is removed. That leaves `ExtractedAtmosphere` in the
/// render world, which keeps the atmospheric PBR pipeline specialization and
/// its per-fragment transmittance work alive after a camera disables atmosphere.
pub(in crate::presentation) fn install_atmosphere_cleanup_backport(app: &mut App) {
    AtmosphereShaderCorrections::install(app);
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

/// Cache the completed environment lighting and retire its bake probe.
/// Keep the atmosphere on the camera: it owns direct-light transmittance,
/// planetary horizon occlusion, the physical solar disc and aerial perspective.
#[expect(
    clippy::type_complexity,
    clippy::too_many_arguments,
    reason = "the Bevy query observes both generated and installed environment-map consumers on the bake probe"
)]
pub(in crate::presentation) fn cache_initialized_atmosphere(
    mut commands: Commands,
    celestial: Res<PresentedCelestialLighting>,
    environments: Query<&SceneEnvironment>,
    atmospheres: Query<(Entity, Ref<Atmosphere>, &GlobalTransform)>,
    media: Res<Assets<ScatteringMedium>>,
    mut status: ResMut<AtmosphereIblCache>,
    gpu_bake: Res<AtmosphereBakeGpu>,
    corrections: Option<Res<AtmosphereShaderCorrections>>,
    settings: Option<Res<TacticalGraphicsSettings>>,
    camera: Single<
        (Entity, &GlobalTransform, Has<AtmosphereSettings>),
        With<TacticalGameplayCamera>,
    >,
    probe: Query<
        (
            Entity,
            Option<&GeneratedEnvironmentMapLight>,
            Option<&EnvironmentMapLight>,
        ),
        With<AtmosphereBakeProbe>,
    >,
) {
    let (camera_entity, camera_transform, has_atmosphere) = camera.into_inner();
    let enabled = settings
        .as_ref()
        .is_none_or(|s| s.config.rendering.atmosphere.enabled);
    let current = celestial
        .snapshot
        .as_ref()
        .and_then(|snapshot| {
            environments
                .get(snapshot.scene)
                .ok()
                .map(|environment| (snapshot, environment))
        })
        .filter(|_| enabled && corrections.as_ref().is_none_or(|c| c.installed));
    let Some((snapshot, environment)) = current else {
        status.clear(&mut commands, camera_entity, probe.iter().map(|p| p.0));
        if !enabled {
            commands
                .entity(camera_entity)
                .remove::<AtmosphereSettings>();
        }
        return;
    };
    if !has_atmosphere {
        commands
            .entity(camera_entity)
            .insert(AtmosphereSettings::default());
    }
    if media.is_changed() {
        status.medium_revision += 1;
    }
    let Some(key) = AtmosphereIblKey::new(
        snapshot.scene,
        environment,
        settings.as_deref(),
        &atmospheres,
        camera_transform.translation(),
        status.medium_revision,
        corrections
            .as_ref()
            .map_or(0, |corrections| corrections.revision),
    ) else {
        status.clear(&mut commands, camera_entity, probe.iter().map(|p| p.0));
        return;
    };
    if status.begin_if_changed(
        &mut commands,
        key,
        camera_entity,
        camera_transform.translation(),
        probe.iter().map(|p| p.0),
    ) {
        return;
    }
    status.complete_if_ready(&mut commands, camera_entity, probe.single().ok(), &gpu_bake);
}

impl AtmosphereIblCache {
    fn complete_if_ready(
        &mut self,
        commands: &mut Commands,
        camera: Entity,
        probe: Option<(
            Entity,
            Option<&GeneratedEnvironmentMapLight>,
            Option<&EnvironmentMapLight>,
        )>,
        gpu_bake: &AtmosphereBakeGpu,
    ) {
        let AtmosphereIblPhase::Baking { scene } = self.phase else {
            return;
        };
        let Some((probe, Some(generated), Some(filtered))) = probe else {
            return;
        };
        if gpu_bake.is_complete(scene, camera, probe, generated, filtered) {
            self.complete(commands, camera, probe, generated, filtered);
        }
    }

    fn complete(
        &mut self,
        commands: &mut Commands,
        camera: Entity,
        probe: Entity,
        generated: &GeneratedEnvironmentMapLight,
        filtered: &EnvironmentMapLight,
    ) {
        let mut camera_commands = commands.entity(camera);
        if self.key.as_ref().is_some_and(|key| key.environment_light) {
            camera_commands.insert(filtered.clone());
        }
        camera_commands.insert(CachedAtmosphereProbeAssets {
            _environment_map: generated.environment_map.clone(),
            _diffuse_map: filtered.diffuse_map.clone(),
            _specular_map: filtered.specular_map.clone(),
        });
        // The camera retains all three completed textures. Despawning also retires
        // Bevy's private extracted atmosphere-map producer component on the probe.
        commands.entity(probe).despawn();
        self.phase = AtmosphereIblPhase::Cached;
        self.completed_bakes += 1;
    }

    fn clear(
        &mut self,
        commands: &mut Commands,
        camera: Entity,
        probes: impl Iterator<Item = Entity>,
    ) {
        for probe in probes {
            commands.entity(probe).despawn();
        }
        commands
            .entity(camera)
            .remove::<(EnvironmentMapLight, CachedAtmosphereProbeAssets)>();
        self.key = None;
        self.phase = AtmosphereIblPhase::WaitingForScene;
    }

    fn begin_if_changed(
        &mut self,
        commands: &mut Commands,
        key: AtmosphereIblKey,
        camera: Entity,
        observer: Vec3,
        probes: impl Iterator<Item = Entity>,
    ) -> bool {
        if self.key.as_ref() == Some(&key) {
            return false;
        }
        // New probe and image identities reject an old GPU completion even
        // when a scene changes while its previous bake is still in flight.
        self.clear(commands, camera, probes);
        spawn_bake_probe(commands, observer, key.size);
        self.phase = AtmosphereIblPhase::Baking { scene: key.scene };
        self.key = Some(key);
        true
    }
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
    fn cache_key_tracks_the_selected_atmosphere_and_its_physical_transform() {
        let mut world = World::new();
        let first = world
            .spawn((
                Atmosphere::earth(Handle::default()),
                GlobalTransform::from_translation(Vec3::X),
            ))
            .id();
        let second = world
            .spawn((
                Atmosphere::earth(Handle::default()),
                GlobalTransform::from_translation(Vec3::X * 100.0),
            ))
            .id();
        let environment = SceneEnvironmentFixture::TemperateHills.snapshot("selection-test");
        let mut state = bevy::ecs::system::SystemState::<
            Query<(Entity, Ref<Atmosphere>, &GlobalTransform)>,
        >::new(&mut world);
        let initial = AtmosphereIblKey::new(
            first,
            &environment,
            None,
            &state.get(&world).unwrap(),
            Vec3::ZERO,
            0,
            0,
        )
        .unwrap();
        assert_eq!(initial.atmosphere, first);
        world
            .entity_mut(first)
            .insert(GlobalTransform::from_scale(Vec3::splat(2.0)));
        let scaled = AtmosphereIblKey::new(
            first,
            &environment,
            None,
            &state.get(&world).unwrap(),
            Vec3::ZERO,
            0,
            0,
        )
        .unwrap();
        assert_ne!(scaled, initial);
        world
            .entity_mut(first)
            .insert(GlobalTransform::from_translation(Vec3::X * 200.0));
        let selected = AtmosphereIblKey::new(
            first,
            &environment,
            None,
            &state.get(&world).unwrap(),
            Vec3::ZERO,
            0,
            0,
        )
        .unwrap();
        assert_eq!(selected.atmosphere, second);
        assert_ne!(selected, scaled);
        world.despawn(second);
        world.despawn(first);
        assert!(
            AtmosphereIblKey::new(
                first,
                &environment,
                None,
                &state.get(&world).unwrap(),
                Vec3::ZERO,
                0,
                0
            )
            .is_none()
        );
    }

    #[test]
    fn cached_cube_budget_is_bounded() {
        assert_eq!(ATMOSPHERE_IBL_CUBEMAP_SIZE.pow(2) * 6 * 8, 192 * 1024);
    }

    #[test]
    fn completed_ibl_keeps_atmospheric_direct_light_and_retires_probe() {
        let mut app = App::new();
        app.init_resource::<Assets<ScatteringMedium>>()
            .init_resource::<AtmosphereIblCache>()
            .init_resource::<AtmosphereBakeGpu>()
            .init_resource::<PresentedCelestialLighting>()
            .init_resource::<ActiveTacticalScene>()
            .add_systems(
                Update,
                (
                    update_presented_celestial_lighting,
                    cache_initialized_atmosphere,
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
        app.world_mut().spawn((
            Atmosphere::earth(Handle::default()),
            GlobalTransform::default(),
        ));

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
        assert!(!camera_ref.contains::<Skybox>());
        assert!(camera_ref.contains::<EnvironmentMapLight>());
        assert!(camera_ref.contains::<AtmosphereSettings>());
        assert!(
            app.world()
                .entity(camera)
                .contains::<CachedAtmosphereProbeAssets>()
        );

        assert!(app.world().get_entity(probe).is_err());
        assert_eq!(
            app.world().resource::<AtmosphereIblCache>().completed_bakes,
            1
        );
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<Atmosphere>>()
                .iter(app.world())
                .count(),
            1
        );
        // Camera exposure changes the display, not the unexposed sky radiance.
        app.world_mut()
            .entity_mut(camera)
            .insert(Exposure { ev100: 12.0 });
        app.update();
        assert_eq!(
            app.world().resource::<AtmosphereIblCache>().completed_bakes,
            1
        );
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<AtmosphereBakeProbe>>()
                .iter(app.world())
                .count(),
            0
        );

        let mut environment = app
            .world()
            .entity(scene)
            .get::<SceneEnvironment>()
            .unwrap()
            .clone();
        environment.absolute_minute += 60;
        app.world_mut()
            .entity_mut(scene)
            .insert(environment.clone());
        app.update();
        assert!(!app.world().entity(camera).contains::<EnvironmentMapLight>());
        let next_probe = app
            .world_mut()
            .query_filtered::<Entity, With<AtmosphereBakeProbe>>()
            .single(app.world())
            .unwrap();
        app.world_mut().entity_mut(next_probe).insert((
            GeneratedEnvironmentMapLight::default(),
            EnvironmentMapLight::default(),
        ));
        app.update();
        app.world()
            .resource::<AtmosphereBakeGpu>()
            .complete_for_test();
        // A completed old request must not win over a new weather snapshot.
        environment.weather.precipitation = Precipitation::Rain;
        environment.weather.intensity_bps = 7_000;
        app.world_mut().entity_mut(scene).insert(environment);
        app.update();
        assert!(app.world().get_entity(next_probe).is_err());
        assert!(!app.world().entity(camera).contains::<EnvironmentMapLight>());
        assert!(app.world().entity(camera).contains::<AtmosphereSettings>());
        assert_eq!(
            app.world().resource::<AtmosphereIblCache>().completed_bakes,
            1
        );
        let weather_probe = app
            .world_mut()
            .query_filtered::<Entity, With<AtmosphereBakeProbe>>()
            .single(app.world())
            .unwrap();
        app.world_mut().entity_mut(weather_probe).insert((
            GeneratedEnvironmentMapLight::default(),
            EnvironmentMapLight::default(),
        ));
        app.update();
        assert!(!app.world().entity(camera).contains::<EnvironmentMapLight>());
        app.world()
            .resource::<AtmosphereBakeGpu>()
            .complete_for_test();
        app.update();
        assert!(app.world().entity(camera).contains::<EnvironmentMapLight>());
        assert_eq!(
            app.world().resource::<AtmosphereIblCache>().completed_bakes,
            2
        );
        // A missing/reloading canonical shader must retire both an installed
        // cube and an in-flight probe even when scene and weather stay fixed.
        app.insert_resource(AtmosphereShaderCorrections::default());
        app.update();
        assert!(!app.world().entity(camera).contains::<EnvironmentMapLight>());
        assert!(app.world().resource::<AtmosphereIblCache>().key.is_none());
        {
            let mut corrections = app
                .world_mut()
                .resource_mut::<AtmosphereShaderCorrections>();
            corrections.installed = true;
            corrections.revision = 1;
        }
        app.update();
        let interrupted_probe = app
            .world_mut()
            .query_filtered::<Entity, With<AtmosphereBakeProbe>>()
            .single(app.world())
            .unwrap();
        app.world_mut()
            .resource_mut::<AtmosphereShaderCorrections>()
            .installed = false;
        app.update();
        assert!(app.world().get_entity(interrupted_probe).is_err());
        assert!(app.world().resource::<AtmosphereIblCache>().key.is_none());
        {
            let mut corrections = app
                .world_mut()
                .resource_mut::<AtmosphereShaderCorrections>();
            corrections.installed = true;
            corrections.revision = 2;
        }
        app.update();
        let restored_probe = app
            .world_mut()
            .query_filtered::<Entity, With<AtmosphereBakeProbe>>()
            .single(app.world())
            .unwrap();
        assert_ne!(restored_probe, interrupted_probe);
        assert_eq!(
            app.world()
                .resource::<AtmosphereIblCache>()
                .key
                .as_ref()
                .unwrap()
                .shader_revision,
            2
        );
        app.world_mut().resource_mut::<ActiveTacticalScene>().entity = None;
        app.update();
        assert!(!app.world().entity(camera).contains::<EnvironmentMapLight>());
        assert!(
            !app.world()
                .entity(camera)
                .contains::<CachedAtmosphereProbeAssets>()
        );
        assert!(app.world().resource::<AtmosphereIblCache>().key.is_none());
        app.world_mut().resource_mut::<ActiveTacticalScene>().entity = Some(scene);
        app.update();
        let pending_probe = app
            .world_mut()
            .query_filtered::<Entity, With<AtmosphereBakeProbe>>()
            .single(app.world())
            .unwrap();
        let mut settings = TacticalGraphicsSettings::default();
        settings.config.rendering.atmosphere.enabled = false;
        app.insert_resource(settings);
        app.update();
        assert!(app.world().get_entity(pending_probe).is_err());
        assert!(!app.world().entity(camera).contains::<AtmosphereSettings>());
        assert!(app.world().resource::<AtmosphereIblCache>().key.is_none());
    }
}
