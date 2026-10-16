//! Freeze body-dependent cut topology once, before fitting morph realizations.
use anyhow::Result;

use crate::armor_frames::FitRegion;
use crate::armor_recipes::ParametricDesign;
use crate::body_cloth_plan::BodyClothPlan;
use crate::device_fit::{FitBody, Realization, frame};
use crate::gown_plan::GownPlan;
use crate::partlet_plan::PartletPlan;
use crate::skirt_join::SkirtJoinPlan;
use fabelgeist_armor::ArmorGpu;

pub(crate) enum RecipePlan {
    None,
    Skirt(SkirtJoinPlan),
    Bodice(BodyClothPlan),
    Gown(GownPlan),
    Partlet(PartletPlan),
}

impl RecipePlan {
    pub fn new(
        gpu: &ArmorGpu,
        body: &FitBody<'_>,
        wearer: &Realization<'_>,
        design: &ParametricDesign,
    ) -> Result<Self> {
        match design {
            ParametricDesign::Partlet(partlet) => {
                let head = frame(gpu, body, wearer, FitRegion::Head)?;
                Ok(Self::Partlet(PartletPlan::new(
                    partlet,
                    &wearer.wearer(body),
                    &head,
                )?))
            }
            ParametricDesign::Gown(gown) => {
                let hips = frame(gpu, body, wearer, FitRegion::Hips)?;
                let head = frame(gpu, body, wearer, FitRegion::Head)?;
                Ok(Self::Gown(GownPlan::new(
                    gown,
                    &wearer.wearer(body),
                    &hips,
                    &head,
                )?))
            }
            ParametricDesign::Skirt(skirt) if skirt.bodice.is_some() => {
                let hips = frame(gpu, body, wearer, FitRegion::Hips)?;
                let head = frame(gpu, body, wearer, FitRegion::Head)?;
                Ok(Self::Skirt(SkirtJoinPlan::new(
                    skirt,
                    &wearer.wearer(body),
                    &hips,
                    &head,
                )?))
            }
            ParametricDesign::Bodice(bodice) => {
                let head = frame(gpu, body, wearer, FitRegion::Head)?;
                Ok(Self::Bodice(BodyClothPlan::new(
                    bodice,
                    &wearer.wearer(body),
                    &head,
                )?))
            }
            _ => Ok(Self::None),
        }
    }
}
