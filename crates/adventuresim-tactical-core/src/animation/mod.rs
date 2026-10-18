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

pub use elapsed::AnimationElapsed;
pub use evaluation::*;
pub use landing::{LandingImpactSpeed, LandingSequence};
pub use locomotion::*;
pub use pack_id::AnimationPackId;
pub use packs::*;
pub use sample_tick::AnimationSampleTick;
pub use semantic::*;
pub use state::*;
pub use value_error::{AnimationQuantity, AnimationValueError, AnimationValueResult};

mod attack_commitment;
mod elapsed;
mod evaluation;
mod landing;
mod locomotion;
mod pack_id;
mod packs;
mod sample_tick;
mod semantic;
mod state;
mod value_error;

#[cfg(test)]
use evaluation::gait_pair;

include!("tests.rs");
