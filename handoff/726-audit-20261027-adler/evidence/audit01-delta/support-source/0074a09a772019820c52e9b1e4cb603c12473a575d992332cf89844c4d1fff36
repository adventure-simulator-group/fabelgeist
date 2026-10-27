//! Axis-aligned boxes, rays, and the triangle queries a mesh collider needs.

use fabelgeist_math::Vec3;

/// An axis-aligned bounding box.
///
/// The empty box is `min = +inf`, `max = -inf`, so that [`Aabb::union`] and
/// [`Aabb::extend`] start from [`Aabb::EMPTY`] without a special case.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Default for Aabb {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl Aabb {
    pub const EMPTY: Self = Self {
        min: Vec3 {
            x: f32::INFINITY,
            y: f32::INFINITY,
            z: f32::INFINITY,
        },
        max: Vec3 {
            x: f32::NEG_INFINITY,
            y: f32::NEG_INFINITY,
            z: f32::NEG_INFINITY,
        },
    };

    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn point(point: Vec3) -> Self {
        Self {
            min: point,
            max: point,
        }
    }

    pub fn from_points(points: impl IntoIterator<Item = Vec3>) -> Self {
        points
            .into_iter()
            .fold(Self::EMPTY, |bounds, point| bounds.extend(point))
    }

    /// An empty box has no volume to speak of; every query misses it.
    pub fn is_empty(&self) -> bool {
        self.min.x > self.max.x || self.min.y > self.max.y || self.min.z > self.max.z
    }

    pub fn extend(self, point: Vec3) -> Self {
        Self {
            min: self.min.min(point),
            max: self.max.max(point),
        }
    }

    pub fn union(self, other: Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    /// Grow by `amount` in every direction. Used to give contacts a margin so
    /// that a pair is found before it is already interpenetrating.
    pub fn expand(self, amount: f32) -> Self {
        if self.is_empty() {
            return self;
        }
        let amount = Vec3::splat(amount);
        Self {
            min: self.min - amount,
            max: self.max + amount,
        }
    }

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn extent(&self) -> Vec3 {
        if self.is_empty() {
            Vec3::default()
        } else {
            self.max - self.min
        }
    }

    /// Surface area, the cost term in a SAH split.
    pub fn surface_area(&self) -> f32 {
        if self.is_empty() {
            return 0.0;
        }
        let e = self.max - self.min;
        2.0 * (e.x * e.y + e.y * e.z + e.z * e.x)
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    pub fn contains(&self, point: Vec3) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }

    /// Nearest point of the box to `point`; `point` itself if it is inside.
    pub fn clamp(&self, point: Vec3) -> Vec3 {
        point.max(self.min).min(self.max)
    }

    /// Squared distance from `point` to the box; zero inside.
    pub fn distance_squared(&self, point: Vec3) -> f32 {
        if self.is_empty() {
            return f32::INFINITY;
        }
        let delta = point - self.clamp(point);
        delta.length_squared()
    }

    /// Normalised position of `point` inside the box, per axis, clamped to
    /// `[0, 1]`. A degenerate axis maps to the middle rather than to a
    /// division by zero.
    pub fn normalize(&self, point: Vec3) -> Vec3 {
        let extent = self.extent();
        let component = |value: f32, min: f32, size: f32| {
            if size > 0.0 {
                ((value - min) / size).clamp(0.0, 1.0)
            } else {
                0.5
            }
        };
        Vec3::new(
            component(point.x, self.min.x, extent.x),
            component(point.y, self.min.y, extent.y),
            component(point.z, self.min.z, extent.z),
        )
    }
}

/// A ray, with the reciprocal direction precomputed for the slab test.
#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
    inverse_direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction,
            // A zero component gives an infinity here on purpose: the slab
            // test then compares against +/-inf, which is the correct answer
            // for a ray parallel to that axis.
            inverse_direction: Vec3::new(1.0 / direction.x, 1.0 / direction.y, 1.0 / direction.z),
        }
    }

    pub fn at(&self, distance: f32) -> Vec3 {
        self.origin + self.direction * distance
    }

    /// Distance along the ray at which it enters `bounds`, if it enters within
    /// `[0, max_distance]`. Zero when the origin is already inside.
    pub fn hits(&self, bounds: &Aabb, max_distance: f32) -> Option<f32> {
        let t0 = (bounds.min - self.origin) * self.inverse_direction;
        let t1 = (bounds.max - self.origin) * self.inverse_direction;
        let near = t0.min(t1);
        let far = t0.max(t1);
        let enter = near.max_component().max(0.0);
        let exit = far.min_component().min(max_distance);
        // `<=` so that a ray grazing a flat box -- a triangle's own bounds --
        // still counts as a hit.
        (enter <= exit).then_some(enter)
    }
}

/// The point of triangle `abc` closest to `point`.
///
/// Ericson's *Real-Time Collision Detection*, section 5.1.5: test the three
/// vertex regions and the three edge regions before falling through to the
/// face, so that the result is right for a point outside the triangle's prism
/// as well as inside it.
pub fn closest_point_on_triangle(point: Vec3, a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    let ab = b - a;
    let ac = c - a;
    let ap = point - a;

    let d1 = ab.dot(ap);
    let d2 = ac.dot(ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return a;
    }

    let bp = point - b;
    let d3 = ab.dot(bp);
    let d4 = ac.dot(bp);
    if d3 >= 0.0 && d4 <= d3 {
        return b;
    }

    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let denominator = d1 - d3;
        let v = if denominator != 0.0 {
            d1 / denominator
        } else {
            0.0
        };
        return a + ab * v;
    }

    let cp = point - c;
    let d5 = ab.dot(cp);
    let d6 = ac.dot(cp);
    if d6 >= 0.0 && d5 <= d6 {
        return c;
    }

    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let denominator = d2 - d6;
        let w = if denominator != 0.0 {
            d2 / denominator
        } else {
            0.0
        };
        return a + ac * w;
    }

    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let denominator = (d4 - d3) + (d5 - d6);
        let w = if denominator != 0.0 {
            (d4 - d3) / denominator
        } else {
            0.0
        };
        return b + (c - b) * w;
    }

    let denominator = va + vb + vc;
    if denominator == 0.0 {
        // A degenerate triangle -- three collinear or coincident points. Any
        // vertex is as good an answer as another.
        return a;
    }
    let v = vb / denominator;
    let w = vc / denominator;
    a + ab * v + ac * w
}

/// Moller-Trumbore. Returns the distance along the ray, for hits in
/// `(epsilon, max_distance]` from either face.
pub fn ray_triangle(ray: &Ray, a: Vec3, b: Vec3, c: Vec3, max_distance: f32) -> Option<f32> {
    const EPSILON: f32 = 1e-8;

    let edge1 = b - a;
    let edge2 = c - a;
    let h = ray.direction.cross(edge2);
    let determinant = edge1.dot(h);
    if determinant.abs() < EPSILON {
        return None;
    }

    let inverse = 1.0 / determinant;
    let s = ray.origin - a;
    let u = inverse * s.dot(h);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }

    let q = s.cross(edge1);
    let v = inverse * ray.direction.dot(q);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    let distance = inverse * edge2.dot(q);
    (distance > EPSILON && distance <= max_distance).then_some(distance)
}
