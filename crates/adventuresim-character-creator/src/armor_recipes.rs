//! Catalog boundary for the authored armor recipes and anatomical fit regions.

use adventuresim_armor_model::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::LazyLock};

use crate::armor_frames::{FitRegion, Side, Wearer};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ParametricDesign {
    Helmet(HelmetDesign),
    Limb(LimbArmorDesign),
    Garment(GarmentArmorDesign),
    Underlayer(crate::underlayer::UnderlayerDesign),
}

impl ParametricDesign {
    pub fn generate(&self, frame: &PartFrame) -> Result<PartMesh> {
        Ok(match self {
            Self::Helmet(d) => generate_helmet(d, frame)?,
            Self::Limb(d) => generate_limb_armor(d, frame)?,
            Self::Garment(d) => generate_garment_armor(d, frame)?,
            Self::Underlayer(_) => {
                anyhow::bail!("body-conforming garments require source body triangles")
            }
        })
    }
}

const CATALOG_SOURCE: &str = include_str!("../../../assets_src/equipment/armor-designs.json");

static CATALOG: LazyLock<BTreeMap<String, ParametricDesign>> = LazyLock::new(|| {
    crate::armor_design_input::decode(CATALOG_SOURCE.as_bytes()).unwrap_or_else(|error| {
        panic!("invalid embedded assets_src/equipment/armor-designs.json: {error:#}")
    })
});

/// Look up the authored item recipe in the embedded equipment catalog.
pub fn recipe(id: &str) -> Option<ParametricDesign> {
    CATALOG.get(id).cloned()
}

pub fn is_parametric(id: &str) -> bool {
    matches!(id, "vambrace" | "breastplate" | "cuirass") || recipe(id).is_some()
}

pub fn fit_region(design: &ParametricDesign, placement: &str) -> Result<FitRegion> {
    use FitRegion as F;
    use GarmentArmorKind as G;
    let side = || Side::from_placement(placement);
    Ok(match design {
        ParametricDesign::Underlayer(d) => match d.kind {
            crate::underlayer::UnderlayerKind::PaddedHose => F::WholeLeg(side()?),
            crate::underlayer::UnderlayerKind::MailBrayette => F::Hips,
            crate::underlayer::UnderlayerKind::MailKneeVoider => F::Knee(side()?),
            crate::underlayer::UnderlayerKind::MailStandard => F::Neck,
            _ => F::Torso,
        },
        ParametricDesign::Helmet(_) => F::Head,
        ParametricDesign::Limb(d) => match d {
            LimbArmorDesign::Greave(_) => F::LowerLeg(side()?),
            LimbArmorDesign::Cuisse(_) => F::Thigh(side()?),
            LimbArmorDesign::Rerebrace(_) => F::UpperArm(side()?),
            LimbArmorDesign::Poleyn(_) => F::Knee(side()?),
            LimbArmorDesign::Couter(_) => F::Elbow(side()?),
            LimbArmorDesign::Spaulder(_) => F::Shoulder(side()?),
            LimbArmorDesign::MittenGauntlet(_) => F::Hand(side()?),
            LimbArmorDesign::Sabaton(_) | LimbArmorDesign::LeatherBoot(_) => F::Foot(side()?),
        },
        ParametricDesign::Garment(d) => match d.kind {
            G::ArmingDoublet | G::Brigandine | G::JackOfPlates | G::MailShirt => F::Torso,
            G::Fauld | G::MailSkirt | G::PaddedSkirt | G::Tassets => F::Hips,
            G::MailChausses | G::PaddedChausses => F::WholeLeg(side()?),
            G::MailSleeve | G::QuiltedSleeve => F::WholeArm(side()?),
            G::Gorget => F::Neck,
        },
    })
}

pub fn fitted_mesh(
    design: &ParametricDesign,
    placement: &str,
    wearer: &Wearer<'_>,
) -> Result<PartMesh> {
    if let ParametricDesign::Underlayer(d) = design {
        let pattern =
            crate::underlayer::UnderlayerPattern::new(d, placement, wearer, wearer.faces)?;
        return Ok(pattern.evaluate(d, wearer));
    }
    if let ParametricDesign::Helmet(HelmetDesign::CloseHelmet(helmet)) = design {
        return crate::close_helmet_fit::fit(helmet, wearer);
    }
    if let ParametricDesign::Helmet(HelmetDesign::MailCoif(coif)) = design {
        return crate::coif_fit::fit(coif, wearer);
    }
    if let ParametricDesign::Garment(garment) = design {
        return crate::garment_fit::fitted_garment(garment, placement, wearer);
    }
    if let ParametricDesign::Limb(limb) = design {
        return crate::limb_fit::fitted_limb(limb, wearer, fit_region(design, placement)?);
    }
    let frame = wearer.frame(fit_region(design, placement)?)?;
    let mesh = design.generate(&frame)?;
    Ok(mesh)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::armor_design_input::decode;

    #[test]
    fn embedded_catalog_preserves_every_item_and_construction_family() {
        let expected = [
            ("arming_cap", "Helmet", "ArmingCap"),
            ("arming_doublet", "Underlayer", "ArmingDoublet"),
            ("barbute", "Helmet", "Barbute"),
            ("brigandine", "Garment", "Brigandine"),
            ("burgonet", "Helmet", "Burgonet"),
            ("close_helmet", "Helmet", "CloseHelmet"),
            ("couter", "Limb", "Couter"),
            ("cuisse", "Limb", "Cuisse"),
            ("fauld", "Garment", "Fauld"),
            ("gorget", "Garment", "Gorget"),
            ("greave", "Limb", "Greave"),
            ("jack_of_plates", "Garment", "JackOfPlates"),
            ("kettle_hat", "Helmet", "KettleHat"),
            ("leather_boot", "Limb", "LeatherBoot"),
            ("mail_chausses", "Garment", "MailChausses"),
            ("mail_coif", "Helmet", "MailCoif"),
            ("mail_shirt", "Garment", "MailShirt"),
            ("mail_skirt", "Garment", "MailSkirt"),
            ("mail_sleeve", "Garment", "MailSleeve"),
            ("mail_voiders", "Underlayer", "MailVoiders"),
            ("mail_brayette", "Underlayer", "MailBrayette"),
            ("mail_knee_voider", "Underlayer", "MailKneeVoider"),
            ("mail_standard", "Underlayer", "MailStandard"),
            ("mitten_gauntlet", "Limb", "MittenGauntlet"),
            ("morion", "Helmet", "Morion"),
            ("padded_chausses", "Underlayer", "PaddedHose"),
            ("padded_skirt", "Garment", "PaddedSkirt"),
            ("poleyn", "Limb", "Poleyn"),
            ("quilted_sleeve", "Garment", "QuiltedSleeve"),
            ("rerebrace", "Limb", "Rerebrace"),
            ("sabaton", "Limb", "Sabaton"),
            ("sallet", "Helmet", "Sallet"),
            ("spaulder", "Limb", "Spaulder"),
            ("tassets", "Garment", "Tassets"),
            ("visored_sallet", "Helmet", "VisoredSallet"),
        ];
        let catalog = decode(CATALOG_SOURCE.as_bytes()).unwrap();
        assert_eq!(catalog.len(), expected.len());
        for (id, category, family) in expected {
            let encoded = serde_json::to_value(catalog.get(id).unwrap()).unwrap();
            let shape = &encoded[category];
            if matches!(category, "Garment" | "Underlayer") {
                assert_eq!(shape["kind"], family, "{id}");
            } else {
                assert!(shape.get(family).is_some(), "{id} lost its {family} family");
            }
        }
        let content: crate::item_catalog_schema::ItemCatalogDocument =
            serde_json::from_str(include_str!("../../../content/items/catalog.yaml")).unwrap();
        for item in &content.items {
            if matches!(
                item.kind,
                crate::item_catalog_schema::ItemKind::Armor { .. }
            ) {
                assert!(is_parametric(&item.id), "missing armor recipe {}", item.id);
            }
        }
        for id in catalog.keys() {
            assert!(
                content.items.iter().any(|item| &item.id == id),
                "orphan recipe {id}"
            );
        }
    }

    #[test]
    fn recipe_lookup_returns_authored_shapes_without_unknown_item_fallback() {
        let authored: serde_json::Value = serde_json::from_str(CATALOG_SOURCE).unwrap();
        let selected = recipe("barbute").unwrap();
        assert_eq!(
            serde_json::to_value(&selected).unwrap(),
            authored["barbute"]
        );
        assert_ne!(
            serde_json::to_value(selected).unwrap(),
            serde_json::to_value(ParametricDesign::Helmet(HelmetDesign::catalog(
                HelmetKind::Barbute
            )))
            .unwrap()
        );
        assert!(recipe("unknown_armor").is_none());
        for id in ["vambrace", "breastplate", "cuirass"] {
            assert!(is_parametric(id));
            assert!(recipe(id).is_none());
        }
    }

    #[test]
    fn embedded_schema_rejects_missing_unknown_and_invalid_controls() {
        let source: serde_json::Value = serde_json::from_str(CATALOG_SOURCE).unwrap();
        let mut missing = source.clone();
        missing["morion"]["Helmet"]["Morion"]["fit"]
            .as_object_mut()
            .unwrap()
            .remove("clearance");
        assert!(decode(&serde_json::to_vec(&missing).unwrap()).is_err());
        let mut unknown = source.clone();
        unknown["morion"]["Helmet"]["Morion"]["fit"]["clearence"] = 8.into();
        assert!(decode(&serde_json::to_vec(&unknown).unwrap()).is_err());
        let mut invalid = source;
        invalid["morion"]["Helmet"]["Morion"]["fit"]["wall_thickness"] = 0.into();
        let error = decode(&serde_json::to_vec(&invalid).unwrap()).unwrap_err();
        assert!(format!("{error:#}").contains("morion"));
    }
}
