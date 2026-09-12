//! Strict typed armor recipe overrides shared by preview and both export paths.
use crate::armor_recipes::{self, ParametricDesign};
use anyhow::{Context, Result, ensure};
use std::{collections::BTreeMap, path::Path};

use serde::{Deserialize, Serialize};

/// Equipment placements accepted by the parametric armor catalog.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArmorPlacement {
    Left,
    Right,
    Worn,
}

impl ArmorPlacement {
    pub fn parse(value: &str) -> Option<Self> {
        Self::deserialize(serde::de::value::StrDeserializer::<serde::de::value::Error>::new(value))
            .ok()
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
            Self::Worn => "worn",
        }
    }
}

/// Saved edits: shared item defaults and explicit placement-specific choices.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArmorDesigns {
    pub defaults: BTreeMap<String, ParametricDesign>,
    pub placements: BTreeMap<String, BTreeMap<ArmorPlacement, ParametricDesign>>,
}

impl ArmorDesigns {
    pub fn selected(&self, id: &str, placement: ArmorPlacement) -> Option<&ParametricDesign> {
        self.placements
            .get(id)
            .and_then(|choices| choices.get(&placement))
            .or_else(|| self.defaults.get(id))
    }
}

/// Use the same schema and parameter checks for saved edits and loaded recipes.
pub fn encode(designs: &ArmorDesigns) -> Result<Vec<u8>> {
    let bytes = serde_json::to_vec_pretty(designs)?;
    parse(&bytes)?;
    Ok(bytes)
}

pub fn load(path: Option<&Path>) -> Result<ArmorDesigns> {
    let Some(path) = path else {
        return Ok(ArmorDesigns::default());
    };
    parse(&std::fs::read(path)?)
        .with_context(|| format!("reading armor design recipes {}", path.display()))
}

fn parse(bytes: &[u8]) -> Result<ArmorDesigns> {
    let designs: ArmorDesigns = serde_json::from_slice(bytes)?;
    reject_unknown_fields(
        &serde_json::from_slice(bytes)?,
        &serde_json::to_value(&designs)?,
    )?;
    let catalog: crate::item_catalog_schema::ItemCatalogDocument =
        serde_json::from_str(include_str!("../../../content/items/catalog.yaml"))?;
    for (id, choices) in &designs.placements {
        ensure!(
            armor_recipes::recipe(id).is_some(),
            "unknown parametric armor recipe {id}"
        );
        let item = catalog
            .items
            .iter()
            .find(|item| item.id == *id)
            .with_context(|| format!("unknown armor item {id}"))?;
        let equipment = item
            .equipment
            .as_ref()
            .context("armor item has no placements")?;
        for placement in choices.keys() {
            ensure!(
                equipment
                    .placements
                    .iter()
                    .any(|p| p.id == placement.as_str()),
                "armor item {id} has no {} placement",
                placement.as_str()
            );
        }
    }
    for (id, design) in designs.defaults.iter().chain(
        designs
            .placements
            .iter()
            .flat_map(|(id, choices)| choices.values().map(move |design| (id, design))),
    ) {
        validate_design(id, design)?;
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
            (ParametricDesign::Underlayer(a), ParametricDesign::Underlayer(b)) => a.kind == b.kind,
            (ParametricDesign::WaistAssembly(_), ParametricDesign::WaistAssembly(_)) => true,
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
pub(crate) fn decode(bytes: &[u8]) -> Result<BTreeMap<String, ParametricDesign>> {
    let designs: BTreeMap<String, ParametricDesign> = serde_json::from_slice(bytes)?;
    reject_unknown_fields(
        &serde_json::from_slice(bytes)?,
        &serde_json::to_value(&designs)?,
    )?;
    for (id, design) in &designs {
        validate_design(id, design)?;
    }
    Ok(designs)
}

fn validate_design(id: &str, design: &ParametricDesign) -> Result<()> {
    match design {
        ParametricDesign::Helmet(d) => d.validate().map_err(anyhow::Error::new),
        ParametricDesign::Limb(d) => d.validate().map_err(anyhow::Error::new),
        ParametricDesign::Garment(d) => d.validate().map_err(anyhow::Error::new),
        ParametricDesign::Underlayer(d) => d.validate(),
        ParametricDesign::WaistAssembly(d) => d.validate().map_err(anyhow::Error::new),
    }
    .with_context(|| format!("invalid armor design for item {id}"))
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
    fn museum_controls_round_trip_in_defaults_and_independent_placement_recipes() {
        use adventuresim_armor_model::*;
        let default = armor_recipes::recipe("spaulder").unwrap();
        let mut left = default.clone();
        let ParametricDesign::Limb(LimbArmorDesign::Spaulder(d)) = &mut left else {
            panic!()
        };
        d.crown_coverage = Permille(200);
        let burgonet = ParametricDesign::Helmet(HelmetDesign::Burgonet(BurgonetDesign {
            peak_rise: Millimeters(20),
            buffe: Some(BuffeDesign {
                breaths: Some(VisorBreaths::buffe()),
                courses: Some(BuffeCourses::default()),
                chin_width: Permille(500),
                ridge_sharpness: Permille(1000),
                ..Default::default()
            }),
            ..Default::default()
        }));
        let mut tassets = armor_recipes::recipe("tassets").unwrap();
        let ParametricDesign::WaistAssembly(d) = &mut tassets else {
            panic!()
        };
        d.tassets.plate_shape = GarmentPlateShape::WrappedTassets(WrappedTassetDesign {
            inner_gap: Millimeters(80),
            section_break: 0,
            ..Default::default()
        });
        let choices = ArmorDesigns {
            defaults: BTreeMap::from([
                ("spaulder".into(), default.clone()),
                ("burgonet".into(), burgonet),
                ("tassets".into(), tassets),
            ]),
            placements: BTreeMap::from([(
                "spaulder".into(),
                BTreeMap::from([(ArmorPlacement::Left, left.clone())]),
            )]),
        };
        let restored = parse(&super::encode(&choices).unwrap()).unwrap();
        assert_eq!(
            serde_json::to_value(&choices).unwrap(),
            serde_json::to_value(&restored).unwrap()
        );
        assert_eq!(
            serde_json::to_value(restored.selected("spaulder", ArmorPlacement::Right)).unwrap(),
            serde_json::to_value(default).unwrap()
        );
        let mut bad = restored;
        let ParametricDesign::Limb(LimbArmorDesign::Spaulder(d)) = bad
            .placements
            .get_mut("spaulder")
            .unwrap()
            .get_mut(&ArmorPlacement::Left)
            .unwrap()
        else {
            panic!()
        };
        d.crown_coverage = Permille(199);
        assert!(super::encode(&bad).is_err());
    }

    #[test]
    fn placement_edit_round_trips_and_keeps_other_side_on_item_default() {
        use adventuresim_armor_model::{LimbArmorDesign, Millimeters};
        let default = armor_recipes::recipe("pauldron").unwrap();
        let mut left = default.clone();
        let ParametricDesign::Limb(LimbArmorDesign::Pauldron(d)) = &mut left else {
            panic!()
        };
        d.outline.front_extension = Millimeters(65);
        d.outline.front_return = adventuresim_armor_model::Milliradians(1800);
        d.outline.rear_return = adventuresim_armor_model::Milliradians(2800);
        let designs = ArmorDesigns {
            defaults: BTreeMap::from([("pauldron".into(), default.clone())]),
            placements: BTreeMap::from([(
                "pauldron".into(),
                BTreeMap::from([(ArmorPlacement::Left, left.clone())]),
            )]),
        };
        let restored = parse(&encode(&designs).unwrap()).unwrap();
        assert_eq!(
            serde_json::to_value(restored.selected("pauldron", ArmorPlacement::Left)).unwrap(),
            serde_json::to_value(&left).unwrap()
        );
        assert_eq!(
            serde_json::to_value(restored.selected("pauldron", ArmorPlacement::Right)).unwrap(),
            serde_json::to_value(&default).unwrap()
        );
        let frame = adventuresim_armor_model::PartFrame {
            origin: [0.0; 3],
            axes: [[0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            half_extents: [0.070, 0.085, 0.065],
        };
        let selected = |placement| {
            restored
                .selected("pauldron", placement)
                .unwrap()
                .generate(&frame)
                .unwrap()
        };
        assert_ne!(
            selected(ArmorPlacement::Left).positions,
            selected(ArmorPlacement::Right).positions
        );
    }

    #[test]
    fn rejects_unknown_placements_items_and_placement_family_swaps() {
        let pauldron = armor_recipes::recipe("pauldron").unwrap();
        for placements in [
            serde_json::json!({"pauldron": {"front": pauldron}}),
            serde_json::json!({"morion": {"left": armor_recipes::recipe("morion").unwrap()}}),
            serde_json::json!({"unknown": {"left": pauldron}}),
            serde_json::json!({"pauldron": {"left": armor_recipes::recipe("spaulder").unwrap()}}),
        ] {
            let document = serde_json::json!({"defaults": {}, "placements": placements});
            assert!(parse(&serde_json::to_vec(&document).unwrap()).is_err());
        }
    }

    #[test]
    fn absent_placement_edits_preserve_default_selection() {
        let default = armor_recipes::recipe("pauldron").unwrap();
        let designs = ArmorDesigns {
            defaults: BTreeMap::from([("pauldron".into(), default)]),
            placements: BTreeMap::new(),
        };
        assert!(std::ptr::eq(
            designs.selected("pauldron", ArmorPlacement::Left).unwrap(),
            designs.selected("pauldron", ArmorPlacement::Right).unwrap()
        ));
        assert!(
            ArmorDesigns::default()
                .selected("pauldron", ArmorPlacement::Left)
                .is_none()
        );
    }

    #[test]
    fn invalid_edits_cannot_be_saved_as_loadable_recipes() {
        let mut designs = ArmorDesigns::default();
        let mut helmet = adventuresim_armor_model::CloseHelmetDesign::default();
        helmet.breaths.count_per_row = 8;
        helmet.breaths.span = adventuresim_armor_model::Millimeters(20);
        designs.defaults.insert(
            "close_helmet".into(),
            ParametricDesign::Helmet(adventuresim_armor_model::HelmetDesign::CloseHelmet(helmet)),
        );
        assert!(encode(&designs).is_err());
    }

    #[test]
    fn rejects_misspelled_nested_controls_and_wrong_construction_families() {
        let design = armor_recipes::recipe("morion").unwrap();
        let mut document = serde_json::json!({"defaults": {"morion": design}, "placements": {}});
        document["defaults"]["morion"]["Helmet"]["Morion"]["fit"]["clearence"] =
            serde_json::json!(8);
        assert!(parse(&serde_json::to_vec(&document).unwrap()).is_err());
        let swapped = serde_json::json!({"defaults": {"morion": armor_recipes::recipe("barbute").unwrap()}, "placements": {}});
        assert!(parse(&serde_json::to_vec(&swapped).unwrap()).is_err());
    }

    #[test]
    fn every_authored_family_round_trips_through_validated_input() {
        let catalog: crate::item_catalog_schema::ItemCatalogDocument =
            serde_json::from_str(include_str!("../../../content/items/catalog.yaml")).unwrap();
        let defaults = catalog
            .items
            .iter()
            .filter_map(|item| {
                armor_recipes::recipe(&item.id).map(|design| (item.id.clone(), design))
            })
            .collect();
        let designs = ArmorDesigns {
            defaults,
            placements: BTreeMap::new(),
        };
        let encoded = serde_json::to_vec(&designs).unwrap();
        assert_eq!(
            serde_json::to_value(parse(&encoded).unwrap()).unwrap(),
            serde_json::to_value(designs).unwrap()
        );
    }
}
