//! Shared sewing boundaries for independently selectable garment components.
use super::{dot, lerp, subtract};
use crate::armor_frames::Side;
use adventuresim_armor_model::{
    GARMENT_ARMPIT_ROW as ARMPIT, GARMENT_PANEL_ACROSS as ACROSS, GARMENT_PANEL_ALONG as ALONG,
    GARMENT_SHOULDER_DEPTH_SEGMENTS as SHOULDER, PartFrame, PartMesh,
};
use std::f32::consts::TAU;

pub(crate) struct AttachmentRing {
    points: Vec<(f32, [f32; 3])>,
}

impl AttachmentRing {
    fn new(points: Vec<[f32; 3]>, frame: &PartFrame) -> Self {
        let center = std::array::from_fn(|axis| {
            points.iter().map(|p| p[axis]).sum::<f32>() / points.len() as f32
        });
        let angle = |point: [f32; 3]| {
            let offset = subtract(point, center);
            dot(offset, frame.axes[0]).atan2(dot(offset, frame.axes[2]))
        };
        let mut points = points;
        let winding = (0..points.len())
            .map(|index| {
                let a = angle(points[index]);
                let b = angle(points[(index + 1) % points.len()]);
                (b - a + std::f32::consts::PI).rem_euclid(TAU) - std::f32::consts::PI
            })
            .sum::<f32>();
        if winding < 0.0 {
            points.reverse();
        }
        let start = points
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| angle(**a).abs().total_cmp(&angle(**b).abs()))
            .map_or(0, |(index, _)| index);
        points.rotate_left(start);
        let lengths = (0..points.len())
            .map(|index| distance(points[index], points[(index + 1) % points.len()]))
            .collect::<Vec<_>>();
        let perimeter = lengths.iter().sum::<f32>();
        let mut along = 0.0;
        let points = points
            .into_iter()
            .zip(lengths)
            .map(|(point, length)| {
                let fraction = along / perimeter;
                along += length;
                (fraction, point)
            })
            .collect();
        Self { points }
    }

    pub(crate) fn armhole(torso: &PartMesh, side: Side, frame: &PartFrame) -> Self {
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

    pub(crate) fn at(&self, angle: f32) -> [f32; 3] {
        let fraction = angle.rem_euclid(TAU) / TAU;
        let next = self.points.partition_point(|(a, _)| *a < fraction) % self.points.len();
        let previous = (next + self.points.len() - 1) % self.points.len();
        let (a, p) = self.points[previous];
        let (b, q) = self.points[next];
        let span = (b - a).rem_euclid(1.0);
        let t = if span > f32::EPSILON {
            (fraction - a).rem_euclid(1.0) / span
        } else {
            0.0
        };
        lerp(p, q, t)
    }
}

fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    subtract(a, b)
        .into_iter()
        .map(|value| value * value)
        .sum::<f32>()
        .sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_armor_model::ArmorDetail;

    #[test]
    fn concave_armhole_keeps_its_authored_contour_order() {
        let frame = PartFrame {
            detail: ArmorDetail::BakeSource,
            origin: [0.0; 3],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            half_extents: [1.0; 3],
        };
        let contour = vec![
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 0.4],
            [0.2, 0.0, 0.1],
            [1.0, 0.0, -0.4],
            [0.0, 0.0, -1.0],
            [-1.0, 0.0, -0.4],
            [-0.2, 0.0, 0.1],
            [-1.0, 0.0, 0.4],
        ];
        let ring = AttachmentRing::new(contour.clone(), &frame);
        let ordered = ring
            .points
            .iter()
            .map(|(_, point)| *point)
            .collect::<Vec<_>>();
        for pair in ordered.windows(2) {
            let a = contour.iter().position(|point| point == &pair[0]).unwrap();
            let b = contour.iter().position(|point| point == &pair[1]).unwrap();
            assert!(a.abs_diff(b) == 1 || a.abs_diff(b) == contour.len() - 1);
        }
    }
}
