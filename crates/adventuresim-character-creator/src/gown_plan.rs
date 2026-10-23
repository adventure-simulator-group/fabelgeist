//! Full fitted upper cloth and hanging skirt share one sewn carrier boundary.
use anyhow::{Context, Result};
use fabelgeist_armor::{ArmorComponentRole, ArmorGpu, GownDesign, PartFrame};

use crate::armor_frames::Wearer;
use crate::body_cloth_plan::BodyClothPlan;
use crate::device_fit::{DevicePiece, FitBody, Realization, fit_piece_async};
use crate::skirt_join::{SewnWaist, SkirtJoinPlan};
use crate::surface_cut::SurfaceCut;

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
        let straight_fall = design.is_straight_hanging();
        let mut inner =
            BodyClothPlan::for_gown(design, body, head).context("constructing gown panels")?;
        let source_geometry = inner.source.geometry(body)?;
        let tailored_body = source_geometry.wearer(body);
        let body = &tailored_body;
        let face = inner
            .pop_source_layer()
            .context("gown bodice has no face cloth")?;
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
            )
            .context("planning released upper panels")?;
            inner.surface = loft
                .upper_patch(inner.surface, body, head, release)
                .context("building released upper panel")?;
        }
        let mut waist_anchor = crate::cloth_torso_frame::ClothTorsoFrame::new(body, head)?.waist;
        waist_anchor[1] += design.bodice.hem_rise.metres();
        let mut join = SkirtJoinPlan::from_cut(
            SurfaceCut {
                apertures: inner.surface.apertures.clone(),
                points: inner.surface.points.clone(),
                faces: inner.surface.faces.clone(),
            },
            body,
            hips,
            head,
            SewnWaist {
                anchor: waist_anchor,
                first_row: design.first_skirt_row(),
                hanging_rows: fabelgeist_armor::gpu::SkirtHangingRows::for_design(
                    &design.skirt,
                    design.first_skirt_row(),
                )?,
                ring_segments: design.skirt.mesh_ring_segments(),
                offset_metres: face.offset_metres,
                face_thickness: design.bodice.thickness,
            },
        )
        .context("joining panel waist to hanging skirt")?;
        join.source = inner.source.clone();
        join.release_body(design, &inner.surface, body, hips, head, enclosure_table)
            .context("releasing gown fullness")?;
        let carrier_role = if design.skirt.puff_and_slash.is_some() {
            ArmorComponentRole::Undercloth
        } else {
            ArmorComponentRole::OuterFabric
        };
        Self::share_body_source(&mut join, &mut inner)?;
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
        layers: &[&fabelgeist_armor::GeneratedArmor],
    ) -> Result<()> {
        let first = self.join.patch.carrier_point_count();
        let skin = self
            .join
            .source
            .skin(&self.join.table, &wearer.wearer(body))?;
        for (vertex, &source) in piece.skin_aliases.iter().enumerate() {
            let source = self
                .join
                .patch
                .dressing_opening
                .as_ref()
                .map_or(source, |opening| opening.source_vertex(source));
            if let Some(raw) = source.checked_sub(first)
                && raw < skin.indices.len()
            {
                piece
                    .skin
                    .joint_indices
                    .set_vertex(vertex, skin.indices.vertex(raw))?;
                piece.skin.joint_weights[vertex] = skin.weights[raw];
            }
        }

        // Source ordinals refer to the unsplit outer carrier. Attach before
        // material partitioning duplicates and reorders those vertices.

        if !self.inner.layers.is_empty() {
            let mut lining = fit_piece_async(gpu, body, wearer, morphs, |device, batch, target| {
                let surfaces = layers
                    .iter()
                    .map(|layer| {
                        crate::armor_layer::ArmorLayerSurface::from_generated(layer, target)
                    })
                    .collect::<Result<Vec<_>>>()?;
                device.record_body_cloth(batch, &self.inner, &surfaces)
            })
            .await?;
            self.inner.bind_skin(&mut lining, &wearer.wearer(body))?;
            self.inner.join_layer_interfaces(&mut lining)?;
            piece.append(lining)?;
            self.inner.join_gown_interface(piece, &self.join)?;
        }
        self.join.finish(piece, self.carrier_role)?;
        crate::cloth_triangle_budget::check(&piece.base)?;
        Ok(())
    }
    /// Couple the fitting domain while retaining independent material cuts.
    fn share_body_source(join: &mut SkirtJoinPlan, inner: &mut BodyClothPlan) -> Result<()> {
        // The face and hidden sheets must solve the same lower-layer fit.
        // Their render domains stay independent, but their source field does
        // not change when a lining reaches past a facing or waist seam.
        join.source.share_support(
            crate::cloth_layer_seating::CutTopology {
                table: &join.table,
                faces: &join.source_faces,
            },
            crate::cloth_layer_seating::CutTopology {
                table: &inner.table,
                faces: &inner.source_faces,
            },
        )?;
        inner.source = join.source.clone();
        Ok(())
    }
}
