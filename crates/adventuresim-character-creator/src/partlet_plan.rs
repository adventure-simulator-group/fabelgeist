//! The complete panel and collar carrier is thickened once, after sewing.
use crate::armor_frames::Wearer;
use crate::body_cloth_plan::BodyClothPlan;
use crate::cloth_ribbon_plan::{ClothRibbonBoundary, ClothRibbonPlan};
use crate::cloth_tie_plan::ClothTiePlan;
use crate::device_fit::DevicePiece;
use crate::partlet_collar::{CollarBandPlacement, CollarPlan};
use crate::partlet_cut::PartletCut;
use crate::surface_cut::CUT_VERTEX_SNAP_METRES;
use anyhow::Result;
use fabelgeist_armor::gpu::BodyClothLayer;
use fabelgeist_armor::{ArmorComponentRole, PartFrame, PartletDesign};
use std::collections::BTreeSet;

pub(crate) struct PartletPlan {
    pub cloth: BodyClothPlan,
    pub collar: Option<CollarPlan>,
    pub binding: Option<ClothRibbonPlan>,
    pub bands: Vec<CollarBandPlacement>,
    pub ties: Option<ClothTiePlan>,
}

impl PartletPlan {
    pub fn new(design: &PartletDesign, body: &Wearer<'_>, head: &PartFrame) -> Result<Self> {
        let cut = PartletCut::new(design, body, head)?;
        let mut indices = cut
            .surface
            .faces
            .iter()
            .flatten()
            .copied()
            .collect::<Vec<_>>();
        let collar = design
            .collar
            .as_ref()
            .map(|collar| CollarPlan::new(collar, &cut, design.thickness.metres(), &mut indices))
            .transpose()?;
        let point_count =
            cut.surface.points.len() as u32 + collar.as_ref().map_or(0, |c| c.rows * c.columns);
        let mut cloth = BodyClothPlan {
            table: cut.surface.source_table(body.faces, body.faces),
            layers: vec![BodyClothLayer {
                point_count,
                indices,
                role: ArmorComponentRole::OuterFabric,
                offset_metres: design.clearance.metres() + design.thickness.metres(),
                thickness_metres: design.thickness.metres(),
            }],
            surface: cut.surface.clone(),
        };
        let bands = collar
            .as_ref()
            .map_or_else(Vec::new, |c| c.bands(design, &mut cloth.layers));
        let point_count = cloth.layers.iter().map(|l| l.point_count).sum();
        let binding = design
            .binding
            .as_ref()
            .map(|binding| {
                ClothRibbonPlan::new(
                    binding_boundary(&cut, design, binding),
                    binding,
                    point_count,
                    &mut cloth.layers,
                )
            })
            .transpose()?;
        let ties = design
            .ties
            .as_ref()
            .map(|ties| -> Result<_> {
                Ok(ClothTiePlan::new(
                    &cut.tie_anchors()?,
                    ties,
                    &mut cloth.layers,
                ))
            })
            .transpose()?;
        Ok(Self {
            cloth,
            collar,
            binding,
            bands,
            ties,
        })
    }

    pub fn finish(&self, piece: &mut DevicePiece) -> Result<()> {
        if self.collar.is_some() {
            let source = piece.skin_aliases.clone();
            let body_count = self.cloth.surface.points.len();
            piece.partition_first_component(|triangle| {
                if triangle.iter().any(|&v| source[v as usize] >= body_count) {
                    ArmorComponentRole::CollarFabric
                } else {
                    ArmorComponentRole::OuterFabric
                }
            })?;
        }
        piece.normalize_cloth_charts(&[
            ArmorComponentRole::OuterFabric,
            ArmorComponentRole::CollarFabric,
            ArmorComponentRole::Neckband,
            ArmorComponentRole::CollarTrim,
        ])?;
        Ok(())
    }
}

fn binding_boundary(
    cut: &PartletCut,
    design: &PartletDesign,
    binding: &fabelgeist_armor::ClothBinding,
) -> ClothRibbonBoundary {
    let sewn_neck = if design.collar.is_some() {
        cut.neck
            .windows(2)
            .map(|p| [p[0], p[1]])
            .collect::<BTreeSet<_>>()
    } else {
        BTreeSet::new()
    };
    let hem = cut
        .reference
        .iter()
        .map(|p| p[1])
        .fold(f32::INFINITY, f32::min);
    ClothRibbonBoundary {
        edges: cut
            .surface
            .borders(&cut.reference)
            .into_iter()
            .filter(|e| !sewn_neck.contains(e))
            .collect(),
        widths: cut
            .reference
            .iter()
            .map(|p| {
                if (p[1] - hem).abs() < 2. * CUT_VERTEX_SNAP_METRES {
                    binding.hem_width
                } else {
                    binding.width
                }
            })
            .collect(),
    }
}
