//! Improve coarse anatomical triangulation without adding source points.
//! Native boundary: canonical metre positions and frozen source vertex ordinals.
//! Only interior diagonals change; physical cut boundaries stay untouched.
use super::ClothPatch;
use bevy::math::DVec3;
use std::collections::BTreeMap;

/// An altitude below one eighth of the longest edge creates a narrow cell
/// whose normal can exaggerate a small body-sampling error into a cloth crease.
const MINIMUM_ANATOMICAL_ALTITUDE_SHARE: f64 = 0.125;

struct EdgeFlip {
    first: usize,
    second: usize,
    replacements: [[u32; 3]; 2],
}

impl ClothPatch {
    pub(super) fn condition_edges(&mut self) {
        // Each accepted flip strictly raises the pair's minimum aspect quality.
        // The globally sorted face-quality sequence therefore improves; a
        // reverse flip cannot be admitted. No new vertex or morph source is made.
        while let Some(flip) = self.next_conditioning_flip() {
            let bank = self.source_facets[flip.first]
                .union(&self.source_facets[flip.second])
                .copied()
                .collect::<std::collections::BTreeSet<_>>();
            self.source_facets[flip.first] = bank.clone();
            self.source_facets[flip.second] = bank;
            self.faces[flip.first] = flip.replacements[0];
            self.faces[flip.second] = flip.replacements[1];
        }
    }

    fn next_conditioning_flip(&self) -> Option<EdgeFlip> {
        let mut edges = BTreeMap::<[u32; 2], Vec<usize>>::new();
        for (index, face) in self.faces.iter().enumerate() {
            for corner in 0..3 {
                let mut edge = [face[corner], face[(corner + 1) % 3]];
                edge.sort_unstable();
                edges.entry(edge).or_default().push(index);
            }
        }
        for (edge, adjacent) in &edges {
            let [first, second] = adjacent.as_slice() else {
                continue;
            };
            let before = [self.faces[*first], self.faces[*second]];
            let quality = before.map(|face| aspect(self.positions_for(face)));
            let minimum = quality[0].min(quality[1]);
            if minimum >= MINIMUM_ANATOMICAL_ALTITUDE_SHARE {
                continue;
            }
            let Some(corner) = (0..3).find(|&i| {
                let mut candidate = [before[0][i], before[0][(i + 1) % 3]];
                candidate.sort_unstable();
                candidate == *edge
            }) else {
                continue;
            };
            let [a, b, c] = [
                before[0][corner],
                before[0][(corner + 1) % 3],
                before[0][(corner + 2) % 3],
            ];
            let Some(reverse) = (0..3).find(|&i| before[1][i] == b && before[1][(i + 1) % 3] == a)
            else {
                continue;
            };
            let d = before[1][(reverse + 2) % 3];
            let mut diagonal = [c, d];
            diagonal.sort_unstable();
            if c == d || edges.contains_key(&diagonal) {
                continue;
            }
            let replacements = [[c, a, d], [c, d, b]];
            let after = replacements.map(|face| self.positions_for(face));
            if after.iter().any(|&points| aspect(points) <= minimum) {
                continue;
            }
            let normals = before.map(|face| normal(self.positions_for(face)));
            // Preserve orientation against both sides of the original quad.
            // This also rejects a diagonal outside a concave local boundary.
            if after.iter().any(|&points| {
                let candidate = normal(points);
                !candidate.is_finite() || normals.iter().any(|normal| candidate.dot(*normal) <= 0.)
            }) {
                continue;
            }
            return Some(EdgeFlip {
                first: *first,
                second: *second,
                replacements,
            });
        }
        None
    }

    fn positions_for(&self, face: [u32; 3]) -> [DVec3; 3] {
        face.map(|vertex| self.positions[vertex as usize])
    }
}

fn normal([a, b, c]: [DVec3; 3]) -> DVec3 {
    (b - a).cross(c - a)
}

/// Dimensionless altitude divided by longest edge, not an area in metres.
fn aspect(points: [DVec3; 3]) -> f64 {
    let longest_squared = (0..3)
        .map(|i| points[i].distance_squared(points[(i + 1) % 3]))
        .fold(0., f64::max);
    if longest_squared == 0. {
        return 0.;
    }
    normal(points).length() / longest_squared
}
