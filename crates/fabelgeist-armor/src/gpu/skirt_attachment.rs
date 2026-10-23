//! Native topology adapter for a cloned body patch and its shared waist seam.

use crate::{GenerateError, SKIRT_RING_SEGMENTS};

/// Fitted points follow the ordinary skirt profile in the shared fit buffer.
/// Each has local position and local normal, packed as six native floats.
pub const SKIRT_ATTACHMENT_POINT_WORDS: u32 = 6;
/// Interior rings in the transition; waist and skirt endpoints are shared.
pub(crate) const TRANSITION_RINGS: usize = 4;

/// GPU topology boundary. Body triangles use patch-local vertex indices;
/// waist vertices are ordered by increasing radians around the fitted Y axis.
pub struct SkirtAttachment {
    pub point_count: u32,
    pub indices: Vec<u32>,
    pub waist: Vec<SkirtWaistVertex>,
}

pub struct SkirtWaistVertex {
    pub vertex: u32,
    pub angle: f32,
}

impl SkirtAttachment {
    pub(crate) fn append(
        &self,
        coords: &mut Vec<[f32; 4]>,
        indices: &mut Vec<u32>,
    ) -> Result<(), GenerateError> {
        if self.point_count == 0
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
            .map(|p| SkirtWaistVertex {
                vertex: p.vertex + first,
                angle: p.angle,
            })
            .collect::<Vec<_>>();
        for ring in 1..=TRANSITION_RINGS {
            let first_ring = coords.len() as u32;
            let progress = ring as f32 / (TRANSITION_RINGS + 1) as f32;
            coords.extend((0..SKIRT_RING_SEGMENTS).map(|column| {
                [
                    (self.point_count as usize + column) as f32,
                    column as f32,
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

fn angular_ring(first: u32) -> Vec<SkirtWaistVertex> {
    (0..SKIRT_RING_SEGMENTS)
        .map(|column| SkirtWaistVertex {
            vertex: first + column as u32,
            angle: std::f32::consts::TAU * column as f32 / SKIRT_RING_SEGMENTS as f32,
        })
        .collect()
}

/// A zipper between unequal closed rings. Each ring edge is consumed once;
/// no spatial welding or independently thickened seam is needed.
fn stitch(upper: &[SkirtWaistVertex], lower: &[SkirtWaistVertex], indices: &mut Vec<u32>) {
    let (mut top, mut bottom) = (0, 0);
    let next_angle = |ring: &[SkirtWaistVertex], next: usize| {
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
