use super::{dot, lerp, local_point, subtract};
use crate::armor_frames::Wearer;
use adventuresim_armor_model::PartFrame;
const SECTION_HALF_WIDTH_M: f32 = 0.018;
const SURFACE_SAMPLE_COUNT: usize = 12;
/// A shared smooth depth datum keeps neighboring plate laps on the same surface.
pub(super) struct PlateDepth {
    samples: Vec<f32>,
    extents: [f32; 3],
}
impl PlateDepth {
    const SIDE: usize = 17;
    pub(super) fn new(surface: &SurfaceSampler, extents: [f32; 3]) -> Self {
        let mut samples = Vec::new();
        for row in 0..Self::SIDE {
            for column in 0..Self::SIDE {
                let x = (2.0 * column as f32 / (Self::SIDE - 1) as f32 - 1.0) * extents[0] * 1.2;
                let y = (2.0 * row as f32 / (Self::SIDE - 1) as f32 - 1.0) * extents[1] * 1.2;
                samples.push(surface.depth([x, y, extents[2]], 1.0));
            }
        }
        Self { samples, extents }
    }
    pub(super) fn at(&self, point: [f32; 3]) -> f32 {
        let axis = |i: usize| {
            let position =
                ((point[i] / (self.extents[i] * 1.2) + 1.0) * 0.5 * (Self::SIDE - 1) as f32)
                    .clamp(0.0, (Self::SIDE - 1) as f32);
            let base = position.floor() as isize;
            let t = position - base as f32;
            let weights = [
                (1.0 - t).powi(3) / 6.0,
                (3.0 * t.powi(3) - 6.0 * t * t + 4.0) / 6.0,
                (-3.0 * t.powi(3) + 3.0 * t * t + 3.0 * t + 1.0) / 6.0,
                t.powi(3) / 6.0,
            ];
            (base, weights)
        };
        let (x, wx) = axis(0);
        let (y, wy) = axis(1);
        let mut depth = 0.0;
        for (j, b) in wy.iter().enumerate() {
            for (i, a) in wx.iter().enumerate() {
                let column = (x + i as isize - 1).clamp(0, Self::SIDE as isize - 1) as usize;
                let row = (y + j as isize - 1).clamp(0, Self::SIDE as isize - 1) as usize;
                depth += a * b * self.samples[row * Self::SIDE + column];
            }
        }
        depth
    }
}
/// Low-resolution anatomical cross sections control the entire garment surface.
/// Interpolating this cage preserves smooth flow across all authored patches.
pub(super) struct SectionCage {
    sections: Vec<(f32, [f32; 3], [f32; 3])>,
}
impl SectionCage {
    pub(super) fn new(wearer: &Wearer<'_>, support: &[usize], frame: &PartFrame) -> Self {
        let sections = (0..=16)
            .map(|row| {
                let y = -frame.half_extents[1] + 2.0 * frame.half_extents[1] * row as f32 / 16.0;
                let center = frame.point([0.0, y, 0.0]);
                let (lo, hi) = section(wearer, support, center, frame.axes);
                (y, lo, hi)
            })
            .collect();
        Self { sections }
    }
    pub(super) fn at(&self, y: f32) -> ([f32; 3], [f32; 3]) {
        let upper = self
            .sections
            .partition_point(|entry| entry.0 < y)
            .min(self.sections.len() - 1);
        let lower = upper.saturating_sub(1);
        let (a, lo_a, hi_a) = self.sections[lower];
        let (b, lo_b, hi_b) = self.sections[upper];
        let t = if b > a {
            ((y - a) / (b - a)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        (lerp(lo_a, lo_b, t), lerp(hi_a, hi_b, t))
    }
}
pub(super) struct SurfaceSampler {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
}

impl SurfaceSampler {
    pub(super) fn new(wearer: &Wearer<'_>, support: &[usize], frame: &PartFrame) -> Self {
        Self {
            positions: support
                .iter()
                .map(|i| local_point(frame, wearer.positions[*i]))
                .collect(),
            normals: support
                .iter()
                .map(|i| frame.axes.map(|axis| dot(axis, wearer.normals[*i])))
                .collect(),
        }
    }

    pub(super) fn depth(&self, point: [f32; 3], side: f32) -> f32 {
        self.sample(point, 2, |_, normal| normal[2] * side > 0.12)
    }

    fn sample(
        &self,
        point: [f32; 3],
        output: usize,
        facing: impl Fn([f32; 3], [f32; 3]) -> bool,
    ) -> f32 {
        let mut candidates = self
            .positions
            .iter()
            .zip(&self.normals)
            .filter(|(position, normal)| facing(**position, **normal))
            .map(|(position, _)| {
                let distance = (0..3)
                    .filter(|axis| *axis != output)
                    .map(|axis| (position[axis] - point[axis]).powi(2))
                    .sum::<f32>();
                (distance, position[output])
            })
            .collect::<Vec<_>>();
        candidates.sort_unstable_by(|a, b| a.0.total_cmp(&b.0));
        let selected = &candidates[..SURFACE_SAMPLE_COUNT.min(candidates.len())];
        let (sum, weight) = selected
            .iter()
            .fold((0.0, 0.0), |(sum, weight), (distance, value)| {
                let influence = 1.0 / (distance + 0.0001);
                (sum + value * influence, weight + influence)
            });
        if weight > 0.0 {
            sum / weight
        } else {
            point[output]
        }
    }
}

pub(super) fn section(
    wearer: &Wearer<'_>,
    support: &[usize],
    center: [f32; 3],
    axes: [[f32; 3]; 3],
) -> ([f32; 3], [f32; 3]) {
    section_in_band(wearer, support, center, axes, SECTION_HALF_WIDTH_M)
}

pub(super) fn section_in_band(
    wearer: &Wearer<'_>,
    support: &[usize],
    center: [f32; 3],
    axes: [[f32; 3]; 3],
    half_width: f32,
) -> ([f32; 3], [f32; 3]) {
    let samples = support
        .iter()
        .map(|index| axes.map(|axis| dot(axis, subtract(wearer.positions[*index], center))))
        .collect::<Vec<_>>();
    sample_bounds(samples, half_width)
}

fn sample_bounds(mut samples: Vec<[f32; 3]>, half_width: f32) -> ([f32; 3], [f32; 3]) {
    samples.sort_unstable_by(|a, b| a[1].abs().total_cmp(&b[1].abs()));
    let limit = samples
        .iter()
        .take_while(|point| point[1].abs() < half_width)
        .count()
        .max(12)
        .min(samples.len());
    let mut lo = [f32::INFINITY; 3];
    let mut hi = [f32::NEG_INFINITY; 3];
    for point in &samples[..limit] {
        for axis in 0..3 {
            lo[axis] = lo[axis].min(point[axis]);
            hi[axis] = hi[axis].max(point[axis]);
        }
    }
    (lo, hi)
}

pub(super) fn enclosing_section(
    wearer: &Wearer<'_>,
    support: &[usize],
    center: [f32; 3],
    axes: [[f32; 3]; 3],
) -> ([f32; 3], [f32; 3]) {
    let (mut lo, mut hi) = section(wearer, support, center, axes);
    let radial_center = [(lo[0] + hi[0]) * 0.5, (lo[2] + hi[2]) * 0.5];
    let radius = [
        ((hi[0] - lo[0]) * 0.5).max(0.005),
        ((hi[2] - lo[2]) * 0.5).max(0.005),
    ];
    let scale = support
        .iter()
        .map(|index| axes.map(|axis| dot(axis, subtract(wearer.positions[*index], center))))
        .filter(|point| point[1].abs() < SECTION_HALF_WIDTH_M)
        .map(|point| {
            (((point[0] - radial_center[0]) / radius[0]).powi(2)
                + ((point[2] - radial_center[1]) / radius[1]).powi(2))
            .sqrt()
        })
        .fold(1.0, f32::max);
    for axis in 0..2 {
        lo[axis * 2] = radial_center[axis] - radius[axis] * scale;
        hi[axis * 2] = radial_center[axis] + radius[axis] * scale;
    }
    (lo, hi)
}
