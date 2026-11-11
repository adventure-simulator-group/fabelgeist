//! The shapes a surface is described by, and their signed distances.
//!
//! One layout serves every kind, so that one buffer holds a whole scene:
//! three `vec4`s whose meaning the kind sets, then the object, payload,
//! reach and a word holding the kind and the layers.

/// What a [`Shape`] is.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShapeKind {
    /// Points within a radius of a segment, the radius running linearly
    /// from one end to the other: a sphere, a capsule, a branch.
    RoundCone = 0,
    /// A box, rotated and with rounded edges.
    Box = 1,
    /// A triangle, as a thin shell around it.
    Triangle = 2,
    /// One cell of a heightfield: the ground below a bilinear patch, down to
    /// a depth. A tile answers only over its own footprint, so tiles side by
    /// side make one continuous ground.
    HeightTile = 3,
}

impl ShapeKind {
    fn from_bits(bits: u32) -> Self {
        match bits & 0xff {
            1 => Self::Box,
            2 => Self::Triangle,
            3 => Self::HeightTile,
            _ => Self::RoundCone,
        }
    }
}

/// Every layer: a [`Shape`]'s default, and a [`crate::Filter`]'s.
pub const ALL_LAYERS: u32 = 0x00ff_ffff;

/// A simple shape, part of some object's surface.
///
/// | kind        | `p0`                      | `p1`                           | `p2`             |
/// |-------------|---------------------------|--------------------------------|------------------|
/// | round cone  | one end, its radius       | the other end, its radius      | --               |
/// | box         | centre, edge rounding     | half extents                   | rotation (quat)  |
/// | triangle    | a, half thickness         | b                              | c                |
/// | height tile | `x0, z0, x1, z1`          | heights at `(x0,z0) (x1,z0) (x0,z1) (x1,z1)` | depth |
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Shape {
    pub p0: [f32; 4],
    pub p1: [f32; 4],
    pub p2: [f32; 4],
    /// The object it is part of. A query can leave one object out: a
    /// surface does not blend with its own pieces, which already fit.
    pub object: u32,
    /// The caller's: what a shader looks the surface's attributes up by.
    pub payload: u32,
    /// How far from its surface the shape is found, and so how far it
    /// blends into others: `k` of the smooth minimum, the size of the fillet
    /// it grows. A pair blends by the smaller of their two.
    pub reach: f32,
    /// The kind in the low byte, the layers it is on above.
    pub mask: u32,
}

impl Shape {
    fn new(kind: ShapeKind, p0: [f32; 4], p1: [f32; 4], p2: [f32; 4]) -> Self {
        Self {
            p0,
            p1,
            p2,
            object: 0,
            payload: 0,
            reach: 0.0,
            mask: kind as u32 | (ALL_LAYERS << 8),
        }
    }

    /// A round cone from `a`, radius `ra`, to `b`, radius `rb`.
    pub fn round_cone(a: [f32; 3], ra: f32, b: [f32; 3], rb: f32) -> Self {
        Self::new(
            ShapeKind::RoundCone,
            [a[0], a[1], a[2], ra],
            [b[0], b[1], b[2], rb],
            [0.0; 4],
        )
    }

    pub fn sphere(center: [f32; 3], radius: f32) -> Self {
        Self::round_cone(center, radius, center, radius)
    }

    pub fn capsule(a: [f32; 3], b: [f32; 3], radius: f32) -> Self {
        Self::round_cone(a, radius, b, radius)
    }

    /// A box of `half` extents about `center`, turned by the unit quaternion
    /// `rotation` (`x, y, z, w`), its edges rounded by `rounding`.
    pub fn oriented_box(
        center: [f32; 3],
        half: [f32; 3],
        rotation: [f32; 4],
        rounding: f32,
    ) -> Self {
        Self::new(
            ShapeKind::Box,
            [center[0], center[1], center[2], rounding],
            [half[0], half[1], half[2], 0.0],
            rotation,
        )
    }

    /// A triangle, thickened into a shell `half_thickness` either side.
    pub fn triangle(a: [f32; 3], b: [f32; 3], c: [f32; 3], half_thickness: f32) -> Self {
        Self::new(
            ShapeKind::Triangle,
            [a[0], a[1], a[2], half_thickness],
            [b[0], b[1], b[2], 0.0],
            [c[0], c[1], c[2], 0.0],
        )
    }

    /// The ground under the cell `[x0, x1) x [z0, z1)` of a heightfield,
    /// given its corner heights `[h(x0,z0), h(x1,z0), h(x0,z1), h(x1,z1)]`,
    /// down to `depth` below its lowest corner: a slab, bounded like every
    /// other shape so that it can be binned.
    pub fn height_tile(min: [f32; 2], max: [f32; 2], heights: [f32; 4], depth: f32) -> Self {
        Self::new(
            ShapeKind::HeightTile,
            [min[0], min[1], max[0], max[1]],
            heights,
            [depth, 0.0, 0.0, 0.0],
        )
    }

    pub fn with_object(mut self, object: u32) -> Self {
        self.object = object;
        self
    }

    pub fn with_payload(mut self, payload: u32) -> Self {
        self.payload = payload;
        self
    }

    pub fn with_reach(mut self, reach: f32) -> Self {
        self.reach = reach;
        self
    }

    /// Put it on `layers` (a bit set of 24) only.
    pub fn with_layers(mut self, layers: u32) -> Self {
        self.mask = (self.mask & 0xff) | ((layers & ALL_LAYERS) << 8);
        self
    }

    pub fn kind(&self) -> ShapeKind {
        ShapeKind::from_bits(self.mask)
    }

    pub fn layers(&self) -> u32 {
        self.mask >> 8
    }

    /// Signed distance from `p` to the surface: negative inside. Exact but
    /// for a height tile's, which is the vertical distance to the patch over
    /// the slope's secant -- first-order, and exact on flat ground.
    pub fn distance(&self, p: [f32; 3]) -> f32 {
        match self.kind() {
            ShapeKind::RoundCone => round_cone(p, self.p0, self.p1),
            ShapeKind::Box => rounded_box(p, self.p0, self.p1, self.p2),
            ShapeKind::Triangle => triangle(p, self.p0, self.p1, self.p2),
            ShapeKind::HeightTile => height_tile(p, self.p0, self.p1, self.p2[0]),
        }
    }

    /// Everything within `reach` of the surface, as a box.
    pub(crate) fn bounds(&self, reach: f32) -> ([f32; 3], [f32; 3]) {
        let (lo, hi) = match self.kind() {
            ShapeKind::RoundCone => {
                let (a, b) = (self.p0, self.p1);
                (
                    [0, 1, 2].map(|k| (a[k] - a[3]).min(b[k] - b[3])),
                    [0, 1, 2].map(|k| (a[k] + a[3]).max(b[k] + b[3])),
                )
            }
            ShapeKind::Box => {
                // The rotated box's own bounds: each world axis gathers the
                // box's extents along it.
                let r = rotation_rows(self.p2);
                let half = [self.p1[0], self.p1[1], self.p1[2]];
                let extent = [0, 1, 2]
                    .map(|k| (0..3).map(|j| r[k][j].abs() * half[j]).sum::<f32>() + self.p0[3]);
                (
                    [0, 1, 2].map(|k| self.p0[k] - extent[k]),
                    [0, 1, 2].map(|k| self.p0[k] + extent[k]),
                )
            }
            ShapeKind::Triangle => {
                let t = self.p0[3];
                (
                    [0, 1, 2].map(|k| self.p0[k].min(self.p1[k]).min(self.p2[k]) - t),
                    [0, 1, 2].map(|k| self.p0[k].max(self.p1[k]).max(self.p2[k]) + t),
                )
            }
            ShapeKind::HeightTile => {
                // Only over its footprint: a neighbour answers beyond. Its
                // distance is the height over the slope's secant, so it is
                // found up to `reach` times the steepest secant above or
                // below.
                let h = self.p1;
                let (low, high) = (
                    h[0].min(h[1]).min(h[2]).min(h[3]),
                    h[0].max(h[1]).max(h[2]).max(h[3]),
                );
                let w = (self.p0[2] - self.p0[0]).max(1e-9);
                let d = (self.p0[3] - self.p0[1]).max(1e-9);
                let gx = (h[1] - h[0]).abs().max((h[3] - h[2]).abs()) / w;
                let gz = (h[2] - h[0]).abs().max((h[3] - h[1]).abs()) / d;
                let vertical = reach * (1.0 + gx * gx + gz * gz).sqrt();
                return (
                    [self.p0[0], low - self.p2[0] - vertical, self.p0[1]],
                    [self.p0[2], high + vertical, self.p0[3]],
                );
            }
        };
        (lo.map(|x| x - reach), hi.map(|x| x + reach))
    }
}

pub(crate) fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub(crate) fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn scale(a: [f32; 3], s: f32) -> [f32; 3] {
    a.map(|x| x * s)
}

fn xyz(v: [f32; 4]) -> [f32; 3] {
    [v[0], v[1], v[2]]
}

/// Quílez's round cone.
fn round_cone(p: [f32; 3], a: [f32; 4], b: [f32; 4]) -> f32 {
    let (pa, pb) = (xyz(a), xyz(b));
    let (r1, r2) = (a[3], b[3]);
    let ba = sub(pb, pa);
    let l2 = dot(ba, ba);
    if l2 < 1e-12 {
        let d = sub(p, pa);
        return dot(d, d).sqrt() - r1.max(r2);
    }
    let rr = r1 - r2;
    let a2 = l2 - rr * rr;
    let il2 = 1.0 / l2;
    let pa_ = sub(p, pa);
    let y = dot(pa_, ba);
    let z = y - l2;
    let xv = sub(scale(pa_, l2), scale(ba, y));
    let x2 = dot(xv, xv);
    let y2 = y * y * l2;
    let z2 = z * z * l2;
    let k = rr.signum() * rr * rr * x2;
    if z.signum() * a2 * z2 > k {
        return (x2 + z2).sqrt() * il2 - r2;
    }
    if y.signum() * a2 * y2 < k {
        return (x2 + y2).sqrt() * il2 - r1;
    }
    ((x2 * a2 * il2).sqrt() + y * rr) * il2 - r1
}

/// The rows of the rotation matrix of the unit quaternion `q`.
fn rotation_rows(q: [f32; 4]) -> [[f32; 3]; 3] {
    let [x, y, z, w] = q;
    [
        [
            1.0 - 2.0 * (y * y + z * z),
            2.0 * (x * y - z * w),
            2.0 * (x * z + y * w),
        ],
        [
            2.0 * (x * y + z * w),
            1.0 - 2.0 * (x * x + z * z),
            2.0 * (y * z - x * w),
        ],
        [
            2.0 * (x * z - y * w),
            2.0 * (y * z + x * w),
            1.0 - 2.0 * (x * x + y * y),
        ],
    ]
}

/// A rotated, rounded box.
fn rounded_box(p: [f32; 3], center: [f32; 4], half: [f32; 4], rotation: [f32; 4]) -> f32 {
    // Into the box's frame: the transpose of its rotation.
    let r = rotation_rows(rotation);
    let d = sub(p, xyz(center));
    let local = [0, 1, 2].map(|j| r[0][j] * d[0] + r[1][j] * d[1] + r[2][j] * d[2]);
    let rounding = center[3];
    let q = [0, 1, 2].map(|k| local[k].abs() - (half[k] - rounding));
    let outside = q.map(|x| x.max(0.0));
    dot(outside, outside).sqrt() + q[0].max(q[1]).max(q[2]).min(0.0) - rounding
}

fn dot2(v: [f32; 3]) -> f32 {
    dot(v, v)
}

/// Quílez's unsigned triangle distance, less the shell's half thickness.
fn triangle(p: [f32; 3], a: [f32; 4], b: [f32; 4], c: [f32; 4]) -> f32 {
    let (a3, b3, c3) = (xyz(a), xyz(b), xyz(c));
    let (ba, pa) = (sub(b3, a3), sub(p, a3));
    let (cb, pb) = (sub(c3, b3), sub(p, b3));
    let (ac, pc) = (sub(a3, c3), sub(p, c3));
    let nor = cross(ba, ac);
    let side = dot(cross(ba, nor), pa).signum()
        + dot(cross(cb, nor), pb).signum()
        + dot(cross(ac, nor), pc).signum();
    let squared = if side < 2.0 {
        let edge = |e: [f32; 3], q: [f32; 3]| {
            dot2(sub(scale(e, (dot(e, q) / dot2(e)).clamp(0.0, 1.0)), q))
        };
        edge(ba, pa).min(edge(cb, pb)).min(edge(ac, pc))
    } else {
        let d = dot(nor, pa);
        d * d / dot2(nor)
    };
    squared.sqrt() - a[3]
}

/// The ground under a bilinear patch down to `depth` below its lowest
/// corner, over its footprint only.
fn height_tile(p: [f32; 3], footprint: [f32; 4], h: [f32; 4], depth: f32) -> f32 {
    let [x0, z0, x1, z1] = footprint;
    if p[0] < x0 || p[0] >= x1 || p[2] < z0 || p[2] >= z1 {
        return f32::INFINITY;
    }
    let (w, d) = ((x1 - x0).max(1e-9), (z1 - z0).max(1e-9));
    let u = (p[0] - x0) / w;
    let v = (p[2] - z0) / d;
    let height =
        h[0] * (1.0 - u) * (1.0 - v) + h[1] * u * (1.0 - v) + h[2] * (1.0 - u) * v + h[3] * u * v;
    let gx = ((h[1] - h[0]) * (1.0 - v) + (h[3] - h[2]) * v) / w;
    let gz = ((h[2] - h[0]) * (1.0 - u) + (h[3] - h[1]) * u) / d;
    let top = (p[1] - height) / (1.0 + gx * gx + gz * gz).sqrt();
    let bottom = h[0].min(h[1]).min(h[2]).min(h[3]) - depth;
    top.max(bottom - p[1])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_round_cone_measures_its_own_surface() {
        let cone = Shape::round_cone([0.0, 0.0, 0.0], 1.0, [0.0, 4.0, 0.0], 0.5);
        assert!(cone.distance([0.0, 0.0, 0.0]) < -0.9);
        assert!(cone.distance([0.0, -1.0, 0.0]).abs() < 1e-5);
        assert!(cone.distance([0.0, 4.5, 0.0]).abs() < 1e-5);
        assert!((cone.distance([3.0, 0.0, 0.0]) - 2.0).abs() < 1e-5);
        let sphere = Shape::sphere([1.0, 2.0, 3.0], 0.5);
        assert!((sphere.distance([1.0, 2.0, 4.0]) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn a_turned_box_measures_along_its_own_axes() {
        // A quarter turn about y: its long x extent lies along z.
        let s = std::f32::consts::FRAC_1_SQRT_2;
        let b = Shape::oriented_box([0.0; 3], [2.0, 1.0, 0.5], [0.0, s, 0.0, s], 0.0);
        assert!((b.distance([0.0, 0.0, 3.0]) - 1.0).abs() < 1e-5);
        assert!((b.distance([1.0, 0.0, 0.0]) - 0.5).abs() < 1e-5);
        assert!((b.distance([0.0, 0.0, 0.0]) + 0.5).abs() < 1e-5);
        let (lo, hi) = b.bounds(0.0);
        assert!((hi[2] - 2.0).abs() < 1e-5 && (lo[0] + 0.5).abs() < 1e-5);
    }

    #[test]
    fn a_triangle_is_a_thin_shell() {
        let t = Shape::triangle([0.0; 3], [2.0, 0.0, 0.0], [0.0, 0.0, 2.0], 0.1);
        assert!((t.distance([0.5, 1.0, 0.5]) - 0.9).abs() < 1e-5);
        assert!((t.distance([0.5, 0.0, 0.5]) + 0.1).abs() < 1e-5);
        assert!((t.distance([-1.0, 0.0, 0.0]) - 0.9).abs() < 1e-5);
    }

    #[test]
    fn height_tiles_make_one_ground() {
        // Two tiles of a slope, side by side: the distance runs on across
        // their shared edge, and each answers only over itself.
        let slope = |x: f32| 0.5 * x;
        let left = Shape::height_tile(
            [0.0, 0.0],
            [1.0, 1.0],
            [slope(0.0), slope(1.0), slope(0.0), slope(1.0)],
            2.0,
        );
        let right = Shape::height_tile(
            [1.0, 0.0],
            [2.0, 1.0],
            [slope(1.0), slope(2.0), slope(1.0), slope(2.0)],
            2.0,
        );
        let ground = |p: [f32; 3]| left.distance(p).min(right.distance(p));
        let before = ground([0.9999, 2.0, 0.5]);
        let after = ground([1.0, 2.0, 0.5]);
        assert!((before - after).abs() < 1e-3);
        // Perpendicular to the slope: 1.5 above over sqrt(1 + 0.25).
        assert!((after - 1.5 / 1.25f32.sqrt()).abs() < 1e-5);
        assert_eq!(left.distance([1.5, 0.0, 0.5]), f32::INFINITY);
        assert!(ground([0.5, 0.0, 0.5]) < 0.0, "below ground is inside");
        assert!(
            (ground([0.5, -3.0, 0.5]) - 1.0).abs() < 1e-5,
            "and below the slab, out"
        );
    }

    #[test]
    fn layers_and_kind_share_a_word() {
        let s = Shape::triangle([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.0).with_layers(0b101);
        assert_eq!(s.kind(), ShapeKind::Triangle);
        assert_eq!(s.layers(), 0b101);
        assert_eq!(Shape::sphere([0.0; 3], 1.0).layers(), ALL_LAYERS);
    }
}
