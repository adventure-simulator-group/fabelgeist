use serde::{Deserialize, Serialize};

use super::Vec3;

/// A vector in double precision, for positions too far apart for `f32`.
///
/// An `f32` keeps 24 bits of mantissa: a metre's resolution runs out about
/// 16 000 km from the origin, a millimetre's at 16 km. A planet's surface is
/// 6 400 km from its centre and an orbit is 150 million km from its star, so
/// world positions are kept in `DVec3` on the host and only the difference to
/// an origin nearby -- the camera, a tile's centre -- is handed to the GPU,
/// in `f32`, through [`DVec3::relative_to`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[repr(C)]
pub struct DVec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl DVec3 {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);
    pub const X: Self = Self::new(1.0, 0.0, 0.0);
    pub const Y: Self = Self::new(0.0, 1.0, 0.0);
    pub const Z: Self = Self::new(0.0, 0.0, 1.0);

    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub const fn splat(value: f64) -> Self {
        Self::new(value, value, value)
    }

    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    pub fn length_squared(self) -> f64 {
        self.dot(self)
    }

    pub fn length(self) -> f64 {
        self.length_squared().sqrt()
    }

    /// The unit vector along `self`; zero stays zero.
    pub fn normalize(self) -> Self {
        let length = self.length();
        if length > 0.0 { self / length } else { self }
    }

    pub fn distance(self, other: Self) -> f64 {
        (self - other).length()
    }

    pub fn lerp(self, other: Self, t: f64) -> Self {
        self + (other - self) * t
    }

    pub fn min(self, other: Self) -> Self {
        Self::new(
            self.x.min(other.x),
            self.y.min(other.y),
            self.z.min(other.z),
        )
    }

    pub fn max(self, other: Self) -> Self {
        Self::new(
            self.x.max(other.x),
            self.y.max(other.y),
            self.z.max(other.z),
        )
    }

    pub fn abs(self) -> Self {
        Self::new(self.x.abs(), self.y.abs(), self.z.abs())
    }

    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }

    pub fn to_array(self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }

    pub fn from_array(values: [f64; 3]) -> Self {
        Self::new(values[0], values[1], values[2])
    }

    /// `self - origin` in single precision: the only way a world position
    /// should reach a shader. The subtraction happens in `f64`, so the result
    /// is exact to `f32`'s precision *at its own size* -- a point a metre from
    /// the camera keeps micrometres, wherever in the universe the two are.
    pub fn relative_to(self, origin: Self) -> Vec3 {
        (self - origin).as_vec3()
    }

    /// Rounded to single precision as it stands. Only for directions and for
    /// positions already near the origin; see [`Self::relative_to`].
    pub fn as_vec3(self) -> Vec3 {
        Vec3::new(self.x as f32, self.y as f32, self.z as f32)
    }
}

impl From<Vec3> for DVec3 {
    fn from(v: Vec3) -> Self {
        Self::new(v.x as f64, v.y as f64, v.z as f64)
    }
}

impl std::ops::Add for DVec3 {
    type Output = Self;
    fn add(self, o: Self) -> Self {
        Self::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }
}

impl std::ops::Sub for DVec3 {
    type Output = Self;
    fn sub(self, o: Self) -> Self {
        Self::new(self.x - o.x, self.y - o.y, self.z - o.z)
    }
}

impl std::ops::Mul<f64> for DVec3 {
    type Output = Self;
    fn mul(self, s: f64) -> Self {
        Self::new(self.x * s, self.y * s, self.z * s)
    }
}

impl std::ops::Mul<DVec3> for f64 {
    type Output = DVec3;
    fn mul(self, v: DVec3) -> DVec3 {
        v * self
    }
}

/// Component-wise.
impl std::ops::Mul for DVec3 {
    type Output = Self;
    fn mul(self, o: Self) -> Self {
        Self::new(self.x * o.x, self.y * o.y, self.z * o.z)
    }
}

impl std::ops::Div<f64> for DVec3 {
    type Output = Self;
    fn div(self, s: f64) -> Self {
        Self::new(self.x / s, self.y / s, self.z / s)
    }
}

impl std::ops::Neg for DVec3 {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }
}

impl std::ops::AddAssign for DVec3 {
    fn add_assign(&mut self, o: Self) {
        *self = *self + o;
    }
}

impl std::ops::SubAssign for DVec3 {
    fn sub_assign(&mut self, o: Self) {
        *self = *self - o;
    }
}

impl std::ops::MulAssign<f64> for DVec3 {
    fn mul_assign(&mut self, s: f64) {
        *self = *self * s;
    }
}

impl std::fmt::Display for DVec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_positions_keep_their_precision_far_from_the_origin() {
        // A camera on an orbit at 1 AU, a point 1.5 mm in front of it.
        let camera = DVec3::new(1.495_978_707e11, 0.0, 0.0);
        let point = camera + DVec3::new(0.0, 0.0, -1.5e-3);
        let local = point.relative_to(camera);
        assert!((local.z + 1.5e-3).abs() < 1e-9, "{local}");
        // The same subtraction in f32 loses it entirely.
        let lost = (
            point.x as f32 - camera.x as f32,
            point.z as f32 - camera.z as f32,
        );
        assert_eq!(lost.0, 0.0);
    }

    #[test]
    fn cross_follows_the_right_hand() {
        assert_eq!(DVec3::X.cross(DVec3::Y), DVec3::Z);
        assert_eq!(DVec3::Y.cross(DVec3::Z), DVec3::X);
    }
}
