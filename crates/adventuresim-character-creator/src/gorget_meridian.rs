//! A formed collar-to-hem curve in anatomical radius and height.
//! The collar tangent fixes the join. After any anatomical throat turn, ordered
//! Bézier controls carry the measured middle support outward to the hem.
use anyhow::{Result, ensure};
use bevy::math::Vec2;

const MINIMUM_TANGENT_LENGTH_M: f32 = 0.000001;

pub(super) struct Meridian {
    controls: [Vec2; 4],
}

impl Meridian {
    pub fn new(first: Vec2, middle: Vec2, mut last: Vec2, tangent: Vec2) -> Result<Self> {
        ensure!(
            [first, middle, last, tangent]
                .iter()
                .all(|point| point.is_finite())
                && first.x > 0.0
                && middle.x > 0.0
                && last.x > 0.0
                && tangent.length() > MINIMUM_TANGENT_LENGTH_M
                && tangent.y <= 0.0,
            "gorget meridian requires a descending collar tangent and outward shoulder support: join={first:?}, middle={middle:?}, hem={last:?}, tangent={tangent:?}"
        );
        let scale = first.distance(last) / (3.0 * tangent.length());
        let second = first + tangent * scale;
        ensure!(
            second.x > 0.0,
            "gorget collar tangent crosses the neck axis"
        );
        // Interpolate the measured middle while keeping the control polygon
        // ordered. If that support needs more radius, extend the hem instead
        // of turning the curve inward beyond the middle control.
        let mut third = (8.0 * middle - first - last) / 3.0 - second;
        third.x = third.x.max(second.x);
        last.x = last.x.max(third.x);
        // The rim must be approached from above or level. A terminal control
        // below the hem creates a trough and reverses the thin return strip.
        third.y = third.y.max(last.y);
        // Height follows the measured crown and hem while radius progresses
        // outward. A short neck can have a shoulder above the collar base;
        // ordering heights would either erase that support or collapse the
        // tangent as the hem passes through the join height.
        Ok(Self {
            controls: [first, second, third, last],
        })
    }

    pub fn point(&self, t: f32) -> Vec2 {
        let s = 1.0 - t;
        self.controls[0] * s.powi(3)
            + self.controls[1] * (3.0 * s * s * t)
            + self.controls[2] * (3.0 * s * t * t)
            + self.controls[3] * t.powi(3)
    }
}

/// The supported middle of the bib owns its full clearance metric. The upper
/// half transitions continuously from the independently authored collar inset.
pub(super) fn clearance_blend(t: f32) -> f32 {
    const SUPPORTED_MIDDLE: f32 = 0.5;
    let t = (t / SUPPORTED_MIDDLE).min(1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measured_bulges_keep_radius_order_and_the_collar_tangent() {
        for middle in [Vec2::new(0.13, 0.05), Vec2::new(0.19, 0.08)] {
            let first = Vec2::new(0.08, 0.09);
            let previous = Vec2::new(0.079, 0.095);
            let curve =
                Meridian::new(first, middle, Vec2::new(0.15, 0.01), first - previous).unwrap();
            let joined = curve.controls[1] - first;
            let incoming = first - previous;
            assert!(joined.perp_dot(incoming).abs() < 1e-8);
            assert!(joined.dot(incoming) > 0.0);
            let mut previous = curve.point(0.0);
            for index in 1..=128 {
                let point = curve.point(index as f32 / 128.0);
                assert!(point.x >= previous.x - 1e-7);
                previous = point;
            }
            assert!(curve.point(1.0).x >= 0.15);
        }
    }

    #[test]
    fn inward_throat_join_keeps_its_tangent_before_one_outward_turn() {
        let curve = Meridian::new(
            Vec2::new(0.08, 0.09),
            Vec2::new(0.13, 0.05),
            Vec2::new(0.15, 0.01),
            Vec2::new(-0.001, -0.005),
        )
        .unwrap();
        assert!(curve.controls[1].x < curve.controls[0].x);
        let incoming = Vec2::new(-0.001, -0.005);
        assert!(
            (curve.controls[1] - curve.controls[0])
                .perp_dot(incoming)
                .abs()
                < 1e-8
        );
        let mut increasing = false;
        let mut previous = curve.point(0.0).x;
        for index in 1..=128 {
            let radius = curve.point(index as f32 / 128.0).x;
            if increasing {
                assert!(radius >= previous);
            }
            increasing |= radius >= previous;
            previous = radius;
        }
    }

    #[test]
    fn elevated_shoulder_keeps_the_descending_collar_join_and_measured_crown() {
        let first = Vec2::new(0.11, 0.04);
        let middle = Vec2::new(0.13, 0.043);
        let last = Vec2::new(0.14, 0.041);
        let previous = Vec2::new(0.109, 0.0401);
        let curve = Meridian::new(first, middle, last, first - previous).unwrap();
        assert!((curve.point(0.5).y - middle.y).abs() < 1e-7);
        assert_eq!(curve.point(1.0).y, last.y);
        let incoming = first - previous;
        let joined = curve.controls[1] - first;
        assert!(joined.y < 0.0 && joined.perp_dot(incoming).abs() < 1e-8);
        let mut previous_radius = first.x;
        for index in 1..=128 {
            let point = curve.point(index as f32 / 128.0);
            assert!(point.is_finite() && point.x >= previous_radius);
            previous_radius = point.x;
        }
    }

    #[test]
    fn moving_the_hem_through_join_height_keeps_the_curve_continuous() {
        let curves = [-0.00001, 0.0, 0.00001].map(|offset| {
            Meridian::new(
                Vec2::new(0.11, 0.04),
                Vec2::new(0.13, 0.043),
                Vec2::new(0.14, 0.04 + offset),
                Vec2::new(0.001, -0.0001),
            )
            .unwrap()
        });
        for index in 0..=128 {
            let t = index as f32 / 128.0;
            assert!(curves[0].point(t).distance(curves[1].point(t)) < 0.00002);
            assert!(curves[2].point(t).distance(curves[1].point(t)) < 0.00002);
        }
        for curve in curves {
            assert!(curve.controls[1].distance(curve.controls[0]) > 0.005);
        }
    }

    #[test]
    fn low_middle_support_cannot_pull_the_terminal_strip_below_its_hem() {
        let last = Vec2::new(0.15, 0.02);
        let curve = Meridian::new(
            Vec2::new(0.08, 0.09),
            Vec2::new(0.13, 0.012),
            last,
            Vec2::new(0.001, -0.005),
        )
        .unwrap();
        for index in 0..=128 {
            assert!(curve.point(index as f32 / 128.0).y >= last.y - 1e-7);
        }
        assert!(curve.controls[2].y >= curve.controls[3].y);
    }
}
