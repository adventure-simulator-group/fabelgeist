use super::*;
use std::collections::{BTreeMap, BTreeSet};

use adventuresim_building_generator::signs::ShopName;
use adventuresim_world_schema::person_names::RenderedPersonalName;
use adventuresim_world_schema::settlement_buildings::BusinessId;

/// Immutable operator and display identity for one business in a tactical scene.
/// The mission snapshot owns `operator_name`; validation requires `shop_name`
/// to match that captured name and building use. Later missions recapture it.
#[derive(Clone, Debug, Eq, PartialEq, Component, Serialize, Deserialize)]
#[component(immutable)]
#[serde(deny_unknown_fields)]
pub struct SceneEstablishment {
    pub building_id: crate::scene_input::SceneBuildingId,
    pub business_id: BusinessId,
    pub operator_character_id: crate::player::CharacterId,
    pub operator_name: RenderedPersonalName,
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
        if establishment.building_id.0 == 0
            || establishment.operator_character_id.0 == 0
            || establishment.operator_name.as_str().is_empty()
        {
            return invalid(SceneValidationError::EstablishmentIdentity);
        }
        if !building_ids.insert(establishment.building_id)
            || !business_ids.insert(&establishment.business_id)
            || !operator_ids.insert(establishment.operator_character_id)
        {
            return invalid(SceneValidationError::EstablishmentDuplicate);
        }
        if settlement_id
            .replace(&establishment.business_id.settlement_id)
            .is_some_and(|expected| expected != establishment.business_id.settlement_id)
        {
            return invalid(SceneValidationError::EstablishmentSettlement);
        }
        let Some(usage) = buildings.get(&establishment.building_id).copied().flatten() else {
            return invalid(SceneValidationError::EstablishmentBuilding);
        };
        if usage != establishment.business_id.key.usage {
            return invalid(SceneValidationError::EstablishmentUsage);
        }
        if establishment.shop_name != ShopName::for_operator(&establishment.operator_name, usage) {
            return invalid(SceneValidationError::EstablishmentShopName);
        }
    }
    Ok(())
}
