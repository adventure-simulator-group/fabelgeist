//! The solids buildings are made of, as exact signed distances: negative
//! inside, the Euclidean distance to the surface outside.
//!
//! Every solid stands upright on its base and may be turned about the
//! vertical. In its own frame the base's middle is the origin, `x` runs
//! across its width and `z` across its depth; `size` is its half width (a
//! round solid's radius), its height and its half depth.
//!
//! The formulas are Inigo Quilez's exact ones for each shape; the gable is
//! his isosceles triangle extruded, the pyramid his square pyramid scaled.
//! [`SOLID_WGSL`] is the same arithmetic line for line, so a GPU pass and the
//! host agree on what is inside.

use bytemuck::{Pod, Zeroable};

/// What shape a solid has.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum Shape {
    /// A box `2 size.x` wide, `size.y` tall, `2 size.z` deep.
    Box = 0,
    /// A vertical cylinder of radius `size.x`, `size.y` tall.
    Cylinder = 1,
    /// A cone of base radius `size.x` narrowing to a point `size.y` up.
    Cone = 2,
    /// A gabled roof: a triangular prism `2 size.x` long with its ridge
    /// along `x`, `size.y` tall at the ridge and `2 size.z` wide at the eaves.
    Gable = 3,
    /// A pyramid on a square base `2 size.x` across, `size.y` tall.
    Pyramid = 4,
}

/// One solid: its shape, where its base stands and how it is turned.
///
/// Two `vec4`s, which is how WGSL reads it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct Solid {
    /// The middle of its base.
    pub base: [f32; 3],
    /// A [`Shape`].
    pub shape: u32,
    /// Half width (or radius), height, half depth.
    pub size: [f32; 3],
    /// Turn about the vertical, radians: local `x` points along
    /// `(cos yaw, 0, -sin yaw)`.
    pub yaw: f32,
}

/// `sign` as WGSL has it: zero at zero.
fn sign(x: f32) -> f32 {
    if x > 0.0 {
        1.0
    } else if x < 0.0 {
        -1.0
    } else {
        0.0
    }
}

fn dot2(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[0] + a[1] * b[1]
}

/// A 2D shape of signed distance `d` (across) extruded `half` either way
/// along `w` (the extrusion axis coordinate).
fn extrude(d: f32, w: f32, half: f32) -> f32 {
    let q = [d, w.abs() - half];
    q[0].max(q[1]).min(0.0) + (q[0].max(0.0).powi(2) + q[1].max(0.0).powi(2)).sqrt()
}

/// A cone around `y`, `r1` across at `y = -h` narrowing to `r2` at `y = h`.
fn capped_cone(q: [f32; 2], h: f32, r1: f32, r2: f32) -> f32 {
    let k1 = [r2, h];
    let k2 = [r2 - r1, 2.0 * h];
    let ca = [
        q[0] - q[0].min(if q[1] < 0.0 { r1 } else { r2 }),
        q[1].abs() - h,
    ];
    let t = (dot2([k1[0] - q[0], k1[1] - q[1]], k2) / dot2(k2, k2)).clamp(0.0, 1.0);
    let cb = [q[0] - k1[0] + k2[0] * t, q[1] - k1[1] + k2[1] * t];
    let s = if cb[0] < 0.0 && ca[1] < 0.0 {
        -1.0
    } else {
        1.0
    };
    s * dot2(ca, ca).min(dot2(cb, cb)).sqrt()
}

/// An isosceles triangle with its apex at the origin and its base `q.y`
/// below it (towards `+y`), `2 q.x` wide.
fn isosceles(p: [f32; 2], q: [f32; 2]) -> f32 {
    let p = [p[0].abs(), p[1]];
    let ta = (dot2(p, q) / dot2(q, q)).clamp(0.0, 1.0);
    let a = [p[0] - q[0] * ta, p[1] - q[1] * ta];
    let tb = (p[0] / q[0]).clamp(0.0, 1.0);
    let b = [p[0] - q[0] * tb, p[1] - q[1]];
    let s = -sign(q[1]);
    let d = [
        dot2(a, a).min(dot2(b, b)),
        (s * (p[0] * q[1] - p[1] * q[0])).min(s * (p[1] - q[1])),
    ];
    -d[0].sqrt() * sign(d[1])
}

/// A pyramid on a unit square base centred at the origin, its apex `h` up.
/// Below the base the nearest point is on the base, which leans out no
/// further than any slope does.
fn pyramid(p: [f32; 3], h: f32) -> f32 {
    let m2 = h * h + 0.25;
    let (mut x, mut z) = (p[0].abs(), p[2].abs());
    if p[1] < 0.0 {
        let (dx, dz) = ((x - 0.5).max(0.0), (z - 0.5).max(0.0));
        return (dx * dx + dz * dz + p[1] * p[1]).sqrt();
    }
    if z > x {
        std::mem::swap(&mut x, &mut z);
    }
    x -= 0.5;
    z -= 0.5;
    let y = p[1];
    let q = [z, h * y - 0.5 * x, h * x + 0.5 * y];
    let s = (-q[0]).max(0.0);
    let t = ((q[1] - 0.5 * z) / (m2 + 0.25)).clamp(0.0, 1.0);
    let a = m2 * (q[0] + s) * (q[0] + s) + q[1] * q[1];
    let b = m2 * (q[0] + 0.5 * t) * (q[0] + 0.5 * t) + (q[1] - m2 * t) * (q[1] - m2 * t);
    let d2 = if q[1].min(-q[0] * m2 - q[1] * 0.5) > 0.0 {
        0.0
    } else {
        a.min(b)
    };
    ((d2 + q[2] * q[2]) / m2).sqrt() * sign(q[2].max(-y))
}

impl Solid {
    pub fn new(shape: Shape, base: [f32; 3], size: [f32; 3], yaw: f32) -> Self {
        Self {
            base,
            shape: shape as u32,
            size,
            yaw,
        }
    }

    pub fn shape(&self) -> Option<Shape> {
        Some(match self.shape {
            0 => Shape::Box,
            1 => Shape::Cylinder,
            2 => Shape::Cone,
            3 => Shape::Gable,
            4 => Shape::Pyramid,
            _ => return None,
        })
    }

    /// `p` in the solid's own frame.
    pub fn local(&self, p: [f32; 3]) -> [f32; 3] {
        let (s, c) = self.yaw.sin_cos();
        let d = [
            p[0] - self.base[0],
            p[1] - self.base[1],
            p[2] - self.base[2],
        ];
        [c * d[0] - s * d[2], d[1], s * d[0] + c * d[2]]
    }

    /// A point of the solid's own frame in the world.
    pub fn world(&self, l: [f32; 3]) -> [f32; 3] {
        let (s, c) = self.yaw.sin_cos();
        [
            self.base[0] + c * l[0] + s * l[2],
            self.base[1] + l[1],
            self.base[2] - s * l[0] + c * l[2],
        ]
    }

    /// Signed distance from `p` to the solid's surface.
    pub fn distance(&self, p: [f32; 3]) -> f32 {
        let [x, y, z] = self.local(p);
        let [a, h, b] = self.size;
        match self.shape() {
            Some(Shape::Box) => {
                let q = [x.abs() - a, (y - 0.5 * h).abs() - 0.5 * h, z.abs() - b];
                let outside =
                    (q[0].max(0.0).powi(2) + q[1].max(0.0).powi(2) + q[2].max(0.0).powi(2)).sqrt();
                outside + q[0].max(q[1]).max(q[2]).min(0.0)
            }
            Some(Shape::Cylinder) => extrude((x * x + z * z).sqrt() - a, y - 0.5 * h, 0.5 * h),
            Some(Shape::Cone) => {
                capped_cone([(x * x + z * z).sqrt(), y - 0.5 * h], 0.5 * h, a, 0.0)
            }
            Some(Shape::Gable) => extrude(isosceles([z, h - y], [b, h]), x, a),
            Some(Shape::Pyramid) => {
                let side = 2.0 * a;
                pyramid([x / side, y / side, z / side], h / side) * side
            }
            None => f32::INFINITY,
        }
    }

    /// The radius of a vertical cylinder about the base's middle that holds
    /// the whole solid.
    pub fn reach(&self) -> f32 {
        let [a, _, b] = self.size;
        match self.shape() {
            Some(Shape::Cylinder | Shape::Cone) => a,
            Some(Shape::Pyramid) => a * std::f32::consts::SQRT_2,
            _ => (a * a + b * b).sqrt(),
        }
    }

    /// Lower and upper corner of an upright box holding the solid.
    pub fn bounds(&self) -> [[f32; 3]; 2] {
        let r = self.reach();
        let [x, y, z] = self.base;
        [[x - r, y, z - r], [x + r, y + self.size[1], z + r]]
    }
}

/// [`Solid`], [`Solid::distance`] and [`Solid::reach`] in WGSL: `struct
/// Solid`, `fn solid_distance(solid: Solid, p: vec3<f32>) -> f32` and `fn
/// solid_reach(solid: Solid) -> f32`.
pub const SOLID_WGSL: &str = r#"
struct Solid {
    base: vec3<f32>,
    shape: u32,
    size: vec3<f32>,
    yaw: f32,
};

fn solid_extrude(d: f32, w: f32, half: f32) -> f32 {
    let q = vec2<f32>(d, abs(w) - half);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0)));
}

fn solid_capped_cone(q: vec2<f32>, h: f32, r1: f32, r2: f32) -> f32 {
    let k1 = vec2<f32>(r2, h);
    let k2 = vec2<f32>(r2 - r1, 2.0 * h);
    let ca = vec2<f32>(q.x - min(q.x, select(r2, r1, q.y < 0.0)), abs(q.y) - h);
    let t = clamp(dot(k1 - q, k2) / dot(k2, k2), 0.0, 1.0);
    let cb = q - k1 + k2 * t;
    let s = select(1.0, -1.0, cb.x < 0.0 && ca.y < 0.0);
    return s * sqrt(min(dot(ca, ca), dot(cb, cb)));
}

fn solid_isosceles(p_in: vec2<f32>, q: vec2<f32>) -> f32 {
    let p = vec2<f32>(abs(p_in.x), p_in.y);
    let a = p - q * clamp(dot(p, q) / dot(q, q), 0.0, 1.0);
    let b = p - q * vec2<f32>(clamp(p.x / q.x, 0.0, 1.0), 1.0);
    let s = -sign(q.y);
    let d = vec2<f32>(
        min(dot(a, a), dot(b, b)),
        min(s * (p.x * q.y - p.y * q.x), s * (p.y - q.y)),
    );
    return -sqrt(d.x) * sign(d.y);
}

fn solid_pyramid(p: vec3<f32>, h: f32) -> f32 {
    let m2 = h * h + 0.25;
    var x = abs(p.x);
    var z = abs(p.z);
    if (p.y < 0.0) {
        return length(vec3<f32>(max(x - 0.5, 0.0), p.y, max(z - 0.5, 0.0)));
    }
    if (z > x) {
        let w = x;
        x = z;
        z = w;
    }
    x -= 0.5;
    z -= 0.5;
    let y = p.y;
    let q = vec3<f32>(z, h * y - 0.5 * x, h * x + 0.5 * y);
    let s = max(-q.x, 0.0);
    let t = clamp((q.y - 0.5 * z) / (m2 + 0.25), 0.0, 1.0);
    let a = m2 * (q.x + s) * (q.x + s) + q.y * q.y;
    let b = m2 * (q.x + 0.5 * t) * (q.x + 0.5 * t) + (q.y - m2 * t) * (q.y - m2 * t);
    var d2 = min(a, b);
    if (min(q.y, -q.x * m2 - q.y * 0.5) > 0.0) {
        d2 = 0.0;
    }
    return sqrt((d2 + q.z * q.z) / m2) * sign(max(q.z, -y));
}

fn solid_reach(solid: Solid) -> f32 {
    switch solid.shape {
        case 1u, 2u: {
            return solid.size.x;
        }
        case 4u: {
            return solid.size.x * 1.41421356;
        }
        default: {
            return length(solid.size.xz);
        }
    }
}

fn solid_distance(solid: Solid, p: vec3<f32>) -> f32 {
    let s = sin(solid.yaw);
    let c = cos(solid.yaw);
    let d = p - solid.base;
    let x = c * d.x - s * d.z;
    let y = d.y;
    let z = s * d.x + c * d.z;
    let a = solid.size.x;
    let h = solid.size.y;
    let b = solid.size.z;
    switch solid.shape {
        case 0u: {
            let q = vec3<f32>(abs(x) - a, abs(y - 0.5 * h) - 0.5 * h, abs(z) - b);
            return length(max(q, vec3<f32>(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
        }
        case 1u: {
            return solid_extrude(sqrt(x * x + z * z) - a, y - 0.5 * h, 0.5 * h);
        }
        case 2u: {
            return solid_capped_cone(vec2<f32>(sqrt(x * x + z * z), y - 0.5 * h), 0.5 * h, a, 0.0);
        }
        case 3u: {
            return solid_extrude(solid_isosceles(vec2<f32>(z, h - y), vec2<f32>(b, h)), x, a);
        }
        case 4u: {
            let side = 2.0 * a;
            return solid_pyramid(vec3<f32>(x, y, z) / side, h / side) * side;
        }
        default: {
            return 1e30;
        }
    }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    #[test]
    fn a_box_measures_from_its_faces_edges_and_corners() {
        let solid = Solid::new(Shape::Box, [1.0, 2.0, 3.0], [2.0, 4.0, 1.0], 0.0);
        // Inside: the nearest face.
        assert!(close(solid.distance([1.0, 4.0, 3.0]), -1.0));
        // Off a face, an edge, a corner.
        assert!(close(solid.distance([1.0, 4.0, 5.0]), 1.0));
        assert!(close(solid.distance([4.0, 4.0, 5.0]), 2f32.sqrt()));
        assert!(close(solid.distance([4.0, 7.0, 5.0]), 3f32.sqrt()));
        // Below its base.
        assert!(close(solid.distance([1.0, 1.0, 3.0]), 1.0));
    }

    #[test]
    fn turning_a_solid_turns_its_distances() {
        let yaw = 0.7;
        let solid = Solid::new(Shape::Box, [0.0; 3], [3.0, 2.0, 0.5], yaw);
        // A point off the end of its long axis, wherever that now points.
        let end = solid.world([4.0, 1.0, 0.0]);
        assert!(close(solid.distance(end), 1.0));
        assert!(close(end[0], 4.0 * yaw.cos()) && close(end[2], -4.0 * yaw.sin()));
        assert!(close(solid.local(end)[0], 4.0));
    }

    #[test]
    fn round_solids_measure_radially_and_along_their_slant() {
        let drum = Solid::new(Shape::Cylinder, [0.0; 3], [2.0, 10.0, 0.0], 0.0);
        assert!(close(drum.distance([5.0, 5.0, 0.0]), 3.0));
        assert!(close(drum.distance([0.0, 5.0, 0.0]), -2.0));
        assert!(close(drum.distance([0.0, 12.0, 0.0]), 2.0));
        // A cone of slope 45 degrees: from a point on its axis above the
        // apex, the apex; from beside its flank, along the flank's normal.
        let cone = Solid::new(Shape::Cone, [0.0; 3], [1.0, 1.0, 0.0], 0.0);
        assert!(close(cone.distance([0.0, 3.0, 0.0]), 2.0));
        let flank = [1.0, 1.0, 0.0];
        assert!(close(cone.distance(flank), 0.5f32.sqrt()));
        assert!(cone.distance([0.2, 0.2, 0.0]) < 0.0);
    }

    #[test]
    fn a_gable_and_a_pyramid_are_their_slopes() {
        // A ridge 1 high over eaves 1 either side: 45 degree slopes.
        let gable = Solid::new(Shape::Gable, [0.0; 3], [5.0, 1.0, 1.0], 0.0);
        assert!(close(gable.distance([0.0, 2.0, 0.0]), 1.0));
        assert!(close(gable.distance([0.0, 1.0, 1.0]), 0.5f32.sqrt()));
        assert!(gable.distance([0.0, 0.3, 0.0]) < 0.0);
        assert!(close(gable.distance([6.0, 0.5, 0.0]), 1.0));
        let pyramid = Solid::new(Shape::Pyramid, [0.0; 3], [1.0, 1.0, 1.0], 0.0);
        assert!(close(pyramid.distance([0.0, 2.0, 0.0]), 1.0));
        assert!(close(pyramid.distance([0.0, -1.0, 0.0]), 1.0));
        assert!(close(pyramid.distance([1.0, 1.0, 0.0]), 0.5f32.sqrt()));
        assert!(pyramid.distance([0.0, 0.5, 0.0]) < 0.0);
    }

    #[test]
    fn the_wgsl_validates() {
        let source = format!(
            "{SOLID_WGSL}\n@group(0) @binding(0) var<storage, read> solids: array<Solid>;\n\
             @group(0) @binding(1) var<storage, read_write> out: array<f32>;\n\
             @compute @workgroup_size(1) fn main() {{ out[0] = solid_distance(solids[0], vec3<f32>(1.0)); }}"
        );
        let module = naga::front::wgsl::parse_str(&source)
            .unwrap_or_else(|e| panic!("{}", e.emit_to_string(&source)));
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .unwrap_or_else(|e| panic!("{}", e.emit_to_string(&source)));
    }
}
