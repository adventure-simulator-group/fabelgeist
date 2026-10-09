//! Small linear-algebra helpers.
//!
//! The reference implementation leans on `numpy` and
//! `scipy.spatial.transform.Rotation`; this module provides just the pieces
//! GarmentCode actually uses, with the same conventions.

pub type V2 = [f64; 2];
pub type V3 = [f64; 3];

pub const TOL: f64 = 1e-4;

// ----- 2D -----

pub fn sub2(a: V2, b: V2) -> V2 {
    [a[0] - b[0], a[1] - b[1]]
}

pub fn add2(a: V2, b: V2) -> V2 {
    [a[0] + b[0], a[1] + b[1]]
}

pub fn scale2(a: V2, s: f64) -> V2 {
    [a[0] * s, a[1] * s]
}

pub fn dot2(a: V2, b: V2) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}

/// Z component of the 3D cross product of two 2D vectors.
pub fn cross2(a: V2, b: V2) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}

pub fn norm2(a: V2) -> f64 {
    dot2(a, a).sqrt()
}

pub fn dist2(a: V2, b: V2) -> f64 {
    norm2(sub2(a, b))
}

pub fn normalize2(a: V2) -> V2 {
    scale2(a, 1.0 / norm2(a))
}

// ----- 3D -----

pub fn sub3(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub fn add3(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

pub fn scale3(a: V3, s: f64) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}

pub fn dot3(a: V3, b: V3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn cross3(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

pub fn norm3(a: V3) -> f64 {
    dot3(a, a).sqrt()
}

pub fn min3(a: V3, b: V3) -> V3 {
    [a[0].min(b[0]), a[1].min(b[1]), a[2].min(b[2])]
}

pub fn max3(a: V3, b: V3) -> V3 {
    [a[0].max(b[0]), a[1].max(b[1]), a[2].max(b[2])]
}

// ----- generic utils (pygarment.garmentcode.utils) -----

/// Compare two floats with a tolerance -- `utils.close_enough`.
pub fn close_enough(f1: f64, f2: f64, tol: f64) -> bool {
    (f1 - f2).abs() < tol
}

pub fn close_to_zero(f: f64) -> bool {
    f.abs() < TOL
}

/// Signed angle between two 2D vectors -- `utils.vector_angle`.
///
/// NOTE: the sign is only applied when the cross product is meaningfully
/// non-zero, matching the reference (which leaves the angle unsigned for
/// near-parallel vectors).
pub fn vector_angle(v1: V2, v2: V2) -> f64 {
    let cos = dot2(v1, v2) / (norm2(v1) * norm2(v2));
    let mut angle = cos.acos();
    let cross = cross2(v1, v2);
    if cross.abs() > 1e-5 {
        angle *= cross.signum();
    }
    angle
}

/// `pattern.utils.vector_angle` -- same as above but clamps `cos` first.
pub fn vector_angle_clamped(v1: V2, v2: V2) -> f64 {
    let cos = (dot2(v1, v2) / (norm2(v1) * norm2(v2))).clamp(-1.0, 1.0);
    let mut angle = cos.acos();
    let cross = cross2(v1, v2);
    if cross.abs() > 1e-5 {
        angle *= cross.signum();
    }
    angle
}

/// 2D rotation matrix by `angle` (radians), applied as `r2d(a) * v`.
pub fn r2d(angle: f64) -> [[f64; 2]; 2] {
    let (s, c) = angle.sin_cos();
    [[c, -s], [s, c]]
}

pub fn apply_r2d(m: [[f64; 2]; 2], v: V2) -> V2 {
    [
        m[0][0] * v[0] + m[0][1] * v[1],
        m[1][0] * v[0] + m[1][1] * v[1],
    ]
}

/// Linear interpolation: `factor == 0` gives `val1`, `factor == 1` gives `val2`.
pub fn lin_interpolation(val1: f64, val2: f64, factor: f64) -> f64 {
    assert!(
        (0.0..=1.0).contains(&factor),
        "lin_interpolation::ERROR::Expected a factor in [0, 1], got {factor}"
    );
    (1.0 - factor) * val1 + factor * val2
}

// ----- Local edge coordinate frames (pattern.utils) -----

/// Convert a point expressed in the frame local to edge `[start, end]` into
/// the global (panel) frame.
pub fn rel_to_abs_2d(start: V2, end: V2, rel_point: V2) -> V2 {
    let edge = sub2(end, start);
    let edge_perp = [-edge[1], edge[0]];
    let abs_start = add2(start, scale2(edge, rel_point[0]));
    add2(abs_start, scale2(edge_perp, rel_point[1]))
}

/// Convert a global point (or vector, with `as_vector`) into the frame local to
/// edge `[start, end]`.
pub fn abs_to_rel_2d(start: V2, end: V2, abs_point: V2, as_vector: bool) -> V2 {
    let edge_vec = sub2(end, start);
    let edge_len = norm2(edge_vec);
    let point_vec = if as_vector {
        abs_point
    } else {
        sub2(abs_point, start)
    };

    let projected_len = dot2(edge_vec, point_vec) / edge_len;
    let x = projected_len / edge_len;

    let projected = scale2(edge_vec, x);
    let vert_comp = sub2(point_vec, projected);
    let mut y = norm2(vert_comp) / edge_len;

    // Distinguish left & right curvature.
    y *= cross2(edge_vec, point_vec).signum();

    [x, y]
}

// ----- Rotations -----

/// A 3D rotation, mirroring the slice of `scipy.spatial.transform.Rotation`
/// that GarmentCode uses.
///
/// Internally a rotation matrix. Euler angles use scipy's `XYZ` convention
/// (upper case == *intrinsic* rotations), i.e. `R = Rx(a) * Ry(b) * Rz(c)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rotation {
    m: [[f64; 3]; 3],
}

fn rot_x(t: f64) -> [[f64; 3]; 3] {
    let (s, c) = t.sin_cos();
    [[1.0, 0.0, 0.0], [0.0, c, -s], [0.0, s, c]]
}

fn rot_y(t: f64) -> [[f64; 3]; 3] {
    let (s, c) = t.sin_cos();
    [[c, 0.0, s], [0.0, 1.0, 0.0], [-s, 0.0, c]]
}

fn rot_z(t: f64) -> [[f64; 3]; 3] {
    let (s, c) = t.sin_cos();
    [[c, -s, 0.0], [s, c, 0.0], [0.0, 0.0, 1.0]]
}

pub fn matmul3(a: [[f64; 3]; 3], b: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let mut out = [[0.0; 3]; 3];
    for (i, row) in out.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            *cell = (0..3).map(|k| a[i][k] * b[k][j]).sum();
        }
    }
    out
}

impl Rotation {
    pub fn identity() -> Self {
        Self {
            m: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        }
    }

    pub fn from_matrix(m: [[f64; 3]; 3]) -> Self {
        Self { m }
    }

    pub fn as_matrix(&self) -> [[f64; 3]; 3] {
        self.m
    }

    /// scipy `Rotation.from_euler("XYZ", angles, degrees=degrees)`.
    pub fn from_euler_xyz(angles: V3, degrees: bool) -> Self {
        let a = if degrees {
            [
                angles[0].to_radians(),
                angles[1].to_radians(),
                angles[2].to_radians(),
            ]
        } else {
            angles
        };
        Self {
            m: matmul3(matmul3(rot_x(a[0]), rot_y(a[1])), rot_z(a[2])),
        }
    }

    /// scipy `Rotation.as_euler("XYZ", degrees=degrees)`.
    ///
    /// Recovers intrinsic X-Y-Z angles from the matrix. For
    /// `R = Rx(a) Ry(b) Rz(c)`: `R[0][2] = sin(b)`,
    /// `R[0][0..2] = cos(b) * [cos(c), -sin(c)]` and
    /// `R[1][2], R[2][2] = cos(b) * [-sin(a), cos(a)]`.
    pub fn as_euler_xyz(&self, degrees: bool) -> V3 {
        let m = self.m;
        let sy = m[0][2].clamp(-1.0, 1.0);
        let cy = (1.0 - sy * sy).max(0.0).sqrt();

        let (x, y, z) = if cy > 1e-10 {
            (
                (-m[1][2]).atan2(m[2][2]),
                sy.asin(),
                (-m[0][1]).atan2(m[0][0]),
            )
        } else {
            // Gimbal lock: pin the third angle to zero, as scipy does.
            (m[2][1].atan2(m[1][1]), sy.asin(), 0.0)
        };

        if degrees {
            [x.to_degrees(), y.to_degrees(), z.to_degrees()]
        } else {
            [x, y, z]
        }
    }

    /// scipy `Rotation.from_rotvec` -- axis-angle, magnitude is the angle.
    pub fn from_rotvec(v: V3) -> Self {
        let theta = norm3(v);
        if theta < 1e-12 {
            return Self::identity();
        }
        let k = scale3(v, 1.0 / theta);
        let (s, c) = theta.sin_cos();
        let one_c = 1.0 - c;
        Self {
            m: [
                [
                    c + k[0] * k[0] * one_c,
                    k[0] * k[1] * one_c - k[2] * s,
                    k[0] * k[2] * one_c + k[1] * s,
                ],
                [
                    k[1] * k[0] * one_c + k[2] * s,
                    c + k[1] * k[1] * one_c,
                    k[1] * k[2] * one_c - k[0] * s,
                ],
                [
                    k[2] * k[0] * one_c - k[1] * s,
                    k[2] * k[1] * one_c + k[0] * s,
                    c + k[2] * k[2] * one_c,
                ],
            ],
        }
    }

    pub fn apply(&self, v: V3) -> V3 {
        [
            self.m[0][0] * v[0] + self.m[0][1] * v[1] + self.m[0][2] * v[2],
            self.m[1][0] * v[0] + self.m[1][1] * v[1] + self.m[1][2] * v[2],
            self.m[2][0] * v[0] + self.m[2][1] * v[1] + self.m[2][2] * v[2],
        ]
    }

    /// `self * other` -- apply `other` first, then `self`.
    pub fn mul(&self, other: &Rotation) -> Rotation {
        Rotation {
            m: matmul3(self.m, other.m),
        }
    }
}

/// Rotation that takes `v1` onto `v2` -- `utils.vector_align_3D`.
pub fn vector_align_3d(v1: V3, v2: V3) -> Rotation {
    let cos = (dot3(v1, v2) / (norm3(v1) * norm3(v2))).clamp(-1.0, 1.0);
    let angle = cos.acos();

    let cross = cross3(v1, v2);
    let cross = scale3(cross, 1.0 / norm3(cross));

    Rotation::from_rotvec(scale3(cross, angle))
}

// ----- Maya-style euler conversion (pygarment.pattern.rotation) -----
//
// NOTE: this is a *different* convention from `Rotation`'s: it builds
// `Rz * Ry * Rx` (extrinsic xyz). The reference implementation genuinely uses
// both -- panels serialize scipy `XYZ` angles, while the pattern module reads
// them back through these routines -- so the port keeps both.

/// `rotation.euler_xyz_to_R` -- input in degrees.
pub fn euler_xyz_to_r(euler: V3) -> [[f64; 3]; 3] {
    matmul3(
        matmul3(rot_z(euler[2].to_radians()), rot_y(euler[1].to_radians())),
        rot_x(euler[0].to_radians()),
    )
}

/// `rotation.R_to_euler` -- output in degrees.
pub fn r_to_euler(r: [[f64; 3]; 3]) -> V3 {
    let tol = f64::EPSILON * 10.0;

    let (eul1, eul2, eul3);
    if r[0][0].abs() < tol && r[1][0].abs() < tol {
        eul1 = 0.0;
        eul2 = (-r[2][0]).atan2(r[0][0]);
        eul3 = (-r[1][2]).atan2(r[1][1]);
    } else {
        eul1 = r[1][0].atan2(r[0][0]);
        let sp = eul1.sin();
        let cp = eul1.cos();
        eul2 = (-r[2][0]).atan2(cp * r[0][0] + sp * r[1][0]);
        eul3 = (sp * r[0][2] - cp * r[1][2]).atan2(cp * r[1][1] - sp * r[0][1]);
    }

    [eul3.to_degrees(), eul2.to_degrees(), eul1.to_degrees()]
}

/// `BasicPattern._point_in_3D` -- 2D panel-local point to world coordinates
/// using the Maya-style rotation matrix.
pub fn point_in_3d(local: V2, rot: [[f64; 3]; 3], translation: V3) -> V3 {
    let p = [local[0], local[1], 0.0];
    let rotated = [
        rot[0][0] * p[0] + rot[0][1] * p[1] + rot[0][2] * p[2],
        rot[1][0] * p[0] + rot[1][1] * p[1] + rot[1][2] * p[2],
        rot[2][0] * p[0] + rot[2][1] * p[1] + rot[2][2] * p[2],
    ];
    add3(rotated, translation)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Oracle: `Rotation.from_euler("XYZ", [10, 20, 30], degrees=True).as_matrix()`.
    #[test]
    fn scipy_xyz_matrix() {
        let r = Rotation::from_euler_xyz([10.0, 20.0, 30.0], true);
        let m = r.as_matrix();
        let expected = [
            [0.81379768, -0.46984631, 0.34202014],
            [0.54383814, 0.82317294, -0.16317591],
            [-0.20487413, 0.31879578, 0.92541658],
        ];
        for i in 0..3 {
            for j in 0..3 {
                assert!(
                    (m[i][j] - expected[i][j]).abs() < 1e-8,
                    "m[{i}][{j}] = {} != {}",
                    m[i][j],
                    expected[i][j]
                );
            }
        }
    }

    #[test]
    fn euler_roundtrip() {
        for angles in [
            [10.0, 20.0, 30.0],
            [-45.0, 5.0, 100.0],
            [0.0, 0.0, 0.0],
            [12.0, -80.0, -3.0],
        ] {
            let r = Rotation::from_euler_xyz(angles, true);
            let back = r.as_euler_xyz(true);
            for i in 0..3 {
                assert!((back[i] - angles[i]).abs() < 1e-9, "{back:?} != {angles:?}");
            }
        }
    }

    /// The Maya-convention pair must round-trip through each other.
    #[test]
    fn maya_euler_roundtrip() {
        let angles = [15.0, -25.0, 40.0];
        let m = euler_xyz_to_r(angles);
        let back = r_to_euler(m);
        for i in 0..3 {
            assert!((back[i] - angles[i]).abs() < 1e-9, "{back:?}");
        }
    }

    /// The two conventions are genuinely different -- guard against silently
    /// unifying them.
    #[test]
    fn conventions_differ() {
        let angles = [10.0, 20.0, 30.0];
        let scipy = Rotation::from_euler_xyz(angles, true).as_matrix();
        let maya = euler_xyz_to_r(angles);
        assert!((scipy[0][1] - maya[0][1]).abs() > 1e-3);
    }

    #[test]
    fn rel_abs_roundtrip() {
        let start = [1.0, 2.0];
        let end = [5.0, 7.0];
        let p = [3.0, -4.0];
        let rel = abs_to_rel_2d(start, end, p, false);
        let abs = rel_to_abs_2d(start, end, rel);
        assert!(dist2(abs, p) < 1e-12, "{abs:?}");
    }

    #[test]
    fn align_3d() {
        let r = vector_align_3d([1.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        let out = r.apply([1.0, 0.0, 0.0]);
        assert!(norm3(sub3(out, [0.0, 0.0, 1.0])) < 1e-12, "{out:?}");
    }
}
