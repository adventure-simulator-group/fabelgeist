//! Shared surface construction; families decide boundaries and connectivity.

use std::f32::consts::{FRAC_PI_2, TAU};

use crate::{GenerateError, parametric::PartMesh};

pub(super) const AROUND: usize = 48;
const DOME_RINGS: usize = 12;

#[derive(Default)]
pub(super) struct Surface {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

impl Surface {
    pub fn vertex(&mut self, point: [f32; 3]) -> u32 {
        let id = self.positions.len() as u32;
        self.positions.push(point);
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

    /// Axisymmetric skull bowl with a true single pole, not collapsed quads.
    pub fn dome(&mut self, radii: [f32; 3], brow: f32) -> Vec<u32> {
        self.full_dome(radii, brow, 1.0)
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

    pub fn shell(self, thickness: f32) -> Result<PartMesh, GenerateError> {
        PartMesh::from_surface(
            self.positions,
            self.indices,
            thickness,
            crate::BoundaryNormals::Smooth,
            crate::ShellExtrusion::Normal,
        )
    }
}

/// Independent polygonal sheet with real thickness, useful for a sagittal comb.
pub(super) fn comb(
    radii: [f32; 3],
    brow: f32,
    height: f32,
    thickness: f32,
) -> Result<PartMesh, GenerateError> {
    const COMB_SEGMENTS: usize = 24;
    const COMB_BASE_INSET: f32 = 0.004;
    const COMB_HALF_WIDTH: f32 = 0.003;
    let mut surface = Surface::default();
    for i in 0..=COMB_SEGMENTS {
        let t = i as f32 / COMB_SEGMENTS as f32;
        let angle = (0.08 + 0.84 * t) * std::f32::consts::PI;
        let base = brow + (radii[1] - brow) * angle.sin();
        let z = radii[2] * angle.cos();
        surface.vertex([COMB_HALF_WIDTH, base - COMB_BASE_INSET, z]);
        let crest = (std::f32::consts::PI * t).sin().max(0.0).powf(0.8);
        surface.vertex([COMB_HALF_WIDTH, base + height * crest, z]);
    }
    for i in 0..COMB_SEGMENTS as u32 {
        let a = i * 2;
        surface.indices.extend([a, a + 2, a + 3, a, a + 3, a + 1]);
    }
    surface.shell(thickness.max(COMB_HALF_WIDTH * 2.0))
}
