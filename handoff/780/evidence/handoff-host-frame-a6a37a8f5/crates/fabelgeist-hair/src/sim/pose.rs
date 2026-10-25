//! Head pose in world coordinates and the native matrix representation.

use fabelgeist_math::Vec3;
use crate::strands::safe_normalize;

/// A rotation and a translation: where the head is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rigid {
    /// Columns of the rotation.
    pub axes: [Vec3; 3],
    pub translation: Vec3,
}

impl Default for Rigid {
    fn default() -> Self {
        Self {
            axes: [Vec3::right(), Vec3::up(), Vec3::new(0.0, 0.0, 1.0)],
            translation: Vec3::default(),
        }
    }
}

impl Rigid {
    /// Turned by `yaw` about the vertical, then `pitch` about the side axis
    /// and `roll` about the front, about `pivot` (the neck), and moved by
    /// `offset`.
    pub fn from_angles(yaw: f32, pitch: f32, roll: f32, pivot: Vec3, offset: Vec3) -> Self {
        let (sy, cy) = yaw.sin_cos();
        let (sp, cp) = pitch.sin_cos();
        let (sr, cr) = roll.sin_cos();
        // R = Ry * Rx * Rz, as columns.
        let ry = [Vec3::new(cy, 0.0, -sy), Vec3::up(), Vec3::new(sy, 0.0, cy)];
        let rx = [
            Vec3::right(),
            Vec3::new(0.0, cp, sp),
            Vec3::new(0.0, -sp, cp),
        ];
        let rz = [
            Vec3::new(cr, sr, 0.0),
            Vec3::new(-sr, cr, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        ];
        let apply = |m: &[Vec3; 3], v: Vec3| m[0] * v.x + m[1] * v.y + m[2] * v.z;
        let axes = [0, 1, 2].map(|k| apply(&ry, apply(&rx, rz[k])));
        let rotated_pivot = apply(&axes, pivot);
        Self {
            axes,
            translation: pivot - rotated_pivot + offset,
        }
    }

    pub fn rotate(&self, v: Vec3) -> Vec3 {
        self.axes[0] * v.x + self.axes[1] * v.y + self.axes[2] * v.z
    }

    pub fn apply(&self, p: Vec3) -> Vec3 {
        self.rotate(p) + self.translation
    }

    /// From the world into the head's frame.
    pub fn unapply(&self, p: Vec3) -> Vec3 {
        let q = p - self.translation;
        Vec3::new(
            self.axes[0].dot(q),
            self.axes[1].dot(q),
            self.axes[2].dot(q),
        )
    }

    /// Column-major, as WGSL's `mat4x4<f32>` reads it.
    pub fn matrix(&self) -> [[f32; 4]; 4] {
        let [a, b, c] = self.axes;
        let t = self.translation;
        [
            [a.x, a.y, a.z, 0.0],
            [b.x, b.y, b.z, 0.0],
            [c.x, c.y, c.z, 0.0],
            [t.x, t.y, t.z, 1.0],
        ]
    }
}

/// Part of the way from one pose to another (rotation by normalised lerp
/// of the axes, which is close enough within a frame).
pub(crate) fn blend(a: &Rigid, b: &Rigid, t: f32) -> Rigid {
    let x = safe_normalize(a.axes[0].lerp(b.axes[0], t), b.axes[0]);
    let y0 = a.axes[1].lerp(b.axes[1], t);
    let y = safe_normalize(y0 - x * y0.dot(x), b.axes[1]);
    Rigid {
        axes: [x, y, x.cross(y)],
        translation: a.translation.lerp(b.translation, t),
    }
}

