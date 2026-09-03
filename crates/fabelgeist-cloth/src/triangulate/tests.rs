use super::*;

fn rectangle(width: f32, height: f32) -> Vec<Vec2> {
    vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(width, 0.0),
        Vec2::new(width, height),
        Vec2::new(0.0, height),
    ]
}

/// A panel with a hole cut out of one side -- an armhole, in effect. The
/// concavity is what separates a real triangulation from a convex-hull one.
fn notched() -> Vec<Vec2> {
    vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(0.6, 1.0),
        Vec2::new(0.6, 0.4),
        Vec2::new(0.4, 0.4),
        Vec2::new(0.4, 1.0),
        Vec2::new(0.0, 1.0),
    ]
}

fn total_area(mesh: &PanelMesh) -> f32 {
    mesh.triangles
        .iter()
        .map(|t| {
            let a = mesh.vertices[t[0] as usize];
            let b = mesh.vertices[t[1] as usize];
            let c = mesh.vertices[t[2] as usize];
            ((b - a).x * (c - a).y - (b - a).y * (c - a).x).abs() * 0.5
        })
        .sum()
}

/// Every invariant the cloth build relies on.
fn check(mesh: &PanelMesh, outline: &[Vec2]) {
    assert!(!mesh.triangles.is_empty(), "no triangles");

    for triangle in &mesh.triangles {
        assert!(
            triangle.iter().all(|&v| (v as usize) < mesh.vertices.len()),
            "a triangle indexes past the vertices"
        );
        let a = mesh.vertices[triangle[0] as usize];
        let b = mesh.vertices[triangle[1] as usize];
        let c = mesh.vertices[triangle[2] as usize];
        let cross = (b - a).x * (c - a).y - (b - a).y * (c - a).x;
        assert!(
            cross > 0.0,
            "triangle {triangle:?} is clockwise or degenerate (2A = {cross})"
        );
    }

    // The chains have to cover the outline: one chain per input edge, sharing
    // endpoints, closing the loop.
    assert_eq!(mesh.edge_chains.len(), outline.len());
    for (index, chain) in mesh.edge_chains.iter().enumerate() {
        assert!(chain.len() >= 2, "edge {index} has no chain");
        let next = &mesh.edge_chains[(index + 1) % outline.len()];
        assert_eq!(
            *chain.last().unwrap(),
            next[0],
            "edge {index} does not hand over to the next"
        );
    }
    assert_eq!(
        mesh.edge_chains[0][0],
        *mesh.edge_chains.last().unwrap().last().unwrap(),
        "the boundary does not close"
    );

    // Every vertex must be used, or the cloth carries particles nothing holds.
    let mut used = vec![false; mesh.vertices.len()];
    for triangle in &mesh.triangles {
        for &v in triangle {
            used[v as usize] = true;
        }
    }
    assert!(
        used.iter().all(|&u| u),
        "{} vertices are in no triangle",
        used.iter().filter(|&&u| !u).count()
    );

    // The triangles must tile the outline, not overlap it. Overlap is what a
    // Delaunay build produces when its in-circle test answers a tie
    // inconsistently, and it is invisible in every other check here: the
    // triangles are all inside, all wound correctly, and every vertex is used.
    // Only the total area gives it away.
    let enclosed = signed_area(outline).abs();
    let covered = total_area(mesh);
    assert!(
        (covered - enclosed).abs() < enclosed * 0.05,
        "the mesh covers {covered} but the outline encloses {enclosed};          triangles are overlapping or missing"
    );
}

#[test]
fn meshes_a_rectangle() {
    let outline = rectangle(1.0, 0.6);
    let mesh = triangulate(&outline, 0.08);
    check(&mesh, &outline);

    let area = total_area(&mesh);
    assert!(
        (area - 0.6).abs() < 0.6 * 0.02,
        "the mesh covers {area}, the rectangle is 0.6"
    );
}

/// The resolution knob has to actually do something, and roughly as expected:
/// halving the edge length should roughly quadruple the vertex count.
#[test]
fn resolution_scales_with_the_target_edge() {
    let outline = rectangle(1.0, 1.0);
    let coarse = triangulate(&outline, 0.2);
    let fine = triangulate(&outline, 0.1);
    check(&coarse, &outline);
    check(&fine, &outline);

    let ratio = fine.vertices.len() as f32 / coarse.vertices.len() as f32;
    assert!(
        (2.5..=6.0).contains(&ratio),
        "halving the edge changed the vertex count by {ratio}x"
    );
}

#[test]
fn edges_are_near_the_target_length() {
    let outline = rectangle(1.0, 1.0);
    let target = 0.1f32;
    let mesh = triangulate(&outline, target);

    let mut lengths = Vec::new();
    for triangle in &mesh.triangles {
        for pair in [[0, 1], [1, 2], [2, 0]] {
            let a = mesh.vertices[triangle[pair[0]] as usize];
            let b = mesh.vertices[triangle[pair[1]] as usize];
            lengths.push((b - a).length());
        }
    }
    let mean = lengths.iter().sum::<f32>() / lengths.len() as f32;
    assert!(
        (mean / target - 1.0).abs() < 0.35,
        "mean edge is {mean}, target was {target}"
    );
    // A few slivers near the boundary are unavoidable, but nothing should be
    // an order of magnitude off -- that is what makes one stretch constraint
    // vastly stiffer than its neighbours.
    let shortest = lengths.iter().copied().fold(f32::INFINITY, f32::min);
    assert!(
        shortest > target * 0.1,
        "shortest edge is {shortest}, target was {target}"
    );
}

/// The concave case: a triangulation that ignored the outline would span the
/// notch and the area would come out too big.
#[test]
fn respects_a_concave_outline() {
    let outline = notched();
    let mesh = triangulate(&outline, 0.05);
    check(&mesh, &outline);

    let expected = signed_area(&outline).abs();
    let area = total_area(&mesh);
    assert!(
        (area - expected).abs() < expected * 0.05,
        "the mesh covers {area}, the outline encloses {expected}"
    );

    // And nothing crosses the notch.
    for triangle in &mesh.triangles {
        let a = mesh.vertices[triangle[0] as usize];
        let b = mesh.vertices[triangle[1] as usize];
        let c = mesh.vertices[triangle[2] as usize];
        let centroid = (a + b + c) * (1.0 / 3.0);
        assert!(
            contains(&outline, centroid),
            "a triangle sits outside the outline at {centroid}"
        );
    }
}

/// A clockwise outline must produce the same mesh as its counter-clockwise
/// twin. Panel outlines come from a pattern library and their winding is not
/// something the caller should have to think about.
#[test]
fn accepts_either_winding() {
    let outline = rectangle(1.0, 0.5);
    let mut reversed = outline.clone();
    reversed.reverse();

    let forward = triangulate(&outline, 0.1);
    let backward = triangulate(&reversed, 0.1);
    check(&forward, &outline);
    check(&backward, &reversed);

    assert!(
        (total_area(&forward) - total_area(&backward)).abs() < 1e-3,
        "the two windings cover different areas"
    );
}

#[test]
fn handles_degenerate_input() {
    assert!(triangulate(&[], 0.1).is_empty());
    assert!(triangulate(&[Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0)], 0.1).is_empty());
    assert!(triangulate(&rectangle(1.0, 1.0), 0.0).is_empty());
    assert!(triangulate(&rectangle(1.0, 1.0), -1.0).is_empty());
}

/// A panel smaller than the target edge still has to produce something, or a
/// tiny pattern piece silently vanishes from the garment.
#[test]
fn meshes_a_panel_smaller_than_one_edge() {
    let outline = rectangle(0.01, 0.01);
    let mesh = triangulate(&outline, 0.5);
    check(&mesh, &outline);
}

#[test]
fn point_in_polygon_handles_vertices_on_the_ray() {
    // The ray from a test point passes exactly through two vertices; counting
    // each crossing twice would report the inside as outside.
    let outline = vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(0.0, 1.0),
    ];
    assert!(contains(&outline, Vec2::new(0.5, 0.5)));
    assert!(!contains(&outline, Vec2::new(1.5, 0.5)));
    assert!(!contains(&outline, Vec2::new(-0.5, 0.5)));

    let diamond = vec![
        Vec2::new(0.0, -1.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(0.0, 1.0),
        Vec2::new(-1.0, 0.0),
    ];
    // Level with two vertices exactly.
    assert!(contains(&diamond, Vec2::new(0.0, 0.0)));
    assert!(!contains(&diamond, Vec2::new(2.0, 0.0)));
}

#[test]
fn delaunay_covers_a_point_set() {
    let points: Vec<Vec2> = (0..8)
        .flat_map(|x| (0..8).map(move |y| Vec2::new(x as f32, y as f32)))
        .collect();
    let triangles = delaunay(&points);

    // A Delaunay triangulation of a point set in general position has
    // 2n - 2 - h triangles, where h is the hull size. A square grid has
    // 8*8 = 64 points and a hull of 28.
    assert_eq!(triangles.len(), 2 * 64 - 2 - 28);

    let area: f32 = triangles
        .iter()
        .map(|t| {
            let a = points[t[0] as usize];
            let b = points[t[1] as usize];
            let c = points[t[2] as usize];
            ((b - a).x * (c - a).y - (b - a).y * (c - a).x).abs() * 0.5
        })
        .sum();
    assert!(
        (area - 49.0).abs() < 1e-3,
        "the hull is 7x7, covered {area}"
    );
}

#[test]
fn delaunay_handles_collinear_and_duplicate_points() {
    let collinear: Vec<Vec2> = (0..5).map(|i| Vec2::new(i as f32, 0.0)).collect();
    // No triangle has any area; producing none is the honest answer.
    assert!(delaunay(&collinear).is_empty());

    let duplicates = vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(0.0, 1.0),
    ];
    // It must not hang or panic; the degenerate pair may or may not be used.
    let triangles = delaunay(&duplicates);
    assert!(triangles.len() <= 2);
}
