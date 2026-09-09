//! Shared sewing boundaries for independently selectable garment components.
use super::{dot, lerp, subtract};
use crate::armor_frames::Side;
use adventuresim_armor_model::{
    GARMENT_ARMPIT_ROW as ARMPIT, GARMENT_PANEL_ACROSS as ACROSS, GARMENT_PANEL_ALONG as ALONG,
    GARMENT_SHOULDER_DEPTH_SEGMENTS as SHOULDER, PartFrame, PartMesh,
};
use std::f32::consts::TAU;

pub(super) struct AttachmentRing {
    points: Vec<(f32, [f32; 3])>,
}

impl AttachmentRing {
    fn new(points: Vec<[f32; 3]>, frame: &PartFrame) -> Self {
        let center = std::array::from_fn(|axis| {
            points.iter().map(|p| p[axis]).sum::<f32>() / points.len() as f32
        });
        let mut points = points
            .into_iter()
            .map(|point| {
                let offset = subtract(point, center);
                let angle = dot(offset, frame.axes[0])
                    .atan2(dot(offset, frame.axes[2]))
                    .rem_euclid(TAU);
                (angle, point)
            })
            .collect::<Vec<_>>();
        points.sort_by(|a, b| a.0.total_cmp(&b.0));
        Self { points }
    }

    pub(super) fn armhole(torso: &PartMesh, side: Side, frame: &PartFrame) -> Self {
        let stride = ACROSS + 1;
        let panel = (ALONG + 1) * stride;
        let left = matches!(side, Side::Left);
        let col = if left { ACROSS } else { 0 };
        let mut points = (ARMPIT..=ALONG)
            .map(|row| torso.positions[row * stride + col])
            .collect::<Vec<_>>();
        let band_width = ACROSS / 4 + 1;
        let band = if left { 1 } else { 0 };
        let band_col = if left { band_width - 1 } else { 0 };
        for depth in 0..SHOULDER - 1 {
            let index =
                2 * panel + band * (SHOULDER - 1) * band_width + depth * band_width + band_col;
            points.push(torso.positions[index]);
        }
        points.extend(
            (ARMPIT..=ALONG)
                .rev()
                .map(|row| torso.positions[panel + row * stride + col]),
        );
        let flanks_start = 2 * panel + 2 * (SHOULDER - 1) * band_width;
        let flank = if left { 1 } else { 0 };
        let flank_start = flanks_start + flank * (ARMPIT + 1) * (SHOULDER - 1);
        points.extend(
            (0..SHOULDER - 1)
                .rev()
                .map(|depth| torso.positions[flank_start + ARMPIT * (SHOULDER - 1) + depth]),
        );
        Self::new(points, frame)
    }

    pub(super) fn hem(torso: &PartMesh, frame: &PartFrame) -> Self {
        let panel = (ALONG + 1) * (ACROSS + 1);
        let mut points = (0..=ACROSS)
            .map(|col| torso.positions[col])
            .collect::<Vec<_>>();
        points.extend((0..=ACROSS).rev().map(|col| torso.positions[panel + col]));
        let flanks_start = 2 * panel + 2 * (SHOULDER - 1) * (ACROSS / 4 + 1);
        for flank in 0..2 {
            let start = flanks_start + flank * (ARMPIT + 1) * (SHOULDER - 1);
            points.extend((0..SHOULDER - 1).map(|depth| torso.positions[start + depth]));
        }
        Self::new(points, frame)
    }

    pub(super) fn at(&self, angle: f32) -> [f32; 3] {
        let angle = angle.rem_euclid(TAU);
        let next = self.points.partition_point(|(a, _)| *a < angle) % self.points.len();
        let previous = (next + self.points.len() - 1) % self.points.len();
        let (a, p) = self.points[previous];
        let (b, q) = self.points[next];
        let span = (b - a).rem_euclid(TAU);
        let t = if span > f32::EPSILON {
            (angle - a).rem_euclid(TAU) / span
        } else {
            0.0
        };
        lerp(p, q, t)
    }
}
