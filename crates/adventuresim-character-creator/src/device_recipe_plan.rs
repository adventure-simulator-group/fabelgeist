//! Freeze body-dependent cut topology once, before fitting morph realizations.
use anyhow::Result;

use crate::armor_frames::FitRegion;
use crate::armor_recipes::ParametricDesign;
use crate::body_cloth_plan::BodyClothPlan;
use crate::device_fit::{FitBody, Realization, frame};
use crate::gown_plan::GownPlan;
use crate::partlet_plan::PartletPlan;
use crate::skirt_join::SkirtJoinPlan;
use fabelgeist_armor::{ArmorComponentRole, ArmorGpu};

pub(crate) enum RecipePlan {
    None,
    Skirt {
        join: SkirtJoinPlan,
        carrier: ArmorComponentRole,
    },
    Bodice(BodyClothPlan),
    Gown(GownPlan),
    Partlet(PartletPlan),
}

impl RecipePlan {
    pub async fn finish(
        &self,
        gpu: &ArmorGpu,
        body: &FitBody<'_>,
        wearer: &Realization<'_>,
        morphs: &[(&str, Realization<'_>)],
        piece: &mut crate::device_fit::DevicePiece,
    ) -> Result<()> {
        match self {
            Self::Gown(gown) => gown.finish(gpu, body, wearer, morphs, piece).await?,
            Self::Partlet(partlet) => partlet.finish(piece)?,
            Self::Skirt { join, carrier } => join.finish(piece, *carrier)?,
            Self::None | Self::Bodice(_) => {}
        }
        Ok(())
    }

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
                Ok(Self::Skirt {
                    join: SkirtJoinPlan::new(skirt, &wearer.wearer(body), &hips, &head)?,
                    carrier: if skirt.puff_and_slash.is_some() {
                        ArmorComponentRole::Undercloth
                    } else {
                        ArmorComponentRole::OuterFabric
                    },
                })
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
