//! The creator stores substep cardinalities as native scalar JSON values.
use fabelgeist_shell::SubstepCount;
use serde::{Deserialize, Deserializer, Serializer};

pub(super) fn serialize<S: Serializer>(
    count: &SubstepCount,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_u32(u32::from(*count))
}

pub(super) fn deserialize<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<SubstepCount, D::Error> {
    u32::deserialize(deserializer).map(SubstepCount::from)
}
