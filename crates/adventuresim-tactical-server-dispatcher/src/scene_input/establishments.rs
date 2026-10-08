//! Exact settlement business and operator bindings for occupied scenes.
use super::*;

pub(super) fn bind_establishments(
    settlement: Option<&SettlementSceneProfile>,
    layout: &adventuresim_tactical_core::city_layout::CitySceneLayout,
) -> Result<Vec<SceneEstablishment>, String> {
    let Some(settlement) = settlement else {
        return Ok(Vec::new());
    };
    let mut operators = BTreeMap::new();
    for operator in &settlement.operators {
        if operator.business_id.settlement_id != settlement.id {
            return Err("business operator crosses the requested settlement boundary".into());
        }
        if operators
            .insert(operator.business_id.key, operator)
            .is_some()
        {
            return Err("business operator identity is duplicated".into());
        }
    }
    let mut establishments = Vec::with_capacity(layout.businesses.len());
    for site in &layout.businesses {
        let operator = operators
            .remove(&site.key)
            .ok_or("placed business is missing its resident operator")?;
        let operator_name = operator.operator_name.clone();
        establishments.push(SceneEstablishment {
            building_id: site.building_id,
            business_id: operator.business_id.clone(),
            operator_character_id: operator.operator_character_id,
            operator_name: operator_name.clone(),
            shop_name: ShopName::for_operator(&operator_name, site.key.usage),
        });
    }
    if !operators.is_empty() {
        return Err("business operator has no placed establishment".into());
    }
    establishments.sort_by_key(|establishment| establishment.building_id);
    Ok(establishments)
}
