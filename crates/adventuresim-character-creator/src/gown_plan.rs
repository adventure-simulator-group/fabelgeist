//! Full fitted upper cloth and hanging skirt share one sewn carrier boundary.
use anyhow::{Context, Result};
use fabelgeist_armor::{ArmorComponentRole, ArmorGpu, GownDesign, PartFrame};

use crate::armor_frames::Wearer;
use crate::body_cloth_plan::BodyClothPlan;
use crate::device_fit::{DevicePiece, FitBody, Realization, fit_piece_async};
use crate::skirt_join::{SewnWaist, SkirtJoinPlan};
use crate::surface_cut::{POINT_WORDS, SurfaceCut};

pub(crate) struct GownPlan {
    pub join: SkirtJoinPlan,
    inner: BodyClothPlan,
    carrier_role: ArmorComponentRole,
}

impl GownPlan {
    pub fn new(
        design: &GownDesign,
        body: &Wearer<'_>,
        hips: &PartFrame,
        head: &PartFrame,
    ) -> Result<Self> {
        design.validate()?;
        let straight_fall = matches!(
            design.skirt.profile,
            fabelgeist_armor::SkirtProfile::Straight
        ) && design.skirt.pleats.is_none()
            && design.first_skirt_row() as f32 / fabelgeist_armor::SKIRT_AXIAL_SEGMENTS as f32
                >= design.skirt.fitted_fraction.unit();
        let mut inner =
            BodyClothPlan::with_waist_band(&design.bodice, body, head, design.waist_band)?;
        let face = inner
            .layers
            .pop()
            .context("gown bodice has no face cloth")?;
        inner
            .table
            .truncate(inner.table.len() - face.point_count as usize * POINT_WORDS);
        let enclosure_table = inner.surface.source_table(body.faces, body.faces);
        if let fabelgeist_armor::GownBodyFit::Loose { release, .. } = design.body_fit
            && straight_fall
        {
            let loft = crate::gown_ease::GownEasePlan::new(
                &inner.surface,
                body,
                hips,
                head,
                design.bodice.hem_rise,
            )?;
            inner.surface = loft.upper_patch(inner.surface, body, head, release)?;
        }
        let mut join = SkirtJoinPlan::from_cut(
            SurfaceCut {
                points: inner.surface.points.clone(),
                faces: inner.surface.faces.clone(),
            },
            body,
            hips,
            head,
            SewnWaist {
                first_row: design.first_skirt_row(),
                offset_metres: face.offset_metres,
            },
        )?;
        if let fabelgeist_armor::GownBodyFit::Loose { ease, release } = design.body_fit {
            join.enclosure_table = enclosure_table;
            join.ease = ease;
            join.release = release;
            let ease_plan = crate::gown_ease::GownEasePlan::new(
                &inner.surface,
                body,
                hips,
                head,
                design.bodice.hem_rise,
            )?;
            join.release_weights = ease_plan.weights;
            join.neckline_samples = ease_plan.neckline_samples;
            if straight_fall {
                // A straight hanging sheet has no intermediate silhouette bend.
                const STRAIGHT_CLOTH_INTERVALS: usize = 2;
                join.patch.hanging_intervals = STRAIGHT_CLOTH_INTERVALS;
                join.patch.seam_layout = fabelgeist_armor::gpu::SkirtSeamLayout::StraightFall;
            }
        }
        if let Some(sleeves) = &design.sleeves {
            join.sleeves = Some(crate::sewn_sleeves::SewnSleevePlan::new(
                &inner.surface,
                body,
                head,
                &mut join,
                sleeves,
            )?);
        }
        join.bind_neckline(&inner.surface, body, head, design)?;
        let carrier_role = if design.skirt.puff_and_slash.is_some() {
            ArmorComponentRole::Undercloth
        } else {
            ArmorComponentRole::OuterFabric
        };
        Ok(Self {
            join,
            inner,
            carrier_role,
        })
    }

    pub async fn finish(
        &self,
        gpu: &ArmorGpu,
        body: &FitBody<'_>,
        wearer: &Realization<'_>,
        morphs: &[(&str, Realization<'_>)],
        piece: &mut DevicePiece,
    ) -> Result<()> {
        if let Some(sleeves) = &self.join.sleeves {
            piece.bind_sewn_sleeves(
                body,
                wearer,
                sleeves,
                self.join.patch.carrier_point_count(),
            )?;
            sleeves.attach_closures(piece, self.join.patch.carrier_point_count())?;
        }
        self.join.finish(piece, self.carrier_role)?;
        if !self.inner.layers.is_empty() {
            let lining = fit_piece_async(gpu, body, wearer, morphs, |device, batch, _| {
                device.record_body_cloth(batch, &self.inner)
            })
            .await?;
            piece.append(lining)?;
        }
        Ok(())
    }
}
