//! Frozen real aperture contours own sewn thread and lace routing centers.
use crate::armor_frames::Wearer;
use crate::cloth_opening::ClothSurfaceSample;
use crate::surface_cut::{ClothAperture, SurfaceCut};
use anyhow::{Result, bail, ensure};
use fabelgeist_armor::ClothLacing;
/// Native final outer-carrier ordinals, retained for every direct refit.
pub(crate) struct SewnEyelet {
    pub boundary: Vec<usize>,
    /// Positive affine span across an empty aperture, not cloth-triangle membership.
    pub center: ClothSurfaceSample,
}
pub(crate) struct SewnEyeletPair {
    pub left: SewnEyelet,
    pub right: SewnEyelet,
}
impl SewnEyeletPair {
    pub fn from_cut(
        cut: &SurfaceCut,
        body: &Wearer<'_>,
        design: &ClothLacing,
    ) -> Result<Vec<Self>> {
        let positions = cut
            .points
            .iter()
            .map(|p| {
                crate::surface_cut::interpolate(
                    body.faces[p.triangle].map(|v| body.positions[v as usize]),
                    p.weights,
                )
            })
            .collect::<Vec<_>>();
        let boundaries = cut.aperture_loops(&positions)?;
        ensure!(
            boundaries.len() == 2 * design.positions.len(),
            "sewn eyelet station ownership is incomplete"
        );
        let mut eyelets = cut
            .apertures
            .iter()
            .zip(boundaries)
            .map(|(aperture, boundary)| SewnEyelet::new(aperture, boundary, &positions, design))
            .collect::<Result<Vec<_>>>()?
            .into_iter();
        let mut pairs = Vec::new();
        while let Some(left) = eyelets.next() {
            let Some(right) = eyelets.next() else {
                bail!("sewn eyelet station has no matching side");
            };
            pairs.push(Self { left, right });
        }
        Ok(pairs)
    }
}
impl SewnEyelet {
    fn new(
        aperture: &ClothAperture,
        boundary: Vec<u32>,
        positions: &[[f32; 3]],
        design: &ClothLacing,
    ) -> Result<Self> {
        let chart = boundary
            .iter()
            .map(|&v| aperture.project(positions[v as usize]))
            .collect::<Vec<_>>();
        // Native aperture-plane coordinates in metres. Measure actual finite
        // segments in f64: a tiny edge's extended line is not a cloth obstacle.
        let mut inside = false;
        for i in 0..chart.len() {
            let a = chart[i].map(f64::from);
            let b = chart[(i + 1) % chart.len()].map(f64::from);
            let edge = [b[0] - a[0], b[1] - a[1]];
            let squared = edge[0] * edge[0] + edge[1] * edge[1];
            let progress = if squared > 0. {
                (-(a[0] * edge[0] + a[1] * edge[1]) / squared).clamp(0., 1.)
            } else {
                0.
            };
            let distance = (a[0] + edge[0] * progress).hypot(a[1] + edge[1] * progress);
            ensure!(
                distance > f64::from(design.cord.metres()) * 0.5,
                "sewn eyelet aperture clearance {}mm cannot admit cord diameter {}mm",
                distance * 1000.,
                design.cord.metres() * 1000.
            );
            if (a[1] > 0.) != (b[1] > 0.) && a[0] - a[1] * edge[0] / edge[1] > 0. {
                inside = !inside;
            }
        }
        ensure!(inside, "sewn eyelet does not enclose its cord center");
        for a in 0..chart.len() {
            for b in a + 1..chart.len() {
                for c in b + 1..chart.len() {
                    let triangle = [chart[a], chart[b], chart[c]].map(|point| point.map(f64::from));
                    if let Some(weights) = crate::cloth_chart_triangle::weights(triangle, [0., 0.])
                    {
                        return Ok(Self {
                            center: ClothSurfaceSample {
                                vertices: [
                                    boundary[a] as usize,
                                    boundary[b] as usize,
                                    boundary[c] as usize,
                                ],
                                weights,
                            },
                            boundary: boundary.into_iter().map(|v| v as usize).collect(),
                        });
                    }
                }
            }
        }
        bail!("sewn eyelet lacks a positive center span inside its actual boundary")
    }
}
