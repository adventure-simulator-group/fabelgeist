use super::*;
use std::collections::{BTreeMap, BTreeSet};

use adventuresim_building_generator::signs::ShopName;
use adventuresim_world_schema::settlement_buildings::BusinessId;

/// Immutable operator and display identity for one business in a tactical scene.
#[derive(Clone, Debug, Eq, PartialEq, Component, Serialize, Deserialize)]
#[component(immutable)]
#[serde(deny_unknown_fields)]
pub struct SceneEstablishment {
    pub building_id: u64,
    pub business_id: BusinessId,
    pub operator_character_id: u64,
    pub operator_name: String,
    pub shop_name: Option<ShopName>,
}

pub(super) fn validate(input: &TacticalSceneInput) -> Result<(), SceneInputError> {
    let buildings = input
        .buildings
        .iter()
        .map(|building| (building.id, building.program.usage))
        .chain(
            input
                .distant_buildings
                .iter()
                .map(|building| (building.id, building.usage)),
        )
        .collect::<BTreeMap<_, _>>();
    let mut building_ids = BTreeSet::new();
    let mut business_ids = BTreeSet::new();
    let mut operator_ids = BTreeSet::new();
    let mut settlement_id = None::<&str>;
    for establishment in &input.establishments {
        if establishment.building_id == 0
            || establishment.operator_character_id == 0
            || establishment.operator_name.trim().is_empty()
        {
            return invalid("establishment identity or operator is empty");
        }
        if !building_ids.insert(establishment.building_id)
            || !business_ids.insert(&establishment.business_id)
            || !operator_ids.insert(establishment.operator_character_id)
        {
            return invalid("establishment building, business, or operator is duplicated");
        }
        if settlement_id
            .replace(&establishment.business_id.settlement_id)
            .is_some_and(|expected| expected != establishment.business_id.settlement_id)
        {
            return invalid("establishments cross settlement boundaries");
        }
        let Some(usage) = buildings.get(&establishment.building_id).copied().flatten() else {
            return invalid("establishment references an unknown business building");
        };
        if usage != establishment.business_id.key.usage {
            return invalid("establishment business use does not match its building");
        }
        if establishment.shop_name != ShopName::for_operator(&establishment.operator_name, usage) {
            return invalid("establishment shop name does not match its operator and building use");
        }
    }
    Ok(())
}
