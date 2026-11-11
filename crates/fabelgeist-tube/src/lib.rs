//! Generalized cylinders.
//!
//! A skeleton -- the branches of a tree, a vine, a strand of hair, a blood
//! vessel -- is a set of segments with a radius at each end. Its surface is a
//! truncated cone around every segment. This crate sweeps those cones.
//!
//! Every segment is meshed on its own: a ring of `sides + 1` vertices at each
//! end (the extra one carries the texture seam) and `2 * sides` triangles
//! between them. No segment depends on another, so the whole skeleton is one
//! embarrassingly parallel pass -- [`gpu::TubeMesher`] runs it as a single
//! compute dispatch straight into the buffers a [`fabelgeist_mesh::GpuMesh`]
//! draws from, and [`mesh`] is the same thing on the host. Joints are left to
//! the overlap of neighbouring cones, which at the tapering radii of a
//! branching structure is invisible and costs nothing.
//!
//! The ring frame comes from the branchless orthonormal basis of Duff et al.
//! (2017), which is continuous everywhere except straight down the negative
//! axis, so consecutive segments of a smoothly curving strand get rings that
//! barely twist relative to each other.

#[cfg(feature = "gpu")]
pub mod gpu;

use fabelgeist_mesh::{FrontFace, MeshData, PrimitiveTopology};

/// One segment of a skeleton: a truncated cone from `start` to `end`.
///
/// Laid out as two `vec4<f32>` so that a GPU buffer of these reads directly
/// in WGSL.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TubeSegment {
    pub start: [f32; 3],
    pub start_radius: f32,
    pub end: [f32; 3],
    pub end_radius: f32,
}

impl TubeSegment {
    pub fn new(start: [f32; 3], start_radius: f32, end: [f32; 3], end_radius: f32) -> Self {
        Self {
            start,
            start_radius,
            end,
            end_radius,
        }
    }
}

/// Vertices one segment emits.
pub const fn vertices_per_segment(sides: u32) -> u32 {
    2 * (sides + 1)
}

/// Indices one segment emits.
pub const fn indices_per_segment(sides: u32) -> u32 {
    6 * sides
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn length(a: [f32; 3]) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

/// Duff et al.'s branchless orthonormal basis around a unit `n`.
pub fn orthonormal_basis(n: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let sign = 1.0f32.copysign(n[2]);
    let a = -1.0 / (sign + n[2]);
    let b = n[0] * n[1] * a;
    (
        [1.0 + sign * n[0] * n[0] * a, sign * b, -sign * n[0]],
        [b, sign + n[1] * n[1] * a, -n[1]],
    )
}

/// Sweep every segment into one triangle mesh, `sides` quads around each.
pub fn mesh(segments: &[TubeSegment], sides: u32) -> MeshData {
    let sides = sides.max(3);
    let ring = sides + 1;
    let vertex_count = segments.len() * vertices_per_segment(sides) as usize;
    let mut positions = Vec::with_capacity(vertex_count);
    let mut normals = Vec::with_capacity(vertex_count);
    let mut tex_coords = Vec::with_capacity(vertex_count);
    let mut indices = Vec::with_capacity(segments.len() * indices_per_segment(sides) as usize);

    for (index, segment) in segments.iter().enumerate() {
        let axis = sub(segment.end, segment.start);
        let len = length(axis);
        let t = if len > 1e-12 {
            [axis[0] / len, axis[1] / len, axis[2] / len]
        } else {
            [0.0, 1.0, 0.0]
        };
        let (b1, b2) = orthonormal_basis(t);
        // A cone's normal leans back along the axis by its taper.
        let slope = if len > 1e-12 {
            (segment.start_radius - segment.end_radius) / len
        } else {
            0.0
        };

        for (end, center, radius) in [
            (0.0, segment.start, segment.start_radius),
            (len, segment.end, segment.end_radius),
        ] {
            for k in 0..ring {
                let theta = std::f32::consts::TAU * k as f32 / sides as f32;
                let (s, c) = theta.sin_cos();
                let n = [
                    c * b1[0] + s * b2[0],
                    c * b1[1] + s * b2[1],
                    c * b1[2] + s * b2[2],
                ];
                positions.push([
                    center[0] + radius * n[0],
                    center[1] + radius * n[1],
                    center[2] + radius * n[2],
                ]);
                let normal = [
                    n[0] + slope * t[0],
                    n[1] + slope * t[1],
                    n[2] + slope * t[2],
                ];
                let inverse = 1.0 / length(normal).max(1e-12);
                normals.push([
                    normal[0] * inverse,
                    normal[1] * inverse,
                    normal[2] * inverse,
                ]);
                tex_coords.push([k as f32 / sides as f32, end]);
            }
        }

        let base = index as u32 * vertices_per_segment(sides);
        for k in 0..sides {
            let a = base + k;
            let b = a + 1;
            let c = a + ring;
            let d = c + 1;
            indices.extend_from_slice(&[a, b, c, b, d, c]);
        }
    }

    MeshData {
        positions,
        normals,
        tex_coords,
        indices: Some(indices),
        joints: None,
        weights: None,
        topology: PrimitiveTopology::TriangleList,
        front_face: FrontFace::Ccw,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basis_is_orthonormal_everywhere() {
        for n in [
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, -1.0],
            [0.6, 0.0, 0.8],
            [-0.48, 0.6, -0.64],
        ] {
            let (a, b) = orthonormal_basis(n);
            let dot = |x: [f32; 3], y: [f32; 3]| x[0] * y[0] + x[1] * y[1] + x[2] * y[2];
            assert!((dot(a, a) - 1.0).abs() < 1e-5);
            assert!((dot(b, b) - 1.0).abs() < 1e-5);
            assert!(dot(a, b).abs() < 1e-5 && dot(a, n).abs() < 1e-5 && dot(b, n).abs() < 1e-5);
            // Right-handed: a x b = n, which the winding relies on.
            let cross = [
                a[1] * b[2] - a[2] * b[1],
                a[2] * b[0] - a[0] * b[2],
                a[0] * b[1] - a[1] * b[0],
            ];
            assert!((dot(cross, n) - 1.0).abs() < 1e-5);
        }
    }

    #[test]
    fn a_cylinder_has_its_radius_and_faces_outwards() {
        let segment = TubeSegment::new([0.0, 0.0, 0.0], 0.5, [0.0, 2.0, 0.0], 0.5);
        let data = mesh(&[segment], 8);
        data.validate().unwrap();
        assert_eq!(data.positions.len(), 18);
        for (p, n) in data.positions.iter().zip(&data.normals) {
            assert!(((p[0] * p[0] + p[2] * p[2]).sqrt() - 0.5).abs() < 1e-5);
            assert!(n[0] * p[0] + n[2] * p[2] > 0.0);
        }
        // Every triangle's geometric normal agrees with its vertex normals.
        for t in data.triangles().unwrap() {
            let [a, b, c] = t.map(|i| data.positions[i as usize]);
            let e1 = sub(b, a);
            let e2 = sub(c, a);
            let g = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            let n = data.normals[t[0] as usize];
            assert!(g[0] * n[0] + g[1] * n[1] + g[2] * n[2] > 0.0);
        }
    }
}
