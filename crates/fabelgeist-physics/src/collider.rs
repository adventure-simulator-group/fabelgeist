//! Analytic colliders: the shapes a solver can test against in closed form.
//!
//! A body could be collided against as a triangle mesh alone -- and
//! [`crate::mesh`] does that -- but a handful of capsules approximating limbs
//! is far cheaper and, for a garment that only needs to stay outside the body,
//! often enough. Both are available, and they compose: a mesh for the torso
//! where the fit matters, capsules for the arms.

use fabelgeist_math::Vec3;

/// Bytes per collider on the GPU: three `vec4` of shape data plus a `vec4` of
/// kind, friction and padding.
pub const COLLIDER_BYTES: usize = 64;

/// A shape, in world space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    /// Everything on the negative side of the plane is inside.
    Plane {
        normal: Vec3,
        offset: f32,
    },
    Sphere {
        center: Vec3,
        radius: f32,
    },
    /// A swept sphere: the segment `a`-`b` thickened by `radius`. This is what
    /// a limb is.
    Capsule {
        a: Vec3,
        b: Vec3,
        radius: f32,
    },
    /// An axis-aligned box, optionally rotated by `rotation` about its centre.
    /// The rotation is a unit quaternion, `xyzw`.
    Box {
        center: Vec3,
        half_extents: Vec3,
        rotation: [f32; 4],
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Collider {
    pub shape: Shape,
    /// Coulomb friction against this surface. Zero is frictionless; a garment
    /// staying on a shoulder rather than sliding off it wants roughly 0.3.
    pub friction: f32,
    /// Extra separation held between a particle's centre and the surface. Half
    /// the fabric's thickness, plus whatever margin keeps the two visually
    /// apart.
    pub thickness: f32,
}

impl Collider {
    pub fn new(shape: Shape) -> Self {
        Self {
            shape,
            friction: 0.3,
            thickness: 0.002,
        }
    }

    pub fn plane(normal: Vec3, offset: f32) -> Self {
        Self::new(Shape::Plane { normal, offset })
    }

    /// The ground: `y = height`.
    pub fn ground(height: f32) -> Self {
        Self::plane(Vec3::new(0.0, 1.0, 0.0), height)
    }

    pub fn sphere(center: Vec3, radius: f32) -> Self {
        Self::new(Shape::Sphere { center, radius })
    }

    pub fn capsule(a: Vec3, b: Vec3, radius: f32) -> Self {
        Self::new(Shape::Capsule { a, b, radius })
    }

    pub fn with_friction(mut self, friction: f32) -> Self {
        self.friction = friction;
        self
    }

    pub fn with_thickness(mut self, thickness: f32) -> Self {
        self.thickness = thickness;
        self
    }

    fn kind(&self) -> u32 {
        match self.shape {
            Shape::Plane { .. } => 0,
            Shape::Sphere { .. } => 1,
            Shape::Capsule { .. } => 2,
            Shape::Box { .. } => 3,
        }
    }

    /// The 16 floats the WGSL `Collider` struct expects.
    pub fn pack(&self) -> [f32; 16] {
        let mut packed = [0.0f32; 16];
        match self.shape {
            Shape::Plane { normal, offset } => {
                packed[0..4].copy_from_slice(&[normal.x, normal.y, normal.z, offset]);
            }
            Shape::Sphere { center, radius } => {
                packed[0..4].copy_from_slice(&[center.x, center.y, center.z, radius]);
            }
            Shape::Capsule { a, b, radius } => {
                packed[0..4].copy_from_slice(&[a.x, a.y, a.z, radius]);
                packed[4..8].copy_from_slice(&[b.x, b.y, b.z, 0.0]);
            }
            Shape::Box {
                center,
                half_extents,
                rotation,
            } => {
                packed[0..4].copy_from_slice(&[center.x, center.y, center.z, 0.0]);
                packed[4..8].copy_from_slice(&[
                    half_extents.x,
                    half_extents.y,
                    half_extents.z,
                    0.0,
                ]);
                packed[8..12].copy_from_slice(&rotation);
            }
        }
        packed[12] = f32::from_bits(self.kind());
        packed[13] = self.friction;
        packed[14] = self.thickness;
        packed
    }

    /// Signed distance from `point` to the surface, and the outward direction.
    /// Negative distance means inside.
    ///
    /// The host mirror of what the WGSL does, for tests and for host-side
    /// queries.
    pub fn signed_distance(&self, point: Vec3) -> (f32, Vec3) {
        match self.shape {
            Shape::Plane { normal, offset } => (point.dot(normal) - offset, normal),
            Shape::Sphere { center, radius } => {
                let delta = point - center;
                let length = delta.length();
                let direction = if length > 1e-9 {
                    delta / length
                } else {
                    Vec3::new(0.0, 1.0, 0.0)
                };
                (length - radius, direction)
            }
            Shape::Capsule { a, b, radius } => {
                let axis = b - a;
                let length_squared = axis.length_squared();
                // A degenerate capsule is a sphere at `a`.
                let t = if length_squared > 1e-12 {
                    ((point - a).dot(axis) / length_squared).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let closest = a + axis * t;
                let delta = point - closest;
                let length = delta.length();
                let direction = if length > 1e-9 {
                    delta / length
                } else {
                    Vec3::new(0.0, 1.0, 0.0)
                };
                (length - radius, direction)
            }
            Shape::Box {
                center,
                half_extents,
                rotation,
            } => {
                let local = quaternion_rotate(quaternion_conjugate(rotation), point - center);
                let q = local.abs() - half_extents;
                let outside = q.max(Vec3::default());
                let outside_length = outside.length();
                // Inside, the nearest face is the least negative component.
                let inside = q.max_component().min(0.0);
                let distance = outside_length + inside;

                let local_direction = if outside_length > 1e-9 {
                    outside / outside_length
                        * Vec3::new(local.x.signum(), local.y.signum(), local.z.signum())
                } else {
                    let axis = q.max_axis();
                    let mut direction = Vec3::default();
                    direction.set_axis(axis, local.axis(axis).signum());
                    direction
                };
                (distance, quaternion_rotate(rotation, local_direction))
            }
        }
    }
}

/// `xyzw` quaternion applied to a vector.
pub fn quaternion_rotate(q: [f32; 4], v: Vec3) -> Vec3 {
    let u = Vec3::new(q[0], q[1], q[2]);
    let w = q[3];
    u * (2.0 * u.dot(v)) + v * (w * w - u.dot(u)) + u.cross(v) * (2.0 * w)
}

pub fn quaternion_conjugate(q: [f32; 4]) -> [f32; 4] {
    [-q[0], -q[1], -q[2], q[3]]
}

/// Pack a whole list for upload.
pub fn pack_colliders(colliders: &[Collider]) -> Vec<f32> {
    let mut packed = Vec::with_capacity(colliders.len() * 16);
    for collider in colliders {
        packed.extend_from_slice(&collider.pack());
    }
    packed
}
