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
        let mut inner =
            BodyClothPlan::with_waist_band(&design.bodice, body, head, design.waist_band)?;
        let face = inner
            .layers
            .pop()
            .context("gown bodice has no face cloth")?;
        inner
            .table
            .truncate(inner.table.len() - face.point_count as usize * POINT_WORDS);
        let join = SkirtJoinPlan::from_cut(
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
