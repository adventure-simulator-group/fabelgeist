//! Native topology adapter for a cloned body patch and its shared waist seam.

use crate::skirt::SKIRT_MESH_RING_SEGMENTS;
use crate::{GenerateError, SKIRT_RING_SEGMENTS};

/// Fitted points follow the ordinary skirt profile in the shared fit buffer.
/// Each has local position and local normal, packed as six native floats.
pub const SKIRT_ATTACHMENT_POINT_WORDS: u32 = 6;
/// Interior rings in the transition; waist and skirt endpoints are shared.
pub(crate) const TRANSITION_RINGS: usize = 2;

/// GPU topology boundary. Body triangles use patch-local vertex indices;
/// waist vertices are ordered by increasing radians around the fitted Y axis.
pub struct SkirtAttachment {
    pub seam_layout: SkirtSeamLayout,
    /// Native render intervals; fitting still samples the dense section grid.
    pub hanging_intervals: usize,
    /// First hanging skirt row after the sewn waist transition.
    pub first_row: usize,
    pub point_count: u32,
    pub indices: Vec<u32>,
    pub waist: Vec<ClothRingVertex>,
    pub layers: Vec<AttachedClothLayer>,
}

/// A finishing sheet over already evaluated points in the attachment table.
/// Offsets are relative to that cloth face, not to the wearer's bare surface.
pub struct AttachedClothLayer {
    pub first_point: u32,
    pub layer: super::BodyClothLayer,
}

pub struct ClothRingVertex {
    pub vertex: u32,
    pub angle: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SkirtSeamLayout {
    /// Independent hanging grid with smooth transition rings.
    Transition,
    /// Extrude a fully released waist's own columns vertically to the hem.
    StraightFall,
}

impl SkirtAttachment {
    pub fn hanging_columns(&self) -> usize {
        match self.seam_layout {
            SkirtSeamLayout::Transition => SKIRT_MESH_RING_SEGMENTS,
            SkirtSeamLayout::StraightFall => self.waist.len(),
        }
    }

    pub fn carrier_point_count(&self) -> usize {
        (self.hanging_intervals + 1) * self.hanging_columns()
    }
    pub(crate) fn append(
        &self,
        coords: &mut Vec<[f32; 4]>,
        indices: &mut Vec<u32>,
    ) -> Result<(), GenerateError> {
        if self.first_row == 0
            || !(1..=crate::SKIRT_MESH_AXIAL_SEGMENTS).contains(&self.hanging_intervals)
            || self.first_row >= crate::SKIRT_AXIAL_SEGMENTS
            || self.point_count == 0
            || self.waist.len() < 3
            || self.indices.iter().any(|&i| i >= self.point_count)
            || self
                .waist
                .iter()
                .any(|p| p.vertex >= self.point_count || !p.angle.is_finite())
            || self.waist.windows(2).any(|p| p[0].angle >= p[1].angle)
        {
            return Err(GenerateError::InvalidSurface);
        }
        let first = coords.len() as u32;
        coords.extend((0..self.point_count).map(|point| [point as f32, 0.0, 0.0, 1.0]));
        indices.extend(self.indices.iter().map(|i| i + first));
        let mut upper = self
            .waist
            .iter()
            .map(|p| ClothRingVertex {
                vertex: p.vertex + first,
                angle: p.angle,
            })
            .collect::<Vec<_>>();
        if self.seam_layout == SkirtSeamLayout::StraightFall {
            let lower = self
                .waist
                .iter()
                .enumerate()
                .map(|(i, p)| ClothRingVertex {
                    vertex: i as u32,
                    angle: p.angle,
                })
                .collect::<Vec<_>>();
            stitch(&upper, &lower, indices);
            return Ok(());
        }
        for ring in 1..=TRANSITION_RINGS {
            let first_ring = coords.len() as u32;
            let progress = ring as f32 / (TRANSITION_RINGS + 1) as f32;
            coords.extend((0..SKIRT_MESH_RING_SEGMENTS).map(|column| {
                let sample = column * SKIRT_RING_SEGMENTS / SKIRT_MESH_RING_SEGMENTS;
                [
                    (self.point_count as usize + sample) as f32,
                    sample as f32,
                    progress,
                    2.0,
                ]
            }));
            let lower = angular_ring(first_ring);
            stitch(&upper, &lower, indices);
            upper = lower;
        }
        stitch(&upper, &angular_ring(0), indices);
        Ok(())
    }
}

fn angular_ring(first: u32) -> Vec<ClothRingVertex> {
    (0..SKIRT_MESH_RING_SEGMENTS)
        .map(|column| ClothRingVertex {
            vertex: first + column as u32,
            angle: std::f32::consts::TAU * column as f32 / SKIRT_MESH_RING_SEGMENTS as f32,
        })
        .collect()
}

/// A zipper between unequal closed rings. Each ring edge is consumed once;
/// no spatial welding or independently thickened seam is needed.
fn stitch(upper: &[ClothRingVertex], lower: &[ClothRingVertex], indices: &mut Vec<u32>) {
    let (mut top, mut bottom) = (0, 0);
    let next_angle = |ring: &[ClothRingVertex], next: usize| {
        if next >= ring.len() {
            ring[0].angle + std::f32::consts::TAU
        } else {
            ring[next].angle
        }
    };
    while top < upper.len() || bottom < lower.len() {
        let a = upper[top % upper.len()].vertex;
        let b = lower[bottom % lower.len()].vertex;
        if top < upper.len()
            && (bottom == lower.len() || next_angle(upper, top + 1) < next_angle(lower, bottom + 1))
        {
            indices.extend([a, b, upper[(top + 1) % upper.len()].vertex]);
            top += 1;
        } else {
            indices.extend([a, b, lower[(bottom + 1) % lower.len()].vertex]);
            bottom += 1;
        }
    }
}
