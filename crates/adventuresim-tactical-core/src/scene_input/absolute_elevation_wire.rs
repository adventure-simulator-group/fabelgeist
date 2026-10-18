//! Scalar metre document port for the checked world elevation owner.
use adventuresim_world_schema::ElevationMeters;
use serde::{Deserialize, Deserializer, Serializer};
pub fn serialize<S: Serializer>(value: &ElevationMeters, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_i16(value.get())
}
pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<ElevationMeters, D::Error> {
    ElevationMeters::new(i16::deserialize(d)?)
        .ok_or_else(|| serde::de::Error::custom("absolute scene elevation is outside world bounds"))
}
