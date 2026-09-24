//! Dimensioned blade and receiving socket construction.
use super::*;

/// A blind tapered cavity beneath a solid transition to the blade section.
/// All heights are measured upward from the open socket rim. The neck occupies
/// the final `neck_length` of `length`; it is not added to the blade length.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpearSocket {
    pub length: Metres,
    pub neck_length: Metres,
    pub base_radius: Metres,
    pub wall: Metres,
    pub cavity_depth: Metres,
    pub insertion_depth: Metres,
    /// Radius at the blind end of the tapered receiving cavity.
    pub bore_tip_radius: Metres,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::deserialize_present"
    )]
    pub stops: Option<BasalStops>,
}

/// Bilateral plates formed continuously with a socket. The top is horizontal,
/// the ends are square, and a quadratic underside rises from the root.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BasalStops {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::deserialize_present"
    )]
    pub root_blend: Option<Metres>,
    pub span: Metres,
    pub center_height: Metres,
    pub end_height: Metres,
    pub root_height: Metres,
    pub thickness: Metres,
    /// Rotation around the socket axis; zero lies in the blade's width plane.
    pub orientation: Degrees,
}

impl SpearSocket {
    pub(crate) fn outer_barrel_radius(&self, root_radius: f64, height: f64) -> f64 {
        let progress = (height / (self.length.get() - self.neck_length.get())).clamp(0.0, 1.0);
        self.base_radius.get() + (root_radius - self.base_radius.get()) * progress
    }
    fn neck_axis_coefficients(&self, root: f64, end: f64, end_slope: f64) -> [f64; 4] {
        let start_slope =
            (root - self.base_radius.get()) / (self.length.get() - self.neck_length.get());
        let start_tangent = self.neck_length.get() * start_slope;
        let end_tangent = self.neck_length.get() * end_slope;
        [
            2.0 * root + start_tangent - 2.0 * end + end_tangent,
            -3.0 * root - 2.0 * start_tangent + 3.0 * end - end_tangent,
            start_tangent,
            root,
        ]
    }
    pub(crate) fn neck_axis(&self, root: f64, end: f64, end_slope: f64, t: f64) -> f64 {
        let [a, b, c, d] = self.neck_axis_coefficients(root, end, end_slope);
        ((a * t + b) * t + c) * t + d
    }
    /// Exact cubic extrema prevent a tangent from turning a neck axis inside out
    /// between sampled rings. This is independent of rendering detail.
    pub(crate) fn minimum_neck_axis(&self, root: f64, end: f64, end_slope: f64) -> f64 {
        let [a, b, c, _] = self.neck_axis_coefficients(root, end, end_slope);
        let mut minimum = root.min(end);
        let mut sample = |t: f64| {
            if t > 0.0 && t < 1.0 {
                minimum = minimum.min(self.neck_axis(root, end, end_slope, t));
            }
        };
        if a == 0.0 {
            if b != 0.0 {
                sample(-c / (2.0 * b));
            }
        } else {
            let discriminant = b * b - 3.0 * a * c;
            if discriminant >= 0.0 {
                let root = discriminant.sqrt();
                sample((-b - root) / (3.0 * a));
                sample((-b + root) / (3.0 * a));
            }
        }
        minimum
    }
    pub(crate) fn bore_radius(&self, height: f64) -> f64 {
        let bottom = self.base_radius.get() - self.wall.get();
        bottom + (self.bore_tip_radius.get() - bottom) * height / self.cavity_depth.get()
    }
}
