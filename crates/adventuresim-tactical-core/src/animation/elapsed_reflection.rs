//! Bevy reflection of the canonical framework-free elapsed-seconds value.

use adventuresim_core::elapsed::AnimationElapsed;
use bevy::reflect::{ReflectDeserialize, ReflectSerialize, reflect_remote};
use serde::{Deserialize, Serialize};

#[reflect_remote(AnimationElapsed)]
/// Opaque reflection never exposes a mutable raw seconds field. Deserialization
/// delegates to AnimationElapsed's checked scalar admission.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
#[reflect(opaque)]
#[reflect(Serialize, Deserialize)]
pub struct ReflectedAnimationElapsed;
