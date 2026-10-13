//! Stable row orders reduce permutation symmetry without changing geometry.
use super::*;

#[derive(Clone, Copy)]
struct RowConstraint {
    first: PackingDomainIndex,
    second: PackingDomainIndex,
    gap: f64,
}

/// Explore combinations by total candidate rank, so one row cannot starve others.
pub(super) struct Combinations<'a> {
    rows: &'a [Vec<Vec<PackingDomainIndex>>],
    pending: std::collections::BinaryHeap<std::cmp::Reverse<(usize, [usize; 4])>>,
    visited: std::collections::BTreeSet<[usize; 4]>,
}
impl<'a> Combinations<'a> {
    pub(super) fn new(rows: &'a [Vec<Vec<PackingDomainIndex>>]) -> Self {
        let pending = std::collections::BinaryHeap::from([std::cmp::Reverse((0, [0; 4]))]);
        Self {
            rows,
            pending,
            visited: std::collections::BTreeSet::from([[0; 4]]),
        }
    }
}
impl<'a> Iterator for Combinations<'a> {
    type Item = [&'a [PackingDomainIndex]; 4];
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

pub(super) fn rows(domains: &[PlacementDomain]) -> Vec<Vec<Vec<PackingDomainIndex>>> {
    (0..4)
        .map(|edge| {
            let mut original = domains
                .iter()
                .enumerate()
                .filter_map(|(index, d)| {
                    (d.frontage.edge == edge).then_some(PackingDomainIndex::new(index))
                })
                .collect::<Vec<_>>();
            original.sort_by(|&a, &b| {
                let projection = |index: PackingDomainIndex| {
                    domains[index.index()]
                        .proposed
                        .reservation
                        .centre_metres()
                        .as_dvec2()
                        .dot(domains[index.index()].frontage.tangent().as_dvec2())
                };
                projection(a)
                    .total_cmp(&projection(b))
                    .then(domains[a.index()].owner.cmp(&domains[b.index()].owner))
            });
            let mut orders = vec![original.clone()];
            let mut reversed = original.clone();
            reversed.reverse();
            orders.push(reversed);
            for component in [1, 0] {
                let mut sorted = original.clone();
                sorted.sort_by(|&a, &b| {
                    domains[a.index()].proposed.reservation.dimensions_metres()[component]
                        .total_cmp(
                            &domains[b.index()].proposed.reservation.dimensions_metres()[component],
                        )
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
    selected: &[&[PackingDomainIndex]],
) -> Result<Option<Vec<PlacementDomain>>, CoupledPackingIssue> {
    let geometry = domains
        .iter()
        .map(PlacementDomain::geometry_at_zero)
        .collect::<Result<Vec<_>, _>>()?;
    let mut narrowed = domains.to_vec();
    for row in selected {
        let constraints = row
            .iter()
            .enumerate()
            .flat_map(|(rank, &first)| {
                let geometry = &geometry;
                row[rank + 1..].iter().map(move |&second| {
                    let a = &domains[first.index()];
                    let axis = a.frontage.tangent().as_dvec2();
                    let (_, first_maximum) =
                        Envelope::Ground(&geometry[first.index()]).projection(axis);
                    let (second_minimum, _) =
                        Envelope::Ground(&geometry[second.index()]).projection(axis);
                    let reach = first_maximum - second_minimum
                        + intervals::PackingClearance::PropertyBoundary.metres()
                        + CityPlotBounds::COORDINATE_TOLERANCE_METRES;
                    RowConstraint {
                        first,
                        second,
                        gap: reach / axis.length_squared(),
                    }
                })
            })
            .collect::<Vec<_>>();
        for &RowConstraint { first, second, gap } in &constraints {
            let minimum = narrowed[first.index()].allowed.minimum_metres() + gap;
            let Some(allowed) = narrowed[second.index()]
                .allowed
                .with_half_plane(-minimum, 1.0)
                .map_err(|_| CoupledPackingIssue::NumericalFailure)?
            else {
                return Ok(None);
            };
            narrowed[second.index()].allowed = allowed;
        }
        for &RowConstraint { first, second, gap } in constraints.iter().rev() {
            let maximum = narrowed[second.index()].allowed.maximum_metres() - gap;
            let Some(allowed) = narrowed[first.index()]
                .allowed
                .with_half_plane(maximum, -1.0)
                .map_err(|_| CoupledPackingIssue::NumericalFailure)?
            else {
                return Ok(None);
            };
            narrowed[first.index()].allowed = allowed;
        }
    }
    Ok(narrowed
        .iter()
        .all(|d| d.allowed.minimum_metres() <= d.allowed.maximum_metres())
        .then_some(narrowed))
}

pub(super) fn apply(
    model: &Model<'_>,
    selected: &[&[PackingDomainIndex]],
) -> Result<Problem, CoupledPackingIssue> {
    let geometry = model
        .domains
        .iter()
        .map(PlacementDomain::geometry_at_zero)
        .collect::<Result<Vec<_>, _>>()?;
    let mut problem = model.problem.clone();
    for row in selected {
        for (rank, &first) in row.iter().enumerate() {
            for &second in &row[rank + 1..] {
                let a = &model.domains[first.index()];
                let axis = a.frontage.tangent().as_dvec2();
                let coefficient = axis.dot(axis);
                let (_, first_maximum) =
                    Envelope::Ground(&geometry[first.index()]).projection(axis);
                let (second_minimum, _) =
                    Envelope::Ground(&geometry[second.index()]).projection(axis);
                let reach = first_maximum - second_minimum
                    + intervals::PackingClearance::PropertyBoundary.metres()
                    + CityPlotBounds::COORDINATE_TOLERANCE_METRES;
                problem.add_constraint(
                    [
                        (model.variables[first.index()], -coefficient),
                        (model.variables[second.index()], coefficient),
                    ],
                    ComparisonOp::Ge,
                    reach,
                );
            }
        }
    }
    Ok(problem)
}
