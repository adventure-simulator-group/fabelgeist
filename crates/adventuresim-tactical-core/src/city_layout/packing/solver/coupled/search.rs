//! Branch only on measured conflicts in a continuous, jointly feasible placement.
use super::*;

pub(super) struct Search<'a> {
    model: &'a Model<'a>,
    nodes: ExploredSearchNodes,
    maximum: SearchBudget,
    numerical_failure: bool,
}

struct SelectedBranch {
    pair_index: usize,
    children: Vec<Candidate>,
}

struct Candidate {
    problem: Problem,
    positions: Vec<f64>,
}
impl<'a> Search<'a> {
    pub(super) fn new(model: &'a Model<'a>, maximum: SearchBudget) -> Self {
        Self {
            model,
            nodes: ExploredSearchNodes::default(),
            maximum,
            numerical_failure: false,
        }
    }
    pub(super) fn solve_counted(mut self) -> CountedSearchOutcome {
        let remaining = (0..self.model.pairs.len()).collect::<Vec<_>>();
        let result = self
            .visit(self.model.problem.clone(), remaining)
            .and_then(|p| {
                p.ok_or(if self.numerical_failure {
                    CoupledPackingIssue::NumericalFailure
                } else {
                    CoupledPackingIssue::SolverRejected
                })
            });
        CountedSearchOutcome {
            outcome: result.and_then(FrontageDisplacements::from_solver),
            explored_nodes: self.nodes,
        }
    }
    fn visit(
        &mut self,
        problem: Problem,
        remaining: Vec<usize>,
    ) -> Result<Option<Vec<f64>>, CoupledPackingIssue> {
        let Some(positions) = self.solve_problem(&problem)? else {
            return Ok(None);
        };
        self.advance(problem, remaining, positions)
    }
    fn solve_problem(
        &mut self,
        problem: &Problem,
    ) -> Result<Option<Vec<f64>>, CoupledPackingIssue> {
        if !self.nodes.attempt(self.maximum) {
            return Err(CoupledPackingIssue::SearchBudget {
                explored: self.nodes,
                maximum: self.maximum,
            });
        }
        let solution = match problem.solve() {
            Ok(outcome) => outcome
                .into_solution()
                .map_err(|_| CoupledPackingIssue::NumericalFailure)?,
            Err(microlp::Error::Infeasible) => return Ok(None),
            Err(microlp::Error::InternalError(message)) => {
                self.numerical_failure = true;
                #[cfg(test)]
                self.record_numerical_failure(&message, problem);
                #[cfg(not(test))]
                let _ = message;
                return Ok(None);
            }
            Err(_) => return Err(CoupledPackingIssue::InvalidModel),
        };
        Ok(Some(
            self.model
                .variables
                .iter()
                .map(|&v| solution.var_value(v))
                .collect(),
        ))
    }
    fn advance(
        &mut self,
        problem: Problem,
        remaining: Vec<usize>,
        positions: Vec<f64>,
    ) -> Result<Option<Vec<f64>>, CoupledPackingIssue> {
        let mut conflicts = remaining
            .iter()
            .copied()
            .filter(|&index| {
                self.model.pairs[index]
                    .conditions
                    .iter()
                    .all(|&condition| self.model.pairs[index].deficit(condition, &positions) > 0.0)
            })
            .collect::<Vec<_>>();
        conflicts.sort_by_key(|&index| (self.model.pairs[index].conditions.len(), index));
        let mut selected: Option<SelectedBranch> = None;
        for index in conflicts {
            let pair = &self.model.pairs[index];
            let mut directions = pair.conditions.clone();
            directions.sort_by(|&a, &b| {
                pair.deficit(a, &positions)
                    .total_cmp(&pair.deficit(b, &positions))
            });
            let mut children = Vec::new();
            for condition in directions {
                let mut branch = problem.clone();
                pair.add(condition, &mut branch, &self.model.variables);
                match self.solve_problem(&branch) {
                    Ok(Some(positions)) => children.push(Candidate {
                        problem: branch,
                        positions,
                    }),
                    Ok(None) => {}
                    Err(issue) => {
                        // A later sibling cannot erase an already complete LP
                        // solution. Rounded world geometry is still verified by
                        // the caller before accepting this selection.
                        if let Some(candidate) = children.iter().find(|candidate| {
                            remaining.iter().all(|&pair_index| {
                                let pair = &self.model.pairs[pair_index];
                                pair.conditions.iter().any(|&condition| {
                                    pair.deficit(condition, &candidate.positions) <= 0.0
                                })
                            })
                        }) {
                            return Ok(Some(candidate.positions.clone()));
                        }
                        return Err(issue);
                    }
                }
            }
            if children.is_empty() {
                return Ok(None);
            }
            if selected
                .as_ref()
                .is_none_or(|best| children.len() < best.children.len())
            {
                let forced = children.len() == 1;
                selected = Some(SelectedBranch {
                    pair_index: index,
                    children,
                });
                if forced {
                    break;
                }
            }
        }
        let Some(SelectedBranch {
            pair_index: index,
            children,
        }) = selected
        else {
            return Ok(Some(positions));
        };
        let remaining = remaining
            .into_iter()
            .filter(|&other| other != index)
            .collect::<Vec<_>>();
        for child in children {
            if let Some(positions) =
                self.advance(child.problem, remaining.clone(), child.positions)?
            {
                return Ok(Some(positions));
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
impl Search<'_> {
    fn record_numerical_failure(&self, message: &str, problem: &Problem) {
        use std::io::Write;
        let Some(directory) = std::env::var_os("FABELGEIST_SUPPORT_DIAGNOSTIC_DIR") else {
            return;
        };
        let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(directory);
        std::fs::create_dir_all(&directory).unwrap();
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(directory.join("coupled-numerical-failures.jsonl"))
            .unwrap();
        writeln!(
            file,
            "{}",
            serde_json::json!({"node":self.nodes,"problem":format!("{problem:?}"),"error":message})
        )
        .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model(objective: f64, bounds: (f64, f64)) -> Model<'static> {
        let mut model = Model::new(&[]);
        let variable = model.problem.add_var(objective, bounds);
        model.variables.push(variable);
        model
    }

    #[test]
    fn lp_infinity_is_intermediate_and_selected_displacements_are_finite() {
        let model = model(0.0, (f64::NEG_INFINITY, f64::INFINITY));
        let counted = Search::new(&model, SearchBudget::new(1)).solve_counted();
        let positions = counted.outcome.unwrap();
        assert!(
            positions
                .iter()
                .all(|position| position.metres().is_finite())
        );
        assert_eq!(counted.explored_nodes.count(), 1);
    }

    #[test]
    fn completed_infeasibility_and_exhaustion_remain_distinct() {
        let mut model = model(0.0, (0.0, 1.0));
        model
            .problem
            .add_constraint([(model.variables[0], 1.0)], ComparisonOp::Ge, 2.0);
        let completed = Search::new(&model, SearchBudget::new(1)).solve_counted();
        assert!(matches!(
            completed.outcome,
            Err(CoupledPackingIssue::SolverRejected)
        ));
        assert_eq!(completed.explored_nodes.count(), 1);
        let denied = Search::new(&model, SearchBudget::new(0)).solve_counted();
        assert!(matches!(
            denied.outcome,
            Err(CoupledPackingIssue::SearchBudget { .. })
        ));
        assert_eq!(denied.explored_nodes.count(), 0);
    }

    #[test]
    fn unbounded_objective_rejects_an_invalid_planning_model() {
        let model = model(-1.0, (0.0, f64::INFINITY));
        let counted = Search::new(&model, SearchBudget::new(2)).solve_counted();
        assert!(matches!(
            counted.outcome,
            Err(CoupledPackingIssue::InvalidModel)
        ));
        assert_eq!(counted.explored_nodes.count(), 1);
    }
}
