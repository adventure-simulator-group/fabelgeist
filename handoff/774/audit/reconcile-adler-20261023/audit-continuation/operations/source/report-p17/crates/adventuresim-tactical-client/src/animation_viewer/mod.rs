//! A deterministic gameplay-presentation fixture for tactical animation review.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{BufWriter, Write},
    path::PathBuf,
};

use adventuresim_tactical_core::animation::{
    AnimationValueResult, UnitInterval, dive_launch_root_rotation,
};
use adventuresim_tactical_core::physics::{
    AdventureSimulatorPhysicsPlugin, TACTICAL_DIVE_HORIZONTAL_SPEED_METRES_PER_SECOND,
    quickstep_force_curve, quickstep_motion_target, quickstep_peak_horizontal_force_newtons,
    quickstep_push_duration, quickstep_target_displacement_metres,
    quickstep_tracking_force_newtons,
};
use adventuresim_tactical_core::prelude::*;
use adventuresim_tactical_netcode::client::WeaponGuardInputState;
use bevy::{
    animation::AnimationTargetId,
    app::AppExit,
    asset::io::AssetSourceBuilder,
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk},
    window::PresentMode,
};
use serde::Serialize;

use crate::animation::jitter::{self, JitterBone, JitterFrame, JitterValidationSummary};
use crate::animation::pose_buffer::PoseBufferMetrics;
use crate::animation::review_scenarios::*;
use crate::animation::{
    AnimationPlayback, AnimationRuntime, ArmIkState, AuthoredBindTransform, BoneRole,
    HumanoidBone, LegIkDiagnostics, LegIkState, LocomotionBodyResponseState, LocomotionHeightState,
    LocomotionPresentationEvent, LocomotionPresentationPayload, PresentedSkeleton,
    ProceduralAnimationClock, RaisedFootworkState, TacticalAnimationPlugin, TerrainIkMode,
    bouncy_bones::BouncyBonesTelemetry,
    capture_animation_target_id, capture_entity_id, locomotion_support_weights,
    measured_ankle_sole_offset_metres,
    semantic_route::{SemanticRoutePath, SemanticRouteTrace},
    sole_contact_tolerance_metres,
};
use crate::{
    camera::{CameraMode, TacticalCameraPlugin, animation_capture_camera_offset},
    player::{LocalCharacterId, PlayerPlugin},
    presentation::{TacticalGameplayCamera, TacticalPresentationPlugin},
};

pub(crate) use capture::frame_clock::capture_fixed_clock;
use capture::*;
pub(crate) use harness::ArmorHarness;
pub(crate) use inspection::CaptureInspection;
use report::*;
use scenarios::terrain_policy::{
    raised_scenario_requires_zero_flight, scenario_uses_terrain_ik, startup_terrain_ik_mode,
    terrain_ik_mode_for_frame,
};
use scenarios::*;

mod capture;
mod harness;
mod inspection;
mod report;
mod scenarios;
mod setup;
mod view;

const CAPTURE_LOAD_FRAME_LIMIT: u32 = 1200;

#[derive(Resource)]
struct CaptureBodyProportions(
    Option<adventuresim_core::character_proportions::CharacterProportions>,
);

pub(crate) struct CaptureOptions {
    pub output: PathBuf,
    pub asset_root: PathBuf,
    pub settle_frames: u32,
    pub scenario: Option<String>,
    pub combat_config: TacticalCombatConfig,
    pub body_proportions: Option<adventuresim_core::character_proportions::CharacterProportions>,
    pub armor_harness: Option<ArmorHarness>,
    pub hidden: bool,
    pub inspection: CaptureInspection,
}

pub(crate) fn run(options: CaptureOptions, fixed_clock: Time<Fixed>) -> AppExit {
    let CaptureOptions {
        output,
        asset_root,
        settle_frames,
        scenario,
        combat_config,
        body_proportions,
        armor_harness,
        hidden,
        inspection,
    } = options;
    let scenario = scenario.as_deref();
    let selected_scenario = scenario.map(ScenarioMetadata::from_label);
    let initial_terrain_ik = startup_terrain_ik_mode(selected_scenario.as_ref());
    let capture = match CaptureSequence::new(output.clone(), settle_frames, selected_scenario) {
        Ok(capture) => capture,
        Err(error) => {
            error!(%error, "capture plan admission failed");
            return AppExit::Error(std::num::NonZeroU8::MIN);
        }
    };
    inspection.publish(&output);
    fs::create_dir_all(&output).unwrap_or_else(|error| {
        panic!("failed to create animation capture directory {output:?}: {error}")
    });
    invalidate_previous_report(&output);
    setup::write_body_proportions(&output, body_proportions);
    let default_character_id = default_tactical_character_id();

    let workspace_asset_source =
        AssetSourceBuilder::platform_default(&asset_root.to_string_lossy(), None);
    App::new()
        .insert_resource(combat_config)
        .insert_resource(inspection)
        .insert_resource(CaptureBodyProportions(body_proportions))
        .insert_resource(harness::ArmorCapture::new(armor_harness, output.clone()))
        .insert_resource(harness::equipment_cache(armor_harness))
        .register_asset_source(
            crate::animation::WORKSPACE_ASSET_SOURCE,
            workspace_asset_source,
        )
        // The live debug client registers the same default through
        // `DebugPlugin`. The fixture does not install that input/network
        // plugin, so mirror its presentation default explicitly.
        .register_required_components_with::<Collider, _>(DebugRender::none)
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Fabelgeist Animation Review Capture".into(),
                        resolution: (960, 720).into(),
                        present_mode: PresentMode::AutoVsync,
                        visible: !hidden,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((
            AdventureSimulatorCorePlugins
                .build()
                .set(AdventureSimulatorPhysicsPlugin {
                    enable_simulation: false,
                    enable_presentation_simulation: true,
                }),
            EnhancedInputPlugin,
        ))
        .add_plugins((
            PlayerPlugin,
            crate::equipment::EquipmentVisualPlugin,
            TacticalAnimationPlugin,
            TacticalCameraPlugin,
            TacticalPresentationPlugin::default(),
        ))
        .insert_resource(LocalCharacterId(default_character_id))
        .insert_resource(CameraMode { third_person: true })
        .insert_resource(WeaponGuardInputState::default())
        .insert_resource(fixed_clock)
        // Individual scenarios select terrain conformity explicitly so the
        // viewer can retain FK-only controls after the live default changed.
        .insert_resource(initial_terrain_ik)
        .insert_resource(ClearColor(Color::srgb(0.08, 0.1, 0.13)))
        .insert_resource(capture)
        .add_plugins(CapturePlugin)
        .run()
}
