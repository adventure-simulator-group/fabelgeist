//! Graph colouring, so that a constraint set can be solved in parallel.
//!
//! Gauss-Seidel is what XPBD wants -- each projection should see the previous
//! one's result -- but a GPU runs a whole dispatch at once, and two
//! constraints sharing a particle would race on it. Colouring the constraint
//! graph fixes both: within a colour no two constraints touch the same
//! particle, so a colour is a safe parallel dispatch, and across colours the
//! order is sequential, which is the Gauss-Seidel sweep.
//!
//! Greedy, in the order the constraints arrive. Optimal colouring is NP-hard
//! and the greedy result is within a couple of colours of it on the meshes
//! this deals with, where the answer is bounded below by the maximum number of
//! constraints on any one particle anyway.

use std::collections::HashMap;

/// Constraints grouped into colours, with the permutation that produced them.
#[derive(Clone, Debug, Default)]
pub struct Coloring {
    /// Original constraint indices, ordered so each colour is contiguous.
    pub order: Vec<u32>,
    /// Start of each colour in `order`, with a final entry equal to the total,
    /// so colour `i` is `ranges[i]..ranges[i + 1]`.
    pub ranges: Vec<u32>,
}

impl Coloring {
    pub fn color_count(&self) -> usize {
        self.ranges.len().saturating_sub(1)
    }

    /// `(first, count)` of colour `index`, in constraint slots.
    pub fn color(&self, index: usize) -> (u32, u32) {
        let first = self.ranges[index];
        (first, self.ranges[index + 1] - first)
    }

    pub fn constraint_count(&self) -> usize {
        self.order.len()
    }

    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }
}

/// Colour constraints given the particles each one touches.
///
/// `constraints` is one slice of particle indices per constraint; the arity
/// may vary, which is what lets a bending constraint (four particles) and a
/// distance constraint (two) go through the same routine.
pub fn color(constraints: &[&[u32]]) -> Coloring {
    if constraints.is_empty() {
        return Coloring {
            order: Vec::new(),
            ranges: vec![0],
        };
    }

    // Highest colour already taken on each particle. Sparse, because a
    // constraint set may touch a small part of a large particle array.
    let mut taken: HashMap<u32, Vec<bool>> = HashMap::new();
    let mut colors: Vec<u32> = Vec::with_capacity(constraints.len());
    let mut color_count = 0usize;

    for particles in constraints {
        // Lowest colour no particle of this constraint has used yet.
        let mut candidate = 0usize;
        loop {
            let free = particles.iter().all(|particle| {
                taken
                    .get(particle)
                    .is_none_or(|used| used.get(candidate).is_none_or(|&u| !u))
            });
            if free {
                break;
            }
            candidate += 1;
        }

        for &particle in particles.iter() {
            let used = taken.entry(particle).or_default();
            if used.len() <= candidate {
                used.resize(candidate + 1, false);
            }
            used[candidate] = true;
        }

        colors.push(candidate as u32);
        color_count = color_count.max(candidate + 1);
    }

    // Counting sort into colour order.
    let mut counts = vec![0u32; color_count + 1];
    for &c in &colors {
        counts[c as usize + 1] += 1;
    }
    for i in 1..counts.len() {
        counts[i] += counts[i - 1];
    }

    let ranges = counts.clone();
    let mut cursor = counts;
    let mut order = vec![0u32; constraints.len()];
    for (index, &c) in colors.iter().enumerate() {
        let slot = &mut cursor[c as usize];
        order[*slot as usize] = index as u32;
        *slot += 1;
    }

    Coloring { order, ranges }
}

/// Convenience for the common case: constraints of fixed arity, stored flat.
pub fn color_fixed(particles: &[u32], arity: usize) -> Coloring {
    let chunks: Vec<&[u32]> = particles.chunks(arity).collect();
    color(&chunks)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The property the solver depends on, checked directly: no two
    /// constraints in a colour share a particle.
    fn check(constraints: &[&[u32]], coloring: &Coloring) {
        assert_eq!(
            coloring.constraint_count(),
            constraints.len(),
            "lost or gained constraints"
        );

        let mut seen = vec![false; constraints.len()];
        for &index in &coloring.order {
            assert!(!seen[index as usize], "constraint {index} appears twice");
            seen[index as usize] = true;
        }
        assert!(seen.iter().all(|&s| s), "order is not a permutation");

        for color_index in 0..coloring.color_count() {
            let (first, count) = coloring.color(color_index);
            let mut used = std::collections::HashSet::new();
            for slot in first..first + count {
                let constraint = coloring.order[slot as usize];
                for &particle in constraints[constraint as usize] {
                    assert!(
                        used.insert(particle),
                        "colour {color_index} uses particle {particle} twice"
                    );
                }
            }
        }
    }

    #[test]
    fn colors_a_chain() {
        // A path graph alternates between two colours.
        let edges: Vec<[u32; 2]> = (0..100).map(|i| [i, i + 1]).collect();
        let slices: Vec<&[u32]> = edges.iter().map(|e| e.as_slice()).collect();
        let coloring = color(&slices);
        check(&slices, &coloring);
        assert_eq!(
            coloring.color_count(),
            2,
            "a chain needs exactly two colours"
        );
    }

    #[test]
    fn colors_a_star() {
        // Every edge shares the hub, so every edge needs its own colour.
        let edges: Vec<[u32; 2]> = (1..20).map(|i| [0, i]).collect();
        let slices: Vec<&[u32]> = edges.iter().map(|e| e.as_slice()).collect();
        let coloring = color(&slices);
        check(&slices, &coloring);
        assert_eq!(coloring.color_count(), 19);
    }

    #[test]
    fn colors_a_grid() {
        // A quad mesh's stretch constraints: the case cloth actually hits.
        let (width, height) = (32u32, 32u32);
        let index = |x: u32, y: u32| y * width + x;
        let mut edges: Vec<[u32; 2]> = Vec::new();
        for y in 0..height {
            for x in 0..width {
                if x + 1 < width {
                    edges.push([index(x, y), index(x + 1, y)]);
                }
                if y + 1 < height {
                    edges.push([index(x, y), index(x, y + 1)]);
                }
            }
        }
        let slices: Vec<&[u32]> = edges.iter().map(|e| e.as_slice()).collect();
        let coloring = color(&slices);
        check(&slices, &coloring);
        // Four incident edges per interior vertex, so four colours is the
        // floor; greedy should not be far off it.
        assert!(
            (4..=6).contains(&coloring.color_count()),
            "expected 4-6 colours, got {}",
            coloring.color_count()
        );
    }

    #[test]
    fn colors_mixed_arity() {
        let constraints: Vec<Vec<u32>> = vec![
            vec![0, 1],
            vec![1, 2, 3, 4],
            vec![4, 5],
            vec![0, 5],
            vec![2, 6, 7],
        ];
        let slices: Vec<&[u32]> = constraints.iter().map(|c| c.as_slice()).collect();
        let coloring = color(&slices);
        check(&slices, &coloring);
    }

    #[test]
    fn colors_nothing() {
        let coloring = color(&[]);
        assert!(coloring.is_empty());
        assert_eq!(coloring.color_count(), 0);
    }

    #[test]
    fn independent_constraints_share_one_color() {
        let edges: Vec<[u32; 2]> = (0..50).map(|i| [i * 2, i * 2 + 1]).collect();
        let slices: Vec<&[u32]> = edges.iter().map(|e| e.as_slice()).collect();
        let coloring = color(&slices);
        check(&slices, &coloring);
        assert_eq!(coloring.color_count(), 1);
    }

    #[test]
    fn fixed_arity_matches_the_general_form() {
        let flat: Vec<u32> = vec![0, 1, 1, 2, 2, 3, 3, 0];
        let chunks: Vec<&[u32]> = flat.chunks(2).collect();
        assert_eq!(color_fixed(&flat, 2).order, color(&chunks).order);
    }
}
