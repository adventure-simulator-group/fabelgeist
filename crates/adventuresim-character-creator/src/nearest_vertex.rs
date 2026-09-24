//! Exact nearest source vertices for transferring anatomical UVs and skinning.

/// Balanced spatial index with the same lowest-index tie break as a linear scan.
pub struct NearestVertices<'a> {
    positions: &'a [[f32; 3]],
    order: Vec<usize>,
}

impl<'a> NearestVertices<'a> {
    /// Positions must be finite and contain at least one vertex.
    pub fn new(positions: &'a [[f32; 3]]) -> Self {
        assert!(!positions.is_empty(), "validated wearer contains vertices");
        let mut order = (0..positions.len()).collect::<Vec<_>>();
        partition(&mut order, positions, 0);
        Self { positions, order }
    }

    /// Return the original source index closest to a finite query position.
    pub fn nearest(&self, point: [f32; 3]) -> usize {
        let mut best = (0, squared_distance(point, self.positions[0]));
        self.search(&self.order, 0, point, &mut best);
        best.0
    }

    fn search(&self, order: &[usize], axis: usize, point: [f32; 3], best: &mut (usize, f32)) {
        if order.is_empty() {
            return;
        }
        let middle = order.len() / 2;
        let index = order[middle];
        let distance = squared_distance(point, self.positions[index]);
        if distance.total_cmp(&best.1).then(index.cmp(&best.0)).is_lt() {
            *best = (index, distance);
        }
        let delta = point[axis] - self.positions[index][axis];
        let (left, right) = (&order[..middle], &order[middle + 1..]);
        let (near, far) = if delta < 0.0 {
            (left, right)
        } else {
            (right, left)
        };
        let next_axis = (axis + 1) % 3;
        self.search(near, next_axis, point, best);
        // Include equality: an equidistant point may have an earlier source index.
        if delta.powi(2) <= best.1 {
            self.search(far, next_axis, point, best);
        }
    }
}

fn partition(order: &mut [usize], positions: &[[f32; 3]], axis: usize) {
    if order.len() <= 1 {
        return;
    }
    let middle = order.len() / 2;
    order.select_nth_unstable_by(middle, |a, b| {
        positions[*a][axis]
            .total_cmp(&positions[*b][axis])
            .then(a.cmp(b))
    });
    let (left, rest) = order.split_at_mut(middle);
    let next_axis = (axis + 1) % 3;
    partition(left, positions, next_axis);
    partition(&mut rest[1..], positions, next_axis);
}

fn squared_distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    (0..3).map(|i| (a[i] - b[i]).powi(2)).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn brute(points: &[[f32; 3]], query: [f32; 3]) -> usize {
        points
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                squared_distance(query, **a).total_cmp(&squared_distance(query, **b))
            })
            .unwrap()
            .0
    }

    #[test]
    fn matches_linear_scan_for_randomized_queries_and_source_orders() {
        let mut rng = fabelgeist_determinism::StreamId::new("test.nearest-vertex").rng(19, &[]);
        let mut random = || rng.range_f32(-10.0, 10.0);
        for count in [1, 2, 3, 17, 128, 1023] {
            let mut points = (0..count)
                .map(|_| std::array::from_fn(|_| random()))
                .collect::<Vec<_>>();
            for _ in 0..2 {
                let tree = NearestVertices::new(&points);
                for _ in 0..200 {
                    let query = std::array::from_fn(|_| random());
                    assert_eq!(tree.nearest(query), brute(&points, query));
                }
                for point in &points {
                    assert_eq!(tree.nearest(*point), brute(&points, *point));
                }
                points.reverse();
            }
        }
    }

    #[test]
    fn ties_and_duplicates_choose_earliest_source_across_split_planes() {
        let points = [
            [1., 0., 0.],
            [-1., 0., 0.],
            [0., 1., 0.],
            [0., -1., 0.],
            [0., 0., 1.],
            [0., 0., -1.],
            [1., 0., 0.],
        ];
        let tree = NearestVertices::new(&points);
        assert_eq!(tree.nearest([0.; 3]), 0);
        assert_eq!(tree.nearest([1., 0., 0.]), 0);
        for count in [2, 17, 100] {
            let duplicates = vec![[0.; 3]; count];
            assert_eq!(NearestVertices::new(&duplicates).nearest([1.; 3]), 0);
        }
    }

    #[test]
    fn planar_and_extreme_finite_points_preserve_f32_distance_ordering() {
        let points = [
            [0., -0., 0.],
            [f32::MAX, 0., 0.],
            [-f32::MAX, 0., 0.],
            [f32::MIN_POSITIVE, 0., 0.],
            [0., 1., 0.],
            [0., -1., 0.],
        ];
        let tree = NearestVertices::new(&points);
        for point in [[0.; 3], [f32::MAX; 3], [-f32::MAX; 3], [0., 2., 0.]] {
            assert_eq!(tree.nearest(point), brute(&points, point));
        }
    }
}
