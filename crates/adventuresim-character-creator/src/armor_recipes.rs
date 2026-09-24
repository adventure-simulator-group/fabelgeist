//! Catalog boundary for the authored armor recipes and anatomical fit regions.

use anyhow::Result;
use fabelgeist_armor::*;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::LazyLock};

use crate::armor_frames::{FitRegion, Side};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ParametricDesign {
    Helmet(HelmetDesign),
    Limb(LimbArmorDesign),
    PuffAndSlash(PuffAndSlashDesign),
    TrunkHose(TrunkHoseDesign),
    Garment(GarmentArmorDesign),
    WaistAssembly(WaistArmorDesign),
    Underlayer(crate::underlayer::UnderlayerDesign),
}

impl ParametricDesign {
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Helmet(d) => d.validate().map_err(anyhow::Error::new),
            Self::Limb(d) => d.validate().map_err(anyhow::Error::new),
            Self::PuffAndSlash(d) => d.validate().map_err(anyhow::Error::new),
            Self::TrunkHose(d) => d.validate().map_err(anyhow::Error::new),
            Self::Garment(d) => d.validate().map_err(anyhow::Error::new),
            Self::WaistAssembly(d) => d.validate().map_err(anyhow::Error::new),
            Self::Underlayer(d) => d.validate(),
        }
    }

    /// Whether both designs build the same construction, so one may replace the other.
    pub fn same_family(&self, other: &Self) -> bool {
        use std::mem::discriminant;
        match (self, other) {
            (Self::Helmet(a), Self::Helmet(b)) => discriminant(a) == discriminant(b),
            (Self::Limb(a), Self::Limb(b)) => discriminant(a) == discriminant(b),
            (Self::PuffAndSlash(a), Self::PuffAndSlash(b)) => a.kind == b.kind,
            (Self::TrunkHose(_), Self::TrunkHose(_)) => true,
            (Self::Garment(a), Self::Garment(b)) => a.kind == b.kind,
            (Self::WaistAssembly(_), Self::WaistAssembly(_)) => true,
            (Self::Underlayer(a), Self::Underlayer(b)) => a.kind == b.kind,
            _ => false,
        }
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

/// Items built by their own generator instead of an embedded catalog recipe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DedicatedGenerator {
    Vambrace,
    Breastplate,
}

impl DedicatedGenerator {
    pub fn for_item(id: &str) -> Option<Self> {
        match id {
            "vambrace" => Some(Self::Vambrace),
            "breastplate" | "cuirass" => Some(Self::Breastplate),
            _ => None,
        }
    }
}

pub fn is_parametric(id: &str) -> bool {
    DedicatedGenerator::for_item(id).is_some() || recipe(id).is_some()
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
        ParametricDesign::PuffAndSlash(d) => match d.kind {
            PuffAndSlashKind::Sleeve => F::WholeArm(side()?),
            PuffAndSlashKind::Hose => F::WholeLeg(side()?),
        },
        ParametricDesign::TrunkHose(_) => F::Hips,
        ParametricDesign::WaistAssembly(_) => F::Hips,
        ParametricDesign::Limb(d) => match d {
            LimbArmorDesign::Greave(_) => F::LowerLeg(side()?),
            LimbArmorDesign::Cuisse(_) => F::Thigh(side()?),
            LimbArmorDesign::Rerebrace(_) => F::UpperArm(side()?),
            LimbArmorDesign::Poleyn(_) => F::Knee(side()?),
            LimbArmorDesign::Couter(_) => F::Elbow(side()?),
            LimbArmorDesign::Spaulder(_) | LimbArmorDesign::Pauldron(_) => F::Shoulder(side()?),
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
            ("pauldron", "Limb", "Pauldron"),
            ("poleyn", "Limb", "Poleyn"),
            ("puffed_hose", "PuffAndSlash", "Hose"),
            ("puffed_sleeve", "PuffAndSlash", "Sleeve"),
            ("quilted_sleeve", "Garment", "QuiltedSleeve"),
            ("rerebrace", "Limb", "Rerebrace"),
            ("sabaton", "Limb", "Sabaton"),
            ("sallet", "Helmet", "Sallet"),
            ("spaulder", "Limb", "Spaulder"),
            ("split_hose", "PuffAndSlash", "Hose"),
            ("tassets", "WaistAssembly", "Tassets"),
            ("trunk_hose", "TrunkHose", "TrunkHose"),
            ("visored_sallet", "Helmet", "VisoredSallet"),
        ];
        let catalog = decode(CATALOG_SOURCE.as_bytes()).unwrap();
        assert_eq!(catalog.len(), expected.len());
        for (id, category, family) in expected {
            let encoded = serde_json::to_value(catalog.get(id).unwrap()).unwrap();
            let shape = &encoded[category];
            if category == "WaistAssembly" {
                assert_eq!(shape["fauld"]["kind"], "Fauld");
                assert_eq!(shape["tassets"]["kind"], family);
            } else if category == "TrunkHose" {
                assert!(shape.is_object(), "{id} lost its {family} family");
            } else if matches!(category, "Garment" | "Underlayer" | "PuffAndSlash") {
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
