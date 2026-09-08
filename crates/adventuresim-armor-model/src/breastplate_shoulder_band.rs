//! A shoulder rim authored by physical arc width on a sparse anatomical crest.

use crate::breastplate_crest_curve::{Cubic, dot, normalized};
use crate::breastplate_crest_policy::CrestQueryPolicy;
use crate::breastplate_topology::{BreastplateBoundaryEdge, SemanticBoundaryRange};
use serde::Serialize;
use std::collections::BTreeMap;

const POLICY_VERSION: &str = "measured-shoulder-band-cubic-neck-tangent-v2";
const POLICY_ENV: &str = "BREASTPLATE_DIAGNOSTIC_SHOULDER_BAND_FRACTION";
const JOINT_POLICY_ENV: &str = "BREASTPLATE_DIAGNOSTIC_JOINT_PROFILES";
const PROFILE_OUTPUT_ENV: &str = "BREASTPLATE_ANGULAR_DUMP";
pub(crate) const CREST_SEED_COUNT: usize = 9;
const ARC_INTERVALS: usize = 1024;
const MIN_GRAPH_NORMAL: f64 = 0.05;
const MAX_ARC_REFINEMENT_ERROR_M: f64 = 1e-6;

#[derive(Clone, Debug, Serialize, thiserror::Error)]
pub(crate) enum BandError {
    #[error("shoulder band fraction must be finite and strictly between zero and one")]
    InvalidFraction,
    #[error("shoulder band diagnostic requires joint profiles")]
    RequiresJointProfiles,
    #[error("superior-first crest query requires an enabled shoulder-band fraction")]
    RequiresShoulderBand,
    #[error("posterior shoulder seat requires superior-first shoulder band")]
    RequiresSuperiorFirst,
    #[error("simple rim diagnostic requires a seated shoulder band")]
    RequiresSeatedBand,
    #[error("posterior shoulder seed certificate failed")]
    InvalidSeed,
    #[error("posterior shoulder sparse arc solve failed to converge monotonically")]
    ArcSolveFailed,
    #[error("shoulder crest fit is nonfinite, folded, or degenerate")]
    InvalidCurve,
    #[error("shoulder crest tangent plane cannot be represented as an anterior graph")]
    InvalidNormal,
    #[error("requested shoulder band exceeds the available anatomical crest arc")]
    UnavailableWidth,
    #[error("shoulder crest measurement failed at side {side}, seed fraction {fraction}")]
    QueryFailed { side: usize, fraction: f32 },
    #[error("shoulder band diagnostic could not be written")]
    DiagnosticWrite,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) struct BandFraction(f64);

impl BandFraction {
    pub(crate) fn new(value: f64) -> Result<Self, BandError> {
        if value.is_finite() && value > 0.0 && value < 1.0 {
            Ok(Self(value))
        } else {
            Err(BandError::InvalidFraction)
        }
    }

    pub(crate) fn from_environment() -> Result<Option<Self>, BandError> {
        let Some(raw) = std::env::var_os(POLICY_ENV) else {
            return Ok(None);
        };
        if std::env::var_os(JOINT_POLICY_ENV).is_none() {
            return Err(BandError::RequiresJointProfiles);
        }
        let value = raw
            .to_str()
            .and_then(|s| s.parse().ok())
            .ok_or(BandError::InvalidFraction)?;
        Self::new(value).map(Some)
    }

    pub(crate) fn hash(self, hash: &mut blake3::Hasher) {
        hash.update(POLICY_VERSION.as_bytes());
        hash.update(&self.0.to_bits().to_le_bytes());
        hash.update(&(CREST_SEED_COUNT as u64).to_le_bytes());
        hash.update(&(ARC_INTERVALS as u64).to_le_bytes());
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) struct CrestSample {
    pub(crate) position: [f64; 3],
    pub(crate) normal: [f64; 3],
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ShoulderBand {
    policy: &'static str,
    query_policy: CrestQueryPolicy,
    sample_space: &'static str,
    fraction: BandFraction,
    arc_intervals: usize,
    full_raw: [CrestSample; CREST_SEED_COUNT],
    available_raw: [CrestSample; CREST_SEED_COUNT],
    full_curve: Cubic,
    curve: Cubic,
    normal_curve: Cubic,
    full_arc_m: f64,
    available_arc_m: f64,
    requested_arc_m: f64,
    realized_arc_m: f64,
    endpoint_parameter: f64,
    arc_refinement_error_m: f64,
    #[serde(skip)]
    arc: Vec<f64>,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) struct BandStation {
    pub(crate) position: [f64; 3],
    pub(crate) normal: [f64; 3],
    pub(crate) tangent: [f64; 3],
    curve_parameter: f64,
}

impl BandStation {
    pub(crate) fn mirrored(self) -> Self {
        Self {
            position: [-self.position[0], self.position[1], self.position[2]],
            normal: [-self.normal[0], self.normal[1], self.normal[2]],
            // Left shoulder traverses outer-to-inner in boundary order.
            tangent: [self.tangent[0], -self.tangent[1], -self.tangent[2]],
            ..self
        }
    }
}

impl ShoulderBand {
    pub(crate) fn from_queries(
        fraction: BandFraction,
        query_policy: CrestQueryPolicy,
        inner_fractions: [f32; 2],
        inner_sample: CrestSample,
        mut query: impl FnMut(usize, f32) -> Result<CrestSample, BandError>,
    ) -> Result<Self, BandError> {
        let mut collect = |starts: [f32; 2]| -> Result<[CrestSample; CREST_SEED_COUNT], BandError> {
            let mut samples = [CrestSample {
                position: [0.0; 3],
                normal: [0.0; 3],
            }; CREST_SEED_COUNT];
            for (i, sample) in samples.iter_mut().enumerate() {
                let t = i as f32 / (CREST_SEED_COUNT - 1) as f32;
                for (side, start) in starts.into_iter().enumerate() {
                    let measured = query(side, start + (1.0 - start) * t)?;
                    for axis in 0..3 {
                        sample.position[axis] += measured.position[axis] * 0.5;
                        sample.normal[axis] += measured.normal[axis] * 0.5;
                    }
                }
                sample.normal = normalized(sample.normal)?;
            }
            Ok(samples)
        };
        let full_raw = collect([0.0; 2])?;
        let mut available_raw = collect(inner_fractions)?;
        available_raw[0] = inner_sample;
        Self::write_dump(
            "shoulder-band-inputs",
            &serde_json::json!({
                "policy": POLICY_VERSION, "fraction": fraction,
                "query_policy": query_policy,
                "sample_space": "normal-padded symmetric crest in torso-local metres",
                "full_raw": full_raw, "available_raw": available_raw,
                "inner_seed_fractions": inner_fractions,
            }),
        )?;
        let mut band = Self::fit(fraction, full_raw, available_raw)?;
        band.query_policy = query_policy;
        Self::write_dump("shoulder-band", &band)?;
        Ok(band)
    }

    pub(crate) fn write_dump(suffix: &str, value: &impl Serialize) -> Result<(), BandError> {
        if let Some(path) = std::env::var_os(PROFILE_OUTPUT_ENV) {
            let path = std::path::PathBuf::from(path).with_extension(format!("{suffix}.json"));
            let bytes = serde_json::to_vec_pretty(value).map_err(|_| BandError::DiagnosticWrite)?;
            std::fs::write(path, bytes).map_err(|_| BandError::DiagnosticWrite)?;
        }
        Ok(())
    }

    pub(crate) fn stations(
        &self,
        layout: &[SemanticBoundaryRange],
    ) -> Result<BTreeMap<usize, BandStation>, BandError> {
        let boundary_count = layout.iter().map(|r| r.segments).sum::<usize>();
        let mut stations = BTreeMap::new();
        let mut roles = Vec::new();
        for edge in [
            BreastplateBoundaryEdge::RightShoulder,
            BreastplateBoundaryEdge::LeftShoulder,
        ] {
            let range = layout
                .iter()
                .find(|r| r.edge == edge)
                .ok_or(BandError::InvalidCurve)?;
            let left = edge == BreastplateBoundaryEdge::LeftShoulder;
            for offset in 0..=range.segments {
                let fraction = offset as f64 / range.segments as f64;
                let sample = self.sample(if left { 1.0 - fraction } else { fraction })?;
                let sample = if left { sample.mirrored() } else { sample };
                stations.insert((range.start + offset) % boundary_count, sample);
            }
            roles.push(serde_json::json!({
                "edge": format!("{edge:?}"), "start": range.start, "segments": range.segments,
                "includes_shared_endpoint": true,
            }));
        }
        Self::write_dump(
            "shoulder-band-stations",
            &serde_json::json!({
                "roles": roles, "boundary_count": boundary_count, "stations": stations,
            }),
        )?;
        Ok(stations)
    }

    pub(crate) fn fit(
        fraction: BandFraction,
        full_raw: [CrestSample; CREST_SEED_COUNT],
        available_raw: [CrestSample; CREST_SEED_COUNT],
    ) -> Result<Self, BandError> {
        let full_curve = Cubic::fit(&full_raw.map(|s| s.position))?;
        let curve = Cubic::fit(&available_raw.map(|s| s.position))?;
        full_curve.validate_lateral_monotonicity()?;
        curve.validate_lateral_monotonicity()?;
        let normal_curve = Cubic::fit(&available_raw.map(|s| s.normal))?;
        let full_arc_m = *Self::arc_table(full_curve, ARC_INTERVALS).last().unwrap();
        let arc = Self::arc_table(curve, ARC_INTERVALS);
        let available_arc_m = *arc.last().unwrap();
        let arc_refinement_error_m = (Self::arc_table(full_curve, ARC_INTERVALS * 2)
            .last()
            .unwrap()
            - full_arc_m)
            .abs()
            .max(
                (Self::arc_table(curve, ARC_INTERVALS * 2).last().unwrap() - available_arc_m).abs(),
            );
        if arc_refinement_error_m > MAX_ARC_REFINEMENT_ERROR_M {
            return Err(BandError::InvalidCurve);
        }
        let requested_arc_m = fraction.0 * full_arc_m;
        if requested_arc_m > available_arc_m {
            return Err(BandError::UnavailableWidth);
        }
        let mut result = Self {
            policy: POLICY_VERSION,
            query_policy: CrestQueryPolicy::AnteriorFirst,
            sample_space: "normal-padded symmetric crest in torso-local metres",
            fraction,
            arc_intervals: ARC_INTERVALS,
            full_raw,
            available_raw,
            full_curve,
            curve,
            normal_curve,
            full_arc_m,
            available_arc_m,
            requested_arc_m,
            realized_arc_m: requested_arc_m,
            endpoint_parameter: 0.0,
            arc_refinement_error_m,
            arc,
        };
        result.endpoint_parameter = result.parameter_at_arc(requested_arc_m);
        for i in 0..=ARC_INTERVALS {
            result.sample(i as f64 / ARC_INTERVALS as f64)?;
        }
        Ok(result)
    }

    fn arc_table(curve: Cubic, intervals: usize) -> Vec<f64> {
        let mut arc = vec![0.0];
        let mut previous = curve.at(0.0);
        for i in 1..=intervals {
            let next = curve.at(i as f64 / intervals as f64);
            let delta = std::array::from_fn(|axis| next[axis] - previous[axis]);
            arc.push(arc.last().unwrap() + dot(delta, delta).sqrt());
            previous = next;
        }
        arc
    }

    fn parameter_at_arc(&self, distance: f64) -> f64 {
        if distance == 0.0 {
            return 0.0;
        }
        let hi = self
            .arc
            .partition_point(|s| *s < distance)
            .min(ARC_INTERVALS);
        let fraction = (distance - self.arc[hi - 1]) / (self.arc[hi] - self.arc[hi - 1]);
        (hi as f64 - 1.0 + fraction) / ARC_INTERVALS as f64
    }

    pub(crate) fn sample(&self, arc_fraction: f64) -> Result<BandStation, BandError> {
        if !arc_fraction.is_finite() || !(0.0..=1.0).contains(&arc_fraction) {
            return Err(BandError::InvalidFraction);
        }
        let t = self.parameter_at_arc(self.realized_arc_m * arc_fraction);
        let tangent = normalized(self.curve.tangent(t))?;
        let measured = self.normal_curve.at(t);
        let along = dot(measured, tangent);
        let normal = normalized(std::array::from_fn(|axis| {
            measured[axis] - tangent[axis] * along
        }))?;
        if normal[2] < MIN_GRAPH_NORMAL {
            return Err(BandError::InvalidNormal);
        }
        Ok(BandStation {
            position: self.curve.at(t),
            normal,
            tangent,
            curve_parameter: t,
        })
    }

    pub(crate) fn requested_arc_m(&self) -> f64 {
        self.requested_arc_m
    }

    pub(crate) fn available_arc_m(&self) -> f64 {
        self.available_arc_m
    }

    pub(crate) fn endpoint_parameter(&self) -> f64 {
        self.endpoint_parameter
    }

    /// Fit only the seated material interval. Its whole realized arc is sampled,
    /// including t=1 even when the width solve lands slightly below its request.
    pub(crate) fn fitted_used_interval(
        &self,
        raw: [CrestSample; CREST_SEED_COUNT],
    ) -> Result<Self, BandError> {
        let curve = Cubic::fit(&raw.map(|s| s.position))?;
        curve.validate_exact_lateral_monotonicity()?;
        let normal_curve = Cubic::fit(&raw.map(|s| s.normal))?;
        let arc = Self::arc_table(curve, ARC_INTERVALS);
        let available_arc_m = *arc.last().unwrap();
        let arc_refinement_error_m =
            (Self::arc_table(curve, ARC_INTERVALS * 2).last().unwrap() - available_arc_m).abs();
        if arc_refinement_error_m > MAX_ARC_REFINEMENT_ERROR_M {
            return Err(BandError::InvalidCurve);
        }
        let result = Self {
            policy: "posterior-seated-used-interval-v1",
            available_raw: raw,
            curve,
            normal_curve,
            arc,
            available_arc_m,
            realized_arc_m: available_arc_m,
            endpoint_parameter: 1.0,
            arc_refinement_error_m,
            ..self.clone()
        };
        for i in 0..=ARC_INTERVALS {
            result.sample(i as f64 / ARC_INTERVALS as f64)?;
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn realized_short_interval_samples_exact_endpoint_without_extrapolation() {
        let baseline = ShoulderBand::fit(
            BandFraction::new(0.3).unwrap(),
            samples(0.0, 1.0, 1.0, false),
            samples(0.2, 1.0, 1.0, false),
        )
        .unwrap();
        let raw = samples(0.2, 0.49998, 1.0, false);
        let used = baseline.fitted_used_interval(raw).unwrap();
        assert!(used.available_arc_m < used.requested_arc_m);
        assert_eq!(used.sample(1.0).unwrap().curve_parameter, 1.0);
        assert_eq!(
            used.sample(1.0).unwrap().position,
            raw[CREST_SEED_COUNT - 1].position
        );
    }

    fn samples(start: f64, end: f64, scale: f64, curved: bool) -> [CrestSample; CREST_SEED_COUNT] {
        std::array::from_fn(|i| {
            let x = start + (end - start) * i as f64 / (CREST_SEED_COUNT - 1) as f64;
            CrestSample {
                position: [x * scale, if curved { x * x * scale } else { 0.0 }, 0.0],
                normal: [0.0, 0.0, 1.0],
            }
        })
    }

    #[test]
    fn physical_width_preserves_inner_endpoint_and_scales_with_wearer() {
        for curved in [false, true] {
            let make = |scale| {
                ShoulderBand::fit(
                    BandFraction::new(0.3).unwrap(),
                    samples(0.0, 1.0, scale, curved),
                    samples(0.2, 1.0, scale, curved),
                )
                .unwrap()
            };
            let a = make(1.0);
            let b = make(2.0);
            assert_eq!(a.sample(0.0).unwrap().position, a.available_raw[0].position);
            assert!((a.requested_arc_m - 0.3 * a.full_arc_m).abs() < 1e-12);
            for t in [0.0, 0.2, 0.7, 1.0] {
                let p = a.sample(t).unwrap();
                let q = b.sample(t).unwrap();
                for axis in 0..3 {
                    assert!((2.0 * p.position[axis] - q.position[axis]).abs() < 1e-12);
                }
                assert!(dot(p.normal, p.tangent).abs() < 1e-12);
                assert_eq!(p.mirrored().mirrored().position, p.position);
            }
            if !curved {
                assert!((a.sample(1.0).unwrap().position[0] - 0.5).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn width_control_is_monotone_and_rejects_unavailable_arc() {
        let make = |fraction| {
            ShoulderBand::fit(
                BandFraction::new(fraction).unwrap(),
                samples(0.0, 1.0, 1.0, true),
                samples(0.2, 1.0, 1.0, true),
            )
        };
        let a = make(0.3).unwrap();
        let b = make(0.45).unwrap();
        assert!(b.sample(1.0).unwrap().position[0] > a.sample(1.0).unwrap().position[0]);
        assert!(matches!(make(0.99), Err(BandError::UnavailableWidth)));
        for value in [f64::NAN, f64::INFINITY, 0.0, -0.1, 1.0] {
            assert!(BandFraction::new(value).is_err());
        }
    }

    #[test]
    fn folded_crest_and_grazing_normals_fail_without_fallback() {
        let full = samples(0.0, 1.0, 1.0, false);
        let mut bad = samples(0.2, 1.0, 1.0, false);
        bad[4].position[0] = -20.0;
        assert!(matches!(
            ShoulderBand::fit(BandFraction::new(0.3).unwrap(), full, bad),
            Err(BandError::InvalidCurve)
        ));
        let bad = samples(0.2, 1.0, 1.0, false).map(|s| CrestSample {
            normal: [0.0, 1.0, 0.0],
            ..s
        });
        assert!(matches!(
            ShoulderBand::fit(BandFraction::new(0.3).unwrap(), full, bad),
            Err(BandError::InvalidNormal)
        ));
    }

    #[test]
    fn policy_hash_separates_widths_and_is_repeatable() {
        let hash = |fraction| {
            let mut hash = blake3::Hasher::new();
            BandFraction::new(fraction).unwrap().hash(&mut hash);
            hash.finalize()
        };
        assert_eq!(hash(0.3), hash(0.3));
        assert_ne!(hash(0.3), hash(0.45));
    }

    #[test]
    fn semantic_stations_include_both_shared_endpoints_and_mirror_traversal() {
        use crate::breastplate_topology::CanonicalBreastplateTopology;
        let band = ShoulderBand::fit(
            BandFraction::new(0.3).unwrap(),
            samples(0.0, 1.0, 1.0, true),
            samples(0.2, 1.0, 1.0, true),
        )
        .unwrap();
        let layout = CanonicalBreastplateTopology::semantic_layout();
        let stations = band.stations(&layout).unwrap();
        let right = layout
            .iter()
            .find(|r| r.edge == BreastplateBoundaryEdge::RightShoulder)
            .unwrap();
        let left = layout
            .iter()
            .find(|r| r.edge == BreastplateBoundaryEdge::LeftShoulder)
            .unwrap();
        let count = layout.iter().map(|r| r.segments).sum::<usize>();
        assert_eq!(stations.len(), 2 * (right.segments + 1));
        for i in 0..=right.segments {
            let a = stations[&(right.start + i)];
            let b = stations[&((left.start + left.segments - i) % count)];
            for axis in 0..3 {
                assert!((a.mirrored().position[axis] - b.position[axis]).abs() < 1e-12);
                assert!((a.mirrored().normal[axis] - b.normal[axis]).abs() < 1e-12);
                assert!((a.mirrored().tangent[axis] - b.tangent[axis]).abs() < 1e-12);
            }
        }
    }
}
