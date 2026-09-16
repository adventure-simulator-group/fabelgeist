//! Bilateral symmetry enforcement and surface fairing for draped garments.
//!
//! Draped garments keep one vertex per panel along each sewn seam. Every
//! operation here works on the welded surface, one position per sewn vertex,
//! and writes that position back to all of its copies: moving copies apart
//! would open the seams.
use super::*;

/// Margin threshold in metres for considering a vertex to lie on the central sagittal plane (X=0).
const SAGITTAL_PLANE_TOLERANCE_M: f32 = 0.005;
/// Maximum radius in metres to match mirrored bilateral vertex pairs.
const SYMMETRY_PAIR_TOLERANCE_M: f32 = 0.06;
/// Number of Laplacian smoothing passes for wrinkle relaxation.
const RELAXATION_PASSES: usize = 4;
/// Step size factor for Laplacian smoothing.
const RELAXATION_FACTOR: f32 = 0.35;

/// Symmetrize and relax wrinkles on a draped garment while keeping clearance from the collision body.
pub fn symmetrize_and_relax(
    positions: &mut [[f32; 3]],
    faces: &[[u32; 3]],
    collision: &fabelgeist_bvh::TriangleBvh,
    clearance_m: f32,
) {
    if positions.is_empty() || faces.is_empty() {
        return;
    }
    let surface = SewnSurface::from_positions(positions, faces);
    let mut sewn = surface.positions(positions);
    symmetrize(&mut sewn);
    relax(&mut sewn, &surface.faces);
    keep_clearance(&mut sewn, collision, clearance_m);
    positions.copy_from_slice(&surface.expand(&sewn));
}

/// Mirror each vertex with its nearest counterpart across the sagittal plane (X = 0).
fn symmetrize(positions: &mut [[f32; 3]]) {
    let mut paired = vec![false; positions.len()];
    for i in 0..positions.len() {
        if paired[i] {
            continue;
        }
        let p_i = positions[i];
        if p_i[0].abs() <= SAGITTAL_PLANE_TOLERANCE_M {
            positions[i][0] = 0.0;
            paired[i] = true;
            continue;
        }
        let mirror = vector([-p_i[0], p_i[1], p_i[2]]);
        let partner = (i + 1..positions.len())
            .filter(|&j| !paired[j] && positions[j][0] * p_i[0] < 0.0)
            .map(|j| (j, (vector(positions[j]) - mirror).length()))
            .filter(|(_, distance)| *distance < SYMMETRY_PAIR_TOLERANCE_M)
            .min_by(|a, b| a.1.total_cmp(&b.1));
        if let Some((j, _)) = partner {
            let p_j = positions[j];
            let x = (p_i[0].abs() + p_j[0].abs()) * 0.5 * p_i[0].signum();
            let y = (p_i[1] + p_j[1]) * 0.5;
            let z = (p_i[2] + p_j[2]) * 0.5;
            positions[i] = [x, y, z];
            positions[j] = [-x, y, z];
            paired[i] = true;
            paired[j] = true;
        }
    }
}

/// Laplacian smoothing of interior vertices; hems and openings keep their outline.
fn relax(positions: &mut [[f32; 3]], faces: &[[u32; 3]]) {
    let mut edges = std::collections::BTreeMap::<(usize, usize), usize>::new();
    for face in faces {
        for k in 0..3 {
            let (a, b) = (face[k] as usize, face[(k + 1) % 3] as usize);
            *edges.entry((a.min(b), a.max(b))).or_default() += 1;
        }
    }
    let mut neighbors = vec![Vec::new(); positions.len()];
    let mut boundary = vec![false; positions.len()];
    for (&(a, b), &count) in &edges {
        neighbors[a].push(b);
        neighbors[b].push(a);
        if count == 1 {
            boundary[a] = true;
            boundary[b] = true;
        }
    }
    for _ in 0..RELAXATION_PASSES {
        let previous = positions.to_vec();
        for (i, position) in positions.iter_mut().enumerate() {
            if boundary[i] || neighbors[i].is_empty() {
                continue;
            }
            let average = neighbors[i]
                .iter()
                .fold(Vec3::default(), |sum, &n| sum + vector(previous[n]))
                / neighbors[i].len() as f32;
            let mut relaxed = array(
                vector(previous[i]) * (1.0 - RELAXATION_FACTOR) + average * RELAXATION_FACTOR,
            );
            // Keep the sagittal line exactly on the plane.
            if previous[i][0] == 0.0 {
                relaxed[0] = 0.0;
            }
            *position = relaxed;
        }
    }
}

/// Push vertices closer to the body than `clearance_m` back out to it.
fn keep_clearance(
    positions: &mut [[f32; 3]],
    collision: &fabelgeist_bvh::TriangleBvh,
    clearance_m: f32,
) {
    for p in positions.iter_mut() {
        let v = vector(*p);
        // Only nearby body surface can violate clearance. Farther vertices may
        // be closest to a triangle edge, where one face's plane says nothing
        // about which side of the body they are on.
        let Some((triangle, closest, distance)) = collision.closest_point(v, clearance_m) else {
            continue;
        };
        let (a, b, c) = collision.triangle(triangle);
        let normal = (b - a).cross(c - a);
        let length = normal.length();
        if length <= 1e-10 || distance >= clearance_m {
            continue;
        }
        *p = array(closest + normal / length * clearance_m);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A plane below the garment, so clearance never moves it.
    fn distant_body() -> fabelgeist_bvh::TriangleBvh {
        fabelgeist_bvh::TriangleBvh::new(
            vec![
                Vec3::new(-5.0, -5.0, 0.0),
                Vec3::new(5.0, -5.0, 0.0),
                Vec3::new(0.0, 5.0, 0.0),
            ],
            vec![[0, 1, 2]],
        )
    }

    #[test]
    fn pairs_and_symmetrizes_bilateral_vertices() {
        let mut positions = vec![
            [0.002, 1.0, 0.5],   // Near center
            [0.21, 1.05, 0.49],  // Left
            [-0.19, 1.03, 0.51], // Right
        ];
        symmetrize_and_relax(&mut positions, &[[0, 1, 2]], &distant_body(), 0.01);

        assert_eq!(positions[0][0], 0.0, "sagittal vertex must be at X=0");
        assert!((positions[1][0] + positions[2][0]).abs() < 1e-5);
        assert!((positions[1][1] - positions[2][1]).abs() < 1e-5);
        assert!((positions[1][2] - positions[2][2]).abs() < 1e-5);
    }

    #[test]
    fn seam_copies_stay_together() {
        // A 3×3 vertex grid cut down its middle column into two panels, each
        // with its own copy of the seam vertices, slightly wrinkled and asymmetric.
        let grid = |x: f32, y: f32| [x + 0.013 * y, y, 0.5 + 0.02 * (x * 9.0 + y * 7.0).sin()];
        let mut positions = Vec::new();
        for xs in [[-0.3, -0.1, 0.1], [0.1, 0.32, 0.5]] {
            for y in [0.0, 0.2, 0.4] {
                for x in xs {
                    positions.push(grid(x, y));
                }
            }
        }
        let quads = |o: u32| {
            [
                [0, 1, 4],
                [0, 4, 3],
                [1, 2, 5],
                [1, 5, 4],
                [3, 4, 7],
                [3, 7, 6],
                [4, 5, 8],
                [4, 8, 7],
            ]
            .map(|f: [u32; 3]| f.map(|i| i + o))
        };
        let faces: Vec<_> = quads(0).into_iter().chain(quads(9)).collect();
        let seams = [(2, 9), (5, 12), (8, 15)];
        for (a, b) in seams {
            assert_eq!(positions[a], positions[b]);
        }

        symmetrize_and_relax(&mut positions, &faces, &distant_body(), 0.01);

        for (a, b) in seams {
            assert_eq!(positions[a], positions[b], "seam {a}/{b} opened");
        }
    }

    #[test]
    fn hems_keep_their_outline_and_distant_cloth_is_not_pulled_to_the_body() {
        let mut positions = vec![
            [-0.2, 0.0, 0.3],
            [0.0, 0.0, 0.3],
            [0.2, 0.0, 0.3],
            [0.0, 0.2, 0.35],
        ];
        let before = positions.clone();
        let faces = [[0, 1, 3], [1, 2, 3]];
        // The body edge nearest the cloth is 0.3 m away, well beyond clearance.
        let body = fabelgeist_bvh::TriangleBvh::new(
            vec![
                Vec3::new(-1.0, -1.0, 0.0),
                Vec3::new(1.0, -1.0, 0.0),
                Vec3::new(0.0, -0.5, 0.0),
            ],
            vec![[0, 1, 2]],
        );
        symmetrize_and_relax(&mut positions, &faces, &body, 0.01);
        assert_eq!(positions, before);
    }
}
