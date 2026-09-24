//! Shared forge quotation, including the inventory representation's limits.
use crate::inventory_measurement::ConsumableFractionMicros;
use adventuresim_weapon_model::{WeaponDesign, derive_material_masses, derive_properties};
use serde::Serialize;
use std::collections::BTreeMap;

const FORGE_MINUTES_BASE: u64 = 60;
const FORGE_MINUTES_PER_KILOGRAM: f32 = 120.0;
const FORGE_MINUTES_PER_COMPONENT: u64 = 12;

#[derive(Debug, Serialize)]
pub struct ForgeQuote {
    pub minutes: u64,
    pub materials: BTreeMap<String, f32>,
    /// Required inventory fractions, rounded up per construction material.
    pub requirements: BTreeMap<String, u32>,
}

pub fn quote_weapon(design: &WeaponDesign) -> Result<ForgeQuote, String> {
    let physical = derive_properties(design).map_err(|errors| format!("{errors:?}"))?;
    let mut quote = ForgeQuote {
        minutes: FORGE_MINUTES_BASE
            + (physical.mass_kg * FORGE_MINUTES_PER_KILOGRAM).ceil() as u64
            + design.recipe.components.len() as u64 * FORGE_MINUTES_PER_COMPONENT,
        materials: BTreeMap::new(),
        requirements: BTreeMap::new(),
    };
    for mass in derive_material_masses(design).map_err(|errors| format!("{errors:?}"))? {
        let stock = mass
            .material
            .forge_stock()
            .ok_or_else(|| format!("No forge stock is traded for {:?}", mass.material))?;
        let required_micros = (f64::from(mass.mass_kg) / f64::from(stock.unit_mass_kg())
            * f64::from(ConsumableFractionMicros::MICROS_PER_WHOLE))
        .ceil();
        if !required_micros.is_finite() || required_micros > f64::from(u32::MAX) {
            return Err("Weapon material requirement is outside the supported range".into());
        }
        let total = quote
            .requirements
            .entry(stock.item_id().into())
            .or_default();
        *total = total
            .checked_add(required_micros as u32)
            .ok_or("Weapon material requirement is outside the supported range")?;
        *quote.materials.entry(stock.item_id().into()).or_default() += mass.mass_kg;
    }
    Ok(quote)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_stock_overflow_rejects_preview_and_transaction_quote() {
        let design: WeaponDesign = serde_json::from_value(serde_json::json!({
            "catalog_id":"arming_sword","recipe":{"components":[
                {"id":"first","kind":"box","size":[1.0,2.0,1.0],"material":"steel","attach":{"to":"weapon.root"}},
                {"id":"second","kind":"box","size":[1.0,2.0,1.0],"material":"darkSteel","attach":{"to":"weapon.root"}}
            ]}
        })).unwrap();
        assert!(adventuresim_weapon_model::generate(&design).is_ok());
        assert!(quote_weapon(&design).is_err());
    }

    #[test]
    fn recipes_drive_material_and_time_quotes() {
        let short =
            quote_weapon(&adventuresim_weapon_model::default_design("rondel_dagger").unwrap())
                .unwrap();
        let long =
            quote_weapon(&adventuresim_weapon_model::default_design("longsword").unwrap()).unwrap();
        assert_ne!(short.requirements, long.requirements);
        assert_ne!(short.minutes, long.minutes);
        assert!(long.materials.values().all(|mass| *mass > 0.0));
    }
}
