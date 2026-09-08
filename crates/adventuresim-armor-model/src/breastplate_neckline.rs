//! A symmetric neckline with explicit shoulder-corner intent.

use crate::breastplate_shoulder_band::{BandError, BandFraction, BandStation};
use crate::breastplate_topology::SemanticBoundaryRange;
use serde::Serialize;

const NECK_CENTER_HANDLE_FRACTION: f64 = 0.50;
const PRIOR_END_HANDLE_X_FRACTION: f64 = 0.12;
const PRIOR_END_HANDLE_Y_FRACTION: f64 = 0.58;
const ARC_INTERVALS: usize = 256;
const ROUNDED_U_ENV: &str = "BREASTPLATE_DIAGNOSTIC_ROUNDED_U_CORNER";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum NecklineTreatment {
    TangentContinuous,
    RoundedUCorner,
}

impl NecklineTreatment {
    pub(crate) fn from_environment(band: Option<BandFraction>) -> Result<Self, BandError> {
        if std::env::var_os(ROUNDED_U_ENV).is_some() {
            if band.is_none() {
                return Err(BandError::RequiresShoulderBand);
            }
            Ok(Self::RoundedUCorner)
        } else {
            Ok(Self::TangentContinuous)
        }
    }

    pub(crate) fn hash(self, hash: &mut blake3::Hasher) {
        if self == Self::RoundedUCorner {
            hash.update(b"rounded-u-deliberate-corner-v1");
        }
    }

    pub(crate) fn continuity(self) -> &'static str {
        match self {
            Self::TangentContinuous => "G1 in native parameters; C1 in physical arc length",
            Self::RoundedUCorner => {
                "shared endpoint with deliberate rounded-U corner; no G1 requirement"
            }
        }
    }

    pub(crate) fn curve(self, center_y: f64, inner: BandStation) -> Result<Neckline, BandError> {
        match self {
            Self::TangentContinuous => Neckline::meeting_band(center_y, inner),
            Self::RoundedUCorner => {
                let [x, y, _] = inner.position;
                Neckline::from_controls([
                    [0.0, center_y],
                    [NECK_CENTER_HANDLE_FRACTION * x, center_y],
                    [
                        (1.0 - PRIOR_END_HANDLE_X_FRACTION) * x,
                        y - PRIOR_END_HANDLE_Y_FRACTION * (y - center_y),
                    ],
                    [x, y],
                ])
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct Neckline {
    controls: [[f64; 2]; 4],
    minimum_derivative_xy: [f64; 2],
}

impl Neckline {
    pub(crate) fn meeting_band(center_y: f64, inner: BandStation) -> Result<Self, BandError> {
        let endpoint = [inner.position[0], inner.position[1]];
        let tangent = [inner.tangent[0], inner.tangent[1]];
        let tangent_length = tangent[0].hypot(tangent[1]);
        if !center_y.is_finite() || tangent_length <= 0.0 || !tangent_length.is_finite() {
            return Err(BandError::InvalidCurve);
        }
        let handle_length = (PRIOR_END_HANDLE_X_FRACTION * endpoint[0])
            .hypot(PRIOR_END_HANDLE_Y_FRACTION * (endpoint[1] - center_y));
        let controls = [
            [0.0, center_y],
            [NECK_CENTER_HANDLE_FRACTION * endpoint[0], center_y],
            std::array::from_fn(|axis| {
                endpoint[axis] - handle_length * tangent[axis] / tangent_length
            }),
            endpoint,
        ];
        Self::from_controls(controls)
    }

    fn from_controls(controls: [[f64; 2]; 4]) -> Result<Self, BandError> {
        if controls.iter().flatten().any(|x| !x.is_finite()) {
            return Err(BandError::InvalidCurve);
        }
        let minimum_derivative_xy = std::array::from_fn(|axis| {
            quadratic_minimum(std::array::from_fn(|i| {
                3.0 * (controls[i + 1][axis] - controls[i][axis])
            }))
        });
        // Exact quadratic extrema permit unordered derivative controls when
        // the actual derivative remains positive throughout the interval.
        if minimum_derivative_xy[0] <= 0.0 || minimum_derivative_xy[1] < 0.0 {
            return Err(BandError::InvalidCurve);
        }
        Ok(Self {
            controls,
            minimum_derivative_xy,
        })
    }

    fn at(&self, t: f64) -> [f64; 2] {
        let s = 1.0 - t;
        let weights = [s.powi(3), 3.0 * s * s * t, 3.0 * s * t * t, t.powi(3)];
        std::array::from_fn(|axis| (0..4).map(|i| weights[i] * self.controls[i][axis]).sum())
    }

    fn derivative(&self, t: f64) -> [f64; 2] {
        let weights = [(1.0 - t).powi(2), 2.0 * t * (1.0 - t), t * t];
        std::array::from_fn(|axis| {
            (0..3)
                .map(|i| 3.0 * weights[i] * (self.controls[i + 1][axis] - self.controls[i][axis]))
                .sum()
        })
    }

    pub(crate) fn endpoint_tangent(&self) -> [f64; 2] {
        self.derivative(1.0)
    }

    /// XY and target-tangent-plane lifted angles; final fitted-field angles
    /// must be evaluated independently from the actual field's normal.
    pub(crate) fn corner_angles(&self, inner: BandStation) -> [f64; 2] {
        let [dx, dy] = self.endpoint_tangent();
        let xy = ((dx * inner.tangent[0] + dy * inner.tangent[1])
            / (dx.hypot(dy) * inner.tangent[0].hypot(inner.tangent[1])))
        .clamp(-1.0, 1.0)
        .acos();
        let dz = -(inner.normal[0] * dx + inner.normal[1] * dy) / inner.normal[2];
        let lifted = ((dx * inner.tangent[0] + dy * inner.tangent[1] + dz * inner.tangent[2])
            / (dx * dx + dy * dy + dz * dz).sqrt())
        .clamp(-1.0, 1.0)
        .acos();
        [xy, lifted]
    }

    pub(crate) fn stations(&self, range: &SemanticBoundaryRange) -> Vec<(usize, [f64; 2])> {
        let dense = (0..=ARC_INTERVALS)
            .map(|i| self.at(i as f64 / ARC_INTERVALS as f64))
            .collect::<Vec<_>>();
        let mut arc = vec![0.0];
        for pair in dense.windows(2) {
            arc.push(
                arc.last().unwrap() + (pair[1][0] - pair[0][0]).hypot(pair[1][1] - pair[0][1]),
            );
        }
        (0..=range.segments)
            .map(|offset| {
                let centered = 2.0 * offset as f64 / range.segments as f64 - 1.0;
                let target = arc.last().unwrap() * centered.abs();
                let upper = arc.partition_point(|s| *s < target).min(ARC_INTERVALS);
                let point = if upper == 0 {
                    dense[0]
                } else {
                    let fraction = (target - arc[upper - 1]) / (arc[upper] - arc[upper - 1]);
                    let t = (upper as f64 - 1.0 + fraction) / ARC_INTERVALS as f64;
                    self.at(t)
                };
                (
                    range.start + offset,
                    [point[0] * centered.signum(), point[1]],
                )
            })
            .collect()
    }
}

fn quadratic_minimum(b: [f64; 3]) -> f64 {
    let mut minimum = b[0].min(b[2]);
    let quadratic = b[0] - 2.0 * b[1] + b[2];
    if quadratic > 0.0 {
        let t = (b[0] - b[1]) / quadratic;
        if t > 0.0 && t < 1.0 {
            minimum =
                minimum.min(b[0] * (1.0 - t).powi(2) + 2.0 * b[1] * t * (1.0 - t) + b[2] * t * t);
        }
    }
    minimum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounded_u_retains_shared_endpoint_but_has_a_deliberate_lifted_corner() {
        let band = band();
        let inner = band.sample(0.0).unwrap();
        let curve = NecklineTreatment::RoundedUCorner
            .curve(1.40, inner)
            .unwrap();
        assert_eq!(curve.at(1.0), [inner.position[0], inner.position[1]]);
        assert_eq!(curve.at(0.0), [0.0, 1.40]);
        assert_eq!(curve.derivative(0.0)[1], 0.0);
        let d = curve.endpoint_tangent();
        let cross = d[0] * inner.tangent[1] - d[1] * inner.tangent[0];
        assert!(cross.abs() > 0.01);
        let lifted = [d[0], d[1], -0.6 * d[0]];
        let dot = lifted
            .iter()
            .zip(inner.tangent)
            .map(|(a, b)| a * b)
            .sum::<f64>();
        let length = lifted.iter().map(|v| v * v).sum::<f64>().sqrt();
        assert!((dot / length).acos() > 0.1);
        let layout = CanonicalBreastplateTopology::semantic_layout();
        let range = layout
            .iter()
            .find(|r| r.edge == BreastplateBoundaryEdge::Neck)
            .unwrap();
        let stations = curve.stations(range);
        assert_eq!(stations.last().unwrap().1, curve.at(1.0));
        assert_eq!(
            stations.first().unwrap().1,
            [-inner.position[0], inner.position[1]]
        );
        let mut unchanged = blake3::Hasher::new();
        let expected = unchanged.finalize();
        NecklineTreatment::TangentContinuous.hash(&mut unchanged);
        assert_eq!(unchanged.finalize(), expected);
        NecklineTreatment::RoundedUCorner.hash(&mut unchanged);
        assert_ne!(unchanged.finalize(), expected);
    }
    use crate::breastplate_shoulder_band::{
        BandFraction, CREST_SEED_COUNT, CrestSample, ShoulderBand,
    };
    use crate::breastplate_topology::{BreastplateBoundaryEdge, CanonicalBreastplateTopology};

    fn band() -> ShoulderBand {
        let samples = |start: f64| {
            std::array::from_fn(|i| {
                let x = start + (0.18 - start) * i as f64 / (CREST_SEED_COUNT - 1) as f64;
                CrestSample {
                    position: [x, 1.45 + 0.02 * (x - 0.08), -0.6 * x],
                    normal: [0.4, 0.65, 0.65],
                }
            })
        };
        ShoulderBand::fit(
            BandFraction::new(0.3).unwrap(),
            samples(0.02),
            samples(0.08),
        )
        .unwrap()
    }

    #[test]
    fn rotated_handle_preserves_endpoints_and_lifted_three_dimensional_tangent() {
        let band = band();
        let inner = band.sample(0.0).unwrap();
        let neckline = Neckline::meeting_band(1.43, inner).unwrap();
        assert_eq!(neckline.at(0.0), [0.0, 1.43]);
        assert_eq!(neckline.at(1.0), [inner.position[0], inner.position[1]]);
        assert_eq!(neckline.derivative(0.0)[1], 0.0);
        let lift_tangent = |d: [f64; 2]| {
            let v = [d[0], d[1], -0.27 * d[0] - 1.13 * d[1]];
            let length = v.iter().map(|x| x * x).sum::<f64>().sqrt();
            v.map(|x| x / length)
        };
        let a = lift_tangent(neckline.endpoint_tangent());
        let b = lift_tangent([inner.tangent[0], inner.tangent[1]]);
        assert!(a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-12));
    }

    #[test]
    fn sampled_shared_station_is_unique_and_nearby_lifted_chords_converge() {
        let band = band();
        let inner = band.sample(0.0).unwrap();
        let neckline = Neckline::meeting_band(1.43, inner).unwrap();
        let layout = CanonicalBreastplateTopology::semantic_layout();
        let neck_range = layout
            .iter()
            .find(|r| r.edge == BreastplateBoundaryEdge::Neck)
            .unwrap();
        let neck = neckline.stations(neck_range);
        let shoulders = band.stations(&layout).unwrap();
        for (index, p) in [&neck[0], neck.last().unwrap()] {
            assert_eq!(
                *p,
                [shoulders[index].position[0], shoulders[index].position[1]]
            );
        }
        let lift = |p: [f64; 2]| {
            [
                p[0],
                p[1],
                0.03 + 0.20 * (1.0 - (p[0] / 0.25).powi(2)).sqrt() - 0.7 * p[1],
            ]
        };
        let gap = |step: f64| {
            let p = lift(neckline.at(1.0));
            let a = lift(neckline.at(1.0 - step));
            let band_next = band.sample(step).unwrap();
            let b = lift([band_next.position[0], band_next.position[1]]);
            let unit = |v: [f64; 3]| {
                let n = v.iter().map(|x| x * x).sum::<f64>().sqrt();
                v.map(|x| x / n)
            };
            let incoming = unit(std::array::from_fn(|i| p[i] - a[i]));
            let outgoing = unit(std::array::from_fn(|i| b[i] - p[i]));
            incoming
                .iter()
                .zip(outgoing)
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f64>()
                .sqrt()
        };
        assert!(gap(1e-5) < gap(1e-3) * 0.02);
        assert!(gap(1e-5) < 1e-4);
    }

    #[test]
    fn exact_extrema_accept_unordered_controls_but_reject_a_real_reversal() {
        assert!(Neckline::from_controls([[0.0, 0.0], [1.0, 0.0], [0.8, 0.5], [1.8, 1.0]]).is_ok());
        assert!(
            Neckline::from_controls([[0.0, 0.0], [1.0, 0.0], [-1.0, 0.5], [0.1, 1.0]]).is_err()
        );
    }
}
