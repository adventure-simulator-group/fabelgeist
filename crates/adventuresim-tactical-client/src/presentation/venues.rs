//! Venue identities and building selection shared by generation and live views.
use adventuresim_tactical_core::scene_input::TacticalSceneInput;
use adventuresim_world_schema::settlement_buildings::BuildingUse;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash)]
pub(crate) struct PlaceId(pub String);

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Hash)]
#[serde(try_from = "String")]
pub(crate) struct PortraitId(pub u64);

impl TryFrom<String> for PortraitId {
    type Error = std::num::ParseIntError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse().map(Self)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum PlaceKind {
    Square,
    Residence,
    Keep,
    Market,
    Smith,
    Armor,
    Tailor,
    Apothecary,
    Books,
    Inn,
    Church,
    Guild,
    Camp,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub(crate) struct Place {
    pub id: PlaceId,
    pub kind: PlaceKind,
}

fn building_use(kind: PlaceKind) -> BuildingUse {
    match kind {
        PlaceKind::Square => BuildingUse::WeighHouse,
        PlaceKind::Market => BuildingUse::GeneralShop,
        PlaceKind::Residence | PlaceKind::Camp => BuildingUse::Dwelling,
        PlaceKind::Keep => BuildingUse::Guardhouse,
        PlaceKind::Smith => BuildingUse::Weaponsmith,
        PlaceKind::Armor => BuildingUse::Armorer,
        PlaceKind::Tailor => BuildingUse::Tailor,
        PlaceKind::Apothecary => BuildingUse::Herbalist,
        PlaceKind::Books => BuildingUse::Bookshop,
        PlaceKind::Inn => BuildingUse::Inn,
        PlaceKind::Church => BuildingUse::ParishChurch,
        PlaceKind::Guild => BuildingUse::Guildhall,
    }
}

pub(crate) fn select_building(
    input: &TacticalSceneInput,
    kind: PlaceKind,
    operators: impl Iterator<Item = u64>,
) -> Option<adventuresim_tactical_core::scene_input::SceneBuildingId> {
    let operators: Vec<_> = operators.collect();
    let usage = building_use(kind);
    input
        .establishments
        .iter()
        .find(|e| operators.contains(&e.operator_character_id.0))
        .map(|e| e.building_id)
        .or_else(|| {
            input
                .buildings
                .iter()
                .find(|b| b.program.usage == Some(usage))
                .map(|b| b.id)
        })
        .or_else(|| {
            input
                .distant_buildings
                .iter()
                .find(|b| b.usage == Some(usage))
                .map(|b| b.id)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn character_identity_survives_javascript_integer_limit() {
        let id: PortraitId = serde_json::from_str("\"18446744073709551615\"").unwrap();
        assert_eq!(id.0, u64::MAX);
        assert!(serde_json::from_str::<PortraitId>("42").is_err());
        assert!(serde_json::from_str::<PortraitId>("\"someone\"").is_err());
    }
}
