//! Quadratic bending geometry and its per-constraint GPU record.
use fabelgeist_math::Vec3;

/// Hinge endpoints followed by the two opposite vertices. The order must agree
/// with the particle record and is retained through weights and rest capture.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BendPoints([Vec3; 4]);
impl From<[Vec3; 4]> for BendPoints {
    fn from(points: [Vec3; 4]) -> Self {
        Self(points)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BendGeometryError {
    CollapsedSpokes,
    CollinearSpokes,
    VanishingMinors,
    NegligibleArea,
}
impl std::fmt::Display for BendGeometryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::CollapsedSpokes => "bending hinge spokes are collapsed",
            Self::CollinearSpokes => "bending hinge spokes are collinear",
            Self::VanishingMinors => "bending affine minors vanish",
            Self::NegligibleArea => "bending hinge area is negligible",
        })
    }
}
impl std::error::Error for BendGeometryError {}

/// The affine weights the quadratic bending constraint is built on.
///
/// Four points that lie in a plane always admit exactly one dependency
/// `sum(k_i) = 0` with `sum(k_i * x_i) = 0`. That combination is unchanged by
/// any affine map of the four points, so it measures departure from flatness
/// and nothing else -- which is precisely bending.
///
/// Found as the null vector of
///
/// ```text
/// [  1    1    1    1  ]
/// [ u0   u1   u2   u3  ]
/// [ v0   v1   v2   v3  ]
/// ```
///
/// where `u` and `v` are coordinates in the points' own plane, via the four
/// signed 3x3 minors.
///
/// The result is normalised and then scaled by the hinge's own length scale,
/// which is what puts the `Fabric` bend compliances into a sensible band --
/// see the presets, which were calibrated against this scaling.
///
/// It does **not** make bending fully resolution-independent. Sweeping the
/// exponent on that length scale (the `bend_scaling_grid` test in
/// `fabelgeist-cloth`, which is `#[ignore]`d and meant to be run by hand) shows
/// the compliance at which a flap starts to droop still shifts by something
/// like a decade when the target edge length is halved, and no exponent tried
/// removed that. Part of the drift is not stiffness at all: a flap only a few
/// hinges wide cannot represent much curvature whatever its compliance, so the
/// coarse end of the sweep is measuring the mesh rather than the material.
/// Expect to retune `bend_compliance` after a large change in resolution.
///
/// `points` is ordered hinge, hinge, wing, wing -- the order
/// [`crate::BendQuad::particles`] produces.
///
/// Degenerate input -- three collinear points, a collapsed triangle -- has a
/// larger null space and no single answer, so admission returns a stage-specific
/// `BendGeometryError` and the caller leaves that hinge unconstrained.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BendWeights([f32; 4]);
impl BendWeights {
    const MINIMUM_SPOKE_LENGTH_SQUARED: f32 = 1e-24;
    const MINIMUM_AFFINE_NORM: f32 = 1e-12;
    const MINIMUM_HINGE_SCALE: f32 = 1e-9;

    pub fn for_points(points: BendPoints) -> Result<Self, BendGeometryError> {
        // A basis for the plane the four points lie in. The longest edge from
        // point 0 gives the most stable first axis.
        let spokes = [
            points.0[1] - points.0[0],
            points.0[2] - points.0[0],
            points.0[3] - points.0[0],
        ];
        let first = spokes
            .iter()
            .copied()
            .max_by(|a: &Vec3, b: &Vec3| -> std::cmp::Ordering {
                a.length_squared().total_cmp(&b.length_squared())
            })
            .expect("three fixed hinge spokes");
        if first.length_squared() < Self::MINIMUM_SPOKE_LENGTH_SQUARED {
            return Err(BendGeometryError::CollapsedSpokes);
        }
        let u = first.normalize();

        // The second axis is whichever spoke is furthest off the first.
        let mut best = Vec3::default();
        let mut best_length = 0.0f32;
        for spoke in spokes {
            let perpendicular = spoke - u * spoke.dot(u);
            if perpendicular.length_squared() > best_length {
                best_length = perpendicular.length_squared();
                best = perpendicular;
            }
        }
        if best_length < Self::MINIMUM_SPOKE_LENGTH_SQUARED {
            return Err(BendGeometryError::CollinearSpokes);
        }
        let v = best.normalize();

        let mut coordinates = BendCoordinates([(0.0, 0.0); 4]);
        for (coordinate, point) in coordinates.0.iter_mut().zip(points.0) {
            let local = point - points.0[0];
            *coordinate = (local.dot(u), local.dot(v));
        }
        let weights = [
            -coordinates.minor(BendCorner::FirstHinge).0,
            coordinates.minor(BendCorner::SecondHinge).0,
            -coordinates.minor(BendCorner::FirstWing).0,
            coordinates.minor(BendCorner::SecondWing).0,
        ];

        let mut norm = 0.0f32;
        for weight in weights {
            norm += weight * weight;
        }
        let norm = norm.sqrt();
        if norm < Self::MINIMUM_AFFINE_NORM {
            return Err(BendGeometryError::VanishingMinors);
        }

        // The local length scale: the root of the two triangles' combined area.
        // More robust than any single edge when the triangles are not equilateral.
        let hinge = (points.0[0], points.0[1]);
        let first_area = (hinge.1 - hinge.0).cross(points.0[2] - hinge.0).length() * 0.5;
        let second_area = (hinge.1 - hinge.0).cross(points.0[3] - hinge.0).length() * 0.5;
        let scale = (first_area + second_area).sqrt();
        if scale < Self::MINIMUM_HINGE_SCALE {
            return Err(BendGeometryError::NegligibleArea);
        }

        let mut scaled = [0.0; 4];
        for (out, weight) in scaled.iter_mut().zip(weights) {
            *out = weight / norm * scale;
        }
        Ok(Self(scaled))
    }
    /// Capture the scaled affine bending measure in this ordered geometry.
    /// The original zero-start, ascending fold and Euclidean norm are retained.
    pub fn measure(self, points: BendPoints) -> BendRestMeasure {
        let mut sum = Vec3::default();
        for (point, weight) in points.0.iter().zip(self.0) {
            sum += *point * weight;
        }
        BendRestMeasure(sum.length())
    }
    pub fn observed_rest(self, points: BendPoints) -> BendRecord {
        BendRecord {
            words: [
                self.0[0],
                self.0[1],
                self.0[2],
                self.0[3],
                self.measure(points).0,
                0.0,
                0.0,
                0.0,
            ],
        }
    }
    /// A sewn hinge's rest target is flat, independent of the placed seam gap.
    pub fn flat_rest(self) -> BendRecord {
        BendRecord {
            words: [
                self.0[0], self.0[1], self.0[2], self.0[3], 0.0, 0.0, 0.0, 0.0,
            ],
        }
    }
}

/// The magnitude of the length-scaled affine combination, distinct from an
/// edge length or compliance. It remains nominal through rest capture.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct BendRestMeasure(f32);

/// One immutable GPU record: four coefficients, rest measure, three zero words.
/// Its private representation is serialized only by the native upload adapter.
///
/// ```compile_fail
/// use fabelgeist_shell::BendWeights;
/// use fabelgeist_xpbd::ConstraintSet;
/// use fabelgeist_gpu::prelude::WgpuContext;
/// fn upload(set: &mut ConstraintSet, context: &WgpuContext, weights: &[BendWeights]) {
///     set.attach(context, "weights", weights).unwrap();
/// }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::NoUninit)]
#[repr(C)]
pub struct BendRecord {
    words: [f32; 8],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BendRecordValidity {
    Finite,
    NonFinite,
}
impl BendRecord {
    /// A seam hinge held slack during closure; all eight words are zero.
    pub fn slack() -> Self {
        Self { words: [0.0; 8] }
    }
    pub fn validity(self) -> BendRecordValidity {
        for value in self.words {
            if !value.is_finite() {
                return BendRecordValidity::NonFinite;
            }
        }
        BendRecordValidity::Finite
    }
}

#[derive(Clone, Copy)]
enum BendCorner {
    FirstHinge,
    SecondHinge,
    FirstWing,
    SecondWing,
}
struct BendCoordinates([(f32, f32); 4]);
struct AffineMinor(f32);
impl BendCoordinates {
    fn minor(&self, omitted: BendCorner) -> AffineMinor {
        let [a, b, c] = match omitted {
            BendCorner::FirstHinge => [self.0[1], self.0[2], self.0[3]],
            BendCorner::SecondHinge => [self.0[0], self.0[2], self.0[3]],
            BendCorner::FirstWing => [self.0[0], self.0[1], self.0[3]],
            BendCorner::SecondWing => [self.0[0], self.0[1], self.0[2]],
        };
        let (ua, va) = a;
        let (ub, vb) = b;
        let (uc, vc) = c;
        AffineMinor(ua * (vb - vc) - va * (ub - uc) + (ub * vc - vb * uc))
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod upload_tests;
