//! Bevy reflection of the canonical framework-free unit-fraction value.

use adventuresim_core::unit_interval::UnitInterval;
use bevy::reflect::{ReflectDeserialize, ReflectSerialize, reflect_remote};
use serde::{Deserialize, Serialize};

#[reflect_remote(UnitInterval)]
/// Opaque reflection never exposes a mutable raw fraction field. Deserialization
/// delegates to UnitInterval's checked scalar admission.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
#[reflect(opaque)]
#[reflect(Serialize, Deserialize)]
pub struct ReflectedUnitInterval;
