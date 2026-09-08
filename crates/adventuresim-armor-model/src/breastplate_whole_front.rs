//! Whole-height front meridian and explicit C2 neck-to-band continuation.
use crate::{
    breastplate_front_envelope::{FrontSection, FrontSupport},
    breastplate_qp::LinearConstraint,
    breastplate_shoulder_band::{BandError, ShoulderBand},
    breastplate_simple_rim::RimPolicy,
    breastplate_transverse_support::{TransverseSupport, solve_augmented},
};
use serde::Serialize;

const WHOLE_FRONT_ENV: &str = "BREASTPLATE_DIAGNOSTIC_WHOLE_FRONT";
const POLICY_KEY: &[u8] = b"whole-height-front-retained-ac-transverse-floor-v3";
const CROWN_NORMALIZATION: f64 = 6.75;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WholeFrontPolicy {
    Baseline,
    WholeHeight,
}
impl WholeFrontPolicy {
    pub(crate) fn from_environment(rim: RimPolicy) -> Result<Self, String> {
        if std::env::var_os(WHOLE_FRONT_ENV).is_none() {
            return Ok(Self::Baseline);
        }
        if rim != RimPolicy::StraightShoulderCorner {
            return Err("Whole front diagnostic requires simple seated rim".into());
        }
        Ok(Self::WholeHeight)
    }
    pub(crate) fn hash(self, hash: &mut blake3::Hasher) {
        if self == Self::WholeHeight {
            hash.update(POLICY_KEY);
        }
    }
}

pub(crate) fn install_guide(
    joint: &mut crate::breastplate_joint_profiles::JointProfiles,
    vertices: &[[f64; 3]],
    faces: &[[u32; 3]],
    crown_m: f64,
    padding_m: f64,
    first_band_y: f64,
) -> Result<(), String> {
    let outline = joint.physical_outline();
    let mut crossings = outline
        .iter()
        .zip(outline.iter().cycle().skip(1))
        .take(outline.len())
        .filter_map(|(a, b)| {
            if a[0].min(b[0]) <= 0. && a[0].max(b[0]) >= 0. && (b[0] - a[0]).abs() > 1e-14 {
                Some(a[1] - a[0] * (b[1] - a[1]) / (b[0] - a[0]))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    crossings.sort_by(f64::total_cmp);
    let neck = *crossings.last().ok_or("No occupied neck centerline")?;
    let heights = [joint.waist_y, neck];
    let sections =
        crate::breastplate_front_envelope::sections(vertices, faces, &outline, heights, padding_m)?;
    let old_neck = joint.base_front_jet(neck);
    let support: Vec<_> = joint
        .front_support
        .iter()
        .filter(|row| row.height_m >= heights[0] && row.height_m <= heights[1])
        .cloned()
        .collect();
    let guide = WholeFront::fit(
        [heights[0], heights[1], first_band_y],
        [joint.base_front_jet(heights[0])[0], old_neck[0]],
        old_neck,
        crown_m,
        &sections,
        &support,
        &joint.main_transverse_support(heights)?,
    )?;
    joint.whole_front = Some(guide);
    joint.fit_bridge_bubble()?;
    joint.validate_composed_support()
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct WholeFront {
    pub waist_y: f64,
    pub neck_y: f64,
    pub first_band_y: f64,
    pub guide_bernstein: [f64; 4],
    pub bridge_power: [f64; 6],
    pub bridge_bubble_m: crate::breastplate_bridge_bubble::BridgeBubble,
}

pub(crate) fn basis(t: f64) -> [f64; 4] {
    [
        (1. - t).powi(3),
        3. * t * (1. - t).powi(2),
        3. * t * t * (1. - t),
        t.powi(3),
    ]
}
fn polynomial_jet<const N: usize>(c: [f64; N], t: f64, span: f64) -> [f64; 3] {
    std::array::from_fn(|k| {
        (k..N)
            .map(|j| {
                let factor = (0..k).fold(1., |p, i| p * (j - i) as f64);
                c[j] * factor * t.powi((j - k) as i32) / span.powi(k as i32)
            })
            .sum()
    })
}
fn power(c: [f64; 4]) -> [f64; 4] {
    [
        c[0],
        3. * (c[1] - c[0]),
        3. * (c[0] - 2. * c[1] + c[2]),
        -c[0] + 3. * c[1] - 3. * c[2] + c[3],
    ]
}
fn dump(name: &str, value: &impl Serialize) -> Result<(), String> {
    ShoulderBand::write_dump(name, value).map_err(|e: BandError| e.to_string())
}

impl WholeFront {
    fn with_bridge(heights: [f64; 3], guide_bernstein: [f64; 4], old_neck_jet: [f64; 3]) -> Self {
        let [waist_y, neck_y, first_band_y] = heights;
        let jet = polynomial_jet(power(guide_bernstein), 1., neck_y - waist_y);
        let delta: [f64; 3] = std::array::from_fn(|i| jet[i] - old_neck_jet[i]);
        let span = first_band_y - neck_y;
        let [a, b, c] = [delta[0], delta[1] * span, 0.5 * delta[2] * span * span];
        let bridge_power = [
            a,
            b,
            c,
            -10. * a - 6. * b - 3. * c,
            15. * a + 8. * b + 3. * c,
            -6. * a - 3. * b - c,
        ];
        Self {
            waist_y,
            neck_y,
            first_band_y,
            guide_bernstein,
            bridge_power,
            bridge_bubble_m: Default::default(),
        }
    }
    fn validate_samples(
        heights: [f64; 2],
        sections: &[FrontSection],
        support: &[FrontSupport],
    ) -> Result<(), String> {
        let intervals = crate::breastplate_front_envelope::SECTION_INTERVALS;
        if sections.len() != intervals + 1
            || sections.iter().enumerate().any(|(i, row)| {
                let expected = heights[0] + (heights[1] - heights[0]) * i as f64 / intervals as f64;
                [
                    row.height_m,
                    row.floor_m,
                    row.raw_body_z_m,
                    row.x_m,
                    row.lateral_limit_m,
                ]
                .iter()
                .any(|v| !v.is_finite())
                    || (row.height_m - expected).abs() > 1e-12
                    || row.lateral_limit_m <= 0.
                    || row.clipped_segments == 0
                    || row.x_m.abs() > row.lateral_limit_m + 1e-12
            })
            || support.iter().any(|r| {
                !r.height_m.is_finite()
                    || !r.floor_m.is_finite()
                    || r.height_m < heights[0]
                    || r.height_m > heights[1]
            })
        {
            return Err("Invalid whole-front support rows or uniform section grid".into());
        }
        Ok(())
    }
    pub(crate) fn fit(
        heights: [f64; 3],
        endpoints: [f64; 2],
        old_neck_jet: [f64; 3],
        crown_m: f64,
        sections: &[FrontSection],
        support: &[FrontSupport],
        transverse: &[TransverseSupport],
    ) -> Result<Self, String> {
        let [waist_y, neck_y, first_band_y] = heights;
        if heights
            .iter()
            .chain(&endpoints)
            .chain(&old_neck_jet)
            .any(|v| !v.is_finite())
            || neck_y <= waist_y
            || first_band_y <= neck_y
            || !crown_m.is_finite()
            || crown_m < 0.
        {
            return Err("Invalid whole-front domain or bridge interval".into());
        }
        Self::validate_samples([waist_y, neck_y], sections, support)?;
        let (h, rhs) = guide_objective([waist_y, neck_y], endpoints, crown_m, sections);
        let mut bounds = Vec::new();
        for (y, floor) in sections
            .iter()
            .map(|r| (r.height_m, r.floor_m))
            .chain(support.iter().map(|r| (r.height_m, r.floor_m)))
        {
            if y < waist_y || y > neck_y {
                continue;
            }
            let b = basis((y - waist_y) / (neck_y - waist_y));
            bounds.push(LinearConstraint {
                coefficients: vec![b[1], b[2]],
                value: floor - b[0] * endpoints[0] - b[3] * endpoints[1],
            });
        }
        bounds.extend([
            LinearConstraint {
                coefficients: vec![2., -1.],
                value: endpoints[0],
            },
            LinearConstraint {
                coefficients: vec![-1., 2.],
                value: endpoints[1],
            },
        ]);
        let original_bound_count = bounds.len();
        let added = transverse
            .iter()
            .map(|r| r.constraint([waist_y, neck_y], endpoints))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Main transverse support: {e}"))?
            .into_iter()
            .flatten()
            .collect();
        let (solve, solve_path) = solve_augmented(&h, &rhs, &mut bounds, added)?;
        let guide_bernstein = [
            endpoints[0],
            solve.coefficients[0],
            solve.coefficients[1],
            endpoints[1],
        ];
        let jet = polynomial_jet(power(guide_bernstein), 1., neck_y - waist_y);
        let span = first_band_y - neck_y;
        let result = Self::with_bridge(heights, guide_bernstein, old_neck_jet);
        dump(
            "whole-front-guide",
            &serde_json::json!({
                "policy":"runtime-whole-height-retained-ac-transverse-guide-v2","representation":result,
                "transverse_support":transverse,"original_bound_count":original_bound_count,"solve_path":solve_path,
                "composition_stage":"authored main/quintic before body-floor bubble; final representation in joint profile and whole-front-bubble",
                "endpoint_policy":"actual base joint field waist and occupied neck; not target-guide depth",
                "support_sections":sections,"stored_support":support,"requested_crown_m":crown_m,
                "crown_target":"6.75*v^2*(1-v), v=occupied waist-to-neck; request not displacement or gap cap",
                "hessian":h,"rhs":rhs,"lower_bounds":bounds.iter().map(|r|serde_json::json!({"coefficients":r.coefficients,"value":r.value})).collect::<Vec<_>>(),
                "qp_diagnostics":format!("{:?}",solve.diagnostics),"multipliers":solve.lower_bound_multipliers,
                "base_joint_qp_role":"provenance for retained a,C and old F only; composed field replaces old shape priors",
                "support_role":"body-front sections plus original main physical floors converted with actual retained a,C; includes conservative inherited rows; no extra padding; sampled floors not full shell clearance",
            }),
        )?;
        dump(
            "whole-front-bridge",
            &serde_json::json!({"representation":result,"old_neck_jet":old_neck_jet,"guide_neck_jet":jet,
                "composition_stage":"pre-bubble quintic provenance; whole-front-bubble owns final coefficient",
                "first_band_height_provenance":"minimum of actual endpoints of policy-proven straight 3D band",
                "first_band_correction_jet":polynomial_jet(result.bridge_power,1.,span),
                "below_waist":"old field; intentional skirt crease may have different slope",
                "bridge":"explicit C2 quintic deltaF; never extrapolated beta; at/above band old field exactly",
            }),
        )?;
        Ok(result)
    }

    pub(crate) fn active(&self, y: f64) -> bool {
        y >= self.waist_y && y < self.first_band_y
    }
    pub(crate) fn jet(&self, y: f64, old: [f64; 3]) -> [f64; 3] {
        if !self.active(y) {
            return old;
        }
        if y <= self.neck_y {
            return polynomial_jet(
                power(self.guide_bernstein),
                (y - self.waist_y) / (self.neck_y - self.waist_y),
                self.neck_y - self.waist_y,
            );
        }
        let mut d = polynomial_jet(
            self.bridge_power,
            (y - self.neck_y) / (self.first_band_y - self.neck_y),
            self.first_band_y - self.neck_y,
        );
        if !self.bridge_bubble_m.is_zero() {
            let b = self.bridge_bubble_m.jet(
                (y - self.neck_y) / (self.first_band_y - self.neck_y),
                self.first_band_y - self.neck_y,
            );
            for k in 0..3 {
                d[k] += b[k];
            }
        }
        std::array::from_fn(|i| old[i] + d[i])
    }
}

fn guide_objective(
    heights: [f64; 2],
    endpoints: [f64; 2],
    crown_m: f64,
    sections: &[FrontSection],
) -> (Vec<Vec<f64>>, Vec<f64>) {
    let mut h = vec![vec![0.; 2]; 2];
    let mut rhs = vec![0.; 2];
    for row in sections {
        let v = (row.height_m - heights[0]) / (heights[1] - heights[0]);
        let b = basis(v);
        let target = row.floor_m + crown_m * CROWN_NORMALIZATION * v * v * (1. - v);
        let end = b[0] * endpoints[0] + b[3] * endpoints[1];
        for i in 0..2 {
            rhs[i] += b[i + 1] * (target - end) / sections.len() as f64;
            for j in 0..2 {
                h[i][j] += b[i + 1] * b[j + 1] / sections.len() as f64;
            }
        }
    }
    (h, rhs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sections() -> Vec<FrontSection> {
        (0..=256)
            .map(|i| FrontSection {
                height_m: 1. + 0.4 * i as f64 / 256.,
                floor_m: 0.05,
                raw_body_z_m: 0.04,
                body_face: 0,
                x_m: 0.,
                lateral_limit_m: 0.2,
                clipped_segments: 1,
            })
            .collect()
    }
    pub(crate) fn fixture() -> WholeFront {
        WholeFront::fit(
            [1., 1.4, 1.45],
            [0.2, 0.1],
            [0.1, -0.2, -1.],
            0.04,
            &sections(),
            &[],
            &[],
        )
        .unwrap()
    }
    #[test]
    fn guide_preserves_endpoints_and_c2_piecewise_jets_without_beta_extrapolation() {
        let f = fixture();
        let old = |y: f64| {
            let d = y - 1.4;
            [0.1 - 0.2 * d - 0.5 * d * d, -0.2 - d, -1.]
        };
        assert!((f.jet(1., old(1.))[0] - 0.2).abs() < 1e-14);
        assert!((f.jet(1.4, old(1.4))[0] - 0.1).abs() < 1e-14);
        let guide = polynomial_jet(power(f.guide_bernstein), 1., 0.4);
        let bridge_start = polynomial_jet(f.bridge_power, 0., 0.05);
        let bridge_end = polynomial_jet(f.bridge_power, 1., 0.05);
        for k in 0..3 {
            assert!((guide[k] - old(1.4)[k] - bridge_start[k]).abs() < 1e-11);
            assert!(bridge_end[k].abs() < 1e-11);
        }
        for y in [0.9, 1.45, 1.5] {
            assert_eq!(f.jet(y, old(y)), old(y));
        }
        for i in 0..=1024 {
            let y = 1. + 0.4 * i as f64 / 1024.;
            let jet = f.jet(y, old(y));
            assert!(jet[0] >= 0.05 - 1e-8);
            assert!(jet[2] <= 1e-7);
        }
        for y in [1.1, 1.3, 1.41, 1.43] {
            let h = 1e-5;
            let jet = f.jet(y, old(y));
            let left = f.jet(y - h, old(y - h))[0];
            let right = f.jet(y + h, old(y + h))[0];
            assert!(((right - left) / (2. * h) - jet[1]).abs() < 1e-6);
            assert!(((right - 2. * jet[0] + left) / (h * h) - jet[2]).abs() < 1e-5);
        }
    }
    #[test]
    fn invalid_interval_and_conflicting_fixed_body_endpoint_are_errors() {
        assert!(WholeFront::validate_samples([1., 1.4], &[], &[]).is_err());
        let mut bad = sections();
        bad[20].height_m = 1.5;
        assert!(WholeFront::validate_samples([1., 1.4], &bad, &[]).is_err());
        let mut bad = sections();
        bad[20].floor_m = f64::NAN;
        assert!(WholeFront::validate_samples([1., 1.4], &bad, &[]).is_err());
        assert!(
            WholeFront::validate_samples(
                [1., 1.4],
                &sections(),
                &[FrontSupport {
                    height_m: 1.5,
                    floor_m: 0.,
                    origin: "test"
                }]
            )
            .is_err()
        );
        assert!(
            WholeFront::fit(
                [1., 1.4, 1.4],
                [0.2, 0.1],
                [0.1, -0.2, -1.],
                0.04,
                &sections(),
                &[],
                &[]
            )
            .is_err()
        );
        let mut body = sections();
        body[0].floor_m = 0.21;
        assert!(
            WholeFront::fit(
                [1., 1.4, 1.45],
                [0.2, 0.1],
                [0.1, -0.2, -1.],
                0.04,
                &body,
                &[],
                &[]
            )
            .is_err()
        );
    }
    #[test]
    fn diagnostic_hash_is_explicit_and_baseline_has_no_new_key() {
        let mut h = blake3::Hasher::new();
        let baseline = h.finalize();
        WholeFrontPolicy::Baseline.hash(&mut h);
        assert_eq!(baseline, h.finalize());
        WholeFrontPolicy::WholeHeight.hash(&mut h);
        assert_ne!(baseline, h.finalize());
    }
}
