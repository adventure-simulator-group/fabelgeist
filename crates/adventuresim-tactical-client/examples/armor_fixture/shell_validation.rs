//! Independent triangle checks on generated walls, beyond edge closure.
use fabelgeist_armor::{GeneratedArmor, PlateFace};
use fabelgeist_bvh::{Aabb, TriangleBvh};
use fabelgeist_math::Vec3;

const CONTACT_DISTANCE_TOLERANCE_METERS: f64 = 1e-10;
const TRIANGLE_NORMAL_DIRECTION_TOLERANCE: f64 = 1e-4;
mod manifold;
mod shared_contact;
pub(super) use manifold::assert_closed_oriented_manifold;

/// Small facets are usable when their f32 direction resolves the actual face.
/// An area floor alone cannot distinguish a small face from a collapsed one.
pub(super) fn assert_triangle_normal_resolved(points: [[f32; 3]; 3]) {
    let [a, b, c] = points.map(Vec3::from_array);
    let normal = (b - a).cross(c - a);
    let length = normal.length_squared().sqrt();
    assert!(length.is_finite() && length > 0.0);
    let [a, b, c] = points.map(|p| p.map(f64::from));
    let precise = cross(
        std::array::from_fn(|k| b[k] - a[k]),
        std::array::from_fn(|k| c[k] - a[k]),
    );
    let precise_length = dot(precise, precise).sqrt();
    let error = normal
        .to_array()
        .into_iter()
        .zip(precise)
        .map(|(found, expected)| {
            (f64::from(found) / f64::from(length) - expected / precise_length).powi(2)
        })
        .sum::<f64>();
    assert!(error < TRIANGLE_NORMAL_DIRECTION_TOLERANCE.powi(2));
}

pub(super) fn assert_no_shell_crossings(armor: &GeneratedArmor) {
    let triangles = armor.indices.as_chunks::<3>().0.to_vec();
    let mesh = TriangleBvh::new(
        armor
            .positions
            .iter()
            .copied()
            .map(Vec3::from_array)
            .collect(),
        triangles.clone(),
    );
    for (id, face) in triangles.into_iter().enumerate() {
        let points = face.map(|i| Vec3::from_array(armor.positions[i as usize]));
        mesh.bvh
            .candidates_aabb(&Aabb::from_points(points), |other| {
                if other as usize <= id {
                    return;
                }
                let (a, b, c) = mesh.triangle(other);
                if shared_contact::beyond_shared_feature(points, [a, b, c]) {
                    assert!(
                        !intersect(points, [a, b, c]),
                        "shell triangles {id} and {other} intersect: {points:?} {:?}",
                        [a, b, c]
                    );
                }
            });
    }
}

pub(super) fn assert_walls_disjoint(armor: &GeneratedArmor) {
    let ranges = if armor.components.is_empty() {
        vec![0..armor.indices.len()]
    } else {
        armor
            .components
            .iter()
            .map(|part| part.indices.clone())
            .collect()
    };
    for (component, range) in ranges.into_iter().enumerate() {
        let wall = |kind| {
            armor.indices[range.clone()]
                .chunks_exact(3)
                .zip(&armor.faces[range.start / 3..range.end / 3])
                .filter(|(_, face)| **face == kind)
                .map(|(face, _)| [face[0], face[1], face[2]])
                .collect::<Vec<_>>()
        };
        let inner = TriangleBvh::new(
            armor
                .positions
                .iter()
                .copied()
                .map(Vec3::from_array)
                .collect(),
            wall(PlateFace::Inner),
        );
        for (outer_id, face) in wall(PlateFace::Outer).into_iter().enumerate() {
            let outer = face.map(|i| Vec3::from_array(armor.positions[i as usize]));
            inner.bvh.candidates_aabb(&Aabb::from_points(outer), |inner_id| {
                let (a, b, c) = inner.triangle(inner_id);
                assert!(
                    !intersect([a, b, c], outer),
                    "component {component} inner {inner_id} crosses outer {outer_id}: {:?} {outer:?}",
                    [a, b, c],
                );
            });
        }
    }
}

fn intersect(a: [Vec3; 3], b: [Vec3; 3]) -> bool {
    let a = a.map(|p| p.to_array().map(f64::from));
    let b = b.map(|p| p.to_array().map(f64::from));
    let edges = |p: [[f64; 3]; 3]| {
        std::array::from_fn::<_, 3, _>(|i| {
            std::array::from_fn::<_, 3, _>(|k| p[(i + 1) % 3][k] - p[i][k])
        })
    };
    let ae = edges(a);
    let be = edges(b);
    let normals = [cross(ae[0], ae[1]), cross(be[0], be[1])];
    let separated = |axis: [f64; 3]| {
        let length = dot(axis, axis).sqrt();
        if length == 0.0 {
            return false;
        }
        let axis = axis.map(|v| v / length);
        let interval = |p: [[f64; 3]; 3]| {
            p.map(|p| dot(p, axis))
                .into_iter()
                .fold((f64::INFINITY, f64::NEG_INFINITY), |(low, high), v| {
                    (low.min(v), high.max(v))
                })
        };
        let (al, ah) = interval(a);
        let (bl, bh) = interval(b);
        // Ten orders below a metre separates rounding noise from contact.
        ah < bl - CONTACT_DISTANCE_TOLERANCE_METERS || bh < al - CONTACT_DISTANCE_TOLERANCE_METERS
    };
    !normals.into_iter().any(separated)
        && !ae
            .into_iter()
            .any(|a| be.into_iter().any(|b| separated(cross(a, b))))
        && !ae.into_iter().any(|e| separated(cross(normals[0], e)))
        && !be.into_iter().any(|e| separated(cross(normals[1], e)))
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    (0..3).map(|k| a[k] * b[k]).sum()
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[test]
fn concave_closed_metal_does_not_require_every_offset_to_support_its_outer_face() {
    let positions = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [0.0, 0.1, 0.19_f32.sqrt()],
    ];
    let [a, b, c] = [3, 4, 5].map(|i| Vec3::from_array(positions[i]));
    let offset = c - Vec3::from_array(positions[2]);
    assert!((b - a).cross(c - a).dot(offset) < 0.0);
    let armor = GeneratedArmor {
        positions,
        indices: vec![
            0, 2, 1, 3, 4, 5, 0, 1, 4, 0, 4, 3, 1, 2, 5, 1, 5, 4, 2, 0, 5, 0, 3, 5,
        ],
        faces: vec![
            PlateFace::Inner,
            PlateFace::Outer,
            PlateFace::Edge,
            PlateFace::Edge,
            PlateFace::Edge,
            PlateFace::Edge,
            PlateFace::Edge,
            PlateFace::Edge,
        ],
        components: Vec::new(),
        design_hash: [0; 32],
        surface_domain: "concave-metal-fixture".into(),
        normals: Vec::new(),
        texcoords: Vec::new(),
        joint_indices: Vec::new(),
        joint_weights: Vec::new(),
        trim: None,
        grids: Vec::new(),
        morphs: Vec::new(),
    };
    assert_closed_oriented_manifold(&armor);
    assert_walls_disjoint(&armor);
    assert_no_shell_crossings(&armor);
}

#[test]
fn wall_crossing_check_includes_coplanar_overlap_and_tiny_triangles() {
    let triangle = [[0.0, 0.0, 0.0], [1e-5, 0.0, 0.0], [0.0, 1e-5, 0.0]].map(Vec3::from_array);
    assert!(intersect(triangle, triangle));
    assert!(intersect(
        triangle,
        triangle.map(|p| p + Vec3::new(1e-6, 0.0, 0.0))
    ));
    assert!(!intersect(
        triangle,
        triangle.map(|p| p + Vec3::new(0.0, 0.0, 1e-6))
    ));
    let crossing =
        [[2e-6, 2e-6, -1e-6], [2e-6, 2e-6, 1e-6], [4e-6, 2e-6, 1e-6]].map(Vec3::from_array);
    assert!(intersect(triangle, crossing));
}
