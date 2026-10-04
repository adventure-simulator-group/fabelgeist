use adventuresim_core::item_catalog_schema::EquipmentMaterial;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Reflect, Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
#[reflect(opaque)]
#[reflect(Component, Serialize, Deserialize)]
pub struct ArmorItem {
    pub material: EquipmentMaterial,
    pub range_of_motion: f32,
    pub coverage: f32,
    pub slot: ArmorSlot,
    pub resistance: f32,
    pub padding: f32,
    pub flexibility: f32,
    pub covered_parts: [bool; 7],
    #[reflect(ignore)]
    pub coverage_geometry: [Option<adventuresim_core::combat::AuthoredArmorCoverage>; 7],
    /// Higher authored equipment channels are physically farther from tissue.
    pub layer_order: u8,
}

/// Stable strategic inventory identity retained on the transient tactical
/// projection so contact consequences can name the exact engaged item.
#[derive(Component, Reflect, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[reflect(opaque)]
#[reflect(Component, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TacticalInventoryItemId(pub adventuresim_core::identity::InventoryItemId);

impl From<u64> for TacticalInventoryItemId {
    fn from(value: u64) -> Self {
        Self(value.into())
    }
}

impl TacticalInventoryItemId {
    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct ArmorLayerContact {
    pub item_id: String,
    pub inventory_item_id: Option<u64>,
    pub material: EquipmentMaterial,
    pub geometry: adventuresim_core::combat::AuthoredArmorCoverage,
    pub intersected: bool,
    pub selected: bool,
    pub surface: adventuresim_core::equipment::ArmorSurface,
}

#[derive(Reflect, Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum ArmorSlot {
    Arms(Option<ArmorSide>),
    Legs(Option<ArmorSide>),
    Head,
    Chest,
    Stomach,
}

#[derive(Reflect, Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum ArmorSide {
    Left,
    Right,
}

#[cfg(test)]
mod identity_reflection_tests {
    use super::*;
    use bevy::reflect::{
        TypeRegistry,
        serde::{TypedReflectDeserializer, TypedReflectSerializer},
    };
    use serde::de::DeserializeSeed;

    #[test]
    fn opaque_inventory_identity_reflection_preserves_scalar_encoding_and_admission() {
        let mut registry = TypeRegistry::new();
        registry.register::<TacticalInventoryItemId>();
        for value in [0, 99, u64::MAX] {
            let identity = TacticalInventoryItemId::from(value);
            let encoded =
                serde_json::to_string(&TypedReflectSerializer::new(&identity, &registry)).unwrap();
            assert_eq!(encoded, serde_json::to_string(&value).unwrap());
            let reflected = TypedReflectDeserializer::of::<TacticalInventoryItemId>(&registry)
                .deserialize(&mut serde_json::Deserializer::from_str(&encoded))
                .unwrap();
            assert_eq!(
                TacticalInventoryItemId::from_reflect(reflected.as_ref()),
                Some(identity)
            );
        }
        for invalid in ["-1", "1.5", "\"99\""] {
            assert!(
                TypedReflectDeserializer::of::<TacticalInventoryItemId>(&registry)
                    .deserialize(&mut serde_json::Deserializer::from_str(invalid))
                    .is_err()
            );
        }
    }
}
