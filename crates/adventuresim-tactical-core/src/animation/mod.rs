//! Semantic animation state shared by the tactical authority and presentation client.
//!
//! The server synchronizes intent and timing through [`SkeletonState`]. Authored
//! clips and evaluated bone transforms remain client-only presentation.

use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    str::FromStr,
};

use adventuresim_core::combat_style::MeleeAttackStyle;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub use adventuresim_core::animation_pack_id::AnimationPackId;
pub use adventuresim_core::elapsed::AnimationElapsed;
pub use adventuresim_core::unit_interval::UnitInterval;
pub use authored_angle::{
    AngularMagnitudeDegrees, AngularSpeedDegrees, AngularSpeedRadians, SignedAngleDegrees,
    SignedAngularSpeedDegrees,
};
pub use cadence_limit::{AuthoredCadenceCap, CadenceDemand, CadenceSelection, StepCadence};
pub use calibration_parameters::{ContactHeightScale, PhaseCalibrationIntervals};
pub use contact::ContactSequence;
pub use continuation_parameters::{AttackContinuation, ContinuationReadiness, IncomingAttackTangent};
pub use elapsed_reflection::ReflectedAnimationElapsed;
pub use evaluation::*;
pub use event_span::{AnimationEventSpan, CoalescedEventBudget};
pub use fraction_reflection::ReflectedUnitInterval;
pub use ground_contact::GroundContact;
pub use height_measurement::{HeightDistance, HeightSpeed};
pub use ik_parameters::{KneeTargetAmplification, SettleProgress};
pub use landing::{LandingImpactSpeed, LandingSequence};
pub use locomotion::*;
pub use measurement::{NonnegativeLength, PositiveAnimationDuration};
pub use motion_modes::{DownedMotion, GuardedLocomotionMode, JumpAnticipation};
pub use packs::*;
pub use phase_response::{PhaseCorrectionRate, PhaseErrorLimit};
pub use pose_coordinates::{ActionProgressDelta, AttackPoseCoordinate};
pub use response_parameters::{
    AngularStiffness, ImpactCompressionCompliance, InertialLeanGain, LookDistribution, MuscleTone,
    ReferenceAcceleration, ResponseRate,
};
pub use sample_rate::{AnimationFrequency, AnimationSampleRate};
pub use sample_tick::{AnimationSampleSpan, AnimationSampleTick};
pub use sample_values::{BlendWeight, ClipMirror, CycleDelta, CyclePhase};
pub use semantic::*;
pub use shaping_time::ResponseShapingTime;
pub use skeleton_family::SkeletonFamily;
pub use state::*;
pub use value_error::{AnimationQuantity, AnimationValueError, AnimationValueResult};

use adventuresim_core::linear_speed::NonnegativeSpeed;

mod attack_commitment;
mod authored_angle;
mod cadence_limit;
mod calibration_parameters;
mod contact;
mod continuation_parameters;
mod elapsed_reflection;
mod evaluation;
mod event_span;
mod fraction_reflection;
mod ground_contact;
mod height_measurement;
mod ik_parameters;
mod landing;
mod locomotion;
mod measurement;
mod motion_modes;
mod packs;
mod phase_response;
mod pose_coordinates;
mod response_parameters;
mod sample_rate;
mod sample_tick;
mod sample_values;
mod semantic;
mod shaping_time;
mod skeleton_family;
mod state;
mod value_error;

#[cfg(test)]
use evaluation::locomotion::gait_pair;

include!("tests.rs");
