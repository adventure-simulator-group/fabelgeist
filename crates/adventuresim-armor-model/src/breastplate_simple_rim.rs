//! Simple authored shoulder/arm boundary, independent of anatomical wrinkles.

use crate::breastplate_seated_band::ShoulderSeat;
use crate::breastplate_shoulder_band::{
    BandError, BandStation, CREST_SEED_COUNT, CrestSample, ShoulderBand,
};
use serde::Serialize;

const SIMPLE_RIM_ENV: &str = "BREASTPLATE_DIAGNOSTIC_SIMPLE_RIM";
const START_LATERAL_HANDLE: f64 = 0.25;
const START_DESCENT_HANDLE: f64 = 0.35;
const END_DESCENT_HANDLE: f64 = 0.70;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum RimPolicy {
    MeasuredCrest,
    StraightShoulderCorner,
}

impl RimPolicy {
    pub(crate) fn from_environment(seat: Option<ShoulderSeat>) -> Result<Self, BandError> {
        if std::env::var_os(SIMPLE_RIM_ENV).is_none() {
            return Ok(Self::MeasuredCrest);
        }
        if seat.is_none() {
            return Err(BandError::RequiresSeatedBand);
        }
        Ok(Self::StraightShoulderCorner)
    }

    pub(crate) fn hash(self, hash: &mut blake3::Hasher) {
        if self == Self::StraightShoulderCorner {
            hash.update(b"straight-shoulder-descending-arm-corner-v1");
        }
    }

    pub(crate) fn arm_opening(
        self,
        band: Option<&ShoulderBand>,
        underarm: [f32; 2],
    ) -> Result<Option<DescendingArmOpening>, BandError> {
        if self == Self::MeasuredCrest {
            return Ok(None);
        }
        let band = band.ok_or(BandError::RequiresSeatedBand)?;
        let endpoint = band.sample(1.0)?;
        let arm = DescendingArmOpening::new(
            [endpoint.position[0] as f32, endpoint.position[1] as f32],
            underarm,
        )?;
        ShoulderBand::write_dump(
            "simple-rim",
            &serde_json::json!({
                "policy": self, "shoulder": band, "arm_opening": arm,
                "arm_endpoint_tangents_xy": arm.endpoint_tangents(),
                "outer_corner_xy_degrees": arm.outer_corner_xy_degrees(endpoint),
                "join_policy": "shared shoulder endpoint, deliberate C0 outer corner; descending arm retains derived angular lower return",
                "query_role": "body queries locate endpoints and validate seat; intermediate anatomy does not author line or normals",
            }),
        )?;
        Ok(Some(arm))
    }

    pub(crate) fn author(
        self,
        measured: [CrestSample; CREST_SEED_COUNT],
    ) -> [CrestSample; CREST_SEED_COUNT] {
        match self {
            Self::MeasuredCrest => measured,
            Self::StraightShoulderCorner => {
                let first = measured[0];
                let last = measured[CREST_SEED_COUNT - 1];
                std::array::from_fn(|i| {
                    let t = i as f64 / (CREST_SEED_COUNT - 1) as f64;
                    CrestSample {
                        position: std::array::from_fn(|axis| {
                            first.position[axis] * (1.0 - t) + last.position[axis] * t
                        }),
                        // Endpoint-only vector interpolation. Normalization and
                        // projection off the authored tangent happen at sampling.
                        normal: std::array::from_fn(|axis| {
                            first.normal[axis] * (1.0 - t) + last.normal[axis] * t
                        }),
                    }
                })
            }
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct DescendingArmOpening {
    controls: [[f64; 2]; 4],
    derivative_x_range: [f64; 2],
    derivative_y_range: [f64; 2],
}

impl DescendingArmOpening {
    pub(crate) fn new(shoulder: [f32; 2], underarm: [f32; 2]) -> Result<Self, BandError> {
        let a = shoulder.map(f64::from);
        let b = underarm.map(f64::from);
        if a.into_iter().chain(b).any(|v| !v.is_finite()) || b[1] >= a[1] || b[0] <= a[0] {
            return Err(BandError::InvalidCurve);
        }
        let controls = [
            a,
            [
                a[0] + START_LATERAL_HANDLE * (b[0] - a[0]),
                a[1] + START_DESCENT_HANDLE * (b[1] - a[1]),
            ],
            [b[0], a[1] + END_DESCENT_HANDLE * (b[1] - a[1])],
            b,
        ];
        let derivative_x_range = derivative_range(controls, 0);
        let derivative_y_range = derivative_range(controls, 1);
        if derivative_x_range[0] < 0.0 || derivative_y_range[1] >= 0.0 {
            return Err(BandError::InvalidCurve);
        }
        Ok(Self {
            controls,
            derivative_x_range,
            derivative_y_range,
        })
    }

    pub(crate) fn at(&self, t: f32) -> [f32; 2] {
        let t = t as f64;
        let s = 1.0 - t;
        let weights = [s * s * s, 3.0 * s * s * t, 3.0 * s * t * t, t * t * t];
        std::array::from_fn(|axis| {
            (0..4)
                .map(|i| weights[i] * self.controls[i][axis])
                .sum::<f64>() as f32
        })
    }

    pub(crate) fn endpoint_tangents(&self) -> [[f64; 2]; 2] {
        [
            std::array::from_fn(|axis| 3.0 * (self.controls[1][axis] - self.controls[0][axis])),
            std::array::from_fn(|axis| 3.0 * (self.controls[3][axis] - self.controls[2][axis])),
        ]
    }

    pub(crate) fn outer_corner_xy_degrees(&self, shoulder: BandStation) -> f64 {
        let arm = self.endpoint_tangents()[0];
        let edge = shoulder.tangent;
        ((arm[0] * edge[0] + arm[1] * edge[1]) / (arm[0].hypot(arm[1]) * edge[0].hypot(edge[1])))
            .clamp(-1.0, 1.0)
            .acos()
            .to_degrees()
    }
}

fn derivative_range(controls: [[f64; 2]; 4], axis: usize) -> [f64; 2] {
    let b: [f64; 3] = std::array::from_fn(|i| 3.0 * (controls[i + 1][axis] - controls[i][axis]));
    let mut values = vec![b[0], b[2]];
    let q = b[0] - 2.0 * b[1] + b[2];
    if q != 0.0 {
        let t = (b[0] - b[1]) / q;
        if (0.0..1.0).contains(&t) {
            values.push(b[0] * (1.0 - t).powi(2) + 2.0 * b[1] * t * (1.0 - t) + b[2] * t * t);
        }
    }
    [
        values.iter().copied().fold(f64::INFINITY, f64::min),
        values.into_iter().fold(f64::NEG_INFINITY, f64::max),
    ]
}

/// Controlled measured-rim comparison uses its original quarter ellipse.
pub(crate) fn armhole_curve(shoulder: [f32; 2], mid_axillary: [f32; 2], t: f32) -> [f32; 2] {
    let t = t.clamp(0.0, 1.0);
    let angle = std::f32::consts::FRAC_PI_2 * t;
    let lateral_station = angle.sin();
    let vertical_station = 1.0 - angle.cos();
    [
        shoulder[0] + (mid_axillary[0] - shoulder[0]) * lateral_station,
        shoulder[1] + (mid_axillary[1] - shoulder[1]) * vertical_station,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::breastplate_shoulder_band::{BandFraction, ShoulderBand};

    #[test]
    fn interior_body_wrinkles_do_not_author_straight_edge_or_normal_guide() {
        let raw = std::array::from_fn(|i| CrestSample {
            position: [0.08 + 0.04 * i as f64 / 8.0, 1.45, -0.03 * i as f64 / 8.0],
            normal: [0.2, 0.7, 0.6],
        });
        let mut wrinkled = raw;
        for sample in &mut wrinkled[1..CREST_SEED_COUNT - 1] {
            sample.position[1] += 0.007;
            sample.normal = [0.5, 0.5, 0.5];
        }
        let a = RimPolicy::StraightShoulderCorner.author(raw);
        let b = RimPolicy::StraightShoulderCorner.author(wrinkled);
        for i in 0..CREST_SEED_COUNT {
            assert_eq!(a[i].position, b[i].position);
            assert_eq!(a[i].normal, b[i].normal);
        }
        assert_ne!(
            RimPolicy::MeasuredCrest.author(wrinkled)[3].position,
            a[3].position
        );
        let baseline = ShoulderBand::fit(BandFraction::new(0.3).unwrap(), raw, raw).unwrap();
        let authored = baseline.fitted_used_interval(a).unwrap();
        let first = authored.sample(0.0).unwrap();
        let last = authored.sample(1.0).unwrap();
        for t in [0.1, 0.4, 0.7] {
            let sample = authored.sample(t).unwrap();
            for axis in 0..3 {
                assert!(
                    (sample.position[axis]
                        - first.position[axis] * (1.0 - t)
                        - last.position[axis] * t)
                        .abs()
                        < 1e-12
                );
            }
            assert!(
                (sample
                    .normal
                    .iter()
                    .zip(sample.tangent)
                    .map(|(a, b)| a * b)
                    .sum::<f64>())
                .abs()
                    < 1e-12
            );
        }
    }

    #[test]
    fn descending_arm_has_exact_endpoints_no_horizontal_roof_and_no_extra_extrema() {
        for scale in [0.8, 1.0, 1.4] {
            let a = [0.12 * scale, 1.48 * scale];
            let b = [0.18 * scale, 1.25 * scale];
            let arm = DescendingArmOpening::new(a, b).unwrap();
            assert_eq!(arm.at(0.0), a);
            assert_eq!(arm.at(1.0), b);
            assert!(arm.derivative_y_range[1] < 0.0);
            assert!(arm.derivative_x_range[0] >= 0.0);
            assert!(arm.endpoint_tangents()[0][1] < 0.0);
            assert_eq!(arm.endpoint_tangents()[1][0], 0.0);
            let points = (0..=1000)
                .map(|i| arm.at(i as f32 / 1000.0))
                .collect::<Vec<_>>();
            assert!(
                points
                    .windows(2)
                    .all(|w| w[1][1] < w[0][1] && w[1][0] >= w[0][0])
            );
            let start = arm.at(0.05);
            assert!(a[1] - start[1] > start[0] - a[0]);
        }
        assert!(DescendingArmOpening::new([0.1, 1.0], [0.2, 1.2]).is_err());
        assert!(DescendingArmOpening::new([0.2, 1.2], [0.1, 1.0]).is_err());
    }

    #[test]
    fn deliberate_outer_corner_owns_one_shared_semantic_station() {
        use crate::breastplate_topology::{BreastplateBoundaryEdge, CanonicalBreastplateTopology};
        let raw = std::array::from_fn(|i| CrestSample {
            position: [0.08 + 0.04 * i as f64 / 8.0, 1.45, -0.03 * i as f64 / 8.0],
            normal: [0.2, 0.7, 0.6],
        });
        let band = ShoulderBand::fit(BandFraction::new(0.3).unwrap(), raw, raw)
            .unwrap()
            .fitted_used_interval(RimPolicy::StraightShoulderCorner.author(raw))
            .unwrap();
        let layout = CanonicalBreastplateTopology::semantic_layout();
        let shoulder = layout
            .iter()
            .find(|r| r.edge == BreastplateBoundaryEdge::RightShoulder)
            .unwrap();
        let opening = layout
            .iter()
            .find(|r| r.edge == BreastplateBoundaryEdge::RightArmhole)
            .unwrap();
        assert_eq!(shoulder.start + shoulder.segments, opening.start);
        let stations = band.stations(&layout).unwrap();
        let end = stations[&opening.start];
        let arm = DescendingArmOpening::new(
            [end.position[0] as f32, end.position[1] as f32],
            [0.18, 1.25],
        )
        .unwrap();
        assert_eq!(
            arm.at(0.0),
            [end.position[0] as f32, end.position[1] as f32]
        );
        assert_eq!(
            stations
                .values()
                .filter(|p| p.position == end.position)
                .count(),
            1
        );
        assert!(arm.outer_corner_xy_degrees(end) > 45.0);
        let preceding = stations[&(opening.start - 1)].position;
        let following = arm.at(0.01);
        assert!(preceding[0] < end.position[0]);
        assert!(following[1] < end.position[1] as f32);
    }

    #[test]
    fn simple_policy_hash_is_distinct_and_measured_comparison_is_unchanged() {
        let mut hash = blake3::Hasher::new();
        let baseline = hash.finalize();
        RimPolicy::MeasuredCrest.hash(&mut hash);
        assert_eq!(hash.finalize(), baseline);
        RimPolicy::StraightShoulderCorner.hash(&mut hash);
        assert_ne!(hash.finalize(), baseline);
    }
}
