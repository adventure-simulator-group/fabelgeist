use crate::animation::{
    AngularMagnitudeDegrees, KneeTargetAmplification, ResponseRate, ResponseShapingTime,
    SignedAngleDegrees, SignedAngularSpeedDegrees,
};
use crate::animation::{
    AngularSpeedRadians, AnimationElapsed, AnimationFrequency, NonnegativeLength,
    PositiveAnimationDuration, UnitInterval,
};
use adventuresim_building_generator::spatial_geometry::{PositiveLength, SignedLength};
use adventuresim_core::linear_speed::{NonnegativeSpeed, SignedSpeed};
use adventuresim_core::{
    body::BodyPart,
    combat::{AutoresolveParameters, CombatResolutionParameters},
};
use bevy::prelude::Resource;
pub use landing::AnimationLandingConfig;
pub use locomotion::{AnimationGaitConfig, AnimationLocomotionConfig};
pub use maneuvers::ManeuverTimingConfig;
pub use playback::AnimationPlaybackConfig;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
pub use state_transitions::AnimationStateTransitionConfig;
use thiserror::Error;

pub use ai_offense::AiOffenseConfig;
pub use incapacitation_forecast::IncapacitationForecastConfig;
pub use ragdoll::{
    MuscleRelease, RagdollCapsuleConfig, RagdollCapsulesConfig, RagdollConfig, RagdollMusclesConfig,
};
pub use runtime::{
    runtime_animation_config, runtime_combat_presentation_config, runtime_melee_authority_config,
};

pub use pose_buffer::PoseBufferConfig;
pub use procedural::{
    BODY_RESPONSE_DEGREE_REPORT_MARGIN_RADIANS, BodyResponseConfig, ProceduralAnimationConfig,
};

mod ai_offense;
mod defaults;
mod incapacitation_forecast;
mod landing;
mod locomotion;
mod maneuvers;
mod playback;
mod pose_buffer;
mod procedural;
mod ragdoll;
mod runtime;
mod state_transitions;

pub const TACTICAL_COMBAT_CONFIG_SCHEMA_VERSION: u16 = 9;

#[derive(Clone, Debug, PartialEq, Resource, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalCombatConfig {
    pub schema_version: u16,
    pub resolution: CombatResolutionParameters,
    pub autoresolve: AutoresolveParameters,
    pub realtime_authority: RealtimeAuthorityConfig,
    pub movement: TacticalMovementConfig,
    pub ai: TacticalAiConfig,
    pub client_input: ClientInputConfig,
    pub targeting: TargetingConfig,
    pub presentation: CombatPresentationConfig,
    pub animation: TacticalAnimationConfig,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalAnimationConfig {
    pub locomotion: AnimationLocomotionConfig,
    pub guard_footwork: GuardFootworkConfig,
    pub state_transitions: AnimationStateTransitionConfig,
    pub playback: AnimationPlaybackConfig,
    pub pose_buffer: PoseBufferConfig,
    pub procedural: ProceduralAnimationConfig,
    pub bouncy_bones: BouncyBonesConfig,
    pub inverse_kinematics: InverseKinematicsConfig,
    pub ragdoll: RagdollConfig,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GuardFootworkConfig {
    pub default_half_width_metres: f32,
    pub contact_margin_metres: f32,
    pub minimum_step_seconds: PositiveAnimationDuration,
    pub maximum_step_seconds: PositiveAnimationDuration,
    pub planning_reach_metres: f32,
    pub reference_leg_length_metres: PositiveLength,
    pub maximum_unsupported_contact_seconds: PositiveAnimationDuration,
}

/// Additive rotational hit springs on the rendered skeleton. A hit kicks
/// the struck bone's spring and the reaction travels the bone chain as a
/// wave; nothing here touches gameplay or physics.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BouncyBonesConfig {
    /// Bone-tip impulse per metre per second of server-applied impact
    /// velocity change. A punch moves the whole body by a couple of metres
    /// per second; the struck limb whips several times faster than that.
    pub impact_impulse_scale: f32,
    /// Impulse (bone-tip metres per second) to spin (radians per second).
    pub strength: f32,
    /// Wobble frequency of every spring.
    pub frequency_hz: f32,
    /// Envelope half-life: how fast a reaction dies out.
    pub halflife_seconds: f32,
    /// Wave speed: radians per second squared of parent spin per radian of
    /// child deflection, before the parent's own response scale.
    pub coupling_per_second_squared: f32,
    /// Instant stiff-body share: kick attenuation per chain link applied on
    /// the hit frame. Zero is a pure wave.
    pub transmission: f32,
    /// Per-bone deflection clamp.
    pub maximum_angle_radians: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InverseKinematicsConfig {
    pub minimum_inter_foot_separation_metres: PositiveLength,
    pub safe_settle_fallback_seconds: AnimationElapsed,
    pub outer_foot_track_metres: PositiveLength,
    pub maximum_plant_discontinuity_metres: SignedLength,
    pub maximum_owner_translation_per_tick_metres: SignedLength,
    pub maximum_owner_rotation_per_tick_degrees: SignedAngleDegrees,
    pub maximum_knee_target_amplification: KneeTargetAmplification,
    pub maximum_knee_step_metres: PositiveLength,
    pub continuity_sample_hz: AnimationFrequency,
    pub run_airborne_owner_step_metres: SignedLength,
    pub run_first_release_owner_step_metres: SignedLength,
    pub airborne_release_step_metres: SignedLength,
    pub raised_settle_pelvis_knee_budget_metres: SignedLength,
    pub raised_settle_step_metres: SignedLength,
    pub guard_reach_pelvis_drop_metres: SignedLength,
    pub stationary_turn_foot_limit_metres: SignedLength,
    pub stationary_turn_step_seconds: ResponseShapingTime,
    pub knee_pole_maximum_foot_facing_offset_degrees: AngularMagnitudeDegrees,
    pub airborne_foot_rotation_speed_degrees_per_second: SignedAngularSpeedDegrees,
    pub first_run_release_foot_rotation_speed_degrees_per_second: SignedAngularSpeedDegrees,
    pub maximum_retained_plant_reach_correction_metres: SignedLength,
    pub pelvis_correction_speed_metres_per_second: NonnegativeSpeed,
    pub run_pelvis_correction_speed_metres_per_second: SignedSpeed,
    pub maximum_pelvis_correction_step_metres: NonnegativeLength,
    pub terrain_blend_speed_per_second: ResponseRate,
    pub minimum_knee_flexion_degrees: SignedAngleDegrees,
    pub minimum_terrain_knee_flexion_degrees: SignedAngleDegrees,
    pub landing_knee_reserve_release_compression_metres: SignedLength,
    pub landing_knee_reserve_full_compression_metres: SignedLength,
    pub measured_ankle_sole_offset_metres: PositiveLength,
    pub sole_contact_tolerance_metres: PositiveLength,
    pub swing_sole_clearance_metres: SignedLength,
    pub run_swing_sole_clearance_metres: SignedLength,
    pub terrain_transition_flight_toe_clearance_metres: SignedLength,
    pub terrain_contact_toe_clearance_metres: SignedLength,
    pub run_swing_minimum_sole_clearance_metres: SignedLength,
    pub run_contact_approach_phase: UnitInterval,
    pub run_contact_chain_settle_phase: UnitInterval,
    pub run_maximum_planned_reach_pelvis_drop_metres: NonnegativeLength,
    pub late_run_contact_plan_phase: UnitInterval,
    pub maximum_run_swing_root_relative_step_metres: SignedLength,
    pub settle_step_seconds: ResponseShapingTime,
    pub settle_step_clearance_metres: SignedLength,
    pub settle_capture_point_margin_metres: SignedLength,
    pub assumed_center_of_mass_height_metres: SignedLength,
    pub maximum_settle_capture_speed_metres_per_second: SignedSpeed,
    pub maximum_hip_drop_metres: NonnegativeLength,
    pub sole_contact_margin_metres: SignedLength,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RealtimeAuthorityConfig {
    pub defense: DefenseAuthorityConfig,
    pub melee: MeleeAuthorityConfig,
    pub ranged: RangedAuthorityConfig,
    pub impact: ImpactConfig,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DefenseAuthorityConfig {
    pub reflex_window_seconds: AnimationElapsed,
    pub roll_dodge_effectiveness: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeleeAuthorityConfig {
    pub replay_cooldown_seconds: AnimationElapsed,
    pub completion_allowance_seconds: AnimationElapsed,
    pub range_latency_tolerance_metres: f32,
    pub windup_jitter_fraction: f32,
    pub maximum_windup_jitter_seconds: AnimationElapsed,
    pub lunge_range_window_metres: f32,
    pub lunge_quickstep_threshold_metres: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RangedAuthorityConfig {
    pub cooldown_seconds: AnimationElapsed,
    pub completion_allowance_seconds: AnimationElapsed,
    pub range_latency_tolerance_metres: f32,
    pub aim_half_angle_degrees: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImpactConfig {
    pub whole_body_velocity_scale: f32,
    pub maximum_velocity_change_metres_per_second: NonnegativeSpeed,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalMovementConfig {
    pub speeds_metres_per_second: MovementSpeedsConfig,
    pub jump_heights_metres: JumpHeightsConfig,
    pub prone_lateral_speed_scale: f32,
    pub prone_effort_scale: f32,
    pub motor: CharacterMotorConfig,
    pub maneuvers: ManeuverTimingConfig,
}

/// Mechanical parameters for the velocity-based kinematic character motor.
///
/// Locomotion changes velocity through bounded forces. The resulting velocity
/// is then resolved by Ahoy's swept collision constraints; Ahoy does not apply
/// a second horizontal acceleration or friction model.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CharacterMotorConfig {
    /// Acceleration due to gravity used by ordinary movement and jump-height
    /// conversion.
    pub gravity_metres_per_second_squared: f32,
    /// Total mass used when a controller has no projected character body.
    pub fallback_character_mass_kg: f32,
    /// Maximum horizontal drive force at the reference leg strength.
    pub reference_ground_drive_force_newtons: f32,
    /// Maximum deliberate braking force at the reference leg strength.
    pub reference_ground_braking_force_newtons: f32,
    /// Peak lateral ground-reaction force for a Strength 3 quickstep,
    /// expressed in multiples of the character's biological body weight.
    pub reference_quickstep_peak_horizontal_force_bodyweights: f32,
    /// Additional peak lateral force, in bodyweights, per point of Strength
    /// above the reference value (and correspondingly less below it).
    pub quickstep_peak_horizontal_force_bodyweights_per_strength: f32,
    /// Supported push duration for an Agility 3 character.
    pub reference_quickstep_push_seconds: PositiveAnimationDuration,
    /// Seconds removed from the supported push for every Agility point above
    /// the reference value (and added below it).
    pub quickstep_push_seconds_reduction_per_agility: f32,
    /// Maximum planar root travel while both quickstep feet remain planted.
    /// Reaching this extension releases support even if force time remains.
    pub quickstep_maximum_supported_root_displacement_metres: f32,
    /// Intended planar displacement for the reference leg length.
    pub reference_quickstep_target_displacement_metres: f32,
    /// Hip-to-ankle chain length for the reference quickstep distance.
    pub reference_quickstep_leg_length_metres: f32,
    /// Normalized authored root displacement at frames 0, 3, 6, 9, and 12.
    /// Runtime clips retain the poses but remove this lateral translation.
    pub quickstep_authored_displacement_profile: [f32; 5],
    /// Upward angle of the propulsive quickstep force above horizontal. The
    /// separate baseline normal force supports body weight while planted.
    pub quickstep_takeoff_angle_degrees: f32,
    /// Reference strength corresponding to the configured forces.
    pub reference_leg_strength: f32,
    /// Reference agility corresponding to the configured lateral control.
    pub reference_leg_agility: f32,
    /// Sustained lateral acceleration available at the reference agility,
    /// expressed as a multiple of gravity.
    pub reference_lateral_acceleration_gravities: f32,
    /// Full-sprint turn radius at the reference Agility 3 anchor. Slower
    /// movement turns more tightly through the shared centripetal model.
    pub reference_sprint_turn_radius_metres: f32,
    /// Full-sprint turn radius for Agility 1, the valid attribute minimum.
    pub agility_one_sprint_turn_radius_metres: f32,
    /// Full-sprint turn radius for Agility 5, the valid attribute maximum.
    pub agility_five_sprint_turn_radius_metres: f32,
    /// Upper bound on ground force imposed by available traction, expressed as
    /// a multiple of normal force.
    pub traction_coefficient: f32,
    /// Kinetic drag on a grounded body slide, expressed as a multiple of
    /// normal force. It begins only at the authored body-contact point.
    pub slide_drag_coefficient: f32,
    /// Fraction of ground drive force available without support.
    pub air_control_force_scale: f32,
    /// Maximum vertical obstacle height treated as a deliberate step.
    pub maximum_step_height_metres: f32,
    /// Steepest surface that can supply ordinary standing support.
    pub maximum_walkable_slope_degrees: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MovementSpeedsConfig {
    pub walk: NonnegativeSpeed,
    pub run: NonnegativeSpeed,
    pub raised_guard: NonnegativeSpeed,
    pub prone_walk: NonnegativeSpeed,
    pub prone: NonnegativeSpeed,
    pub roll: NonnegativeSpeed,
    pub dive: NonnegativeSpeed,
    pub quickstep: NonnegativeSpeed,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JumpHeightsConfig {
    pub ordinary: f32,
    pub dive: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalAiConfig {
    pub ordinary: OrdinaryAiConfig,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OrdinaryAiConfig {
    pub offense: AiOffenseConfig,
    pub defense: AiDefenseConfig,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiDefenseConfig {
    pub dodge_chance: f64,
    pub reaction_delay_min_seconds: AnimationElapsed,
    pub reaction_delay_max_seconds: AnimationElapsed,
    pub frontal_flanking_max: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClientInputConfig {
    pub alternate_attack_hold_seconds: PositiveAnimationDuration,
    pub sprint_hold_seconds: PositiveAnimationDuration,
    pub movement_deadzone: f32,
    pub roll_deadzone: f32,
    pub quickstep_threshold: f32,
    pub aim_trigger_threshold: f32,
    pub gamepad_look_scale: [f32; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetingConfig {
    pub reported_hit_precision: f32,
    pub body_part_hitboxes: Vec<BodyPartHitboxConfig>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BodyPartHitboxConfig {
    pub body_part: BodyPart,
    pub center_metres: [f32; 3],
    pub half_extents_metres: [f32; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CombatPresentationConfig {
    pub incapacitation_forecast: IncapacitationForecastConfig,
    pub dodge_seconds: f32,
    pub body_turn_seconds_per_half_turn: PositiveAnimationDuration,
    pub downed_turn_radians_per_second: AngularSpeedRadians,
    pub attack_curve: AttackCurveConfig,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttackCurveConfig {
    pub inertia_characteristic: f32,
    pub inertia_weight: f32,
    pub skill_weight: f32,
    pub tell_base: f32,
    pub tell_span: f32,
    pub drawback_base: f32,
    pub drawback_span: f32,
    pub follow_through_base: f32,
    pub follow_through_span: f32,
    pub overshoot_base: f32,
    pub overshoot_span: f32,
    pub maximum_drawback: f32,
    pub maximum_overshoot: f32,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TacticalCombatConfigError {
    #[error("incompatible tactical combat config schema version")]
    SchemaVersion,
    #[error("invalid tactical combat config: {0}")]
    Validation(&'static str),
    #[error("could not serialize validated tactical combat config: {0}")]
    Serialization(String),
}

impl From<&'static str> for TacticalCombatConfigError {
    fn from(message: &'static str) -> Self {
        Self::Validation(message)
    }
}

fn validate_schema_version(schema_version: u16) -> Result<(), TacticalCombatConfigError> {
    (schema_version == TACTICAL_COMBAT_CONFIG_SCHEMA_VERSION)
        .then_some(())
        .ok_or(TacticalCombatConfigError::SchemaVersion)
}

impl TacticalCombatConfig {
    pub fn validate(&self) -> Result<(), TacticalCombatConfigError> {
        validate_schema_version(self.schema_version)?;
        let finite_nonnegative = |value: f32| value.is_finite() && value >= 0.0;
        self.resolution.validate()?;
        self.autoresolve.validate()?;
        let authority = &self.realtime_authority;
        let authority_values = [
            authority.defense.roll_dodge_effectiveness,
            authority.melee.range_latency_tolerance_metres,
            authority.melee.windup_jitter_fraction,
            authority.melee.lunge_range_window_metres,
            authority.melee.lunge_quickstep_threshold_metres,
            authority.ranged.range_latency_tolerance_metres,
            authority.ranged.aim_half_angle_degrees,
            authority.impact.whole_body_velocity_scale,
        ];
        if !authority_values.into_iter().all(finite_nonnegative) {
            return Err(TacticalCombatConfigError::Validation(
                "authority values must be finite and non-negative",
            ));
        }
        if authority.defense.roll_dodge_effectiveness > 1.0
            || authority.melee.windup_jitter_fraction > 1.0
            || authority.ranged.aim_half_angle_degrees > 180.0
            || authority.melee.completion_allowance_seconds.seconds() > 5.0
            || authority.ranged.completion_allowance_seconds.seconds() > 5.0
        {
            return Err(TacticalCombatConfigError::Validation(
                "authority values exceed compiled safety bounds",
            ));
        }
        let movement = &self.movement;
        let movement_values = [
            movement.jump_heights_metres.ordinary,
            movement.jump_heights_metres.dive,
            movement.prone_lateral_speed_scale,
            movement.prone_effort_scale,
            movement.motor.gravity_metres_per_second_squared,
            movement.motor.fallback_character_mass_kg,
            movement.motor.reference_ground_drive_force_newtons,
            movement.motor.reference_ground_braking_force_newtons,
            movement
                .motor
                .reference_quickstep_peak_horizontal_force_bodyweights,
            movement
                .motor
                .quickstep_peak_horizontal_force_bodyweights_per_strength,
            movement.motor.quickstep_push_seconds_reduction_per_agility,
            movement
                .motor
                .quickstep_maximum_supported_root_displacement_metres,
            movement
                .motor
                .reference_quickstep_target_displacement_metres,
            movement.motor.reference_quickstep_leg_length_metres,
            movement.motor.quickstep_takeoff_angle_degrees,
            movement.motor.reference_leg_strength,
            movement.motor.reference_leg_agility,
            movement.motor.reference_lateral_acceleration_gravities,
            movement.motor.reference_sprint_turn_radius_metres,
            movement.motor.agility_one_sprint_turn_radius_metres,
            movement.motor.agility_five_sprint_turn_radius_metres,
            movement.motor.traction_coefficient,
            movement.motor.slide_drag_coefficient,
            movement.motor.maximum_step_height_metres,
            movement.motor.maximum_walkable_slope_degrees,
        ];
        if !movement_values
            .into_iter()
            .all(|value| finite_nonnegative(value) && value > 0.0)
        {
            return Err(TacticalCombatConfigError::Validation(
                "movement values must be finite and positive",
            ));
        }
        if !finite_nonnegative(movement.motor.air_control_force_scale)
            || movement.motor.air_control_force_scale > 1.0
            || movement.motor.reference_lateral_acceleration_gravities > 2.0
            || movement.motor.traction_coefficient > 2.0
            || movement.motor.slide_drag_coefficient > 2.0
            || movement.motor.quickstep_takeoff_angle_degrees >= 45.0
            || movement.motor.maximum_step_height_metres > 1.0
            || movement.motor.maximum_walkable_slope_degrees >= 90.0
        {
            return Err(TacticalCombatConfigError::Validation(
                "character motor values exceed compiled safety bounds",
            ));
        }
        let quickstep_profile = movement.motor.quickstep_authored_displacement_profile;
        if quickstep_profile[0].abs() > 1.0e-6
            || (quickstep_profile[4] - 1.0).abs() > 1.0e-6
            || !quickstep_profile
                .into_iter()
                .all(|value| value.is_finite() && (0.0..=1.0).contains(&value))
            || !quickstep_profile.windows(2).all(|pair| pair[0] < pair[1])
        {
            return Err(TacticalCombatConfigError::Validation(
                "quickstep authored displacement profile must increase from zero to one",
            ));
        }
        let low_agility_radius = movement.motor.agility_one_sprint_turn_radius_metres;
        let reference_radius = movement.motor.reference_sprint_turn_radius_metres;
        let high_agility_radius = movement.motor.agility_five_sprint_turn_radius_metres;
        let turn_curve_linear = (high_agility_radius - low_agility_radius) * 0.5;
        let turn_curve_quadratic =
            (high_agility_radius + low_agility_radius) * 0.5 - reference_radius;
        if !(low_agility_radius > reference_radius && reference_radius > high_agility_radius)
            || turn_curve_linear + 2.0 * turn_curve_quadratic.abs() >= 0.0
        {
            return Err(TacticalCombatConfigError::Validation(
                "agility turn radii must define a strictly decreasing quadratic",
            ));
        }

        let ai = &self.ai.ordinary;
        if !ai.defense.dodge_chance.is_finite() || !(0.0..=1.0).contains(&ai.defense.dodge_chance) {
            return Err(TacticalCombatConfigError::Validation(
                "AI dodge probability must be between zero and one",
            ));
        }
        if ai.defense.reaction_delay_max_seconds < ai.defense.reaction_delay_min_seconds
            || !ai.offense.has_valid_intervals()
        {
            return Err(TacticalCombatConfigError::Validation("invalid AI interval"));
        }

        let input = &self.client_input;
        if ![
            input.movement_deadzone,
            input.roll_deadzone,
            input.quickstep_threshold,
            input.aim_trigger_threshold,
        ]
        .into_iter()
        .all(finite_nonnegative)
            || !input.gamepad_look_scale.into_iter().all(f32::is_finite)
            || input.movement_deadzone > 1.0
            || input.roll_deadzone > 1.0
            || input.quickstep_threshold > 1.0
            || input.aim_trigger_threshold > 1.0
        {
            return Err(TacticalCombatConfigError::Validation(
                "invalid client input value",
            ));
        }

        if !self.targeting.reported_hit_precision.is_finite()
            || self.targeting.body_part_hitboxes.len() != 7
        {
            return Err(TacticalCombatConfigError::Validation(
                "invalid targeting config",
            ));
        }
        let mut seen = [false; 7];
        for hitbox in &self.targeting.body_part_hitboxes {
            let index = body_part_index(hitbox.body_part);
            if seen[index]
                || !hitbox.center_metres.into_iter().all(f32::is_finite)
                || !hitbox
                    .half_extents_metres
                    .into_iter()
                    .all(|value| value.is_finite() && value > 0.0)
            {
                return Err(TacticalCombatConfigError::Validation(
                    "invalid body-part hitbox",
                ));
            }
            seen[index] = true;
        }

        let presentation = &self.presentation;
        if ![
            presentation.dodge_seconds,
            presentation
                .downed_turn_radians_per_second
                .radians_per_second(),
        ]
        .into_iter()
        .all(|value| finite_nonnegative(value) && value > 0.0)
        {
            return Err(TacticalCombatConfigError::Validation(
                "presentation timing must be finite and positive",
            ));
        }
        let curve = &presentation.attack_curve;
        if ![
            curve.inertia_characteristic,
            curve.inertia_weight,
            curve.skill_weight,
            curve.tell_base,
            curve.tell_span,
            curve.drawback_base,
            curve.drawback_span,
            curve.follow_through_base,
            curve.follow_through_span,
            curve.overshoot_base,
            curve.overshoot_span,
            curve.maximum_drawback,
            curve.maximum_overshoot,
        ]
        .into_iter()
        .all(finite_nonnegative)
            || curve.inertia_characteristic == 0.0
            || (curve.inertia_weight + curve.skill_weight - 1.0).abs() > 1.0e-6
        {
            return Err(TacticalCombatConfigError::Validation(
                "invalid attack curve",
            ));
        }

        let animation = self.animation;
        let initial_locomotion_speed = animation
            .playback
            .initial_locomotion_speed_metres_per_second;
        animation.locomotion.validate(initial_locomotion_speed)?;
        animation.state_transitions.validate()?;
        let guard = animation.guard_footwork;
        if ![
            guard.default_half_width_metres,
            guard.contact_margin_metres,
            guard.planning_reach_metres,
        ]
        .into_iter()
        .all(|value| finite_nonnegative(value) && value > 0.0)
            || guard.maximum_step_seconds < guard.minimum_step_seconds
            || guard.planning_reach_metres <= guard.contact_margin_metres
        {
            return Err(TacticalCombatConfigError::Validation(
                "invalid guard footwork tuning",
            ));
        }
        animation.playback.validate()?;
        animation.pose_buffer.validate()?;
        animation.procedural.validate()?;
        let bouncy = animation.bouncy_bones;
        if ![
            bouncy.impact_impulse_scale,
            bouncy.strength,
            bouncy.frequency_hz,
            bouncy.halflife_seconds,
            bouncy.coupling_per_second_squared,
            bouncy.transmission,
            bouncy.maximum_angle_radians,
        ]
        .into_iter()
        .all(finite_nonnegative)
            || bouncy.frequency_hz <= 0.0
            || bouncy.halflife_seconds <= 0.0
            || bouncy.transmission > 1.0
        {
            return Err(TacticalCombatConfigError::Validation(
                "invalid bouncy bone tuning",
            ));
        }
        let ik = animation.inverse_kinematics;
        if ![
            ik.maximum_owner_rotation_per_tick_degrees.degrees(),
            ik.maximum_knee_target_amplification.factor(),
            ik.continuity_sample_hz.hertz(),
            ik.knee_pole_maximum_foot_facing_offset_degrees.degrees(),
            ik.airborne_foot_rotation_speed_degrees_per_second
                .degrees_per_second(),
            ik.first_run_release_foot_rotation_speed_degrees_per_second
                .degrees_per_second(),
            ik.terrain_blend_speed_per_second.per_second(),
            ik.minimum_knee_flexion_degrees.degrees(),
            ik.minimum_terrain_knee_flexion_degrees.degrees(),
        ]
        .into_iter()
        .all(f32::is_finite)
            || ik.outer_foot_track_metres.metres()
                <= ik.minimum_inter_foot_separation_metres.metres() * 0.5
            || ik.terrain_blend_speed_per_second.per_second() < 0.0
            || ik.knee_pole_maximum_foot_facing_offset_degrees.degrees() < 0.0
            || ik.maximum_knee_target_amplification.factor() <= 0.0
            || ik.continuity_sample_hz.hertz() <= 0.0
        {
            return Err(TacticalCombatConfigError::Validation(
                "invalid inverse-kinematics tuning",
            ));
        }
        Ok(())
    }

    pub fn digest(&self) -> Result<String, TacticalCombatConfigError> {
        self.validate()?;
        let bytes = serde_json::to_vec(self)
            .map_err(|error| TacticalCombatConfigError::Serialization(error.to_string()))?;
        Ok(Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect())
    }
}

const fn body_part_index(part: BodyPart) -> usize {
    match part {
        BodyPart::LeftArm => 0,
        BodyPart::RightArm => 1,
        BodyPart::LeftLeg => 2,
        BodyPart::RightLeg => 3,
        BodyPart::Chest => 4,
        BodyPart::Stomach => 5,
        BodyPart::Head => 6,
    }
}

impl Default for TacticalCombatConfig {
    fn default() -> Self {
        defaults::canonical().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_config_is_valid_and_digestible() {
        let config = TacticalCombatConfig::default();
        config.validate().unwrap();
        assert_eq!(config.digest().unwrap().len(), 64);
    }

    #[test]
    fn duplicate_hitbox_is_rejected() {
        let mut config = TacticalCombatConfig::default();
        config.targeting.body_part_hitboxes[1].body_part = BodyPart::Head;
        assert!(config.validate().is_err());
    }

    #[test]
    fn nonphysical_combat_resolution_is_rejected() {
        let mut config = TacticalCombatConfig::default();
        config.resolution.armed_attack_energy_transfer = 0.0;
        assert!(config.validate().is_err());

        let mut config = TacticalCombatConfig::default();
        config.resolution.stagger_resistance_joules_per_kg = f32::INFINITY;
        assert!(config.validate().is_err());
    }

    #[test]
    fn invalid_autoresolve_parameters_are_rejected() {
        let mut config = TacticalCombatConfig::default();
        config.autoresolve.minimum_hit_precision = 1.1;
        assert!(config.validate().is_err());

        let mut config = TacticalCombatConfig::default();
        config.autoresolve.minimum_hit_precision = 0.9;
        config.autoresolve.maximum_hit_precision = 0.8;
        assert!(config.validate().is_err());
    }

    #[test]
    fn nonmonotonic_agility_turn_curve_is_rejected() {
        let mut config = TacticalCombatConfig::default();
        config.movement.motor.agility_one_sprint_turn_radius_metres = 4.0;
        config.movement.motor.agility_five_sprint_turn_radius_metres = 2.9;
        assert!(config.validate().is_err());
    }
}
