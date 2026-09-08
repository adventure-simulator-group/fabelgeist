//! Composed analytic ellipse evaluation and final-field runtime support guards.
use super::{JointProfiles, Row, combine, dot, dump};

const DEPTH_CHECK_INTERVALS: usize = 1024;
const SUPPORT_TOLERANCE_M: f64 = 1e-6;

impl JointProfiles {
    pub fn main_transverse_support(
        &self,
        heights: [f64; 2],
    ) -> Result<Vec<crate::breastplate_transverse_support::TransverseSupport>, String> {
        use crate::breastplate_transverse_support::{TransverseInput, TransverseSupport};
        let coefficients = self.base_depth_coefficients();
        self.physical_support
            .iter()
            .enumerate()
            .filter(|(_, (xy, _))| xy[1] >= heights[0] && xy[1] <= heights[1])
            .map(|(i, (xy, floor))| {
                let provenance = self
                    .front_support
                    .get(i)
                    .ok_or_else(|| format!("Missing transverse floor provenance at row {i}"))?;
                if provenance.height_m != xy[1] || provenance.floor_m != *floor {
                    return Err(format!("Mismatched transverse floor provenance at row {i}"));
                }
                TransverseSupport::new(TransverseInput {
                    source_row: i,
                    source_origin: provenance.origin,
                    xy: *xy,
                    floor_m: *floor,
                    radius_m: self.radius(xy[1]),
                    center_m: dot(&self.component(xy[1], false, 0), &coefficients),
                })
                .map_err(|e| format!("Main transverse support: {e}"))
            })
            .collect()
    }
    pub fn fit_bridge_bubble(&mut self) -> Result<(), String> {
        use crate::breastplate_bridge_bubble::{BridgeFloor, BubbleFit, occupied};
        let front = self.whole_front.as_ref().ok_or("Missing composed front")?;
        let heights = [front.neck_y, front.first_band_y];
        let outline = self.physical_outline();
        let mut floors = Vec::new();
        for (xy, floor) in &self.physical_support {
            let u = self.parameter(*xy)?;
            floors.push(BridgeFloor {
                xy: *xy,
                floor_m: *floor,
                actual_before_m: self.evaluate(u, xy[1]).0[2],
                cosine: (u * self.coverage(xy[1])[0]).cos(),
                occupied: occupied(*xy, &outline),
            });
        }
        let fit = BubbleFit::fit(heights, &floors);
        match fit {
            Ok(fit) => {
                self.whole_front.as_mut().unwrap().bridge_bubble_m = fit.coefficient_m;
                dump(
                    "whole-front-bubble",
                    serde_json::json!({"policy":"minimal-nonnegative-occupied-floor-jet-null-v1",
                    "basis":"t^3*(1-t)^3; t=(physicalY-neckY)/(firstBandY-neckY)",
                    "fit":fit,"representation":self.whole_front,
                    "endpoint_policy":"zero through second physical-Y derivative; exact main and band retained",
                    "support_policy":"occupied original physical floors; no invented slack; all original floors revalidated afterward",
                    "occupancy_policy":"physical outline, boundary membership tolerance1e-7m; not a clearance tolerance",
                    "scope":"analytic scalar correction; no convexity or solidified clearance certificate"}),
                )
            }
            Err(error) => {
                dump(
                    "whole-front-bubble",
                    serde_json::json!({"error":error,"heights":heights,"source_rows":floors}),
                )?;
                Err(format!("Body-constrained bridge bubble: {error}"))
            }
        }
    }
    fn base_depth_coefficients(&self) -> Row {
        std::array::from_fn(|j| match j {
            0..=3 => self.lower_c[j],
            4..=7 => self.lower_b[j - 4],
            _ => self.upper.depth_controls[j - 8],
        })
    }
    pub fn base_front_jet(&self, y: f64) -> [f64; 3] {
        let coefficients = self.base_depth_coefficients();
        std::array::from_fn(|k| {
            dot(
                &combine(self.component(y, false, k), self.component(y, true, k), 1.),
                &coefficients,
            )
        })
    }
    pub(super) fn composed_evaluate(&self, u: f64, y: f64) -> Option<([f64; 3], [f64; 3])> {
        let front = self.whole_front.as_ref().filter(|front| front.active(y))?;
        let coefficients = self.base_depth_coefficients();
        let c = dot(&self.component(y, false, 0), &coefficients);
        let dc = dot(&self.component(y, false, 1), &coefficients);
        let jet = front.jet(y, self.base_front_jet(y));
        let b = jet[0] - c;
        let db = jet[1] - dc;
        let a = self.radius(y);
        let da = dot(&self.radius_row(y, 1), &self.radius_coefficients());
        let theta = u * self.coverage(y)[0];
        let n = [
            b * theta.sin(),
            -(b * da * theta.sin().powi(2) + a * theta.cos() * (dc + db * theta.cos())),
            a * theta.cos(),
        ];
        let norm = dot(&n, &n).sqrt();
        Some((
            [a * theta.sin(), y, c + b * theta.cos()],
            n.map(|v| v / norm),
        ))
    }
    pub fn physical_outline(&self) -> Vec<[f64; 2]> {
        self.boundary_domain
            .iter()
            .map(|q| {
                let u = f64::from(q[0]) / self.upper.arc_scale;
                let p = self.evaluate(u, f64::from(q[1])).0;
                [p[0], p[1]]
            })
            .collect()
    }
    fn sampled_composed_depth(&self) -> Result<f64, String> {
        let front = self.whole_front.as_ref().ok_or("Missing composed front")?;
        let coefficients = self.base_depth_coefficients();
        let mut minimum = f64::INFINITY;
        for [start, end] in [
            [front.waist_y, front.neck_y],
            [front.neck_y, front.first_band_y],
        ] {
            for i in 0..=DEPTH_CHECK_INTERVALS {
                let y = start + (end - start) * i as f64 / DEPTH_CHECK_INTERVALS as f64;
                let depth = front.jet(y, self.base_front_jet(y))[0]
                    - dot(&self.component(y, false, 0), &coefficients);
                if !depth.is_finite() || depth <= 0. {
                    return Err(format!(
                        "Non-positive composed ellipse depth {depth} at height {y}"
                    ));
                }
                minimum = minimum.min(depth);
            }
        }
        Ok(minimum)
    }
    pub fn validate_composed_support(&self) -> Result<(), String> {
        let minimum_depth = self.sampled_composed_depth()?;
        let mut rows = Vec::new();
        let mut worst = 0.0_f64;
        for (xy, floor) in &self.physical_support {
            if !floor.is_finite() || xy.iter().any(|v| !v.is_finite()) {
                return Err("Non-finite composed physical support input".into());
            }
            let u = self.parameter(*xy)?;
            let actual = self.evaluate(u, xy[1]).0[2];
            let slack = actual - floor;
            if !actual.is_finite() || !slack.is_finite() {
                return Err("Non-finite composed physical support evaluation".into());
            }
            worst = worst.max(-slack);
            rows.push(
                serde_json::json!({"xy":xy,"floor_m":floor,"actual_z_m":actual,"slack_m":slack}),
            );
        }
        dump(
            "whole-front-physical-support",
            serde_json::json!({"rows":rows,
            "maximum_violation_m":worst,"sampled_minimum_depth_m":minimum_depth,
            "depth_gate":"positive finite F-C at 1025 uniform heights per main/bridge piece; sampled, not exact certificate",
            "scope":"stored original physical floors, not old shape priors; final shell checks still required"}),
        )?;
        if worst > SUPPORT_TOLERANCE_M {
            return Err(format!(
                "Composed whole-front field violates stored body floor by {worst}m"
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{breastplate_front_envelope::FrontSection, breastplate_whole_front::WholeFront};

    fn composed() -> JointProfiles {
        let mut field = super::super::tests::field();
        let rows = (0..=256)
            .map(|i| FrontSection {
                height_m: 1. + 0.4 * i as f64 / 256.,
                floor_m: 0.05,
                raw_body_z_m: 0.04,
                body_face: 0,
                x_m: 0.,
                lateral_limit_m: 0.2,
                clipped_segments: 1,
            })
            .collect::<Vec<_>>();
        field.whole_front = Some(
            WholeFront::fit(
                [1., 1.4, 1.45],
                [field.base_front_jet(1.)[0], field.base_front_jet(1.4)[0]],
                field.base_front_jet(1.4),
                0.04,
                &rows,
                &[],
                &[],
            )
            .unwrap(),
        );
        field
    }
    #[test]
    fn composition_retains_waist_band_radius_and_coronal_center_and_uses_analytic_normals() {
        let mut f = composed();
        let before_bubble = f.clone();
        use crate::breastplate_bridge_bubble::{BridgeFloor, BubbleFit};
        let fit = BubbleFit::fit(
            [1.4, 1.45],
            &[BridgeFloor {
                xy: [0.1, 1.425],
                floor_m: 0.001,
                actual_before_m: 0.,
                cosine: 0.8,
                occupied: true,
            }],
        )
        .unwrap();
        f.whole_front.as_mut().unwrap().bridge_bubble_m = fit.coefficient_m;
        let old = super::super::tests::field();
        for u in [-0.9, 0., 0.8] {
            for y in [1., 1.15, 1.399, 1.4, 1.45, 1.49] {
                assert_eq!(f.evaluate(u, y), before_bubble.evaluate(u, y));
            }
            for j in 0..3 {
                assert!((f.evaluate(u, 1.).0[j] - old.evaluate(u, 1.).0[j]).abs() < 1e-13);
            }
            for y in [0.9, 1.45, 1.49] {
                assert_eq!(f.evaluate(u, y), old.evaluate(u, y));
            }
            for y in [1.05, 1.2, 1.35, 1.42] {
                assert_eq!(f.radius(y), old.radius(y));
                let c = dot(&f.component(y, false, 0), &f.base_depth_coefficients());
                let side_u = std::f64::consts::FRAC_PI_2 / f.coverage(y)[0];
                assert!((f.evaluate(side_u, y).0[2] - c).abs() < 1e-13);
                let h = 1e-6;
                let du: [f64; 3] = std::array::from_fn(|j| {
                    (f.evaluate(u + h, y).0[j] - f.evaluate(u - h, y).0[j]) / (2. * h)
                });
                let dy: [f64; 3] = std::array::from_fn(|j| {
                    (f.evaluate(u, y + h).0[j] - f.evaluate(u, y - h).0[j]) / (2. * h)
                });
                let n = [
                    du[1] * dy[2] - du[2] * dy[1],
                    du[2] * dy[0] - du[0] * dy[2],
                    du[0] * dy[1] - du[1] * dy[0],
                ];
                let norm = dot(&n, &n).sqrt();
                for j in 0..3 {
                    assert!((n[j] / norm - f.evaluate(u, y).1[j]).abs() < 1e-6);
                }
            }
        }
    }
    #[test]
    fn composed_guards_reject_nonfinite_support_and_nonpositive_new_depth() {
        let mut f = composed();
        assert!(f.validate_composed_support().is_ok());
        f.physical_support.push(([0., 1.2], f64::NAN));
        assert!(f.validate_composed_support().is_err());
        f.physical_support.clear();
        f.whole_front.as_mut().unwrap().guide_bernstein = [-1.; 4];
        assert!(f.validate_composed_support().is_err());
    }
}
