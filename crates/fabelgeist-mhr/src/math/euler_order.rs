//! Effective Euler composition order for MHR rotation helpers.

/// Each coordinate axis is applied once, in the named order.
///
/// The first axis is applied first: [`Self::Xyz`] composes `Rz * Ry * Rx`.
/// Angles remain indexed by coordinate axis (`[x, y, z]`) for every order.
///
/// Callers select a named order rather than supplying arbitrary axis indexes.
/// Repeated axes cannot enter the radians helper:
///
/// ```compile_fail
/// use fabelgeist_mhr::math::quat_from_euler;
/// let _ = quat_from_euler([0.1, 0.2, 0.3], [0, 0, 2]);
/// ```
///
/// Out-of-range indexes cannot enter the degrees helper:
///
/// ```compile_fail
/// use fabelgeist_mhr::math::quat_from_euler_degrees;
/// let _ = quat_from_euler_degrees([10.0, 20.0, 30.0], [0, 1, 3]);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EulerOrder {
    Xyz,
    Xzy,
    Yzx,
    Yxz,
    Zxy,
    Zyx,
}

impl EulerOrder {
    /// Interprets the native FBX `RotationOrder` integer as an effective order.
    ///
    /// Codes 1 through 5 select XZY, YZX, YXZ, ZXY, and ZYX respectively.
    /// Code 0 and every other value select XYZ, preserving the rig loader's
    /// fallback policy. This includes spherical code 6: spherical rotation is
    /// unsupported and is treated as XYZ. This constructor neither validates
    /// nor retains the original FBX code.
    pub fn from_fbx_code(code: i64) -> Self {
        match code {
            1 => Self::Xzy,
            2 => Self::Yzx,
            3 => Self::Yxz,
            4 => Self::Zxy,
            5 => Self::Zyx,
            _ => Self::Xyz,
        }
    }

    /// Native `[x, y, z]` array indexes for the quaternion arithmetic kernel.
    pub(super) fn native_indices(self) -> [usize; 3] {
        match self {
            Self::Xyz => [0, 1, 2],
            Self::Xzy => [0, 2, 1],
            Self::Yzx => [1, 2, 0],
            Self::Yxz => [1, 0, 2],
            Self::Zxy => [2, 0, 1],
            Self::Zyx => [2, 1, 0],
        }
    }
}
