//! A disjunction selects at least one real separating plane per geometry pair.
use super::*;
use intervals::PackingClearance;

pub(super) struct Model<'a> {
    pub(super) problem: Problem,
    pub(super) variables: Vec<Variable>,
    pub(super) pairs: Vec<Disjunction>,
    pub(super) domains: &'a [PlacementDomain],
    #[cfg(test)]
    forced: Vec<Disjunction>,
}

#[derive(Clone, Copy, serde::Serialize)]
pub(super) struct Condition {
    pub(super) first: f64,
    pub(super) second: f64,
    pub(super) required: f64,
    minimum: f64,
    maximum: f64,
}

#[derive(serde::Serialize)]
pub(super) struct Disjunction {
    pub(super) first: PackingDomainIndex,
    pub(super) second: PackingDomainIndex,
    pub(super) conditions: Vec<Condition>,
}

impl<'a> Model<'a> {
    pub(super) fn new(domains: &'a [PlacementDomain]) -> Self {
        let mut problem = Problem::new(OptimizationDirection::Minimize);
        let variables = domains
            .iter()
            .map(|domain| {
                let variable = problem.add_var(
                    0.0,
                    (
                        domain.allowed.minimum_metres(),
                        domain.allowed.maximum_metres(),
                    ),
                );
                let distance = problem.add_var(1.0, (0.0, f64::INFINITY));
                problem.add_constraint([(distance, 1.0), (variable, -1.0)], ComparisonOp::Ge, 0.0);
                problem.add_constraint([(distance, 1.0), (variable, 1.0)], ComparisonOp::Ge, 0.0);
                variable
            })
            .collect();
        Self {
            problem,
            variables,
            pairs: Vec::new(),
            domains,
            #[cfg(test)]
            forced: Vec::new(),
        }
    }
    pub(super) fn append_geometry(&mut self) -> Result<(), CoupledPackingIssue> {
        let geometry = self
            .domains
            .iter()
            .map(PlacementDomain::geometry_at_zero)
            .collect::<Result<Vec<_>, _>>()?;
        let gardens = geometry
            .iter()
            .zip(self.domains)
            .map(|(geometry, domain)| {
                geometry
                    .garden
                    .as_ref()
                    .map(|garden| garden.clearance_geometry())
                    .transpose()
                    .map_err(|issue| CoupledPackingIssue::Garden {
                        property: domain.owner,
                        issue,
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        for first in 0..self.domains.len() {
            for second in first + 1..self.domains.len() {
                let first = PackingDomainIndex::new(first);
                let second = PackingDomainIndex::new(second);
                let a = &geometry[first.index()];
                let b = &geometry[second.index()];
                self.pair(
                    first,
                    second,
                    Envelope::Ground(a),
                    Envelope::Ground(b),
                    PackingClearance::PropertyBoundary,
                );
                for (owner, neighbour) in [(first, second), (second, first)] {
                    if let Some(garden) = &gardens[owner.index()] {
                        for &body in &geometry[neighbour.index()].buildings {
                            for &working in &garden.working {
                                self.pair(
                                    owner,
                                    neighbour,
                                    Envelope::Rectangle(working),
                                    Envelope::Rectangle(body),
                                    PackingClearance::GardenWorking,
                                );
                            }
                            for plant in &garden.plants {
                                self.pair(
                                    owner,
                                    neighbour,
                                    Envelope::Polygon(plant),
                                    Envelope::Rectangle(body),
                                    PackingClearance::GardenPlant,
                                );
                            }
                        }
                    }
                }
                for &body_a in &a.buildings {
                    for &body_b in &b.buildings {
                        self.pair(
                            first,
                            second,
                            Envelope::Rectangle(body_a),
                            Envelope::Rectangle(body_b),
                            PackingClearance::BuildingBody,
                        );
                    }
                }
            }
        }
        Ok(())
    }
    fn pair(
        &mut self,
        first: PackingDomainIndex,
        second: PackingDomainIndex,
        a: Envelope<'_>,
        b: Envelope<'_>,
        clearance: PackingClearance,
    ) {
        let mut conditions = a
            .axes()
            .into_iter()
            .chain(b.axes())
            .flat_map(|axis| [axis, -axis])
            .map(|axis| self.condition(first, second, a, b, axis, clearance))
            .collect::<Vec<_>>();
        if conditions.iter().any(|c| c.minimum >= c.required) {
            return;
        }
        conditions.retain(|condition| condition.maximum >= condition.required);
        conditions.sort_by(|a, b| {
            a.first
                .total_cmp(&b.first)
                .then(a.second.total_cmp(&b.second))
                .then(a.required.total_cmp(&b.required))
        });
        conditions.dedup_by(|a, b| {
            a.first == b.first && a.second == b.second && a.required == b.required
        });
        if let [condition] = conditions.as_slice() {
            #[cfg(test)]
            self.forced.push(Disjunction {
                first,
                second,
                conditions: conditions.clone(),
            });
            self.problem.add_constraint(
                [
                    (self.variables[first.index()], condition.first),
                    (self.variables[second.index()], condition.second),
                ],
                ComparisonOp::Ge,
                condition.required,
            );
            return;
        }
        self.pairs.push(Disjunction {
            first,
            second,
            conditions,
        });
    }

    fn condition(
        &self,
        first: PackingDomainIndex,
        second: PackingDomainIndex,
        a: Envelope<'_>,
        b: Envelope<'_>,
        axis: DVec2,
        clearance: PackingClearance,
    ) -> Condition {
        let left = &self.domains[first.index()];
        let right = &self.domains[second.index()];
        let first = -axis.dot(left.frontage.tangent().as_dvec2());
        let second = axis.dot(right.frontage.tangent().as_dvec2());
        let minimum = |rate: f64, range: FrontageInterval| {
            (rate * range.minimum_metres()).min(rate * range.maximum_metres())
        };
        let (_, first_maximum) = a.projection(axis);
        let (second_minimum, _) = b.projection(axis);
        Condition {
            first,
            second,
            // An additional existing coordinate interval protects the cast and
            // world-space f32 translation. Physical clearances are stricter,
            // never relaxed; the returned geometry is checked independently.
            required: first_maximum - second_minimum
                + clearance.metres()
                + CityPlotBounds::COORDINATE_TOLERANCE_METRES,
            minimum: minimum(first, left.allowed) + minimum(second, right.allowed),
            maximum: -minimum(-first, left.allowed) - minimum(-second, right.allowed),
        }
    }
}

#[cfg(test)]
impl Model<'_> {
    pub(super) fn record(&self) {
        let Some(directory) = std::env::var_os("FABELGEIST_SUPPORT_DIAGNOSTIC_DIR") else {
            return;
        };
        let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(directory);
        std::fs::create_dir_all(&directory).unwrap();
        let domains = self
            .domains
            .iter()
            .map(|d| serde_json::json!({"owner":d.owner,"allowed":d.allowed,"base_translation_metres":d.base_translation.metres()}))
            .collect::<Vec<_>>();
        std::fs::write(
            directory.join(format!(
                "coupled-model-{}.json",
                self.domains[0].frontage.block.id.0
            )),
            serde_json::to_vec_pretty(
                &serde_json::json!({"domains":domains,"forced":self.forced,"pairs":self.pairs}),
            )
            .unwrap(),
        )
        .unwrap();
    }
}

impl Disjunction {
    pub(super) fn deficit(&self, c: Condition, p: &[f64]) -> f64 {
        c.required - c.first * p[self.first.index()] - c.second * p[self.second.index()]
    }
    pub(super) fn add(&self, c: Condition, problem: &mut Problem, variables: &[Variable]) {
        problem.add_constraint(
            [
                (variables[self.first.index()], c.first),
                (variables[self.second.index()], c.second),
            ],
            ComparisonOp::Ge,
            c.required,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::super::search::Search;
    use super::*;

    #[test]
    fn complete_lp_solution_survives_later_sibling_budget_exhaustion() {
        let mut model = Model::new(&[]);
        let x = model.problem.add_var(0.0, (-1.0, 1.0));
        let y = model.problem.add_var(0.0, (0.0, 0.0));
        let distance = model.problem.add_var(1.0, (0.0, f64::INFINITY));
        model.variables = vec![x, y];
        model
            .problem
            .add_constraint([(distance, 1.0), (x, -1.0)], ComparisonOp::Ge, 0.0);
        model
            .problem
            .add_constraint([(distance, 1.0), (x, 1.0)], ComparisonOp::Ge, 0.0);
        model.pairs.push(Disjunction {
            first: PackingDomainIndex::new(0),
            second: PackingDomainIndex::new(1),
            conditions: [1.0, -1.0]
                .map(|first| Condition {
                    first,
                    second: 0.0,
                    required: 1.0,
                    minimum: -1.0,
                    maximum: 1.0,
                })
                .to_vec(),
        });
        let outcome = Search::new(&model, SearchBudget::new(2)).solve_counted();
        let positions = outcome
            .outcome
            .unwrap()
            .iter()
            .map(FrontageDisplacement::metres)
            .collect::<Vec<_>>();
        assert_eq!(positions, vec![1.0, 0.0]);
        assert_eq!(outcome.explored_nodes.count(), 2);
        assert!(
            model.pairs[0]
                .conditions
                .iter()
                .any(|&condition| model.pairs[0].deficit(condition, &positions) <= 0.0)
        );
    }
}
