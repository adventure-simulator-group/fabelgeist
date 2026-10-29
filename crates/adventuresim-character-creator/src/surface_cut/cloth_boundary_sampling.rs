//! Bounded replacement of source polylines by coarser cloth cut edges.
use bevy::math::DVec3;
use std::collections::BTreeMap;

pub(super) struct BoundaryChains {
    edges: BTreeMap<[u32; 2], Vec<DVec3>>,
    chord_deviation_metres: f64,
}

struct BoundaryReplacement {
    edge: [u32; 2],
    sources: Vec<DVec3>,
}

impl BoundaryChains {
    pub fn new(faces: &[[u32; 3]], positions: &[DVec3], chord_deviation_metres: f64) -> Self {
        let mut edges = BTreeMap::<[u32; 2], usize>::new();
        for &[a, b, c] in faces {
            for mut edge in [[a, b], [b, c], [c, a]] {
                edge.sort_unstable();
                *edges.entry(edge).or_default() += 1;
            }
        }
        Self {
            chord_deviation_metres,
            edges: edges
                .into_iter()
                .filter(|(_, count)| *count == 1)
                .map(|(edge, _)| (edge, edge.map(|v| positions[v as usize]).to_vec()))
                .collect(),
        }
    }

    pub fn permits(&self, removed: u32, retained: u32, positions: &[DVec3]) -> bool {
        let Some(replacement) = self.replacement(removed, retained) else {
            return false;
        };
        let [start, end] = replacement.edge.map(|v| positions[v as usize]);
        let chord = end - start;
        if !chord.is_finite() || chord.length_squared() == 0. {
            return false;
        }
        replacement.sources.iter().all(|point| {
            let along = ((*point - start).dot(chord) / chord.length_squared()).clamp(0., 1.);
            point.distance(start + chord * along) <= self.chord_deviation_metres
        })
    }

    pub fn collapse(&mut self, removed: u32, retained: u32) {
        let Some(replacement) = self.replacement(removed, retained) else {
            return;
        };
        self.edges.retain(|edge, _| !edge.contains(&removed));
        self.edges.insert(replacement.edge, replacement.sources);
    }

    fn replacement(&self, removed: u32, retained: u32) -> Option<BoundaryReplacement> {
        let mut shared = [removed, retained];
        shared.sort_unstable();
        let first = self.edges.get(&shared)?;
        let mut neighbours = self
            .edges
            .iter()
            .filter(|(edge, _)| edge.contains(&removed) && !edge.contains(&retained));
        let (other, second) = neighbours.next()?;
        if neighbours.next().is_some() {
            return None;
        }
        let mut edge = other.map(|v| if v == removed { retained } else { v });
        edge.sort_unstable();
        // Never close a boundary loop down to fewer than three vertices.
        if self.edges.contains_key(&edge) {
            return None;
        }
        Some(BoundaryReplacement {
            edge,
            sources: first.iter().chain(second).copied().collect(),
        })
    }
}
