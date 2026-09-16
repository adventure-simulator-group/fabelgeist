//! Bilateral symmetry enforcement and surface fairing for draped garments.
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

    // Build vertex adjacency for smoothing.
    let mut neighbors = vec![Vec::new(); positions.len()];
    for face in faces {
        for k in 0..3 {
            let u = face[k] as usize;
            let v = face[(k + 1) % 3] as usize;
            if u < positions.len() && v < positions.len() {
                neighbors[u].push(v);
                neighbors[v].push(u);
            }
        }
    }
    for list in &mut neighbors {
        list.sort_unstable();
        list.dedup();
    }

    // 1. Enforce bilateral symmetry across the sagittal plane (X = 0).
    // Identify sagittal vertices and bilateral pairs.
    let mut paired = vec![false; positions.len()];
    for i in 0..positions.len() {
        if paired[i] {
            continue;
        }
        let p_i = positions[i];
        if p_i[0].abs() <= SAGITTAL_PLANE_TOLERANCE_M {
            // Sagittal center vertex: snap exactly to X = 0.
            positions[i][0] = 0.0;
            paired[i] = true;
            continue;
        }

        // Search for the closest mirrored partner across X = 0: (-x, y, z).
        let target_mirror = [-p_i[0], p_i[1], p_i[2]];
        let mut best_j = None;
        let mut best_dist_sq = SYMMETRY_PAIR_TOLERANCE_M * SYMMETRY_PAIR_TOLERANCE_M;

        for j in (i + 1)..positions.len() {
            if paired[j] {
                continue;
            }
            let p_j = positions[j];
            // Must have opposite sign on X.
            if (p_i[0] > 0.0 && p_j[0] > 0.0) || (p_i[0] < 0.0 && p_j[0] < 0.0) {
                continue;
            }
            let dx = p_j[0] - target_mirror[0];
            let dy = p_j[1] - target_mirror[1];
            let dz = p_j[2] - target_mirror[2];
            let dist_sq = dx * dx + dy * dy + dz * dz;
            if dist_sq < best_dist_sq {
                best_dist_sq = dist_sq;
                best_j = Some(j);
            }
        }

        if let Some(j) = best_j {
            let p_j = positions[j];
            let abs_x = (p_i[0].abs() + p_j[0].abs()) * 0.5;
            let avg_y = (p_i[1] + p_j[1]) * 0.5;
            let avg_z = (p_i[2] + p_j[2]) * 0.5;

            let sign_i = if p_i[0] >= 0.0 { 1.0 } else { -1.0 };
            positions[i] = [abs_x * sign_i, avg_y, avg_z];
            positions[j] = [-abs_x * sign_i, avg_y, avg_z];
            paired[i] = true;
            paired[j] = true;
        }
    }

    // 2. Constrained Laplacian surface smoothing to relax high-frequency simulation wrinkles.
    let mut smoothed = positions.to_vec();
    for _ in 0..RELAXATION_PASSES {
        for i in 0..positions.len() {
            if neighbors[i].is_empty() {
                continue;
            }
            let mut avg_neighbor = [0.0f32; 3];
            for &adj in &neighbors[i] {
                avg_neighbor[0] += positions[adj][0];
                avg_neighbor[1] += positions[adj][1];
                avg_neighbor[2] += positions[adj][2];
            }
            let count = neighbors[i].len() as f32;
            avg_neighbor[0] /= count;
            avg_neighbor[1] /= count;
            avg_neighbor[2] /= count;

            // Blend current position towards neighbor average.
            smoothed[i][0] =
                positions[i][0] * (1.0 - RELAXATION_FACTOR) + avg_neighbor[0] * RELAXATION_FACTOR;
            smoothed[i][1] =
                positions[i][1] * (1.0 - RELAXATION_FACTOR) + avg_neighbor[1] * RELAXATION_FACTOR;
            smoothed[i][2] =
                positions[i][2] * (1.0 - RELAXATION_FACTOR) + avg_neighbor[2] * RELAXATION_FACTOR;
        }

        // Re-symmetrize smoothed positions to preserve exact symmetry.
        for i in 0..positions.len() {
            if smoothed[i][0].abs() <= SAGITTAL_PLANE_TOLERANCE_M {
                smoothed[i][0] = 0.0;
            }
        }
        positions.copy_from_slice(&smoothed);
    }

    // 3. Project any vertices that violated body clearance back to the safety margin.
    for p in positions.iter_mut() {
        let v = vector(*p);
        if let Some((triangle, closest, distance)) = collision.closest_point(v, f32::MAX) {
            let (a, b, c) = collision.triangle(triangle);
            let normal = (b - a).cross(c - a);
            let len = normal.length();
            if len > 1e-10 {
                let norm = normal / len;
                let outward = (v - closest).dot(norm);
                if outward < clearance_m || distance < clearance_m {
                    let adjusted = closest + norm * clearance_m;
                    *p = array(adjusted);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pairs_and_symmetrizes_bilateral_vertices() {
        let mut positions = vec![
            [0.002, 1.0, 0.5],   // Near center
            [0.21, 1.05, 0.49],  // Left
            [-0.19, 1.03, 0.51], // Right
        ];
        let faces = vec![[0, 1, 2]];
        let body = fabelgeist_bvh::TriangleBvh::new(
            vec![
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            ],
            vec![[0, 1, 2]],
        );
        symmetrize_and_relax(&mut positions, &faces, &body, 0.01);

        assert_eq!(positions[0][0], 0.0, "sagittal vertex must be at X=0");
        assert_eq!(
            (positions[1][0] + positions[2][0]).abs() < 1e-5,
            true,
            "X coordinates must be mirrored opposites"
        );
        assert_eq!(
            (positions[1][1] - positions[2][1]).abs() < 1e-5,
            true,
            "Y coordinates must match"
        );
        assert_eq!(
            (positions[1][2] - positions[2][2]).abs() < 1e-5,
            true,
            "Z coordinates must match"
        );
    }
}
