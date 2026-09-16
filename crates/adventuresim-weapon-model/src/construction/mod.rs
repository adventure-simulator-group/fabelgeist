//! Renderer-independent manufacturing operations shared by every weapon recipe.
//!
//! Construction uses double precision until the renderer boundary. Detail is
//! passed explicitly, so generating another weapon cannot change a build's
//! sampling budget.

mod clearance_envelope;
mod crenellated_socket;
mod curves;
mod profile;
pub(crate) use clearance_envelope::ClearanceEnvelope;
pub(crate) use profile::SmoothProfile;
mod section_shell;
pub(crate) use section_shell::LoftEnd;
mod partition;
mod partitioned_curve;
pub(crate) use partitioned_curve::partitioned_cubic;
mod planar_cells;
mod surface_quality;
#[cfg(test)]
mod surface_quality_tests;
mod surface_refinement;
pub(crate) use partition::PlanarCut;
mod polygon;
mod solids;
mod stocks;
#[cfg(test)]
mod surface_tests;
mod sweep;
#[cfg(test)]
mod tests;
mod truncate;

use serde::{Deserialize, Serialize};

pub(crate) use curves::*;
pub(crate) use polygon::*;
pub(crate) use stocks::*;
pub(crate) use sweep::*;

pub(crate) type Point = [f64; 3];
pub(crate) type PlanarPoint = [f64; 2];

/// Supported assembly-frame magnitude, including the float32 renderer boundary.
pub(crate) const MODEL_FRAME_EXTENT: f64 = 20.0;
/// Four rounding intervals at the frame scale define the local strip floor.
pub(crate) const FLOAT32_STRIP_SEPARATION: f64 = 4.0 * MODEL_FRAME_EXTENT * f32::EPSILON as f64;

/// Reject pathological detail requests before allocating their sampled grids.
/// This ceiling permits large authored surfaces while bounding hostile recipes.
pub(crate) fn construction_budget(triangles: f64) -> Result<(), String> {
    const MAX_SOLID_TRIANGLES: f64 = 262144.0;
    if triangles.is_finite() && (0.0..=MAX_SOLID_TRIANGLES).contains(&triangles) {
        Ok(())
    } else {
        Err("solid exceeds its bounded construction budget".into())
    }
}

/// Tessellation budget; every level retains the same structural components.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Detail {
    Low,
    #[default]
    Medium,
    High,
}

impl Detail {
    pub(crate) fn lathe_radial(self, radius: f64, requested: usize, exact: bool) -> usize {
        if exact && requested <= 8 {
            requested
        } else {
            self.radial(radius, requested)
        }
    }
    pub(crate) fn samples(self, requested: usize, minimum: usize) -> usize {
        let scale = match self {
            Self::Low => 0.5,
            Self::Medium => 1.0,
            Self::High => 2.0,
        };
        minimum.max((requested as f64 * scale).ceil() as usize)
    }

    pub(crate) fn error(self, distance: f64) -> f64 {
        distance
            * match self {
                Self::Low => 2.5,
                Self::Medium => 1.0,
                Self::High => 0.4,
            }
    }

    pub(crate) fn radial(self, radius: f64, requested: usize) -> usize {
        let floor = match self {
            Self::Low => 8,
            Self::Medium => 16,
            Self::High => 24,
        };
        let sagitta = radius.min(self.error(0.0003));
        floor
            .max(self.samples(requested, 4))
            .max((std::f64::consts::PI / (1.0 - sagitta / radius).acos()).ceil() as usize)
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Solid {
    pub(crate) positions: Vec<Point>,
    pub(crate) faces: Vec<[usize; 3]>,
    /// Equal nonzero groups permit smoothing across a shared vertex.
    pub(crate) surfaces: Vec<u32>,
}

impl Solid {
    pub(crate) fn triangle(&mut self, a: Point, b: Point, c: Point, surface: u32) {
        let index = self.positions.len();
        self.positions.extend([a, b, c]);
        self.faces.push([index, index + 1, index + 2]);
        self.surfaces.push(surface);
    }

    pub(crate) fn quad(&mut self, a: Point, b: Point, c: Point, d: Point, surface: u32) {
        self.triangle(a, b, c, surface);
        self.triangle(a, c, d, surface);
    }

    pub(crate) fn volume(&self) -> f64 {
        self.faces
            .iter()
            .map(|&[a, b, c]| {
                dot(
                    self.positions[a],
                    cross(self.positions[b], self.positions[c]),
                ) / 6.0
            })
            .sum()
    }

    pub(crate) fn positive(mut self) -> Self {
        if self.volume() < -1e-10 {
            for face in &mut self.faces {
                face.swap(1, 2);
            }
        }
        self
    }

    pub(crate) fn transform(mut self, rotation: Point, offset: Point) -> Self {
        for point in &mut self.positions {
            *point = add(rotate(*point, rotation), offset);
        }
        self
    }
    pub(crate) fn append(&mut self, other: Self) {
        let base = self.positions.len();
        let surface_base = self.surfaces.iter().copied().max().unwrap_or(0);
        self.positions.extend(other.positions);
        self.faces
            .extend(other.faces.into_iter().map(|face| face.map(|i| i + base)));
        self.surfaces
            .extend(other.surfaces.into_iter().map(|surface| {
                if surface == 0 {
                    0
                } else {
                    surface + surface_base
                }
            }));
    }
}

pub(crate) fn add(a: Point, b: Point) -> Point {
    std::array::from_fn(|i| a[i] + b[i])
}
pub(crate) fn sub(a: Point, b: Point) -> Point {
    std::array::from_fn(|i| a[i] - b[i])
}
pub(crate) fn mul(a: Point, scalar: f64) -> Point {
    a.map(|v| v * scalar)
}
pub(crate) fn dot(a: Point, b: Point) -> f64 {
    (0..3).map(|i| a[i] * b[i]).sum()
}
pub(crate) fn cross(a: Point, b: Point) -> Point {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
pub(crate) fn magnitude(a: Point) -> f64 {
    dot(a, a).sqrt()
}
pub(crate) fn normalize(a: Point) -> Point {
    let n = magnitude(a);
    if n == 0.0 { [0.0; 3] } else { mul(a, 1.0 / n) }
}
pub(crate) fn lerp(a: Point, b: Point, t: f64) -> Point {
    add(a, mul(sub(b, a), t))
}

pub(crate) fn rotate(mut point: Point, degrees: Point) -> Point {
    for (axis, angle) in degrees.iter().enumerate() {
        let (s, c) = angle.to_radians().sin_cos();
        let a = (axis + 1) % 3;
        let b = (axis + 2) % 3;
        let x = point[a] * c - point[b] * s;
        let y = point[a] * s + point[b] * c;
        point[a] = x;
        point[b] = y;
    }
    point
}

pub(crate) fn inverse_rotate(mut point: Point, degrees: Point) -> Point {
    for axis in (0..3).rev() {
        let (s, c) = (-degrees[axis]).to_radians().sin_cos();
        let a = (axis + 1) % 3;
        let b = (axis + 2) % 3;
        let x = point[a] * c - point[b] * s;
        let y = point[a] * s + point[b] * c;
        point[a] = x;
        point[b] = y;
    }
    point
}
