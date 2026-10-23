//! Freeze body-dependent cut topology once, before fitting morph realizations.
use anyhow::{Context, Result};

use crate::armor_frames::FitRegion;
use crate::armor_recipes::ParametricDesign;
use crate::body_cloth_plan::BodyClothPlan;
use crate::device_fit::{FitBody, Realization, frame};
use crate::gown_plan::GownPlan;
use crate::skirt_join::SkirtJoinPlan;
use fabelgeist_armor::{ArmorComponentRole, ArmorGpu};

pub(crate) enum RecipePlan {
    None,
    Skirt {
        join: Box<SkirtJoinPlan>,
        carrier: ArmorComponentRole,
    },
    Bodice(Box<BodyClothPlan>),
    Gown(Box<GownPlan>),
}

impl RecipePlan {
    /// Freeze construction once; every base and morph recording shares its plan.
    pub async fn prepare(
        gpu: &ArmorGpu,
        body: &FitBody<'_>,
        wearer: &Realization<'_>,
        design: &ParametricDesign,
    ) -> Result<Self> {
        let plan = Self::new(gpu, body, wearer, design).context("planning textile construction")?;

        Ok(plan)
    }

    pub async fn finish(
        &self,
        gpu: &ArmorGpu,
        body: &FitBody<'_>,
        wearer: &Realization<'_>,
        morphs: &[(&str, Realization<'_>)],
        piece: &mut crate::device_fit::DevicePiece,
        layers: &[&fabelgeist_armor::GeneratedArmor],
    ) -> Result<()> {
        match self {
            Self::Gown(gown) => {
                gown.finish(gpu, body, wearer, morphs, piece, layers)
                    .await?
            }

            Self::Skirt { join, carrier } => join.finish(piece, *carrier)?,
            Self::Bodice(plan) => plan.bind_skin(piece, &wearer.wearer(body))?,
            Self::None => {}
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
            ParametricDesign::Gown(gown) => {
                let hips = frame(gpu, body, wearer, FitRegion::Hips)?;
                let head = frame(gpu, body, wearer, FitRegion::Head)?;
                Ok(Self::Gown(Box::new(GownPlan::new(
                    gown,
                    &wearer.wearer(body),
                    &hips,
                    &head,
                )?)))
            }
            ParametricDesign::Skirt(skirt) if skirt.bodice.is_some() => {
                let hips = frame(gpu, body, wearer, FitRegion::Hips)?;
                let head = frame(gpu, body, wearer, FitRegion::Head)?;
                Ok(Self::Skirt {
                    join: Box::new(SkirtJoinPlan::new(
                        skirt,
                        &wearer.wearer(body),
                        &hips,
                        &head,
                    )?),
                    carrier: if skirt.puff_and_slash.is_some() {
                        ArmorComponentRole::Undercloth
                    } else {
                        ArmorComponentRole::OuterFabric
                    },
                })
            }
            ParametricDesign::Bodice(bodice) => {
                let head = frame(gpu, body, wearer, FitRegion::Head)?;
                Ok(Self::Bodice(Box::new(BodyClothPlan::new(
                    bodice,
                    &wearer.wearer(body),
                    &head,
                )?)))
            }
            _ => Ok(Self::None),
        }
    }
}
