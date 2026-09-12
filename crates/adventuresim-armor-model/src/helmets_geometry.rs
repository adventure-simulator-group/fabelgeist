//! Shared surface construction; families decide boundaries and connectivity.

use std::f32::consts::{FRAC_PI_2, TAU};

use crate::{GenerateError, parametric::PartMesh};

pub(super) const AROUND: usize = 48;
const DOME_RINGS: usize = 12;

#[derive(Default)]
pub(super) struct Surface {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub relief: Vec<f32>,
}

impl Surface {
    /// Redistribute a bowl's angular samples to register a sized face aperture.
    pub fn reshape_front_arc(&mut self, radii: [f32; 3], original: f32, opening: f32) {
        for point in &mut self.positions {
            let x = point[0] / radii[0];
            let z = point[2] / radii[2];
            let angle = x.atan2(z);
            let absolute = angle.abs();
            let mapped = if absolute <= original {
                absolute * opening / original
            } else {
                opening
                    + (std::f32::consts::PI - opening) * (absolute - original)
                        / (std::f32::consts::PI - original)
            } * angle.signum();
            let radius = x.hypot(z);
            point[0] = radii[0] * radius * mapped.sin();
            point[2] = radii[2] * radius * mapped.cos();
        }
    }

    pub fn vertex(&mut self, point: [f32; 3]) -> u32 {
        let id = self.positions.len() as u32;
        self.positions.push(point);
        self.relief.push(0.0);
        id
    }

    pub fn connect(&mut self, upper: &[u32], lower: &[u32], periodic: bool) {
        let count = if periodic {
            upper.len()
        } else {
            upper.len() - 1
        };
        for i in 0..count {
            let next = (i + 1) % upper.len();
            self.indices.extend([
                upper[i],
                lower[i],
                lower[next],
                upper[i],
                lower[next],
                upper[next],
            ]);
        }
    }

    /// A lower exponent retains width higher up a broad skull's meridian.
    pub fn full_dome(&mut self, radii: [f32; 3], brow: f32, exponent: f32) -> Vec<u32> {
        let angles = (0..AROUND)
            .map(|i| i as f32 / AROUND as f32 * TAU)
            .collect::<Vec<_>>();
        self.dome_angles(radii, brow, exponent, &angles)
    }

    pub fn dome_angles(
        &mut self,
        radii: [f32; 3],
        brow: f32,
        exponent: f32,
        angles: &[f32],
    ) -> Vec<u32> {
        let pole = self.vertex([0.0, radii[1], 0.0]);
        let mut previous = Vec::new();
        for row in 1..=DOME_RINGS {
            let latitude = row as f32 / DOME_RINGS as f32 * FRAC_PI_2;
            let ring = angles
                .iter()
                .map(|&angle| {
                    self.vertex([
                        radii[0] * latitude.sin().powf(exponent) * angle.sin(),
                        brow + (radii[1] - brow) * latitude.cos(),
                        radii[2] * latitude.sin().powf(exponent) * angle.cos(),
                    ])
                })
                .collect::<Vec<_>>();
            if previous.is_empty() {
                for i in 0..angles.len() {
                    self.indices
                        .extend([pole, ring[i], ring[(i + 1) % angles.len()]]);
                }
            } else {
                self.connect(&previous, &ring, true);
            }
            previous = ring;
        }
        previous
    }

    pub fn shell(
        self,
        thickness: f32,
        extrusion: crate::ShellExtrusion,
    ) -> Result<PartMesh, GenerateError> {
        let relief = self
            .relief
            .iter()
            .any(|height| *height > 0.0)
            .then_some(self.relief);
        PartMesh::from_relief_surface(
            self.positions,
            self.indices,
            thickness,
            crate::BoundaryNormals::Smooth,
            extrusion,
            relief.map(crate::SurfaceRelief::ShellHeights),
        )
    }
}
