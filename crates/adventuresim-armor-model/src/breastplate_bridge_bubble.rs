//! Minimal body-floor correction in the C2 endpoint-jet nullspace.
use serde::Serialize;

const BOUNDARY_MEMBERSHIP_M: f64 = 1e-7;
const MIN_RESOLVED_GAIN: f64 = f64::EPSILON;

#[derive(Clone, Copy, Debug, Default, Serialize, PartialEq)]
#[serde(transparent)]
pub(crate) struct BridgeBubble(f64);

impl BridgeBubble {
    pub(crate) fn is_zero(self) -> bool {
        self.0 == 0.
    }
    pub(crate) fn jet(self, t: f64, span: f64) -> [f64; 3] {
        let q = t * (1. - t);
        let dq = 1. - 2. * t;
        [
            self.0 * q.powi(3),
            self.0 * 3. * q * q * dq / span,
            self.0 * (6. * q * dq * dq - 6. * q * q) / (span * span),
        ]
    }
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct BridgeFloor {
    pub xy: [f64; 2],
    pub floor_m: f64,
    pub actual_before_m: f64,
    pub cosine: f64,
    pub occupied: bool,
}

#[derive(Debug, Serialize)]
pub(crate) enum BubbleError {
    InvalidDomain,
    InvalidRow { row: usize },
    UnadjustableFloor { row: usize, deficit_m: f64 },
    NonfiniteAmplitude { row: usize },
}
impl std::fmt::Display for BubbleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
enum RowUse {
    OutsideMaterial,
    OutsideBridge,
    EndpointOrUnresolvedGain,
    Adjustable,
}

#[derive(Debug, Serialize)]
struct ConstraintEvidence {
    source_row: usize,
    source: BridgeFloor,
    use_as: RowUse,
    fraction: f64,
    gain: f64,
    required_coefficient_m: Option<f64>,
}

#[derive(Debug, Serialize)]
pub(crate) struct BubbleFit {
    pub coefficient_m: BridgeBubble,
    rows: Vec<ConstraintEvidence>,
    active_rows: Vec<usize>,
}
impl BubbleFit {
    pub(crate) fn fit(heights: [f64; 2], floors: &[BridgeFloor]) -> Result<Self, BubbleError> {
        if heights.iter().any(|v| !v.is_finite()) || heights[1] <= heights[0] {
            return Err(BubbleError::InvalidDomain);
        }
        let mut result = Self {
            coefficient_m: BridgeBubble::default(),
            rows: Vec::new(),
            active_rows: Vec::new(),
        };
        for (i, row) in floors.iter().enumerate() {
            if row
                .xy
                .iter()
                .chain([&row.floor_m, &row.actual_before_m, &row.cosine])
                .any(|v| !v.is_finite())
                || !(-1. ..=1.).contains(&row.cosine)
            {
                return Err(BubbleError::InvalidRow { row: i });
            }
            let t = (row.xy[1] - heights[0]) / (heights[1] - heights[0]);
            let deficit = row.floor_m - row.actual_before_m;
            let gain = if (0. ..=1.).contains(&t) {
                row.cosine * (t * (1. - t)).powi(3)
            } else {
                0.
            };
            let use_as = if !row.occupied {
                RowUse::OutsideMaterial
            } else if !(0. ..=1.).contains(&t) {
                RowUse::OutsideBridge
            } else if gain <= MIN_RESOLVED_GAIN {
                RowUse::EndpointOrUnresolvedGain
            } else {
                RowUse::Adjustable
            };
            let bound = match use_as {
                RowUse::EndpointOrUnresolvedGain if deficit > 0. => {
                    return Err(BubbleError::UnadjustableFloor {
                        row: i,
                        deficit_m: deficit,
                    });
                }
                RowUse::Adjustable => {
                    let bound = deficit / gain;
                    if !bound.is_finite() {
                        return Err(BubbleError::NonfiniteAmplitude { row: i });
                    }
                    result.coefficient_m.0 = result.coefficient_m.0.max(bound);
                    Some(bound)
                }
                _ => None,
            };
            result.rows.push(ConstraintEvidence {
                source_row: i,
                source: row.clone(),
                use_as,
                fraction: t,
                gain,
                required_coefficient_m: bound,
            });
        }
        if !result.coefficient_m.is_zero() {
            result.active_rows = result
                .rows
                .iter()
                .filter(|r| r.required_coefficient_m == Some(result.coefficient_m.0))
                .map(|r| r.source_row)
                .collect();
        }
        Ok(result)
    }
}

pub(crate) fn occupied(point: [f64; 2], outline: &[[f64; 2]]) -> bool {
    let mut inside = false;
    for (a, b) in outline
        .iter()
        .zip(outline.iter().cycle().skip(1))
        .take(outline.len())
    {
        let delta = [b[0] - a[0], b[1] - a[1]];
        let length2 = delta[0] * delta[0] + delta[1] * delta[1];
        let t = if length2 > 0. {
            (((point[0] - a[0]) * delta[0] + (point[1] - a[1]) * delta[1]) / length2).clamp(0., 1.)
        } else {
            0.
        };
        if (point[0] - a[0] - t * delta[0]).hypot(point[1] - a[1] - t * delta[1])
            <= BOUNDARY_MEMBERSHIP_M
        {
            return true;
        }
        if (a[1] > point[1]) != (b[1] > point[1])
            && point[0] < a[0] + delta[0] * (point[1] - a[1]) / delta[1]
        {
            inside = !inside;
        }
    }
    inside
}

#[cfg(test)]
mod tests {
    use super::*;
    fn row(y: f64, deficit: f64, cosine: f64) -> BridgeFloor {
        BridgeFloor {
            xy: [0.1, y],
            floor_m: 0.1 + deficit,
            actual_before_m: 0.1,
            cosine,
            occupied: true,
        }
    }
    #[test]
    fn minimum_nonnegative_fit_satisfies_every_floor_and_zero_is_noop() {
        let rows = [
            row(1.25, 0.0003, 0.8),
            row(1.5, 0.001, 0.7),
            row(1.7, -0.002, 0.8),
        ];
        let fit = BubbleFit::fit([1., 2.], &rows).unwrap();
        for r in &rows {
            assert!(
                r.actual_before_m + r.cosine * fit.coefficient_m.jet(r.xy[1] - 1., 1.)[0]
                    >= r.floor_m - 1e-15
            );
        }
        let active = &rows[fit.active_rows[0]];
        let smaller = BridgeBubble(fit.coefficient_m.0 * (1. - 1e-5));
        assert!(
            active.actual_before_m + active.cosine * smaller.jet(active.xy[1] - 1., 1.)[0]
                < active.floor_m
        );
        assert!(
            BubbleFit::fit([1., 2.], &[row(1.5, -0.001, 0.8)])
                .unwrap()
                .coefficient_m
                .is_zero()
        );
        assert_eq!(BridgeBubble::default().jet(0.4, 0.05), [0.; 3]);
    }
    #[test]
    fn bubble_has_exact_endpoint_null_jets_and_scaled_analytic_derivatives() {
        let b = BridgeBubble(0.0607);
        for t in [0., 1.] {
            assert_eq!(b.jet(t, 0.055), [0.; 3]);
        }
        let t = 0.37;
        let h = 1e-5;
        let span = 0.055;
        let j = b.jet(t, span);
        assert!(
            ((b.jet(t + h, span)[0] - b.jet(t - h, span)[0]) / (2. * h * span) - j[1]).abs() < 1e-9
        );
        assert!(
            ((b.jet(t + h, span)[1] - b.jet(t - h, span)[1]) / (2. * h * span) - j[2]).abs() < 1e-7
        );
    }
    #[test]
    fn impossible_endpoints_and_cosines_are_errors_not_hidden_slack() {
        for r in [
            row(1., 0.001, 0.8),
            row(2., 0.001, 0.8),
            row(1.5, 0.001, 0.),
            row(1.5, 0.001, -0.1),
            row(1. + 1e-8, 0.001, 0.8),
        ] {
            assert!(matches!(
                BubbleFit::fit([1., 2.], &[r]),
                Err(BubbleError::UnadjustableFloor { .. })
            ));
        }
        assert!(matches!(
            BubbleFit::fit([1., 2.], &[row(1.5, 0.001, f64::NAN)]),
            Err(BubbleError::InvalidRow { .. })
        ));
        assert!(matches!(
            BubbleFit::fit([1., 1.], &[]),
            Err(BubbleError::InvalidDomain)
        ));
        // A lower side outside the bridge may lie microscopically past pi/2
        // due to the existing f32 coverage angle. It is not an editable row.
        assert!(
            BubbleFit::fit([1., 2.], &[row(0.9, 0., -4e-8)])
                .unwrap()
                .coefficient_m
                .is_zero()
        );
    }
    #[test]
    fn occupancy_includes_boundary_but_excludes_opening() {
        let poly = [
            [-1., 0.],
            [1., 0.],
            [1., 2.],
            [0.3, 2.],
            [0., 1.],
            [-0.3, 2.],
            [-1., 2.],
        ];
        assert!(occupied([1., 1.], &poly));
        assert!(occupied([0., 0.5], &poly));
        assert!(!occupied([0., 1.8], &poly));
    }
}
