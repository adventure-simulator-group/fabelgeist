use serde::{Deserialize, Serialize};

use crate::{DVec3, Mat3, Vec3};

/// A 3x3 matrix in double precision, mostly for rotations of things far away.
///
/// A rotation in `f32` is good to about 1e-7 radians; applied to an orbit's
/// 1.5e11 m that is 15 km out of place. Frames that turn large vectors --
/// an orbit's plane, a planet's spin -- are kept in `DMat3` and only the
/// rotation of what is near the camera goes to the GPU in single precision.
///
/// Column-major like [`Mat3`]: `columns[c][r]`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DMat3 {
    pub columns: [[f64; 3]; 3],
}

impl Default for DMat3 {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl DMat3 {
    pub const IDENTITY: Self = Self {
        columns: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
    };

    /// The matrix whose columns are `x`, `y` and `z`: for an orthonormal
    /// basis, the rotation taking the axes onto it.
    pub fn from_cols(x: DVec3, y: DVec3, z: DVec3) -> Self {
        Self {
            columns: [x.to_array(), y.to_array(), z.to_array()],
        }
    }

    pub fn col(&self, c: usize) -> DVec3 {
        DVec3::from_array(self.columns[c])
    }

    pub fn row(&self, r: usize) -> DVec3 {
        DVec3::new(self.columns[0][r], self.columns[1][r], self.columns[2][r])
    }

    /// The rotation by `angle` radians about the unit `axis`, counter-clockwise
    /// looking down the axis (Rodrigues).
    pub fn from_axis_angle(axis: DVec3, angle: f64) -> Self {
        let a = axis.normalize();
        let (s, c) = angle.sin_cos();
        let t = 1.0 - c;
        Self::from_cols(
            DVec3::new(
                t * a.x * a.x + c,
                t * a.x * a.y + s * a.z,
                t * a.x * a.z - s * a.y,
            ),
            DVec3::new(
                t * a.x * a.y - s * a.z,
                t * a.y * a.y + c,
                t * a.y * a.z + s * a.x,
            ),
            DVec3::new(
                t * a.x * a.z + s * a.y,
                t * a.y * a.z - s * a.x,
                t * a.z * a.z + c,
            ),
        )
    }

    pub fn rotation_x(angle: f64) -> Self {
        Self::from_axis_angle(DVec3::X, angle)
    }

    pub fn rotation_y(angle: f64) -> Self {
        Self::from_axis_angle(DVec3::Y, angle)
    }

    pub fn rotation_z(angle: f64) -> Self {
        Self::from_axis_angle(DVec3::Z, angle)
    }

    pub fn transpose(&self) -> Self {
        Self::from_cols(self.row(0), self.row(1), self.row(2))
    }

    pub fn mul_vec(&self, v: DVec3) -> DVec3 {
        self.col(0) * v.x + self.col(1) * v.y + self.col(2) * v.z
    }

    pub fn determinant(&self) -> f64 {
        self.col(0).dot(self.col(1).cross(self.col(2)))
    }

    /// Re-orthonormalise a rotation that has drifted after many products
    /// (Gram-Schmidt, keeping the third column's direction).
    pub fn orthonormalize(&self) -> Self {
        let z = self.col(2).normalize();
        let x = self.col(1).cross(z).normalize();
        let y = z.cross(x);
        Self::from_cols(x, y, z)
    }

    /// Rounded to single precision, for a rotation the GPU applies.
    pub fn as_mat3(&self) -> Mat3 {
        let c = |i: usize| {
            Vec3::new(
                self.columns[i][0] as f32,
                self.columns[i][1] as f32,
                self.columns[i][2] as f32,
            )
        };
        Mat3::new(c(0), c(1), c(2))
    }
}

impl std::ops::Mul for DMat3 {
    type Output = Self;
    fn mul(self, o: Self) -> Self {
        Self::from_cols(
            self.mul_vec(o.col(0)),
            self.mul_vec(o.col(1)),
            self.mul_vec(o.col(2)),
        )
    }
}

impl std::ops::Mul<DVec3> for DMat3 {
    type Output = DVec3;
    fn mul(self, v: DVec3) -> DVec3 {
        self.mul_vec(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: DVec3, b: DVec3) -> bool {
        (a - b).length() < 1e-12
    }

    #[test]
    fn a_quarter_turn_about_z_takes_x_to_y() {
        let r = DMat3::rotation_z(std::f64::consts::FRAC_PI_2);
        assert!(close(r * DVec3::X, DVec3::Y));
        assert!(close(r * DVec3::Y, -DVec3::X));
        assert!((r.determinant() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn the_transpose_undoes_a_rotation() {
        let r = DMat3::from_axis_angle(DVec3::new(1.0, 2.0, 3.0), 0.7);
        let v = DVec3::new(-4.0, 5.0, 0.25);
        assert!(close(r.transpose() * (r * v), v));
    }
}
