//! Stable row orders reduce permutation symmetry without changing geometry.
use super::*;

pub(super) fn rows(domains: &[PlacementDomain]) -> Vec<Vec<Vec<usize>>> {
    (0..4)
        .map(|edge| {
            let mut original = domains
                .iter()
                .enumerate()
                .filter_map(|(index, d)| (d.frontage.edge == edge).then_some(index))
                .collect::<Vec<_>>();
            original.sort_by(|&a, &b| {
                let projection = |index: usize| {
                    domains[index]
                        .proposed
                        .reservation
                        .centre_metres
                        .as_dvec2()
                        .dot(domains[index].frontage.tangent().as_dvec2())
                };
                projection(a)
                    .total_cmp(&projection(b))
                    .then(domains[a].owner.cmp(&domains[b].owner))
            });
            let mut orders = vec![original.clone()];
            let mut reversed = original.clone();
            reversed.reverse();
            orders.push(reversed);
            for component in [1, 0] {
                let mut sorted = original.clone();
                sorted.sort_by(|&a, &b| {
                    domains[a].proposed.reservation.dimensions_metres[component]
                        .total_cmp(&domains[b].proposed.reservation.dimensions_metres[component])
                });
                orders.push(sorted.clone());
                let mut reversed = sorted.clone();
                reversed.reverse();
                orders.push(reversed);
                let mut alternating = Vec::with_capacity(sorted.len());
                for index in 0..sorted.len() {
                    let selected = if index % 2 == 0 {
                        index / 2
                    } else {
                        sorted.len() - 1 - index / 2
                    };
                    alternating.push(sorted[selected]);
                }
                orders.push(alternating.clone());
                alternating.reverse();
                orders.push(alternating);
            }
            let base = orders.clone();
            for order in base {
                for index in 0..order.len().saturating_sub(1) {
                    let mut adjacent = order.clone();
                    adjacent.swap(index, index + 1);
                    orders.push(adjacent);
                }
            }
            let mut unique = Vec::new();
            for order in orders {
                if !unique.contains(&order) {
                    unique.push(order);
                }
            }
            unique
        })
        .collect()
}

pub(super) fn constrain(
    domains: &[PlacementDomain],
    selected: &[&[usize]],
) -> Option<Vec<PlacementDomain>> {
    let mut narrowed = domains.to_vec();
    for row in selected {
        let constraints = row
            .iter()
            .enumerate()
            .flat_map(|(rank, &first)| {
                row[rank + 1..].iter().map(move |&second| {
                    let a = &domains[first];
                    let b = &domains[second];
                    let axis = a.frontage.tangent().as_dvec2();
                    let reach = Envelope::Ground(&a.geometry_at_zero()).projection(axis).1
                        - Envelope::Ground(&b.geometry_at_zero()).projection(axis).0
                        + intervals::PackingClearance::PropertyBoundary.metres()
                        + CityPlotBounds::COORDINATE_TOLERANCE_METRES;
                    (first, second, reach / axis.length_squared())
                })
            })
            .collect::<Vec<_>>();
        for &(first, second, gap) in &constraints {
            let minimum = narrowed[first].allowed.minimum_metres + gap;
            narrowed[second].allowed.minimum_metres =
                narrowed[second].allowed.minimum_metres.max(minimum);
        }
        for &(first, second, gap) in constraints.iter().rev() {
            let maximum = narrowed[second].allowed.maximum_metres - gap;
            narrowed[first].allowed.maximum_metres =
                narrowed[first].allowed.maximum_metres.min(maximum);
        }
    }
    narrowed
        .iter()
        .all(|d| d.allowed.minimum_metres <= d.allowed.maximum_metres)
        .then_some(narrowed)
}

pub(super) fn apply(model: &Model<'_>, selected: &[&[usize]]) -> Problem {
    let mut problem = model.problem.clone();
    for row in selected {
        for (rank, &first) in row.iter().enumerate() {
            for &second in &row[rank + 1..] {
                let a = &model.domains[first];
                let b = &model.domains[second];
                let axis = a.frontage.tangent().as_dvec2();
                let coefficient = axis.dot(axis);
                let reach = Envelope::Ground(&a.geometry_at_zero()).projection(axis).1
                    - Envelope::Ground(&b.geometry_at_zero()).projection(axis).0
                    + intervals::PackingClearance::PropertyBoundary.metres()
                    + CityPlotBounds::COORDINATE_TOLERANCE_METRES;
                problem.add_constraint(
                    [
                        (model.variables[first], -coefficient),
                        (model.variables[second], coefficient),
                    ],
                    ComparisonOp::Ge,
                    reach,
                );
            }
        }
    }
    problem
}

/// Explore combinations by total candidate rank, so one row cannot starve others.
pub(super) struct Combinations<'a> {
    rows: &'a [Vec<Vec<usize>>],
    pending: std::collections::BinaryHeap<std::cmp::Reverse<(usize, [usize; 4])>>,
    visited: std::collections::BTreeSet<[usize; 4]>,
}
impl<'a> Combinations<'a> {
    pub(super) fn new(rows: &'a [Vec<Vec<usize>>]) -> Self {
        let pending = std::collections::BinaryHeap::from([std::cmp::Reverse((0, [0; 4]))]);
        Self {
            rows,
            pending,
            visited: std::collections::BTreeSet::from([[0; 4]]),
        }
    }
}
impl<'a> Iterator for Combinations<'a> {
    type Item = [&'a [usize]; 4];
    fn next(&mut self) -> Option<Self::Item> {
        let std::cmp::Reverse((rank, indices)) = self.pending.pop()?;
        for edge in 0..4 {
            let mut next = indices;
            next[edge] += 1;
            if next[edge] < self.rows[edge].len() && self.visited.insert(next) {
                self.pending.push(std::cmp::Reverse((rank + 1, next)));
            }
        }
        Some(std::array::from_fn(|edge| {
            self.rows[edge][indices[edge]].as_slice()
        }))
    }
}
