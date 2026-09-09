//! An authored neck collar and short shoulder bib fitted from sectional bounds.
//! Three broad anatomical sections define each smooth front/back depth curve;
//! individual output vertices are never projected onto body triangles.

use std::f32::consts::TAU;

use adventuresim_armor_model::{GarmentArmorDesign, GarmentArmorKind, PartFrame, PartMesh};
use anyhow::{Context, Result, ensure};

use crate::armor_frames::{FitRegion, Wearer};

const AROUND: usize = 64;
const COLLAR_ROWS: usize = 4;
const BIB_ROWS: usize = 10;
const SECTION_HALF_BAND_M: f32 = 0.006;
const MINIMUM_SECTION_SAMPLES: usize = 4;
const COLLAR_HEIGHT_NECK_RATIO: f32 = 0.20;
const SIDE_TOP_NECK_RATIO: f32 = 0.91;
const FRONT_TOP_NECK_RATIO: f32 = 0.49;
const REAR_TOP_NECK_RATIO: f32 = 0.97;
const FRONT_BIB_DROP_NECK_RATIO: f32 = 0.48;
const REAR_BIB_RISE_NECK_RATIO: f32 = 0.12;
const SIDE_BIB_RISE_NECK_RATIO: f32 = 0.45;
const NECK_FRONT_CROP_RATIO: f32 = 0.95;
const BIB_FLARE_WIDTH_GAIN: f32 = 0.25;
const BIB_HEM_CROSS_SECTION_EXPONENT: f32 = 0.5;
const POSTERIOR_SHOULDER_CROWN_NECK_RATIO: f32 = 0.10;
const FRONT_COLLAR_MORPH_RESERVE_M: f32 = 0.003;

pub fn fit(design: &GarmentArmorDesign, wearer: &Wearer<'_>) -> Result<PartMesh> {
    design.validate()?;
    ensure!(
        design.kind == GarmentArmorKind::Gorget,
        "gorget fitter requires a gorget design"
    );
    let neck = joint(wearer, "c_neck")?;
    let head = joint(wearer, "c_head")?;
    let height = head[1] - neck[1];
    ensure!(height.is_finite() && height > 0.0, "invalid neck landmarks");
    let heading = wearer.frame(FitRegion::Head)?;
    let frame = PartFrame {
        origin: neck,
        axes: heading.axes,
        half_extents: [height; 3],
    };
    let mut support = wearer.support_indices(FitRegion::Neck)?;
    support.extend(wearer.support_indices(FitRegion::Torso)?);
    support.sort_unstable();
    support.dedup();
    let samples = support
        .iter()
        .map(|i| local(&frame, wearer.positions[*i]))
        .collect::<Vec<_>>();
    let cage = CollarCage::new(design, height, &samples)?;
    Ok(cage.mesh(design)?.transformed(&frame))
}

fn joint(wearer: &Wearer<'_>, name: &str) -> Result<[f32; 3]> {
    let index = wearer
        .joint_names
        .iter()
        .position(|n| n == name)
        .with_context(|| format!("missing gorget landmark {name}"))?;
    Ok(std::array::from_fn(|axis| wearer.joints[index][axis]))
}

fn local(frame: &PartFrame, p: [f32; 3]) -> [f32; 3] {
    frame
        .axes
        .map(|axis| (0..3).map(|i| axis[i] * (p[i] - frame.origin[i])).sum())
}

#[derive(Clone, Copy)]
struct Section {
    low: [f32; 3],
    high: [f32; 3],
}

impl Section {
    fn measure(samples: &[[f32; 3]], y: f32, front_limit: Option<f32>) -> Result<Self> {
        Self::measure_bounded(samples, y, front_limit, None)
    }

    fn measure_bounded(
        samples: &[[f32; 3]],
        y: f32,
        front_limit: Option<f32>,
        half_width: Option<f32>,
    ) -> Result<Self> {
        let mut result = Self {
            low: [f32::INFINITY; 3],
            high: [f32::NEG_INFINITY; 3],
        };
        let mut count = 0;
        for point in samples.iter().filter(|p| {
            (p[1] - y).abs() <= SECTION_HALF_BAND_M
                && front_limit.is_none_or(|limit| p[2] < limit)
                && half_width.is_none_or(|limit| p[0].abs() <= limit)
        }) {
            count += 1;
            for (axis, value) in point.iter().enumerate() {
                result.low[axis] = result.low[axis].min(*value);
                result.high[axis] = result.high[axis].max(*value);
            }
        }
        ensure!(
            count >= MINIMUM_SECTION_SAMPLES,
            "insufficient gorget section support at {y}m"
        );
        Ok(result)
    }
}

/// Quadratic through three broad horizontal section bounds.
struct DepthCurve([f32; 3]);

impl DepthCurve {
    fn at(&self, t: f32) -> f32 {
        self.0[0] * (1.0 - t) * (1.0 - 2.0 * t)
            + self.0[1] * 4.0 * t * (1.0 - t)
            + self.0[2] * t * (2.0 * t - 1.0)
    }
}

struct CollarCage {
    center: [f32; 2],
    collar_radius: [f32; 2],
    outer_width: f32,
    top: [f32; 3],
    base: [f32; 3],
    hem: [f32; 3],
    front: DepthCurve,
    back: DepthCurve,
    posterior_shoulder_crown: f32,
}

impl CollarCage {
    fn new(design: &GarmentArmorDesign, height: f32, samples: &[[f32; 3]]) -> Result<Self> {
        let clearance = design.clearance.metres() + design.wall_thickness.metres();
        let top = [
            FRONT_TOP_NECK_RATIO,
            SIDE_TOP_NECK_RATIO,
            REAR_TOP_NECK_RATIO,
        ]
        .map(|v| v * height);
        let base = top.map(|v| v - COLLAR_HEIGHT_NECK_RATIO * height);
        let hem = [
            -FRONT_BIB_DROP_NECK_RATIO * design.length.unit(),
            SIDE_BIB_RISE_NECK_RATIO,
            REAR_BIB_RISE_NECK_RATIO,
        ]
        .map(|v| v * height);
        let neck = Section::measure(samples, base[1], Some(height * NECK_FRONT_CROP_RATIO))?;
        let center = [
            (neck.low[0] + neck.high[0]) * 0.5,
            (neck.low[2] + neck.high[2]) * 0.5,
        ];
        let collar_radius = [
            (neck.high[0] - neck.low[0]) * 0.5 + clearance,
            (neck.high[2] - neck.low[2]) * 0.5 + clearance,
        ];
        let shoulder = Section::measure(samples, hem[1], None)?;
        let outer_width = ((shoulder.high[0] - shoulder.low[0]) * 0.5 + clearance)
            * (1.0 + design.flare.unit() * BIB_FLARE_WIDTH_GAIN);
        let mut front = [0.0; 3];
        let mut back = [0.0; 3];
        for (i, t) in [0.0, 0.5, 1.0].into_iter().enumerate() {
            front[i] = Section::measure_bounded(
                samples,
                base[0] + (hem[0] - base[0]) * t,
                None,
                Some(collar_radius[0]),
            )?
            .high[2]
                - center[1]
                + clearance;
            back[i] = center[1]
                - Section::measure_bounded(
                    samples,
                    base[2] + (hem[2] - base[2]) * t,
                    None,
                    Some(collar_radius[0]),
                )?
                .low[2]
                + clearance;
        }
        Ok(Self {
            center,
            collar_radius,
            outer_width,
            top,
            base,
            hem,
            front: DepthCurve(front),
            back: DepthCurve(back),
            posterior_shoulder_crown: height * POSTERIOR_SHOULDER_CROWN_NECK_RATIO,
        })
    }

    fn mesh(&self, design: &GarmentArmorDesign) -> Result<PartMesh> {
        let mut positions = Vec::new();
        let mut indices = Vec::new();
        for row in 0..=COLLAR_ROWS + BIB_ROWS {
            for i in 0..AROUND {
                let angle = i as f32 / AROUND as f32 * TAU;
                positions.push(if row <= COLLAR_ROWS {
                    self.collar_point(row as f32 / COLLAR_ROWS as f32, angle)
                } else {
                    self.bib_point((row - COLLAR_ROWS) as f32 / BIB_ROWS as f32, angle)
                });
            }
            if row == 0 {
                continue;
            }
            for i in 0..AROUND {
                let a = ((row - 1) * AROUND + i) as u32;
                let b = (row * AROUND + i) as u32;
                let c = (row * AROUND + (i + 1) % AROUND) as u32;
                let d = ((row - 1) * AROUND + (i + 1) % AROUND) as u32;
                indices.extend([a, b, c, a, c, d]);
            }
        }
        Ok(PartMesh::from_surface(
            positions,
            indices,
            design.wall_thickness.metres(),
        )?)
    }

    fn collar_point(&self, t: f32, angle: f32) -> [f32; 3] {
        let start_depth = self.collar_radius[1];
        let end_depth = if angle.cos() >= 0.0 {
            self.front.at(0.0)
        } else {
            self.back.at(0.0)
        };
        let depth = start_depth + (end_depth - start_depth) * t;
        [
            self.center[0] + self.collar_radius[0] * angle.sin(),
            height_at(self.top, angle)
                + (height_at(self.base, angle) - height_at(self.top, angle)) * t
                - FRONT_COLLAR_MORPH_RESERVE_M * angle.cos().max(0.0).powi(4) * (1.0 - t).powi(2),
            self.center[1] + depth * angle.cos(),
        ]
    }

    fn bib_point(&self, t: f32, angle: f32) -> [f32; 3] {
        let width = self.collar_radius[0] + (self.outer_width - self.collar_radius[0]) * t.powi(2);
        let cosine = angle.cos();
        let depth = if cosine >= 0.0 {
            self.front.at(t)
        } else {
            self.back.at(t)
        };
        let profile = cosine.signum()
            * cosine
                .abs()
                .powf(1.0 + (BIB_HEM_CROSS_SECTION_EXPONENT - 1.0) * t);
        let shoulder_crown = self.posterior_shoulder_crown
            * 4.0
            * angle.sin().powi(2)
            * (-cosine).max(0.0).powi(2)
            * (std::f32::consts::PI * t).sin();
        [
            self.center[0] + width * angle.sin(),
            height_at(self.base, angle)
                + (height_at(self.hem, angle) - height_at(self.base, angle)) * t
                + shoulder_crown,
            self.center[1] + depth * profile,
        ]
    }
}

fn height_at(heights: [f32; 3], angle: f32) -> f32 {
    let cosine = angle.cos();
    let end = if cosine >= 0.0 {
        heights[0]
    } else {
        heights[2]
    };
    heights[1] + (end - heights[1]) * cosine.powi(2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn collar_and_bib_form_one_closed_consistently_wound_material_shell() {
        let cage = CollarCage {
            center: [0.0, 0.02],
            collar_radius: [0.078, 0.075],
            outer_width: 0.15,
            top: [0.047, 0.086, 0.092],
            base: [0.028, 0.067, 0.073],
            hem: [-0.046, 0.043, 0.011],
            front: DepthCurve([0.065, 0.071, 0.092]),
            back: DepthCurve([0.078, 0.109, 0.131]),
            posterior_shoulder_crown: 0.0095,
        };
        let mesh = cage
            .mesh(&GarmentArmorDesign::new(GarmentArmorKind::Gorget))
            .unwrap();
        mesh.normals().unwrap();
        let mut edges = BTreeMap::<(u32, u32), Vec<(u32, u32)>>::new();
        for &[a, b, c] in mesh.indices.as_chunks::<3>().0 {
            for (u, v) in [(a, b), (b, c), (c, a)] {
                edges.entry((u.min(v), u.max(v))).or_default().push((u, v));
            }
        }
        for incident in edges.values() {
            assert_eq!(incident.len(), 2);
            assert_eq!(incident[0], (incident[1].1, incident[1].0));
        }
    }

    #[test]
    fn absent_anatomical_section_is_an_explicit_error() {
        assert!(Section::measure(&[], 0.04, None).is_err());
    }
}
