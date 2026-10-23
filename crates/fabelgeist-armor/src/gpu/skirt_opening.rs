//! Frozen topological cut through the carrier before cloth shelling.
use super::coord::CoordShell;
use super::{SkirtAttachment, SkirtSeamLayout};
use crate::{GenerateError, Millimeters, SKIRT_AXIAL_SEGMENTS, SKIRT_RING_SEGMENTS, SkirtDesign};
use std::collections::BTreeMap;

// Numerical upright-chart budget shared by path classification and tip reuse.
pub(super) const VERTICAL_ROUNDOFF_M: f32 = 0.000001;
/// Skirt kinds zero through eight include interpolated cut edges and tip. Lip tags preserve
/// the original xyz payload and encode the original kind plus topological side.
pub(super) const ORIGINAL_COORD_VARIANTS: f32 = 9.;
/// Native canonical chart boundary: angle in radians, upright Y in metres.
#[derive(Clone, Copy, Debug)]
pub struct OpeningChartPoint {
    pub angle: f32,
    pub y: f32,
}
/// Original canonical body chart; later sleeve/collar procedural points are absent.
pub struct SkirtOpeningSource {
    pub points: Vec<OpeningChartPoint>,
    pub start_body_vertex: u32,
    pub axis_angle: f32,
    pub waist_y: f32,
    pub hem_y: f32,
    pub top_y: f32,
    pub tip_y: f32,
    pub radius: f32,
    pub gap: Millimeters,
    pub overlap: Millimeters,
}
/// Original carrier edge, in native vertex ordinals and affine blend share.
#[derive(Clone, Copy)]
pub struct OpeningEdgeSource {
    pub vertices: [usize; 2],
    pub blend: f32,
}
pub struct SkirtDressingOpening {
    pub(super) shell: CoordShell,
    axis_angle: f32,
    gap: Millimeters,
    overlap: Millimeters,
    end_row: f32,
    top_row: f32,
    aliases: BTreeMap<usize, usize>,
    edges: BTreeMap<usize, OpeningEdgeSource>,
    body_vertices: std::ops::Range<usize>,
}
#[derive(Debug, thiserror::Error)]
pub enum SkirtOpeningError {
    #[error("invalid canonical dressing-opening source")]
    Source,
    #[error("no descending carrier edge path reaches the dressing slit tip")]
    Path,
    #[error("dressing slit must terminate inside the ordinary hanging cloth")]
    Tip,
    #[error("dressing slit path has an ambiguous or non-simple cloth vertex fan")]
    Fan,
    #[error(transparent)]
    Carrier(#[from] GenerateError),
}
impl SkirtDressingOpening {
    pub fn new(
        design: &SkirtDesign,
        patch: &SkirtAttachment,
        source: SkirtOpeningSource,
    ) -> Result<Self, SkirtOpeningError> {
        if patch.dressing_opening.is_some()
            || patch.seam_layout != SkirtSeamLayout::Transition
            || !crate::GownDressingSlit::OVERLAP_RANGE.contains(&source.overlap.0)
            || !crate::ClothSlit::GAP_RANGE.contains(&source.gap.0)
            || source.start_body_vertex as usize >= source.points.len()
            || source.points.len() > patch.point_count as usize
            || !source.radius.is_finite()
            || source.radius <= 0.
            || ![
                source.axis_angle,
                source.waist_y,
                source.hem_y,
                source.top_y,
                source.tip_y,
            ]
            .into_iter()
            .all(f32::is_finite)
            || !(source.hem_y < source.tip_y
                && source.tip_y < source.waist_y
                && source.waist_y < source.top_y)
            || source
                .points
                .iter()
                .any(|p| !p.angle.is_finite() || !p.y.is_finite())
        {
            return Err(SkirtOpeningError::Source);
        }
        let mut shell = super::skirt::carrier(design, Some(patch))?;
        let mut chart = shell
            .coords
            .iter()
            .map(|&coord| source.chart(coord, patch))
            .collect::<Vec<_>>();
        let first = patch.carrier_point_count();
        let start = first + source.start_body_vertex as usize;
        if chart.get(start).is_none_or(Option::is_none) {
            return Err(SkirtOpeningError::Source);
        }
        let mut cut = super::skirt_opening_cut::MeridianCut::insert(
            &mut shell,
            &mut chart,
            source.axis_angle,
        )?;
        let mut path = super::skirt_opening_path::path(&chart, &shell.indices, start, &source)?;
        let end_row = (source.waist_y - source.tip_y) / (source.waist_y - source.hem_y)
            * SKIRT_AXIAL_SEGMENTS as f32;
        let top_row = (source.waist_y - source.top_y) / (source.waist_y - source.hem_y)
            * SKIRT_AXIAL_SEGMENTS as f32;
        let aliases = super::skirt_opening_split::split(
            &mut shell,
            &mut path,
            &source,
            &chart,
            &mut cut.sources,
        )?;
        Ok(Self {
            shell,
            axis_angle: source.axis_angle,
            gap: source.gap,
            overlap: source.overlap,
            end_row,
            top_row,
            aliases,
            edges: cut.sources,
            body_vertices: first..first + source.points.len(),
        })
    }
    /// Resolve only extra cut-lip ordinals. Other construction owners are unchanged.
    pub fn overlap_metres(&self) -> f32 {
        self.overlap.metres()
    }
    /// Resolve extra cut-lip ordinals while preserving other source owners.
    pub fn source_vertex(&self, vertex: usize) -> usize {
        let source = self.aliases.get(&vertex).copied().unwrap_or(vertex);
        let Some(edge) = self.edges.get(&source) else {
            return source;
        };
        let [a, b] = edge.vertices.map(|parent| self.source_vertex(parent));
        // An edge spanning the torso boundary belongs to the transition. It
        // cannot acquire a body-only material chart from its other endpoint.
        if !self.body_vertices.contains(&a) {
            return a;
        }
        if !self.body_vertices.contains(&b) {
            return b;
        }
        a.min(b)
    }
    /// Intersections in creation order; each parent precedes its child.
    pub fn edge_sources(&self) -> impl Iterator<Item = (&usize, &OpeningEdgeSource)> {
        self.edges.iter()
    }
    /// The matching interpolated carrier for a shell copy or duplicated lip.
    pub fn edge_source_vertex(&self, vertex: usize) -> Option<usize> {
        let source = self.aliases.get(&vertex).copied().unwrap_or(vertex);
        self.edges.contains_key(&source).then_some(source)
    }
    pub(super) fn shader_words(&self) -> [f32; 5] {
        [
            self.axis_angle,
            self.gap.metres(),
            self.end_row,
            self.top_row,
            // The main skirt side is the lip away from the anterior axis.
            // Signed metres select its topological side at the shader boundary.
            self.overlap.metres()
                * if self.axis_angle.rem_euclid(std::f32::consts::TAU) > std::f32::consts::PI {
                    1.
                } else {
                    -1.
                },
        ]
    }
}
impl SkirtOpeningSource {
    fn chart(&self, coord: [f32; 4], patch: &SkirtAttachment) -> Option<OpeningChartPoint> {
        let [x, y, z, kind] = coord;
        if kind == 0. {
            return Some(OpeningChartPoint {
                angle: x / SKIRT_RING_SEGMENTS as f32 * std::f32::consts::TAU,
                y: self.waist_y + (self.hem_y - self.waist_y) * y / SKIRT_AXIAL_SEGMENTS as f32,
            });
        }
        if kind == 1. {
            return self.points.get(x as usize).copied();
        }
        if kind == 2. {
            let lower = self.waist_y
                + (self.hem_y - self.waist_y) * patch.first_row as f32
                    / SKIRT_AXIAL_SEGMENTS as f32;
            return Some(OpeningChartPoint {
                angle: y / SKIRT_RING_SEGMENTS as f32 * std::f32::consts::TAU,
                y: self.waist_y + (lower - self.waist_y) * z,
            });
        }
        None
    }
}
pub(super) fn angle_delta(angle: f32, axis: f32) -> f32 {
    (angle - axis + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI
}
