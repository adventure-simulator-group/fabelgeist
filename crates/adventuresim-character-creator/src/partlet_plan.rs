//! The complete panel and collar carrier is thickened once, after sewing.
use crate::armor_frames::Wearer;
use crate::body_cloth_plan::BodyClothPlan;
use crate::cloth_ribbon_plan::{ClothRibbonBoundary, ClothRibbonPlan};
use crate::cloth_tie_plan::ClothTiePlan;
use crate::device_fit::DevicePiece;
use crate::partlet_collar::{CollarBandPlacement, CollarPlan};
use crate::partlet_cut::PartletCut;
use crate::surface_cut::CUT_VERTEX_SNAP_METRES;
use anyhow::{Context, Result};
use fabelgeist_armor::gpu::BodyClothLayer;
use fabelgeist_armor::{ArmorComponentRole, PartFrame, PartletDesign};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) struct PartletPlan {
    pub cloth: BodyClothPlan,
    pub collar: Option<CollarPlan>,
    pub binding: Option<ClothRibbonPlan>,
    pub bands: Vec<CollarBandPlacement>,
    pub ties: Option<ClothTiePlan>,
    pub closures: Option<crate::cloth_closure_fit::PlannedClothClosures>,
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
            .map(|collar| CollarPlan::new(collar, &cut, &mut indices))
            .transpose()?;
        let point_count =
            cut.surface.points.len() as u32 + collar.as_ref().map_or(0, |c| c.rows * c.columns);
        let closures = collar
            .as_ref()
            .zip(design.collar.as_ref())
            .map(|(plan, design)| plan.closures(design, cut.surface.points.len()))
            .transpose()?
            .flatten();
        let mut cloth = BodyClothPlan {
            table: cut.surface.source_table(body.faces, body.faces),
            layers: vec![BodyClothLayer {
                chart: None,
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
                    binding.thickness,
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
            closures,
        })
    }

    pub fn finish(&self, piece: &mut DevicePiece) -> Result<()> {
        if let Some(closures) = &self.closures {
            piece.attach_closures(&closures.design, &closures.opening, 0)?;
        }
        if let Some(collar) = &self.collar {
            let source = piece.skin_aliases.clone();
            let seam_progress = collar
                .seam
                .iter()
                .map(|point| (point.vertex as usize, point.progress))
                .collect::<BTreeMap<_, _>>();
            let seam_chart = piece
                .base
                .positions
                .iter()
                .zip(&source)
                .filter_map(|(position, point)| {
                    seam_progress
                        .get(point)
                        .map(|&progress| (position.map(f32::to_bits), [progress, 1.]))
                })
                .collect::<BTreeMap<_, _>>();
            let body_count = self.cloth.surface.points.len();
            let coordinates = piece
                .base
                .texcoords
                .as_mut()
                .context("partlet has no fabric chart")?;
            let collar_end = body_count + (collar.rows * collar.columns) as usize;
            let height = collar.heights[collar.heights.len() - 1];
            for (uv, &point) in coordinates.iter_mut().zip(&source) {
                if point >= body_count && point < collar_end {
                    let local = (point - body_count) as u32;
                    *uv = [
                        (local % collar.columns) as f32 / (collar.columns - 1) as f32,
                        1. - collar.heights[(local / collar.columns) as usize] / height,
                    ];
                }
            }
            let collar_start =
                self.cloth.surface.points.len() + (collar.panel_rows * collar.columns) as usize;
            piece.partition_first_component(|triangle| {
                if triangle.iter().any(|&v| source[v as usize] >= collar_start) {
                    ArmorComponentRole::CollarFabric
                } else {
                    ArmorComponentRole::OuterFabric
                }
            })?;
            let coordinates = piece
                .base
                .texcoords
                .as_mut()
                .context("partlet has no fabric chart")?;
            for component in &piece.base.components {
                if component.role == ArmorComponentRole::CollarFabric {
                    for vertex in component.vertices.clone() {
                        if let Some(&uv) =
                            seam_chart.get(&piece.base.positions[vertex].map(f32::to_bits))
                        {
                            coordinates[vertex] = uv;
                        }
                    }
                }
            }
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
