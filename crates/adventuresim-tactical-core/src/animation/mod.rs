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
pub use authored_angle::{AngularMagnitudeDegrees, AngularSpeedDegrees, AngularSpeedRadians};
pub use contact::ContactSequence;
pub use elapsed_reflection::ReflectedAnimationElapsed;
pub use evaluation::*;
pub use ground_contact::GroundContact;
pub use height_measurement::{HeightDistance, HeightSpeed};
pub use landing::{LandingImpactSpeed, LandingSequence};
pub use locomotion::*;
pub use measurement::{NonnegativeLength, PositiveAnimationDuration};
pub use motion_modes::{DownedMotion, GuardedLocomotionMode, JumpAnticipation};
pub use packs::*;
pub use response_parameters::{
    AngularStiffness, ImpactCompressionCompliance, InertialLeanGain, LookDistribution, MuscleTone,
    ReferenceAcceleration, ResponseRate,
};
pub use sample_rate::{AnimationFrequency, AnimationSampleRate};
pub use sample_tick::{AnimationSampleSpan, AnimationSampleTick};
pub use sample_values::{BlendWeight, ClipMirror, CycleDelta, CyclePhase};
pub use semantic::*;
pub use skeleton_family::SkeletonFamily;
pub use state::*;
pub use value_error::{AnimationQuantity, AnimationValueError, AnimationValueResult};

use adventuresim_core::linear_speed::NonnegativeSpeed;

mod attack_commitment;
mod authored_angle;
mod contact;
mod elapsed_reflection;
mod evaluation;
mod ground_contact;
mod height_measurement;
mod landing;
mod locomotion;
mod measurement;
mod motion_modes;
mod packs;
mod response_parameters;
mod sample_rate;
mod sample_tick;
mod sample_values;
mod semantic;
mod skeleton_family;
mod state;
mod value_error;

#[cfg(test)]
use evaluation::locomotion::gait_pair;

include!("tests.rs");
