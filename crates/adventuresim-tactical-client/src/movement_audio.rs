use adventuresim_tactical_core::prelude::*;
use adventuresim_world_schema::UnitBasisPoints;
use bevy::audio::{AudioSink, AudioSinkPlayback, PlaybackMode, Volume};
use bevy::prelude::*;
use fabelgeist_determinism::{Seed, StreamId};

use crate::{
    animation::{LocomotionPresentationEvent, LocomotionPresentationEventKind},
    audio_config::TacticalAudioConfig,
    presentation::GroundScatterLayer,
};

pub struct MovementAudioPlugin;

impl Plugin for MovementAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(crate::weather_audio::WeatherAudioPlugin)
            .add_systems(Startup, spawn_wind_loop)
            .add_systems(
                Update,
                (
                    play_locomotion_audio,
                    play_grounded_dive_impacts,
                    update_wind_volume,
                ),
            );
    }
}

#[derive(Component)]
struct TacticalWindAudio;

#[derive(Component, Default)]
struct MovementAudioState {
    grounded_dive_active: bool,
}

type GroundedDiveAudioQuery<'world, 'state> = Query<
    'world,
    'state,
    (
        Entity,
        &'static CharacterId,
        &'static GlobalTransform,
        &'static SkeletonState,
        Option<&'static mut MovementAudioState>,
    ),
    Changed<SkeletonState>,
>;

fn spawn_wind_loop(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Name::new("tactical-wind-audio"),
        TacticalWindAudio,
        AudioPlayer::new(asset_server.load("audio/movement/wind_loop.ogg")),
        PlaybackSettings {
            mode: PlaybackMode::Loop,
            volume: Volume::SILENT,
            ..default()
        },
    ));
}

fn update_wind_volume(
    environments: Query<&SceneEnvironment>,
    mut wind: Query<&mut AudioSink, With<TacticalWindAudio>>,
    config: Res<TacticalAudioConfig>,
) {
    let Some(environment) = environments.iter().next() else {
        return;
    };
    let wind_speed = UnitBasisPoints::saturating(environment.weather.wind_speed_bps).as_unit_f32();
    let audible_threshold =
        UnitBasisPoints::saturating(config.movement.wind.minimum_audible_speed_bps).as_unit_f32();
    let wind_fraction =
        ((wind_speed - audible_threshold) / (1.0 - audible_threshold)).clamp(0.0, 1.0);
    let volume = config.movement.wind.maximum_relative_volume
        * wind_fraction.powf(config.movement.wind.response_exponent);
    for mut sink in &mut wind {
        sink.set_volume(Volume::Linear(volume));
    }
}

fn play_locomotion_audio(
    mut commands: Commands,
    mut events: MessageReader<LocomotionPresentationEvent>,
    characters: Query<(&GlobalTransform, &SkeletonState, &CharacterId)>,
    grounds: Query<&SceneGround>,
    understory: Query<(&GlobalTransform, &GroundScatterLayer)>,
    asset_server: Res<AssetServer>,
    config: Res<TacticalAudioConfig>,
) {
    let ground = grounds.iter().next();
    for event in events.read() {
        let Ok((transform, skeleton, character_id)) = characters.get(event.owner) else {
            continue;
        };
        let position = transform.translation();
        let sequence = StreamId::new("audio.locomotion-event")
            .seed(character_id.0, &[event.sequence])
            .to_u64();
        match event.kind {
            LocomotionPresentationEventKind::Contact(_) => {
                let surface = ground.and_then(|ground| ground.ground_at(position.xz()));
                let on_grass = surface.is_some_and(|surface| {
                    matches!(surface.cover, GroundCover::TallGrass | GroundCover::Reeds)
                });
                let family = if on_grass {
                    "footstep_grass_00"
                } else {
                    "footstep_concrete_00"
                };
                spawn_spatial_variant(
                    &mut commands,
                    &asset_server,
                    family,
                    sequence,
                    position,
                    config.movement.footstep_relative_volume,
                    config.movement.pitch_randomization,
                );
                if on_grass {
                    spawn_rustle(
                        &mut commands,
                        &asset_server,
                        sequence,
                        position,
                        config.movement.tall_grass_rustle_relative_volume,
                        config.movement.tall_grass_rustle_pitch,
                        config.movement.pitch_randomization,
                    );
                }
                if near_understory(position, &understory, config.movement.bush_contact_radius_m) {
                    spawn_rustle(
                        &mut commands,
                        &asset_server,
                        StreamId::new("audio.understory-rustle")
                            .seed(sequence, &[])
                            .to_u64(),
                        position,
                        config.movement.bush_rustle_relative_volume,
                        config.movement.bush_rustle_pitch,
                        config.movement.pitch_randomization,
                    );
                }
            }
            LocomotionPresentationEventKind::Landing
                if matches!(
                    skeleton
                        .posture_transition()
                        .map(|transition| transition.kind()),
                    Some(PostureTransitionKind::DiveToDowned { .. })
                ) =>
            {
                spawn_spatial_variant(
                    &mut commands,
                    &asset_server,
                    "impactSoft_heavy_00",
                    sequence,
                    position,
                    config.movement.body_impact_relative_volume,
                    config.movement.pitch_randomization,
                );
            }
            LocomotionPresentationEventKind::Landing => {}
        }
    }
}

fn play_grounded_dive_impacts(
    mut commands: Commands,
    mut characters: GroundedDiveAudioQuery,
    asset_server: Res<AssetServer>,
    config: Res<TacticalAudioConfig>,
) {
    for (entity, character_id, transform, skeleton, state) in &mut characters {
        let grounded_dive_active = matches!(
            skeleton
                .posture_transition()
                .map(|transition| transition.kind()),
            Some(PostureTransitionKind::DiveToDowned {
                trajectory: DiveTrajectory::GroundedSlide,
                ..
            })
        );
        let was_active = state
            .as_deref()
            .is_some_and(|state| state.grounded_dive_active);
        if grounded_dive_active && !was_active {
            spawn_spatial_variant(
                &mut commands,
                &asset_server,
                "impactSoft_heavy_00",
                StreamId::new("audio.grounded-dive")
                    .seed(character_id.0, &[skeleton.locomotion_sample_tick])
                    .to_u64(),
                transform.translation(),
                config.movement.body_impact_relative_volume,
                config.movement.pitch_randomization,
            );
        }
        if let Some(mut state) = state {
            state.grounded_dive_active = grounded_dive_active;
        } else {
            commands.entity(entity).insert(MovementAudioState {
                grounded_dive_active,
            });
        }
    }
}

fn near_understory(
    position: Vec3,
    scatter: &Query<(&GlobalTransform, &GroundScatterLayer)>,
    contact_radius_m: f32,
) -> bool {
    scatter.iter().any(|(transform, layer)| {
        if *layer != GroundScatterLayer::Understory {
            return false;
        }
        let offset = transform.translation() - position;
        offset.xz().length_squared() <= contact_radius_m.powi(2)
    })
}

fn spawn_spatial_variant(
    commands: &mut Commands,
    asset_server: &AssetServer,
    family: &str,
    sequence: u64,
    position: Vec3,
    volume: f32,
    pitch_randomization: [f32; 2],
) {
    let sample =
        sound_seed(sequence, position).child(StreamId::new("audio.family"), &[family.as_bytes()]);
    let variant = sample
        .child(StreamId::new("audio.variant"), &[])
        .rng()
        .index(3);
    let path = format!("audio/movement/{family}{variant}.ogg");
    spawn_spatial_sound(
        commands,
        asset_server.load(path),
        sample,
        position,
        volume,
        1.0,
        pitch_randomization,
    );
}

fn spawn_rustle(
    commands: &mut Commands,
    asset_server: &AssetServer,
    sequence: u64,
    position: Vec3,
    volume: f32,
    base_speed: f32,
    pitch_randomization: [f32; 2],
) {
    let sample = sound_seed(sequence, position);
    spawn_spatial_sound(
        commands,
        asset_server.load("audio/movement/foliage_rustle.ogg"),
        sample,
        position,
        volume,
        base_speed,
        pitch_randomization,
    );
}

fn spawn_spatial_sound(
    commands: &mut Commands,
    source: Handle<AudioSource>,
    sample: Seed,
    position: Vec3,
    volume: f32,
    base_speed: f32,
    pitch_randomization: [f32; 2],
) {
    let pitch_fraction = sample
        .child(StreamId::new("audio.pitch"), &[])
        .rng()
        .inclusive_unit_f32();
    let pitch =
        pitch_randomization[0] + pitch_fraction * (pitch_randomization[1] - pitch_randomization[0]);
    commands.spawn((
        Name::new("tactical-movement-sound"),
        AudioPlayer::new(source),
        PlaybackSettings {
            mode: PlaybackMode::Despawn,
            speed: base_speed * pitch,
            spatial: true,
            volume: Volume::Linear(volume),
            ..default()
        },
        Transform::from_translation(position),
    ));
}

fn sound_seed(sequence: u64, position: Vec3) -> Seed {
    StreamId::new("audio.spatial-event").seed(
        sequence,
        &[
            u64::from(position.x.to_bits()),
            u64::from(position.y.to_bits()),
            u64::from(position.z.to_bits()),
        ],
    )
}
