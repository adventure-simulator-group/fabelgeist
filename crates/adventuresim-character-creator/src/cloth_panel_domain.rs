//! Frozen anatomical ancestry, coarse sampling and coherent plane fitting.
//! Positions at numerical boundaries are canonical unposed metres.
use super::PanelSurface;
use crate::armor_frames::{FitRegion, SKIN_SUPPORT_THRESHOLD, Side, Wearer};
use crate::skin_transfer::SkinWeights;
use crate::surface_cut::{ClothSampling, ConvexRegion, SurfaceCut, connected_patch, interpolate};
use anyhow::Result;
use fabelgeist_armor::Millimeters;
use std::collections::BTreeSet;

/// Two frozen barycentric levels retain the native anatomical domain and the
/// garment's geometric trim independently of subsequent body-shape fitting.
#[derive(Clone)]
pub(super) struct PanelDomain {
    pub(super) carrier: SurfaceCut,
    pub(super) cut: SurfaceCut,
}

pub(super) struct SampledPanel {
    pub surface: PanelDomain,
    pub ancestry: crate::surface_cut::SamplingAncestry,
}

impl PanelDomain {
    /// Cut the same retained anatomical region at either sampling density.
    /// Geometric positions at this boundary are canonical unposed metres.
    pub(super) fn new(
        carrier: SurfaceCut,
        body: &Wearer<'_>,
        include: &[ConvexRegion],
        subtract: &[ConvexRegion],
        center: [f32; 3],
    ) -> Result<Self> {
        let positions = native_points(&carrier, body);
        let cut = SurfaceCut::new(
            &positions,
            &carrier.faces,
            &carrier.faces,
            include,
            subtract,
        );
        let reference = Self::cut_positions(&carrier, &cut, &positions);
        let cut = connected_patch(cut, &reference, center)?;
        let cut = cut
            .sample_cloth(&positions, &carrier.faces, ClothSampling::CoplanarTrim, &[])?
            .surface;
        Ok(Self { carrier, cut })
    }

    pub(super) fn enclosed_surface(
        &self,
        support: &Self,
        skin: &SkinWeights,
        banks: &super::banks::ContactBanks,
        clearance: Millimeters,
        body: &Wearer<'_>,
    ) -> Result<PanelSurface> {
        let mut points = self.positions(body);
        let target_points = support.positions(body);
        let target_skin = super::enclosure::target_skin(support, body)?;
        let target_normals = PanelSurface::fitted_normals(&target_points, &support.cut.faces)?;
        let target = Wearer {
            positions: &target_points,
            normals: &target_normals,
            faces: &support.cut.faces,
            joint_indices: &target_skin.indices,
            joint_weights: &target_skin.weights,
            ..*body
        };
        super::enclosure::fit(
            &mut points,
            &self.cut.faces,
            &target,
            body,
            clearance,
            banks,
        )?;
        PanelSurface::from_fitted_carrier(points, self.cut.faces.clone(), skin.clone())
    }

    pub(super) fn carrier_skin(&self, body: &Wearer<'_>) -> Result<SkinWeights> {
        crate::skin_transfer::interpolate_sources(
            &self.carrier.source_table(body.faces, body.faces),
            body.joint_indices,
            body.joint_weights,
        )
    }

    /// Allocate the coarse mesh to the actual cloth, retaining mixed shoulder
    /// ownership and physical opening boundaries before any morph is fitted.
    pub(super) fn sample(mut self, body: &Wearer<'_>) -> Result<SampledPanel> {
        let carrier_skin = self.carrier_skin(body)?;
        let cut_skin = crate::skin_transfer::interpolate_sources(
            &self
                .cut
                .source_table(&self.carrier.faces, &self.carrier.faces),
            &carrier_skin.indices,
            &carrier_skin.weights,
        )?;
        let torso =
            body.support_weights_for(FitRegion::Torso, &cut_skin.indices, &cut_skin.weights)?;
        let mut preserve = BTreeSet::new();
        for side in [Side::Right, Side::Left] {
            let arm = body.support_weights_for(
                FitRegion::UpperArm(side),
                &cut_skin.indices,
                &cut_skin.weights,
            )?;
            preserve.extend(
                torso
                    .iter()
                    .zip(arm)
                    .enumerate()
                    .filter_map(|(v, (&torso, arm))| {
                        (torso >= SKIN_SUPPORT_THRESHOLD && arm >= SKIN_SUPPORT_THRESHOLD)
                            .then_some(v as u32)
                    }),
            );
        }
        let sampled = self.cut.sample_cloth(
            &native_points(&self.carrier, body),
            &self.carrier.faces,
            ClothSampling::AnatomicalCarrier { preserve },
            &[],
        )?;
        self.cut = sampled.surface;
        Ok(SampledPanel {
            surface: self,
            ancestry: sampled.ancestry,
        })
    }

    pub(super) fn positions(&self, body: &Wearer<'_>) -> Vec<[f32; 3]> {
        Self::cut_positions(
            &self.carrier,
            &self.cut,
            &native_points(&self.carrier, body),
        )
    }

    fn cut_positions(
        carrier: &SurfaceCut,
        cut: &SurfaceCut,
        positions: &[[f32; 3]],
    ) -> Vec<[f32; 3]> {
        cut.points
            .iter()
            .map(|point| {
                interpolate(
                    carrier.faces[point.triangle].map(|v| positions[v as usize]),
                    point.weights,
                )
            })
            .collect()
    }
}

/// Native body-barycentric positions in canonical unposed metres.
fn native_points(cut: &SurfaceCut, body: &Wearer<'_>) -> Vec<[f32; 3]> {
    cut.points
        .iter()
        .map(|point| {
            interpolate(
                body.faces[point.triangle].map(|v| body.positions[v as usize]),
                point.weights,
            )
        })
        .collect()
}
