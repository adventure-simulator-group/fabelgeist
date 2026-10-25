use fabelgeist_math::Vec3;

use super::*;
use crate::aabb::closest_point_on_triangle;
use crate::tests::{Random, brute_force_overlaps, scattered_boxes, sphere_mesh};

/// Every invariant the traversals rely on. Checked structurally rather than by
/// sampling, because a BVH that loses a subtree still answers most queries
/// correctly and only shows up on the query you did not run.
fn check_structure(bvh: &Bvh, bounds: &[Aabb]) {
    if bounds.is_empty() {
        assert!(bvh.nodes.is_empty());
        return;
    }

    assert_eq!(bvh.indices.len(), bounds.len(), "lost or gained primitives");
    let mut seen = vec![false; bounds.len()];
    for &index in &bvh.indices {
        assert!(!seen[index as usize], "primitive {index} appears twice");
        seen[index as usize] = true;
    }
    assert!(seen.iter().all(|&s| s), "indices are not a permutation");

    let mut reached = vec![false; bounds.len()];
    let mut stack = vec![0usize];
    let mut visited_nodes = 0usize;
    while let Some(index) = stack.pop() {
        visited_nodes += 1;
        let node = bvh.nodes[index];
        if node.is_leaf() {
            let first = node.left_first as usize;
            assert!(node.count > 0);
            for &primitive in &bvh.indices[first..first + node.count as usize] {
                reached[primitive as usize] = true;
                assert!(
                    node.bounds.union(bounds[primitive as usize]) == node.bounds,
                    "leaf {index} does not contain primitive {primitive}"
                );
            }
        } else {
            let left = node.left_first as usize;
            assert!(
                left > index,
                "child {left} does not come after parent {index}, so the reverse-order refit would be wrong"
            );
            let combined = bvh.nodes[left].bounds.union(bvh.nodes[left + 1].bounds);
            assert!(
                node.bounds.union(combined) == node.bounds,
                "node {index} does not contain its children"
            );
            stack.push(left);
            stack.push(left + 1);
        }
    }
    assert!(reached.iter().all(|&r| r), "a primitive is unreachable");
    assert_eq!(
        visited_nodes,
        bvh.nodes.len(),
        "some nodes are not reachable from the root"
    );
}

#[test]
fn builds_a_sound_tree() {
    for count in [1, 2, 3, 5, 17, 64, 1000] {
        let bounds = scattered_boxes(count, 3 + count as u32);
        let bvh = Bvh::build(&bounds);
        check_structure(&bvh, &bounds);
    }
}

#[test]
fn builds_over_no_primitives() {
    let bvh = Bvh::build(&[]);
    assert!(bvh.is_empty());
    // A query on an empty tree is a no-op, not a panic.
    bvh.query_aabb(&[], &Aabb::new(Vec3::splat(-1.0), Vec3::splat(1.0)), |_| {
        panic!("an empty tree has nothing to report")
    });
}

/// Coincident boxes are the case that makes a naive median split loop forever
/// and a Morton build degenerate.
#[test]
fn builds_over_coincident_primitives() {
    let bounds = vec![Aabb::new(Vec3::splat(0.0), Vec3::splat(1.0)); 500];
    let bvh = Bvh::build(&bounds);
    check_structure(&bvh, &bounds);
}

#[test]
fn finds_every_overlap() {
    let bounds = scattered_boxes(2000, 29);
    let bvh = Bvh::build(&bounds);
    let mut random = Random::new(31);

    for _ in 0..200 {
        let center = random.point(-12.0, 12.0);
        let half = Vec3::splat(random.range(0.1, 4.0));
        let query = Aabb::new(center - half, center + half);

        let mut found = Vec::new();
        bvh.query_aabb(&bounds, &query, |index| found.push(index));
        found.sort_unstable();

        assert_eq!(found, brute_force_overlaps(&bounds, &query));
    }
}

#[test]
fn refit_matches_a_rebuild() {
    let bounds = scattered_boxes(500, 37);
    let mut bvh = Bvh::build(&bounds);

    // Move everything, keeping the topology.
    let mut random = Random::new(41);
    let moved: Vec<Aabb> = bounds
        .iter()
        .map(|b| {
            let shift = random.point(-0.5, 0.5);
            Aabb::new(b.min + shift, b.max + shift)
        })
        .collect();

    bvh.refit(&moved);
    check_structure(&bvh, &moved);

    // A refit tree still answers every query correctly, even though its
    // topology is now the old geometry's.
    let query = Aabb::new(Vec3::splat(-2.0), Vec3::splat(2.0));
    let mut found = Vec::new();
    bvh.query_aabb(&moved, &query, |index| found.push(index));
    found.sort_unstable();
    assert_eq!(found, brute_force_overlaps(&moved, &query));
}

#[test]
fn closest_point_on_a_sphere_matches_the_analytic_surface() {
    let radius = 2.0;
    let (positions, triangles) = sphere_mesh(48, 96, radius);
    let mesh = TriangleBvh::new(positions, triangles);

    let mut random = Random::new(43);
    for _ in 0..300 {
        let direction = random.point(-1.0, 1.0);
        if direction.length() < 1e-3 {
            continue;
        }
        let distance_from_center = random.range(3.0, 8.0);
        let point = direction.normalize() * distance_from_center;

        let (_, closest, distance) = mesh
            .closest_point(point, 100.0)
            .expect("a point outside the sphere has a nearest surface point");

        // The mesh is inscribed in the sphere, so its surface sits slightly
        // inside; a 48x96 tessellation is within about a millimetre of it.
        let expected = distance_from_center - radius;
        assert!(
            (distance - expected).abs() < 0.01,
            "distance {distance} is not near {expected}"
        );
        assert!(
            (closest.length() - radius).abs() < 0.01,
            "the closest point is not on the sphere"
        );
    }
}

#[test]
fn closest_point_matches_brute_force() {
    let (positions, triangles) = sphere_mesh(8, 12, 1.0);
    let mesh = TriangleBvh::new(positions.clone(), triangles.clone());

    let mut random = Random::new(47);
    for _ in 0..200 {
        let point = random.point(-3.0, 3.0);

        let brute = triangles
            .iter()
            .map(|&[a, b, c]| {
                let candidate = closest_point_on_triangle(
                    point,
                    positions[a as usize],
                    positions[b as usize],
                    positions[c as usize],
                );
                (candidate - point).length()
            })
            .fold(f32::INFINITY, f32::min);

        let (_, _, distance) = mesh.closest_point(point, 100.0).expect("mesh is not empty");
        assert!(
            (distance - brute).abs() < 1e-4,
            "BVH says {distance}, brute force says {brute}"
        );
    }
}

/// Nothing within the radius means nothing is reported -- the case a solver
/// relies on to skip a particle entirely.
#[test]
fn closest_point_respects_its_limit() {
    let (positions, triangles) = sphere_mesh(8, 12, 1.0);
    let mesh = TriangleBvh::new(positions, triangles);
    assert!(mesh.closest_point(Vec3::new(0.0, 50.0, 0.0), 1.0).is_none());
    assert!(
        mesh.closest_point(Vec3::new(0.0, 50.0, 0.0), 100.0)
            .is_some()
    );
}

#[test]
fn raycast_finds_the_nearest_hit() {
    let (positions, triangles) = sphere_mesh(32, 64, 1.0);
    let mesh = TriangleBvh::new(positions, triangles);

    // Straight at the middle from far away: the near face, not the far one.
    let ray = Ray::new(Vec3::new(0.0, 0.0, -5.0), Vec3::new(0.0, 0.0, 1.0));
    let (_, distance) = mesh
        .raycast(&ray, 100.0)
        .expect("the ray crosses the sphere");
    assert!(
        (distance - 4.0).abs() < 0.02,
        "expected to enter at 4.0, got {distance}"
    );

    // Aimed past it.
    let miss = Ray::new(Vec3::new(0.0, 3.0, -5.0), Vec3::new(0.0, 0.0, 1.0));
    assert!(mesh.raycast(&miss, 100.0).is_none());

    // Pointed the wrong way.
    let away = Ray::new(Vec3::new(0.0, 0.0, -5.0), Vec3::new(0.0, 0.0, -1.0));
    assert!(mesh.raycast(&away, 100.0).is_none());
}

#[test]
fn raycast_respects_its_limit() {
    let (positions, triangles) = sphere_mesh(16, 32, 1.0);
    let mesh = TriangleBvh::new(positions, triangles);
    let ray = Ray::new(Vec3::new(0.0, 0.0, -5.0), Vec3::new(0.0, 0.0, 1.0));
    assert!(mesh.raycast(&ray, 3.0).is_none(), "the hit is at 4.0");
    assert!(mesh.raycast(&ray, 5.0).is_some());
}

#[test]
fn updating_positions_keeps_queries_correct() {
    let (positions, triangles) = sphere_mesh(16, 32, 1.0);
    let mut mesh = TriangleBvh::new(positions.clone(), triangles);

    // Scale the sphere up; the topology is unchanged, so this is a refit.
    let scaled: Vec<Vec3> = positions.iter().map(|&p| p * 2.0).collect();
    mesh.update_positions(scaled);

    let ray = Ray::new(Vec3::new(0.0, 0.0, -5.0), Vec3::new(0.0, 0.0, 1.0));
    let (_, distance) = mesh.raycast(&ray, 100.0).expect("still hits");
    assert!(
        (distance - 3.0).abs() < 0.05,
        "a radius-2 sphere is entered at 3.0, not {distance}"
    );
}

/// The vertex, edge and face regions of the closest-point test, each pinned
/// with a case that only that branch answers correctly.
#[test]
fn closest_point_on_triangle_covers_every_region() {
    let a = Vec3::new(0.0, 0.0, 0.0);
    let b = Vec3::new(1.0, 0.0, 0.0);
    let c = Vec3::new(0.0, 1.0, 0.0);

    // Directly above the interior: straight down onto the face.
    let inside = closest_point_on_triangle(Vec3::new(0.25, 0.25, 1.0), a, b, c);
    assert!((inside - Vec3::new(0.25, 0.25, 0.0)).length() < 1e-5);

    // Past each vertex.
    assert!((closest_point_on_triangle(Vec3::new(-1.0, -1.0, 0.0), a, b, c) - a).length() < 1e-5);
    assert!((closest_point_on_triangle(Vec3::new(3.0, -1.0, 0.0), a, b, c) - b).length() < 1e-5);
    assert!((closest_point_on_triangle(Vec3::new(-1.0, 3.0, 0.0), a, b, c) - c).length() < 1e-5);

    // Beside each edge.
    let on_ab = closest_point_on_triangle(Vec3::new(0.5, -1.0, 0.0), a, b, c);
    assert!((on_ab - Vec3::new(0.5, 0.0, 0.0)).length() < 1e-5);
    let on_ac = closest_point_on_triangle(Vec3::new(-1.0, 0.5, 0.0), a, b, c);
    assert!((on_ac - Vec3::new(0.0, 0.5, 0.0)).length() < 1e-5);
    let on_bc = closest_point_on_triangle(Vec3::new(1.0, 1.0, 0.0), a, b, c);
    assert!((on_bc - Vec3::new(0.5, 0.5, 0.0)).length() < 1e-5);

    // A degenerate triangle answers something finite rather than dividing by
    // zero.
    let degenerate = closest_point_on_triangle(Vec3::new(5.0, 5.0, 5.0), a, a, a);
    assert!(degenerate.is_finite());
}

#[test]
fn aabb_arithmetic() {
    assert!(Aabb::EMPTY.is_empty());
    assert_eq!(Aabb::EMPTY.surface_area(), 0.0);
    assert_eq!(
        Aabb::EMPTY.union(Aabb::point(Vec3::splat(1.0))).min,
        Vec3::splat(1.0)
    );

    let unit = Aabb::new(Vec3::splat(0.0), Vec3::splat(1.0));
    assert_eq!(unit.surface_area(), 6.0);
    assert_eq!(unit.center(), Vec3::splat(0.5));
    assert!(unit.contains(Vec3::splat(0.5)));
    assert!(!unit.contains(Vec3::splat(1.5)));
    assert_eq!(unit.distance_squared(Vec3::splat(0.5)), 0.0);
    assert!((unit.distance_squared(Vec3::new(0.5, 0.5, 3.0)) - 4.0).abs() < 1e-6);

    // Touching counts as overlapping: a contact at exactly zero separation is
    // still a contact.
    let touching = Aabb::new(Vec3::new(1.0, 0.0, 0.0), Vec3::new(2.0, 1.0, 1.0));
    assert!(unit.overlaps(&touching));

    // A degenerate axis normalises to the middle rather than to a NaN.
    let flat = Aabb::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 1.0));
    let normalized = flat.normalize(Vec3::new(0.5, 0.0, 0.5));
    assert!(normalized.is_finite());
    assert_eq!(normalized.y, 0.5);
}

#[test]
fn ray_box_test_handles_axis_aligned_rays() {
    let unit = Aabb::new(Vec3::splat(0.0), Vec3::splat(1.0));

    // A ray parallel to two axes gives infinities in the slab test; it must
    // still hit.
    let along_x = Ray::new(Vec3::new(-2.0, 0.5, 0.5), Vec3::new(1.0, 0.0, 0.0));
    assert!((along_x.hits(&unit, 100.0).unwrap() - 2.0).abs() < 1e-5);

    // Parallel but outside the slab: a miss, not a hit at infinity.
    let beside = Ray::new(Vec3::new(-2.0, 5.0, 0.5), Vec3::new(1.0, 0.0, 0.0));
    assert!(beside.hits(&unit, 100.0).is_none());

    // Starting inside reports zero.
    let inside = Ray::new(Vec3::splat(0.5), Vec3::new(1.0, 0.0, 0.0));
    assert_eq!(inside.hits(&unit, 100.0), Some(0.0));

    // A flat box -- a triangle's own bounds -- is still hit edge-on.
    let flat = Aabb::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 1.0));
    let onto_flat = Ray::new(Vec3::new(0.5, 2.0, 0.5), Vec3::new(0.0, -1.0, 0.0));
    assert!(onto_flat.hits(&flat, 100.0).is_some());
}
