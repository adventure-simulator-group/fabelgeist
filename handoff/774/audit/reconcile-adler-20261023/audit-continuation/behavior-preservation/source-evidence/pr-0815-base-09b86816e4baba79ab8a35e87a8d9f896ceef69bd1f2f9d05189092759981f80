//! Generator input for one equipment article, and the catalog's default designs.

use crate::{
    armor_design_input::{self, ArmorDesigns, ArmorPlacement},
    armor_design_output::DesignPaths,
    armor_recipes::{self, DedicatedGenerator, ParametricDesign},
    design_input,
};
use anyhow::{Context, Result, ensure};
use fabelgeist_armor::{BracerDesign, BreastplateDesign};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// The shape a generator builds for one catalog item, whichever generator that is.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemDesign {
    Recipe(ParametricDesign),
    Vambrace(BracerDesign),
    Breastplate(BreastplateDesign),
}

impl ItemDesign {
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Recipe(design) => design.validate(),
            Self::Vambrace(design) => {
                fabelgeist_armor::validate(design).context("invalid vambrace design")
            }
            Self::Breastplate(design) => {
                fabelgeist_armor::validate_breastplate(design).context("invalid breastplate design")
            }
        }
    }

    /// Whether `other` builds the same construction and may replace this design.
    pub fn same_family(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Recipe(a), Self::Recipe(b)) => a.same_family(b),
            (Self::Vambrace(_), Self::Vambrace(_))
            | (Self::Breastplate(_), Self::Breastplate(_)) => true,
            _ => false,
        }
    }

    /// The embedded-catalog recipe, which also selects mail and textile surface maps.
    pub fn recipe(&self) -> Option<&ParametricDesign> {
        match self {
            Self::Recipe(design) => Some(design),
            Self::Vambrace(_) | Self::Breastplate(_) => None,
        }
    }
}

/// Every parametric item's catalog default: authored recipes, with loaded overrides.
#[derive(Clone, Debug, PartialEq)]
pub struct CatalogDesigns {
    /// Recipes that replace the embedded authored ones, keyed by item ID.
    pub overrides: ArmorDesigns,
    pub vambrace: BracerDesign,
    pub breastplate: BreastplateDesign,
}

impl CatalogDesigns {
    /// The authored defaults, replaced by any design file given.
    pub fn load(
        overrides: Option<&Path>,
        vambrace: Option<&Path>,
        breastplate: Option<&Path>,
    ) -> Result<Self> {
        Ok(Self {
            overrides: armor_design_input::load(overrides)?,
            vambrace: design_input::load_bracer_design(vambrace)?,
            breastplate: design_input::load_breastplate_design(breastplate)?,
        })
    }

    pub fn authored() -> Self {
        Self::load(None, None, None).expect("the embedded equipment designs are valid")
    }

    /// The design a newly acquired `item_id` is built from, if it is parametric.
    pub fn default_for(&self, item_id: &str) -> Option<ItemDesign> {
        self.default_in(item_id, None)
    }

    /// The design `item_id` is built from in `placement`: a placement's own
    /// saved recipe, or else the item's default.
    pub fn default_at(&self, item_id: &str, placement: &str) -> Option<ItemDesign> {
        self.default_in(item_id, ArmorPlacement::parse(placement))
    }

    fn default_in(&self, item_id: &str, placement: Option<ArmorPlacement>) -> Option<ItemDesign> {
        match DedicatedGenerator::for_item(item_id) {
            Some(DedicatedGenerator::Vambrace) => Some(ItemDesign::Vambrace(self.vambrace.clone())),
            Some(DedicatedGenerator::Breastplate) => {
                Some(ItemDesign::Breastplate(self.breastplate.clone()))
            }
            None => placement
                .and_then(|placement| self.overrides.selected(item_id, placement))
                .or_else(|| self.overrides.defaults.get(item_id))
                .cloned()
                .or_else(|| armor_recipes::recipe(item_id))
                .map(ItemDesign::Recipe),
        }
    }

    /// Make `design` the catalog default for `item_id`.
    pub fn set_default(&mut self, item_id: &str, design: ItemDesign) -> Result<()> {
        let current = self
            .default_for(item_id)
            .with_context(|| format!("{item_id} has no parametric design"))?;
        ensure!(
            current.same_family(&design),
            "{item_id} must retain its construction family"
        );
        design.validate()?;
        match design {
            ItemDesign::Recipe(recipe) => {
                self.overrides.defaults.insert(item_id.into(), recipe);
            }
            ItemDesign::Vambrace(vambrace) => self.vambrace = vambrace,
            ItemDesign::Breastplate(breastplate) => self.breastplate = breastplate,
        }
        Ok(())
    }

    pub fn save(&self, paths: &DesignPaths<'_>) -> Result<()> {
        paths.save(&self.overrides, &self.vambrace, &self.breastplate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedicated_and_recipe_items_resolve_to_their_generators() {
        let designs = CatalogDesigns::authored();
        assert!(matches!(
            designs.default_for("vambrace"),
            Some(ItemDesign::Vambrace(_))
        ));
        for id in ["breastplate", "cuirass"] {
            assert!(matches!(
                designs.default_for(id),
                Some(ItemDesign::Breastplate(_))
            ));
        }
        assert!(matches!(
            designs.default_for("morion"),
            Some(ItemDesign::Recipe(_))
        ));
        assert!(designs.default_for("linen_tunic").is_none());
    }

    #[test]
    fn catalog_defaults_keep_their_construction_family() {
        let mut designs = CatalogDesigns::authored();
        let barbute = designs.default_for("barbute").unwrap();
        assert!(designs.set_default("morion", barbute).is_err());
        let vambrace = designs.default_for("vambrace").unwrap();
        assert!(designs.set_default("breastplate", vambrace).is_err());

        let ItemDesign::Vambrace(mut edited) = designs.default_for("vambrace").unwrap() else {
            unreachable!()
        };
        edited.center_ridge = fabelgeist_armor::Millimeters(5);
        designs
            .set_default("vambrace", ItemDesign::Vambrace(edited.clone()))
            .unwrap();
        assert_eq!(designs.vambrace, edited);
    }
}
