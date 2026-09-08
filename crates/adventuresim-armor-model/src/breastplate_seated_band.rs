//! Independent posterior seating and a bounded sparse physical-width solve.

use crate::breastplate_crest_curve::normalized;
use crate::breastplate_crest_policy::CrestQueryPolicy;
use crate::breastplate_shoulder_band::{
    BandError, BandFraction, CREST_SEED_COUNT, CrestSample, ShoulderBand,
};
use crate::breastplate_simple_rim::RimPolicy;
use serde::Serialize;

const SEAT_ENV: &str = "BREASTPLATE_DIAGNOSTIC_SHOULDER_SEAT";
const POLICY: &str = "posterior-wall-fraction-used-interval-v1";
const WIDTH_TOLERANCE_M: f64 = 0.00005;
const LENGTH_ORDER_TOLERANCE_M: f64 = 1e-8;
const MAX_SOLVE_TRIALS: usize = 32;
const VALIDATION_INTERVALS: usize = 64;
const MIN_PARAMETER_BRACKET: f64 = 1e-7;

#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) struct ShoulderSeat(f64);

impl ShoulderSeat {
    pub(crate) fn new(value: f64) -> Result<Self, BandError> {
        if value.is_finite() && value > 0.0 && value < 1.0 {
            Ok(Self(value))
        } else {
            Err(BandError::InvalidFraction)
        }
    }

    pub(crate) fn from_environment(
        band: Option<BandFraction>,
        crest: CrestQueryPolicy,
    ) -> Result<Option<Self>, BandError> {
        let Some(raw) = std::env::var_os(SEAT_ENV) else {
            return Ok(None);
        };
        if band.is_none() || crest != CrestQueryPolicy::SuperiorFirst {
            return Err(BandError::RequiresSuperiorFirst);
        }
        Self::new(
            raw.to_str()
                .and_then(|s| s.parse().ok())
                .ok_or(BandError::InvalidFraction)?,
        )
        .map(Some)
    }

    pub(crate) fn fraction(self) -> f64 {
        self.0
    }

    pub(crate) fn hash(self, hash: &mut blake3::Hasher) {
        hash.update(POLICY.as_bytes());
        hash.update(&self.0.to_bits().to_le_bytes());
        hash.update(&WIDTH_TOLERANCE_M.to_bits().to_le_bytes());
    }
}

#[derive(Debug, Serialize)]
struct Trial {
    terminal_remaining_fraction: f64,
    seed_starts: [f32; 2],
    seed_ends: [f32; 2],
    bracket_before: [f64; 2],
    raw: Vec<[CrestSample; 2]>,
    fitted: Option<ShoulderBand>,
    failure: Option<String>,
    failure_code: Option<BandError>,
    first_invalid_remaining_fraction: Option<f64>,
}

impl Trial {
    fn measure(
        terminal: f64,
        bracket: [f64; 2],
        starts: [f32; 2],
        baseline: &ShoulderBand,
        rim: RimPolicy,
        query: &mut impl FnMut(usize, f32) -> Result<CrestSample, BandError>,
    ) -> Self {
        let mut result = Self {
            terminal_remaining_fraction: terminal,
            seed_starts: starts,
            seed_ends: starts.map(|s| s + (1.0 - s) * terminal as f32),
            bracket_before: bracket,
            raw: Vec::new(),
            fitted: None,
            failure: None,
            failure_code: None,
            first_invalid_remaining_fraction: None,
        };
        let mut samples = [CrestSample {
            position: [0.0; 3],
            normal: [0.0; 3],
        }; CREST_SEED_COUNT];
        for (i, sample) in samples.iter_mut().enumerate() {
            let q = terminal * i as f64 / (CREST_SEED_COUNT - 1) as f64;
            let mut pair = [*sample; 2];
            for side in 0..2 {
                let fraction = starts[side] + (1.0 - starts[side]) * q as f32;
                match query(side, fraction) {
                    Ok(hit) => pair[side] = hit,
                    Err(error) => {
                        result.failure = Some(error.to_string());
                        result.failure_code = Some(error);
                        result.first_invalid_remaining_fraction = Some(q);
                        return result;
                    }
                }
            }
            result.raw.push(pair);
            sample.position =
                std::array::from_fn(|axis| (pair[0].position[axis] + pair[1].position[axis]) * 0.5);
            let normal =
                std::array::from_fn(|axis| (pair[0].normal[axis] + pair[1].normal[axis]) * 0.5);
            match normalized(normal) {
                Ok(normal) => sample.normal = normal,
                Err(error) => {
                    result.failure = Some(error.to_string());
                    return result;
                }
            }
        }
        match baseline.fitted_used_interval(rim.author(samples)) {
            Ok(fitted) => result.fitted = Some(fitted),
            Err(error) => result.failure = Some(error.to_string()),
        }
        result
    }
}

pub(crate) fn fit_seated_band(
    baseline: &ShoulderBand,
    seat: ShoulderSeat,
    starts: [f32; 2],
    rim: RimPolicy,
    mut query: impl FnMut(usize, f32) -> Result<CrestSample, BandError>,
) -> Result<ShoulderBand, BandError> {
    let mut trials = Vec::new();
    let mut bracket = [0.0, 1.0];
    let result = solve(baseline, starts, rim, &mut query, &mut trials, &mut bracket);
    ShoulderBand::write_dump(
        "shoulder-seat-trials",
        &serde_json::json!({
            "policy": POLICY, "seat": seat, "unseated_width_baseline": baseline,
            "rim_policy": rim,
            "raw_semantics": "trial.raw is measured body; fitted.available_raw is policy-authored input",
            "terminal_parameter": "common normalized remaining rig interval; end=inner+(1-inner)*q",
            "requested_arc_m": baseline.requested_arc_m(), "width_tolerance_m": WIDTH_TOLERANCE_M,
            "stations": "normalized realized full cubic arc, exact t=1 terminal",
            "validation_intervals": VALIDATION_INTERVALS, "trials": trials, "final_bracket": bracket,
            "bracket_semantics": "parameter search bounds; upper may be a query-invalid prefix cap, not a certified positive arc residual",
            "stopping_reason": result.as_ref().map(|_| "width-tolerance-and-query-validation").map_err(|e| e.to_string()),
            "realized_arc_m": result.as_ref().ok().map(ShoulderBand::available_arc_m),
            "arc_residual_m": result.as_ref().ok().map(|b| b.available_arc_m() - baseline.requested_arc_m()),
        }),
    )?;
    if let Ok(band) = &result {
        ShoulderBand::write_dump("shoulder-band", band)?;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> ShoulderBand {
        let samples = |start| {
            std::array::from_fn(|i| {
                let f = start + (1.0 - start) * i as f64 / (CREST_SEED_COUNT - 1) as f64;
                CrestSample {
                    position: [0.2 * f, 0.0, 0.0],
                    normal: [0.0, 0.0, 1.0],
                }
            })
        };
        ShoulderBand::fit(BandFraction::new(0.3).unwrap(), samples(0.0), samples(0.2)).unwrap()
    }

    #[test]
    fn seat_moves_path_without_renormalizing_physical_width() {
        let baseline = baseline();
        let make = |seat| {
            fit_seated_band(
                &baseline,
                ShoulderSeat::new(seat).unwrap(),
                [0.2; 2],
                RimPolicy::MeasuredCrest,
                |_, f| {
                    Ok(CrestSample {
                        position: [0.3 * f as f64, seat, -seat],
                        normal: [0.0, 0.0, 1.0],
                    })
                },
            )
            .unwrap()
        };
        let a = make(0.125);
        let b = make(0.25);
        assert_eq!(a.requested_arc_m(), baseline.requested_arc_m());
        assert!((a.available_arc_m() - baseline.requested_arc_m()).abs() <= WIDTH_TOLERANCE_M);
        assert_eq!(a.available_arc_m(), b.available_arc_m());
        for t in [0.0, 0.4, 1.0] {
            assert!(
                (b.sample(t).unwrap().position[1] - a.sample(t).unwrap().position[1] - 0.125).abs()
                    < 1e-12
            );
        }
        let endpoint = a.sample(1.0).unwrap();
        assert!((endpoint.position[0] - 0.12).abs() <= WIDTH_TOLERANCE_M);
    }

    #[test]
    fn invalid_oversized_trial_is_a_search_cap_not_immediate_failure() {
        let baseline = baseline();
        let mut trials = Vec::new();
        let mut bracket = [0.0, 1.0];
        let fitted = solve(
            &baseline,
            [0.2; 2],
            RimPolicy::MeasuredCrest,
            &mut |_, f| {
                if f > 0.47 {
                    return Err(BandError::InvalidNormal);
                }
                Ok(CrestSample {
                    position: [0.3 * f as f64, 0.0, 0.0],
                    normal: [0.0, 0.0, 1.0],
                })
            },
            &mut trials,
            &mut bracket,
        )
        .unwrap();
        assert!(trials[0].failure.is_some());
        assert!((fitted.available_arc_m() - baseline.requested_arc_m()).abs() <= WIDTH_TOLERANCE_M);
        assert!(fitted.sample(1.0).unwrap().position[0] < 0.47 * 0.3);
    }

    #[test]
    fn unavailable_width_and_invalid_inner_seed_fail_without_fallback() {
        for limit in [0.1, 0.25] {
            let result = fit_seated_band(
                &baseline(),
                ShoulderSeat::new(0.125).unwrap(),
                [0.2; 2],
                RimPolicy::MeasuredCrest,
                |_, f| {
                    if f > limit {
                        return Err(BandError::InvalidSeed);
                    }
                    Ok(CrestSample {
                        position: [0.3 * f as f64, 0.0, 0.0],
                        normal: [0.0, 0.0, 1.0],
                    })
                },
            );
            if limit < 0.2 {
                assert!(matches!(result, Err(BandError::InvalidSeed)));
            } else {
                assert!(matches!(result, Err(BandError::UnavailableWidth)));
            }
        }
    }

    #[test]
    fn straight_rim_solves_chord_width_and_validates_every_dense_seed() {
        let baseline = baseline();
        let mut queries = Vec::new();
        let mut trials = Vec::new();
        let mut bracket = [0.0, 1.0];
        let band = solve(
            &baseline,
            [0.2; 2],
            RimPolicy::StraightShoulderCorner,
            &mut |side, f| {
                queries.push((side, f));
                Ok(CrestSample {
                    position: [
                        0.3 * f as f64,
                        0.01 * (f as f64 * 20.0).sin(),
                        -0.1 * f as f64,
                    ],
                    normal: [0.0, 0.7, 0.7],
                })
            },
            &mut trials,
            &mut bracket,
        )
        .unwrap();
        let a = band.sample(0.0).unwrap().position;
        let b = band.sample(1.0).unwrap().position;
        let chord = (0..3)
            .map(|axis| (b[axis] - a[axis]).powi(2))
            .sum::<f64>()
            .sqrt();
        assert!((band.available_arc_m() - chord).abs() < 1e-12);
        assert!((chord - baseline.requested_arc_m()).abs() <= WIDTH_TOLERANCE_M);
        let terminal = trials.last().unwrap().terminal_remaining_fraction;
        let validation = &queries[queries.len() - 2 * (VALIDATION_INTERVALS + 1)..];
        for (i, pair) in validation.chunks_exact(2).enumerate() {
            let q = terminal * i as f64 / VALIDATION_INTERVALS as f64;
            let expected = 0.2_f32 + (1.0 - 0.2_f32) * q as f32;
            assert_eq!(pair, [(0, expected), (1, expected)]);
        }
        // A straight authoring policy must not hide a rejected body query
        // between sparse fit stations during final prefix certification.
        let rejected = validation[2].1;
        let result = solve(
            &baseline,
            [0.2; 2],
            RimPolicy::StraightShoulderCorner,
            &mut |_, f| {
                if f == rejected {
                    return Err(BandError::InvalidSeed);
                }
                Ok(CrestSample {
                    position: [
                        0.3 * f as f64,
                        0.01 * (f as f64 * 20.0).sin(),
                        -0.1 * f as f64,
                    ],
                    normal: [0.0, 0.7, 0.7],
                })
            },
            &mut Vec::new(),
            &mut [0.0, 1.0],
        );
        assert!(matches!(result, Err(BandError::InvalidSeed)));
    }

    #[test]
    fn seat_validation_and_cache_identity_are_explicit() {
        for invalid in [0.0, 1.0, -0.1, f64::NAN] {
            assert!(ShoulderSeat::new(invalid).is_err());
        }
        let hash = |value| {
            let mut h = blake3::Hasher::new();
            ShoulderSeat::new(value).unwrap().hash(&mut h);
            h.finalize()
        };
        assert_eq!(hash(0.125), hash(0.125));
        assert_ne!(hash(0.125), hash(0.25));
    }
}

fn solve(
    baseline: &ShoulderBand,
    starts: [f32; 2],
    rim: RimPolicy,
    query: &mut impl FnMut(usize, f32) -> Result<CrestSample, BandError>,
    trials: &mut Vec<Trial>,
    bracket: &mut [f64; 2],
) -> Result<ShoulderBand, BandError> {
    let mut terminal = baseline.endpoint_parameter();
    for _ in 0..MAX_SOLVE_TRIALS {
        let trial = Trial::measure(terminal, *bracket, starts, baseline, rim, query);
        let fitted = trial.fitted.clone();
        let invalid_bound = trial.first_invalid_remaining_fraction.unwrap_or(terminal);
        if trial.first_invalid_remaining_fraction == Some(0.0) {
            let error = trial.failure_code.clone().unwrap_or(BandError::InvalidSeed);
            trials.push(trial);
            return Err(error);
        }
        if let Some(band) = &fitted {
            for prior in trials.iter().filter_map(|t| {
                t.fitted
                    .as_ref()
                    .map(|b| (t.terminal_remaining_fraction, b.available_arc_m()))
            }) {
                let delta = band.available_arc_m() - prior.1;
                if (terminal > prior.0 && delta < -LENGTH_ORDER_TOLERANCE_M)
                    || (terminal < prior.0 && delta > LENGTH_ORDER_TOLERANCE_M)
                {
                    trials.push(trial);
                    return Err(BandError::ArcSolveFailed);
                }
            }
        }
        trials.push(trial);
        if let Some(band) = fitted {
            let residual = band.available_arc_m() - baseline.requested_arc_m();
            if residual.abs() <= WIDTH_TOLERANCE_M {
                // Validation queries do not author vertices or refit the sparse cubic.
                for i in 0..=VALIDATION_INTERVALS {
                    let q = terminal * i as f64 / VALIDATION_INTERVALS as f64;
                    for (side, start) in starts.into_iter().enumerate() {
                        query(side, start + (1.0 - start) * q as f32)?;
                    }
                }
                return Ok(band);
            }
            bracket[usize::from(residual > 0.0)] = terminal;
        } else {
            // A rejected distal query bounds the search; it does not establish
            // that the requested width is impossible within the valid prefix.
            bracket[1] = bracket[1].min(invalid_bound);
        }
        if bracket[1] - bracket[0] <= MIN_PARAMETER_BRACKET {
            return Err(BandError::UnavailableWidth);
        }
        terminal = (bracket[0] + bracket[1]) * 0.5;
    }
    Err(BandError::ArcSolveFailed)
}
