//! Physical loft coordinates from a bound neckline to the hanging profile.
use crate::armor_frames::Wearer;
use crate::surface_cut::{SurfaceCut, dot, interpolate};
use anyhow::{Context, Result, ensure};
use fabelgeist_armor::{PartFrame, gpu::ClothRingVertex};
use fabelgeist_rig::RigJointName;
use std::collections::{BTreeMap, BTreeSet};

/// Native source pair, f32 interpolation bits and fully released border flag.
pub(crate) const NECK_SAMPLE_WORDS: usize = 4;

/// Native per-cut-point chart fractions and neckline source-pair/blend words.
pub(crate) struct GownEasePlan {
    pub weights: Vec<f32>,
    pub neckline_samples: Vec<u32>,
}

impl GownEasePlan {
    pub fn new(
        cut: &SurfaceCut,
        body: &Wearer<'_>,
        hips: &PartFrame,
        head: &PartFrame,
        hem: fabelgeist_armor::Millimeters,
    ) -> Result<Self> {
        let marks = crate::sewn_torso_boundaries::TorsoBoundaries::new(cut, body, head)?;
        let angle = |point: [f32; 3]| {
            let delta = std::array::from_fn(|axis| point[axis] - hips.origin[axis]);
            dot(delta, head.axes[0])
                .atan2(dot(delta, head.axes[2]))
                .rem_euclid(std::f32::consts::TAU)
        };
        let mut neckline = marks
            .neckline
            .iter()
            .map(|&vertex| ClothRingVertex {
                vertex,
                angle: angle(marks.positions[vertex as usize]),
            })
            .collect::<Vec<_>>();
        let winding = (0..neckline.len())
            .map(|i| {
                let delta = neckline[(i + 1) % neckline.len()].angle - neckline[i].angle;
                delta.sin().atan2(delta.cos())
            })
            .sum::<f32>();
        ensure!(
            winding.abs() > std::f32::consts::PI,
            "neckline must enclose its suspension axis"
        );
        if winding < 0. {
            neckline.reverse();
        }
        let first = neckline
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.angle.total_cmp(&b.angle))
            .map(|(i, _)| i)
            .context("empty neckline loft")?;
        neckline.rotate_left(first);
        ensure!(
            neckline.windows(2).all(|p| p[0].angle < p[1].angle),
            "neckline is not star-shaped around the suspension axis"
        );
        let root = RigJointName::ROOT.require_in(body.joint_names)?;
        let waist = body.joints[root][1] + hem.metres();
        let mut neckline_samples = Vec::new();
        let mut weights = Vec::new();
        for (index, &point) in marks.positions.iter().enumerate() {
            let sample = if marks.neckline.contains(&(index as u32)) {
                [index as u32, index as u32, 0.0_f32.to_bits()]
            } else {
                crate::skirt_join::angular_sample(&neckline, angle(point))
            };
            let blend = f32::from_bits(sample[2]);
            let neck = interpolate(
                [
                    marks.positions[sample[0] as usize],
                    marks.positions[sample[1] as usize],
                    point,
                ],
                [1. - blend, blend, 0.],
            );
            ensure!(
                neck[1] > waist,
                "loose neckline must remain above its suspension seam"
            );
            weights.push(((neck[1] - point[1]) / (neck[1] - waist)).clamp(0., 1.));
            neckline_samples.extend(sample);
            neckline_samples.push(u32::from(marks.waist.contains(&(index as u32))));
        }
        Ok(Self {
            weights,
            neckline_samples,
        })
    }

    /// Retain the body-derived topology only until the cloth is fully free.
    /// Whole triangles remain source-correspondent. The jagged sewing border
    /// is extruded in its own columns; no unstructured torso faces reach the
    /// former waist. Armholes and the complete upper band remain intact.
    pub fn upper_patch(
        &self,
        cut: SurfaceCut,
        body: &Wearer<'_>,
        head: &PartFrame,
        release: fabelgeist_armor::Permille,
    ) -> Result<SurfaceCut> {
        let marks = crate::sewn_torso_boundaries::TorsoBoundaries::new(&cut, body, head)?;
        let armpit = marks
            .armholes
            .iter()
            .flat_map(|armhole| armhole.points.iter())
            .map(|&v| marks.positions[v as usize][1])
            .fold(f32::INFINITY, f32::min);
        let faces = cut
            .faces
            .into_iter()
            .filter(|face| {
                face.iter().any(|&v| {
                    self.weights[v as usize] < release.unit()
                        || marks.positions[v as usize][1] >= armpit
                })
            })
            .collect::<Vec<_>>();
        let used = faces.iter().flatten().copied().collect::<BTreeSet<_>>();
        let mapping = used
            .iter()
            .enumerate()
            .map(|(i, &old)| (old, i as u32))
            .collect::<BTreeMap<_, _>>();
        let points = used
            .into_iter()
            .map(|old| cut.points[old as usize])
            .collect();
        let faces = faces
            .into_iter()
            .map(|face| face.map(|old| mapping[&old]))
            .collect();
        Ok(SurfaceCut {
            apertures: cut.apertures,
            points,
            faces,
        })
    }
}
