//! Strict typed armor recipe overrides shared by preview and both export paths.
use crate::armor_recipes::{self, ParametricDesign};
use anyhow::{Context, Result, ensure};
use std::{collections::BTreeMap, path::Path};

pub type ArmorDesigns = BTreeMap<String, ParametricDesign>;

/// Use the same schema and parameter checks for saved edits and loaded recipes.
pub fn encode(designs: &ArmorDesigns) -> Result<Vec<u8>> {
    let bytes = serde_json::to_vec_pretty(designs)?;
    parse(&bytes)?;
    Ok(bytes)
}

pub fn load(path: Option<&Path>) -> Result<ArmorDesigns> {
    let Some(path) = path else {
        return Ok(BTreeMap::new());
    };
    parse(&std::fs::read(path)?)
        .with_context(|| format!("reading armor design recipes {}", path.display()))
}

fn parse(bytes: &[u8]) -> Result<ArmorDesigns> {
    let designs = decode(bytes)?;
    for (id, design) in &designs {
        let default = armor_recipes::recipe(id)
            .with_context(|| format!("unknown parametric armor recipe {id}"))?;
        let same_family = match (&default, design) {
            (ParametricDesign::Helmet(a), ParametricDesign::Helmet(b)) => {
                std::mem::discriminant(a) == std::mem::discriminant(b)
            }
            (ParametricDesign::Limb(a), ParametricDesign::Limb(b)) => {
                std::mem::discriminant(a) == std::mem::discriminant(b)
            }
            (ParametricDesign::Garment(a), ParametricDesign::Garment(b)) => a.kind == b.kind,
            _ => false,
        };
        ensure!(
            same_family,
            "recipe {id} must retain its historical construction family"
        );
    }
    Ok(designs)
}

/// Decode the shared schema before applying override-specific family checks.
pub(crate) fn decode(bytes: &[u8]) -> Result<ArmorDesigns> {
    let designs: ArmorDesigns = serde_json::from_slice(bytes)?;
    reject_unknown_fields(
        &serde_json::from_slice(bytes)?,
        &serde_json::to_value(&designs)?,
    )?;
    for (id, design) in &designs {
        match design {
            ParametricDesign::Helmet(d) => d.validate().map_err(anyhow::Error::new),
            ParametricDesign::Limb(d) => d.validate().map_err(anyhow::Error::new),
            ParametricDesign::Garment(d) => d.validate().map_err(anyhow::Error::new),
        }
        .with_context(|| format!("invalid armor design for item {id}"))?;
    }
    Ok(designs)
}

fn reject_unknown_fields(input: &serde_json::Value, decoded: &serde_json::Value) -> Result<()> {
    if let Some(fields) = input.as_object() {
        for (name, value) in fields {
            let accepted = decoded
                .get(name)
                .with_context(|| format!("unknown armor design field {name}"))?;
            reject_unknown_fields(value, accepted)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_edits_cannot_be_saved_as_loadable_recipes() {
        let mut designs = ArmorDesigns::new();
        let mut helmet = adventuresim_armor_model::CloseHelmetDesign::default();
        helmet.breaths.count_per_row = 8;
        helmet.breaths.span = adventuresim_armor_model::Millimeters(20);
        designs.insert(
            "close_helmet".into(),
            ParametricDesign::Helmet(adventuresim_armor_model::HelmetDesign::CloseHelmet(helmet)),
        );
        assert!(encode(&designs).is_err());
    }

    #[test]
    fn rejects_misspelled_nested_controls_and_wrong_construction_families() {
        let design = armor_recipes::recipe("morion").unwrap();
        let mut document = serde_json::json!({"morion": design});
        document["morion"]["Helmet"]["Morion"]["fit"]["clearence"] = serde_json::json!(8);
        assert!(parse(&serde_json::to_vec(&document).unwrap()).is_err());
        let swapped = serde_json::json!({"morion": armor_recipes::recipe("barbute").unwrap()});
        assert!(parse(&serde_json::to_vec(&swapped).unwrap()).is_err());
    }

    #[test]
    fn every_authored_family_round_trips_through_validated_input() {
        let catalog: crate::item_catalog_schema::ItemCatalogDocument =
            serde_json::from_str(include_str!("../../../content/items/catalog.yaml")).unwrap();
        let designs: ArmorDesigns = catalog
            .items
            .iter()
            .filter_map(|item| {
                armor_recipes::recipe(&item.id).map(|design| (item.id.clone(), design))
            })
            .collect();
        let encoded = serde_json::to_vec(&designs).unwrap();
        assert_eq!(
            serde_json::to_value(parse(&encoded).unwrap()).unwrap(),
            serde_json::to_value(designs).unwrap()
        );
    }
}
