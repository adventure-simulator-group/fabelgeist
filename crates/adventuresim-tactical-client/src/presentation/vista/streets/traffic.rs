//! Static traffic history shared by every overlapping ground patch.
//!
//! Curves are baked once into metre-aligned tiles. The production shader samples
//! those tiles for road shoulders, broad churn and individual wheel marks.
use super::*;
use std::collections::BTreeMap;

mod mask;
mod paths;
#[cfg(test)]
mod tests;

pub(super) use mask::TrafficMask;
pub(super) use paths::TrafficNetwork;

const TILE_METRES: f32 = 64.0;
const TEXELS_PER_METRE: f32 = 4.0;
const TILE_INTERIOR_PIXELS: usize = 256;
const FILTER_GUTTER_PIXELS: usize = 1;
const TILE_PIXELS: usize = TILE_INTERIOR_PIXELS + FILTER_GUTTER_PIXELS * 2;
const ROAD_SHOULDER_METRES: f32 = 1.2;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct TrafficTile(pub i32, pub i32);

impl TrafficTile {
    pub(super) fn at(point: Vec2) -> Self {
        let cell = (point / TILE_METRES).floor().as_ivec2();
        Self(cell.x, cell.y)
    }

    fn origin(self) -> Vec2 {
        Vec2::new(self.0 as f32, self.1 as f32) * TILE_METRES
    }

    pub(super) fn corners(self) -> [Vec2; 4] {
        let origin = self.origin();
        [
            origin,
            origin + Vec2::X * TILE_METRES,
            origin + Vec2::splat(TILE_METRES),
            origin + Vec2::Y * TILE_METRES,
        ]
    }

    pub(super) fn covering(minimum: Vec2, maximum: Vec2) -> impl Iterator<Item = Self> {
        let min = Self::at(minimum);
        let max = Self::at(maximum);
        (min.1..=max.1).flat_map(move |y| (min.0..=max.0).map(move |x| Self(x, y)))
    }
}

#[derive(Clone, Copy)]
struct Road {
    start: Vec2,
    end: Vec2,
    half_width: f32,
}

impl Road {
    fn coordinates(self, point: Vec2) -> Vec2 {
        let forward = (self.end - self.start).normalize();
        let delta = point - self.start;
        Vec2::new(forward.perp_dot(delta), forward.dot(delta))
    }

    fn clearance(self, point: Vec2) -> f32 {
        let local = self.coordinates(point);
        (self.half_width - local.x.abs())
            .min(local.y)
            .min(self.start.distance(self.end) - local.y)
    }
}

#[derive(Clone, Copy)]
struct WheelStroke {
    start: Vec2,
    end: Vec2,
    half_width: f32,
    strength: f32,
}

fn smooth_band(inner: f32, outer: f32, distance: f32) -> f32 {
    let t = ((distance - inner) / (outer - inner)).clamp(0.0, 1.0);
    1.0 - t * t * (3.0 - 2.0 * t)
}

fn quad_clearance(corners: [Vec2; 4], point: Vec2) -> f32 {
    let winding = (corners[1] - corners[0])
        .perp_dot(corners[3] - corners[0])
        .signum();
    (0..4)
        .map(|i| {
            let edge = corners[(i + 1) % 4] - corners[i];
            edge.normalize_or_zero().perp_dot(point - corners[i]) * winding
        })
        .fold(f32::INFINITY, f32::min)
}

// Internal rectangle ends are not boundaries of the road union. Probe only
// near an individual edge; ordinary road interiors use the exact fast path.
fn union_support(point: Vec2, radius: f32, clearance: impl Fn(Vec2) -> f32) -> bool {
    if clearance(point) >= radius {
        return true;
    }
    if clearance(point) < -f32::EPSILON {
        return false;
    }
    const ENVELOPE_DIRECTIONS: usize = 16;
    (0..ENVELOPE_DIRECTIONS).all(|index| {
        let angle = index as f32 * std::f32::consts::TAU / ENVELOPE_DIRECTIONS as f32;
        clearance(point + Vec2::new(angle.cos(), angle.sin()) * radius) >= 0.0
    })
}
