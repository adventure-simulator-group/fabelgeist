//! Seat the upper tasset courses over torso plates without changing the thigh cage.
use crate::{
    armor_frames::{FitRegion, Wearer},
    armor_layer::ArmorLayerSurface,
    garment_fit::drape::{DrapeCage, SkirtProfile},
};
use adventuresim_armor_model::{GarmentArmorDesign, PartFrame, WrappedTassetDesign};
use anyhow::Result;

pub(super) struct TassetLayerSeat {
    cage: DrapeCage,
    center_depth: f32,
    lower_edge: f32,
    transition: f32,
    clearance: f32,
}

impl TassetLayerSeat {
    pub(super) fn new(
        layers: &[ArmorLayerSurface<'_>],
        wearer: &Wearer<'_>,
        design: &GarmentArmorDesign,
        shape: &WrappedTassetDesign,
        bottom: f32,
        top: f32,
    ) -> Result<Option<Self>> {
        let mut points = layers
            .iter()
            .flat_map(|layer| layer.positions.iter().copied())
            .collect::<Vec<_>>();
        if points.is_empty() {
            return Ok(None);
        }
        let hips = wearer.frame(FitRegion::Hips)?;
        let center_depth = hips.origin[2];
        let frame = PartFrame {
            origin: [0.0, 0.0, center_depth],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            half_extents: hips.half_extents,
        };
        let slope = shape.upper_edge_slope.unit();
        let lower_edge = points.iter().map(|p| p[1]).fold(f32::INFINITY, f32::min);
        let upper_edge = top + slope * points.iter().map(|p| p[0].abs()).fold(0.0, f32::max);
        let transition = (top - bottom) / f32::from(design.lame_count);
        // A plate's lowest chevron section may contain only a short front
        // edge. Complete that section with its wearer instead of interpreting
        // the missing back and sides as a collapsed radial envelope.
        for region in [FitRegion::Hips, FitRegion::Torso] {
            points.extend(
                wearer
                    .support_indices(region)?
                    .into_iter()
                    .map(|i| wearer.positions[i]),
            );
        }
        let relief = layers
            .iter()
            .map(|layer| layer.relief.metres())
            .fold(0.0, f32::max);
        Ok(Some(Self {
            cage: DrapeCage::new(
                &points,
                &frame,
                [lower_edge, upper_edge.max(lower_edge + transition)],
                SkirtProfile::FittedPlate,
            ),
            center_depth,
            lower_edge,
            transition,
            clearance: design.clearance.metres() + design.wall_thickness.metres() + relief,
        }))
    }

    pub(super) fn point(&self, point: [f32; 3]) -> [f32; 3] {
        let depth = point[2] - self.center_depth;
        let radius = point[0].hypot(depth);
        let angle = point[0].atan2(depth);
        let required = self
            .cage
            .radius_at_slope(point[1], angle, 0.0, 1.0, self.clearance);
        let height = point[1];
        let blend =
            ((height - self.lower_edge + self.transition) / self.transition).clamp(0.0, 1.0);
        let blend = blend * blend * (3.0 - 2.0 * blend);
        let seated = radius + (required - radius).max(0.0) * blend;
        [
            seated * angle.sin(),
            point[1],
            self.center_depth + seated * angle.cos(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seat() -> TassetLayerSeat {
        let points = (0..=40)
            .flat_map(|row| {
                let y = 1.0 + row as f32 * 0.005;
                let radius = 0.18 + (y - 1.0) * 0.2;
                (0..64).map(move |column| {
                    let angle = std::f32::consts::TAU * column as f32 / 64.0;
                    [radius * angle.sin(), y, radius * angle.cos()]
                })
            })
            .collect::<Vec<_>>();
        let frame = PartFrame {
            origin: [0.0; 3],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            half_extents: [0.2; 3],
        };
        TassetLayerSeat {
            cage: DrapeCage::new(&points, &frame, [1.0, 1.2], SkirtProfile::FittedPlate),
            center_depth: 0.0,
            lower_edge: 1.0,
            transition: 0.05,
            clearance: 0.008,
        }
    }

    #[test]
    fn upper_support_uses_raised_height_and_leaves_the_lower_thigh_unchanged() {
        let seat = seat();
        let source = [0.15, 0.99, 0.0];
        let raised = [source[0], source[1] + 0.06, source[2]];
        let a = seat.point(raised);
        let b = seat.point(source);
        assert!(
            a[0] > b[0] + 0.005,
            "rising support must be sampled at the lifted edge"
        );
        assert_eq!(a[1], raised[1], "layer seating preserves final edge height");
        let opposite = seat.point([-raised[0], raised[1], raised[2]]);
        assert!((a[0] + opposite[0]).abs() < 1e-6);
        let lower = [0.14, 0.6, 0.07];
        let seated = seat.point(lower);
        for axis in 0..3 {
            assert!((lower[axis] - seated[axis]).abs() < 1e-6);
        }
        let already_clear = [0.3, 1.0, 0.0];
        let seated = seat.point(already_clear);
        assert!(
            (seated[0] - already_clear[0]).abs() < 1e-6,
            "support must never pull the anatomical carrier inward"
        );
    }
}
