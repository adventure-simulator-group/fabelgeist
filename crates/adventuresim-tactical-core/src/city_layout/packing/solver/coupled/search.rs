//! Branch only on measured conflicts in a continuous, jointly feasible placement.
use super::*;

pub(super) struct Search<'a> {
    model: &'a Model<'a>,
    nodes: usize,
    maximum: usize,
    numerical_failure: bool,
}
impl<'a> Search<'a> {
    pub(super) fn new(model: &'a Model<'a>, maximum: usize) -> Self {
        Self {
            model,
            nodes: 0,
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
            outcome: result.and_then(FrontageCoordinates::from_solver),
            explored_nodes: ExploredSearchNodes::new(self.nodes),
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
        if self.nodes >= self.maximum {
            return Err(CoupledPackingIssue::SearchBudget {
                explored: self.nodes,
                maximum: self.maximum,
            });
        }
        self.nodes += 1;
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
        let mut selected: Option<(usize, Vec<Candidate>)> = None;
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
                if let Some(positions) = self.solve_problem(&branch)? {
                    children.push(Candidate {
                        problem: branch,
                        positions,
                    });
                }
            }
            if children.is_empty() {
                return Ok(None);
            }
            if selected
                .as_ref()
                .is_none_or(|(_, best)| children.len() < best.len())
            {
                let forced = children.len() == 1;
                selected = Some((index, children));
                if forced {
                    break;
                }
            }
        }
        let Some((index, children)) = selected else {
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

struct Candidate {
    problem: Problem,
    positions: Vec<f64>,
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
