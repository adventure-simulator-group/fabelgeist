//! Scalar f32 serialization port for the shared per-second rate.
use fabelgeist_shell::DampingRate;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub(super) fn serialize<S: Serializer>(
    rate: &DampingRate,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    f32::from(*rate).serialize(serializer)
}

pub(super) fn deserialize<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<DampingRate, D::Error> {
    f32::deserialize(deserializer).map(DampingRate::from)
}
