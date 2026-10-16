//! Full fitted upper cloth and hanging skirt share one sewn carrier boundary.
use anyhow::{Context, Result};
use fabelgeist_armor::{
    ArmorComponentRole, ArmorGpu, GownDesign, PartFrame, SKIRT_AXIAL_SEGMENTS, SKIRT_RING_SEGMENTS,
};

use crate::armor_frames::Wearer;
use crate::bodice_plan::BodicePlan;
use crate::device_fit::{DevicePiece, FitBody, Realization, fit_piece_async};
use crate::skirt_join::{SewnWaist, SkirtJoinPlan};
use crate::surface_cut::{POINT_WORDS, SurfaceCut};

pub(crate) struct GownPlan {
    pub join: SkirtJoinPlan,
    inner: BodicePlan,
}

impl GownPlan {
    pub fn new(
        design: &GownDesign,
        body: &Wearer<'_>,
        hips: &PartFrame,
        head: &PartFrame,
    ) -> Result<Self> {
        design.validate()?;
        let mut inner = BodicePlan::with_waist_band(&design.bodice, body, head, design.waist_band)?;
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
        Ok(Self { join, inner })
    }

    pub async fn finish(
        &self,
        gpu: &ArmorGpu,
        body: &FitBody<'_>,
        wearer: &Realization<'_>,
        morphs: &[(&str, Realization<'_>)],
        piece: &mut DevicePiece,
    ) -> Result<()> {
        // Native shell ancestry is frozen before fitting. The copied material
        // boundary is not a second fitted edge or a capped pair of surfaces.
        let skirt_points =
            (SKIRT_AXIAL_SEGMENTS - self.join.patch.first_row + 1) * SKIRT_RING_SEGMENTS;
        let upper = skirt_points..skirt_points + self.join.patch.point_count as usize;
        let sources = piece.skin_aliases.clone();
        piece.partition_first_component(|triangle| {
            if triangle
                .iter()
                .all(|&v| upper.contains(&sources[v as usize]))
            {
                ArmorComponentRole::BodiceFabric
            } else {
                ArmorComponentRole::OuterFabric
            }
        })?;
        normalize_fabric_charts(&mut piece.base)?;
        if !self.inner.layers.is_empty() {
            let lining = fit_piece_async(gpu, body, wearer, morphs, |device, batch, _| {
                device.record_bodice(batch, &self.inner)
            })
            .await?;
            piece.append(lining)?;
        }
        Ok(())
    }
}

/// Each cut fabric has its own along-cloth pattern domain. The original
/// device chart remains the angular chart; no geometry is evaluated here.
fn normalize_fabric_charts(part: &mut fabelgeist_armor::BuiltPart) -> Result<()> {
    let coordinates = part
        .texcoords
        .as_mut()
        .context("gown has no fabric chart")?;
    for component in &part.components {
        if !matches!(
            component.role,
            ArmorComponentRole::BodiceFabric | ArmorComponentRole::OuterFabric
        ) {
            continue;
        }
        let points = &mut coordinates[component.vertices.clone()];
        let low = points.iter().map(|uv| uv[1]).fold(f32::INFINITY, f32::min);
        let high = points
            .iter()
            .map(|uv| uv[1])
            .fold(f32::NEG_INFINITY, f32::max);
        anyhow::ensure!(high > low, "gown fabric has no along-cloth extent");
        for uv in points {
            uv[1] = (uv[1] - low) / (high - low);
        }
    }
    Ok(())
}
