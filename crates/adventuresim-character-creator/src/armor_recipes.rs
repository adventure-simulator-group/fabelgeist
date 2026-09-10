//! Catalog boundary for the authored armor recipes and anatomical fit regions.

use adventuresim_armor_model::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::armor_frames::{FitRegion, Side, Wearer};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ParametricDesign {
    Helmet(HelmetDesign),
    Limb(LimbArmorDesign),
    Garment(GarmentArmorDesign),
}

impl ParametricDesign {
    pub fn generate(&self, frame: &PartFrame) -> Result<PartMesh> {
        Ok(match self {
            Self::Helmet(d) => generate_helmet(d, frame)?,
            Self::Limb(d) => generate_limb_armor(d, frame)?,
            Self::Garment(d) => generate_garment_armor(d, frame)?,
        })
    }
}

/// Stable item IDs are parsed once at the content boundary.
pub fn recipe(id: &str) -> Option<ParametricDesign> {
    use GarmentArmorKind as G;
    use HelmetKind as H;
    let helmet = match id {
        "morion" => Some(H::Morion),
        "kettle_hat" => Some(H::KettleHat),
        "barbute" => Some(H::Barbute),
        "burgonet" => Some(H::Burgonet),
        "sallet" => Some(H::Sallet),
        "visored_sallet" => Some(H::VisoredSallet),
        "close_helmet" => Some(H::CloseHelmet),
        "arming_cap" => Some(H::ArmingCap),
        "mail_coif" => Some(H::MailCoif),
        _ => None,
    };
    if let Some(kind) = helmet {
        return Some(ParametricDesign::Helmet(HelmetDesign::catalog(kind)));
    }
    let garment = match id {
        "arming_doublet" => Some(G::ArmingDoublet),
        "brigandine" => Some(G::Brigandine),
        "jack_of_plates" => Some(G::JackOfPlates),
        "fauld" => Some(G::Fauld),
        "mail_chausses" => Some(G::MailChausses),
        "mail_shirt" => Some(G::MailShirt),
        "mail_skirt" => Some(G::MailSkirt),
        "mail_sleeve" => Some(G::MailSleeve),
        "padded_chausses" => Some(G::PaddedChausses),
        "padded_skirt" => Some(G::PaddedSkirt),
        "quilted_sleeve" => Some(G::QuiltedSleeve),
        "tassets" => Some(G::Tassets),
        "gorget" => Some(G::Gorget),
        _ => None,
    };
    if let Some(kind) = garment {
        return Some(ParametricDesign::Garment(GarmentArmorDesign::new(kind)));
    }
    let limb = match id {
        "greave" => LimbArmorDesign::Greave(Default::default()),
        "cuisse" => LimbArmorDesign::Cuisse(Default::default()),
        "rerebrace" => LimbArmorDesign::Rerebrace(Default::default()),
        "poleyn" => LimbArmorDesign::Poleyn(JointCupDesign::poleyn()),
        "couter" => LimbArmorDesign::Couter(JointCupDesign::couter()),
        "spaulder" => LimbArmorDesign::Spaulder(Default::default()),
        "mitten_gauntlet" => LimbArmorDesign::MittenGauntlet(Default::default()),
        "sabaton" => LimbArmorDesign::Sabaton(Default::default()),
        "leather_boot" => LimbArmorDesign::LeatherBoot(Default::default()),
        _ => return None,
    };
    Some(ParametricDesign::Limb(limb))
}

pub fn is_parametric(id: &str) -> bool {
    matches!(id, "vambrace" | "breastplate" | "cuirass") || recipe(id).is_some()
}

pub fn fit_region(design: &ParametricDesign, placement: &str) -> Result<FitRegion> {
    use FitRegion as F;
    use GarmentArmorKind as G;
    let side = || Side::from_placement(placement);
    Ok(match design {
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
