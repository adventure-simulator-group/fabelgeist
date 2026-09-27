//! Read-only equipment appearance shared by strategic HTTP and Bevy rendering.
use crate::item_catalog::{EquipmentChannel, EquipmentLocation};
use serde::{Deserialize, Serialize};

/// Decimal transport identity preserves the complete database integer range in JS.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PresentationId(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquipmentAppearance {
    pub id: PresentationId,
    pub item: String,
    pub placement: String,
    pub occupancies: Vec<AppearanceOccupancy>,
    pub weapon: Option<GeneratedAppearance>,
    pub holder: Option<GeneratedAppearance>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppearanceOccupancy {
    pub anchor: AppearanceAnchor,
    pub channel: EquipmentChannel,
    pub order: u16,
    pub requirement_index: u16,
    pub capacity_index: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppearanceAnchor {
    Character(EquipmentLocation),
    Attachment {
        parent: PresentationId,
        point: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneratedAppearance {
    pub generator_version: u16,
    pub design_hash: [u8; 32],
    pub recipe: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CharacterEquipmentAppearance {
    pub id: PresentationId,
    pub equipment: Vec<EquipmentAppearance>,
}
