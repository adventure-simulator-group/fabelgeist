//! Strategic identity projected as an immutable tactical component.
use bevy::prelude::*;
use fabelgeist_determinism::StreamId;
use serde::{Deserialize, Serialize};

/// Strategic character identity projected into the transient tactical world.
/// Network client identity remains a separate transport concern.
#[derive(
    Component, Serialize, Deserialize, Default, Debug, Reflect, Clone, Copy, PartialEq, Eq, Hash,
)]
#[reflect(opaque)]
#[reflect(Component, Serialize, Deserialize)]
#[serde(transparent)]
#[component(immutable)]
pub struct CharacterId(pub adventuresim_core::identity::CharacterId);

impl From<u64> for CharacterId {
    fn from(value: u64) -> Self {
        Self(value.into())
    }
}

impl CharacterId {
    pub fn get(self) -> u64 {
        u64::from(self.0)
    }
}

impl CharacterId {
    /// Get associated color of this player.
    pub fn color(&self) -> Color {
        let mut random = StreamId::new("character.display-color").rng(self.get(), &[]);
        let hue = random.index(360) as f32;
        let saturation = 0.28 + random.inclusive_unit_f32() * 0.18;
        let value = 0.90 + random.inclusive_unit_f32() * 0.08;

        Color::hsv(hue, saturation, value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::reflect::{
        TypeRegistry,
        serde::{TypedReflectDeserializer, TypedReflectSerializer},
    };
    use serde::de::DeserializeSeed;

    #[test]
    fn opaque_identity_reflection_preserves_scalar_encoding_and_admission() {
        let mut registry = TypeRegistry::new();
        registry.register::<CharacterId>();
        for value in [0, 7, u64::MAX] {
            let identity = CharacterId::from(value);
            let encoded =
                serde_json::to_string(&TypedReflectSerializer::new(&identity, &registry)).unwrap();
            assert_eq!(encoded, serde_json::to_string(&value).unwrap());
            let reflected = TypedReflectDeserializer::of::<CharacterId>(&registry)
                .deserialize(&mut serde_json::Deserializer::from_str(&encoded))
                .unwrap();
            assert_eq!(
                CharacterId::from_reflect(reflected.as_ref()),
                Some(identity)
            );
        }
        for invalid in ["-1", "1.5", "\"7\""] {
            assert!(
                TypedReflectDeserializer::of::<CharacterId>(&registry)
                    .deserialize(&mut serde_json::Deserializer::from_str(invalid))
                    .is_err()
            );
        }
    }
}
