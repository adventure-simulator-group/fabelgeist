//! Joint geometric feasibility after inexpensive authored placement candidates.
use super::*;
use geometry::Envelope;
use microlp::{ComparisonOp, OptimizationDirection, Problem, Variable};
use model::Model;

mod geometry;
mod model;
mod orderings;
mod search;
mod setback;
mod verification;

/// Deterministic allocation per stable row-order candidate.
const MAX_ROW_ORDER_SEARCH_NODES: SearchBudget = SearchBudget::new(128);

struct FixedSetbackOutcome {
    selected: Result<Vec<FrontageSelection>, CoupledPackingIssue>,
    explored: ExploredSearchNodes,
}

/// Failed rows do not prevent a later verified solution from winning.
struct Rejections {
    issue: CoupledPackingIssue,
}

impl Rejections {
    fn new() -> Self {
        Self {
            issue: CoupledPackingIssue::SolverRejected,
        }
    }
    fn record(&mut self, issue: CoupledPackingIssue) {
        let precedence = |issue: &CoupledPackingIssue| match issue {
            CoupledPackingIssue::SolverRejected => 0,
            CoupledPackingIssue::SearchBudget { .. } => 1,
            CoupledPackingIssue::NumericalFailure => 3,
            _ => 2,
        };
        if precedence(&issue) > precedence(&self.issue) {
            self.issue = issue;
        }
    }
}

pub(super) fn solve(
    domains: &[PlacementDomain],
    remaining_nodes: SearchBudget,
) -> CityCompileResult<Vec<PropertyTranslation>> {
    let first_budget = remaining_nodes.first_half();
    let first = solve_at_fixed_setback(domains, first_budget);
    let (selected, active) = match first.selected {
        Ok(positions) => (positions, None),
        Err(first_issue) => {
            let seated = setback::seat(domains)?;
            let second = solve_at_fixed_setback(&seated, remaining_nodes.remaining(first.explored));
            let positions = second.selected.map_err(|second_issue| {
                let mut failures = Rejections::new();
                failures.record(first_issue);
                failures.record(second_issue);
                failure(domains, failures.issue)
            })?;
            (positions, Some(seated))
        }
    };
    let active = active.as_deref().unwrap_or(domains);
    selected
        .into_iter()
        .map(|selected| {
            let domain = &active[selected.domain.index()];
            Ok(PropertyTranslation {
                domain: selected.domain,
                displacement: domain
                    .delta_at(selected.frontage_displacement)
                    .map_err(|issue| domain.packing_error(issue))?,
            })
        })
        .collect()
}

fn failure(domains: &[PlacementDomain], issue: CoupledPackingIssue) -> CityCompileError {
    CityCompileError::Packing {
        property: domains[0].owner,
        issue: CityPackingIssue::CoupledSearch {
            block: domains[0].frontage.block.id,
            members: domains.iter().map(|domain| domain.owner).collect(),
            issue,
        },
    }
}

fn solve_at_fixed_setback(
    domains: &[PlacementDomain],
    allocation: SearchBudget,
) -> FixedSetbackOutcome {
    let mut explored = ExploredSearchNodes::default();
    let selected = search_rows(domains, allocation, &mut explored);
    FixedSetbackOutcome { selected, explored }
}

fn search_rows(
    domains: &[PlacementDomain],
    allocation: SearchBudget,
    explored: &mut ExploredSearchNodes,
) -> Result<Vec<FrontageSelection>, CoupledPackingIssue> {
    let exhausted = |explored| CoupledPackingIssue::SearchBudget {
        explored,
        maximum: allocation,
    };
    if allocation.exhausted(*explored) {
        return Err(exhausted(*explored));
    }
    #[cfg(test)]
    {
        let mut unrestricted = Model::new(domains);
        unrestricted.append_geometry()?;
        unrestricted.record();
    }
    let rows = orderings::rows(domains);
    let mut failures = Rejections::new();
    for selected in orderings::Combinations::new(&rows) {
        // A row-order attempt consumes one state before its allocated LP search.
        if !explored.attempt(allocation) || allocation.exhausted(*explored) {
            failures.record(exhausted(*explored));
            return Err(failures.issue);
        }
        let Some(narrowed) = orderings::constrain(domains, &selected)? else {
            continue;
        };
        let mut model = Model::new(&narrowed);
        model.append_geometry()?;
        model.problem = orderings::apply(&model, &selected)?;
        let maximum = allocation
            .remaining(*explored)
            .capped_by(MAX_ROW_ORDER_SEARCH_NODES);
        let counted = search::Search::new(&model, maximum).solve_counted();
        explored.include(counted.explored_nodes)?;
        match counted.outcome {
            Ok(positions) => match verification::validate(domains, &positions) {
                Ok(()) => return Ok(positions.into_selections()),
                Err(issue) => failures.record(issue),
            },
            Err(issue) => failures.record(issue),
        }
        if allocation.exhausted(*explored) {
            failures.record(exhausted(*explored));
            return Err(failures.issue);
        }
    }
    Err(failures.issue)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unsuccessful_outcomes_do_not_invent_budget_exhaustion() {
        let mut failures = Rejections::new();
        assert!(matches!(
            failures.issue,
            CoupledPackingIssue::SolverRejected
        ));
        failures.record(CoupledPackingIssue::InvalidModel);
        assert!(matches!(failures.issue, CoupledPackingIssue::InvalidModel));
        failures.record(CoupledPackingIssue::NumericalFailure);
        failures.record(CoupledPackingIssue::SolverRejected);
        assert!(matches!(
            failures.issue,
            CoupledPackingIssue::NumericalFailure
        ));
        let allocation = SearchBudget::new(128);
        let mut truncated = Rejections::new();
        truncated.record(CoupledPackingIssue::SearchBudget {
            explored: ExploredSearchNodes::new(128),
            maximum: allocation,
        });
        assert!(matches!(
            truncated.issue,
            CoupledPackingIssue::SearchBudget { .. }
        ));
    }
}
