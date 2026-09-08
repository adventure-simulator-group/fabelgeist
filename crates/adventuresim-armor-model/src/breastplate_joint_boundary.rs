//! A derived armscye return expressed directly in the fitted surface's angle.
//! X-only C1 interpolation is insufficient at the full-wrap inverse singularity.

use crate::breastplate_topology::{BreastplateBoundaryEdge, SemanticBoundaryRange};

/// The fitted seam need not coincide with the authored armhole/side junction.
/// Both semantic edges therefore contribute to the derived return interval;
/// neck/shoulder anchors remain excluded even at a coincident height.
pub(super) fn derived_return_stations(
    layout: &[SemanticBoundaryRange],
    outline: &[[f64; 2]],
    seam_y: f64,
    upstream_y: f64,
) -> Vec<usize> {
    let mut stations = layout
        .iter()
        .filter(|range| {
            matches!(
                range.edge,
                BreastplateBoundaryEdge::RightArmhole
                    | BreastplateBoundaryEdge::RightSide
                    | BreastplateBoundaryEdge::LeftSide
                    | BreastplateBoundaryEdge::LeftArmhole
            )
        })
        .flat_map(|range| range.start..=range.start + range.segments)
        .map(|station| station % outline.len())
        .filter(|&station| outline[station][1] > seam_y && outline[station][1] < upstream_y)
        .collect::<Vec<_>>();
    stations.sort_unstable();
    stations.dedup();
    stations
}

#[derive(Clone, Debug, serde::Serialize)]
pub(super) struct JointReturn {
    pub seam_y: f64,
    pub upper_xy: [f64; 2],
    pub upper_dxdy: f64,
    pub angle: f64,
    pub guide_start: usize,
    pub boundary_stations: Vec<usize>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub(super) struct ResolvedJointReturn {
    pub source: JointReturn,
    /// Cubic Bernstein coefficients of u(normalized physical height).
    pub angular_controls: [f64; 4],
    pub stationary_fractions: Vec<f64>,
}

impl JointReturn {
    pub fn validate(&self) -> Result<(), String> {
        if !self.seam_y.is_finite()
            || !self.upper_xy.iter().all(|v| v.is_finite())
            || !self.upper_dxdy.is_finite()
            || self.upper_xy[1] - self.seam_y <= 1e-5
            || !(0.1..=std::f32::consts::FRAC_PI_2 as f64).contains(&self.angle)
        {
            return Err("Invalid joint armscye interval/tangent".into());
        }
        Ok(())
    }
    pub fn resolve(
        &self,
        radius: f64,
        radius_derivative: f64,
    ) -> Result<ResolvedJointReturn, String> {
        self.validate()?;
        let ratio = self.upper_xy[0] / radius;
        if radius <= 0.0 || !ratio.is_finite() || ratio <= 0.0 || ratio >= self.angle.sin() {
            return Err(format!(
                "Joint upstream angular inverse lacks a positive radius guard: ratio={ratio}"
            ));
        }
        let theta = ratio.asin();
        let u = theta / self.angle;
        let derivative = (self.upper_dxdy - radius_derivative * theta.sin())
            / (radius * self.angle * theta.cos());
        if !derivative.is_finite() {
            return Err("Nonfinite joint upstream angular tangent".into());
        }
        let span = self.upper_xy[1] - self.seam_y;
        let controls = [1.0, 1.0, u - span * derivative / 3.0, u];
        let mut resolved = ResolvedJointReturn {
            source: self.clone(),
            angular_controls: controls,
            stationary_fractions: Vec::new(),
        };
        let [p0, p1, p2, p3] = controls;
        let power = [
            p0,
            3.0 * (p1 - p0),
            3.0 * (p2 - 2.0 * p1 + p0),
            p3 - 3.0 * p2 + 3.0 * p1 - p0,
        ];
        let a = 3.0 * power[3];
        let b = 2.0 * power[2];
        let c = power[1];
        if a.abs() < 1e-14 {
            if b.abs() > 1e-14 {
                resolved.stationary_fractions.push(-c / b);
            }
        } else {
            let d = b * b - 4.0 * a * c;
            if d >= 0.0 {
                resolved
                    .stationary_fractions
                    .extend([(-b - d.sqrt()) / (2.0 * a), (-b + d.sqrt()) / (2.0 * a)]);
            }
        }
        resolved
            .stationary_fractions
            .retain(|v| v.is_finite() && *v > 1e-8 && *v < 1.0 - 1e-8);
        resolved.stationary_fractions.sort_by(f64::total_cmp);
        resolved
            .stationary_fractions
            .dedup_by(|a, b| (*a - *b).abs() < 1e-8);
        for t in resolved
            .stationary_fractions
            .iter()
            .copied()
            .chain([0.0, 1.0])
        {
            let value = resolved.at_fraction(t);
            if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                return Err(format!(
                    "Joint angular return leaves semantic range: t={t} u={value} controls={controls:?}"
                ));
            }
        }
        if resolved.stationary_fractions.len() > 1 {
            return Err("Joint angular return oscillates".into());
        }
        Ok(resolved)
    }
}

impl ResolvedJointReturn {
    fn at_fraction(&self, t: f64) -> f64 {
        let b = [
            (1.0 - t).powi(3),
            3.0 * t * (1.0 - t).powi(2),
            3.0 * t * t * (1.0 - t),
            t.powi(3),
        ];
        b.iter()
            .zip(self.angular_controls)
            .map(|(a, b)| a * b)
            .sum()
    }
    pub fn parameter(&self, y: f64) -> f64 {
        self.at_fraction((y - self.source.seam_y) / (self.source.upper_xy[1] - self.source.seam_y))
    }
    pub fn x(&self, y: f64, radius: f64) -> f64 {
        radius * (self.source.angle * self.parameter(y)).sin()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_return_covers_side_stations_above_seam_on_both_sides() {
        use crate::breastplate_topology::CanonicalBreastplateTopology;
        let layout = CanonicalBreastplateTopology::semantic_layout();
        let count = layout.iter().map(|range| range.segments).sum();
        let mut outline = vec![[0.0, 1.4]; count];
        let mut expected = Vec::new();
        for range in &layout {
            let junction = match range.edge {
                BreastplateBoundaryEdge::RightSide => range.start,
                BreastplateBoundaryEdge::LeftSide => range.start + range.segments,
                _ => continue,
            };
            let direction: isize = if range.edge == BreastplateBoundaryEdge::RightSide {
                1
            } else {
                -1
            };
            // Traverse the armhole/side junction in the direction of the waist.
            // The first sample is the unchanged upstream anchor and the last
            // two are at/below the fitted seam, not members of the upper return.
            for (offset, y) in [
                (-2, 1.35),
                (-1, 1.33),
                (0, 1.31),
                (1, 1.30),
                (2, 1.28),
                (3, 1.275),
                (4, 1.26),
            ] {
                let station = (junction as isize + offset * direction) as usize;
                outline[station] = [direction as f64 * 0.2, y];
                if y > 1.275 && y < 1.35 {
                    expected.push(station);
                }
            }
        }
        // Unrelated authored anchors at the same height must not be selected.
        for range in &layout {
            if matches!(
                range.edge,
                BreastplateBoundaryEdge::Neck
                    | BreastplateBoundaryEdge::RightShoulder
                    | BreastplateBoundaryEdge::LeftShoulder
                    | BreastplateBoundaryEdge::Waist
            ) {
                outline[range.start + 1][1] = 1.30;
            }
        }
        expected.sort_unstable();
        let selected = derived_return_stations(&layout, &outline, 1.275, 1.35);
        assert_eq!(selected, expected);
        assert_eq!(selected.len(), 8, "shared edge junctions are selected once");
    }
    fn input(angle: f64) -> JointReturn {
        JointReturn {
            seam_y: 1.2,
            upper_xy: [0.18, 1.35],
            upper_dxdy: -0.3,
            angle,
            guide_start: 0,
            boundary_stations: Vec::new(),
        }
    }
    #[test]
    fn angular_return_preserves_upstream_physical_position_and_tangent() {
        for angle in [1.48, std::f64::consts::FRAC_PI_2] {
            let edge = input(angle);
            let a = |y: f64| 0.22 + 0.02 * (y - 1.2);
            let resolved = edge.resolve(a(1.35), 0.02).unwrap();
            assert!((resolved.x(1.35, a(1.35)) - 0.18).abs() < 1e-13);
            let h = 1e-6;
            let derivative = (resolved.x(1.35, a(1.35)) - resolved.x(1.35 - h, a(1.35 - h))) / h;
            assert!((derivative + 0.3).abs() < 1e-5);
            assert_eq!(resolved.parameter(1.2), 1.0);
        }
    }
    #[test]
    fn full_wrap_return_has_a_continuous_three_dimensional_endpoint_tangent() {
        for angle in [1.48, std::f64::consts::FRAC_PI_2] {
            let edge = input(angle);
            let a = |y: f64| 0.22 + 0.02 * (y - 1.2);
            let resolved = edge.resolve(a(1.35), 0.02).unwrap();
            let position = |y: f64| {
                let theta = angle * resolved.parameter(y);
                [
                    a(y) * theta.sin(),
                    y,
                    0.01 - 0.02 * (y - 1.2) + (0.14 - 0.03 * (y - 1.2)) * theta.cos(),
                ]
            };
            let h = 1e-7;
            let p = position(1.2);
            let q = position(1.2 + h);
            let expected = [0.02 * angle.sin(), 1.0, -0.02 - 0.03 * angle.cos()];
            for j in 0..3 {
                assert!(((q[j] - p[j]) / h - expected[j]).abs() < 1e-5);
            }
        }
    }
    #[test]
    fn invalid_interval_singular_upstream_and_angular_overshoot_are_explicit_errors() {
        let mut edge = input(1.48);
        edge.upper_xy[1] = edge.seam_y + 1e-8;
        assert!(edge.resolve(0.23, 0.0).is_err());
        let mut edge = input(std::f64::consts::FRAC_PI_2);
        edge.upper_xy[0] = 0.23;
        assert!(edge.resolve(0.23, 0.0).is_err());
        let mut edge = input(1.48);
        edge.upper_dxdy = -100.0;
        assert!(edge.resolve(0.23, 0.0).is_err());
    }
}
