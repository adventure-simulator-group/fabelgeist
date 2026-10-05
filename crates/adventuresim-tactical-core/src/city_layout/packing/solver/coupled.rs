//! Joint geometric feasibility after inexpensive authored-position candidates.
use super::*;
use microlp::{ComparisonOp, OptimizationDirection, Problem, Variable};
mod geometry;
mod model;
mod orderings;
mod search;
mod setback;
mod verification;
use geometry::Envelope;
use model::Model;

/// Deterministic allocation per stable row-order candidate.
const MAX_ROW_ORDER_SEARCH_NODES: usize = 128;

pub(super) fn solve(
    domains: &[PlacementDomain],
    remaining_nodes: usize,
) -> Result<Vec<PropertyTranslation>, CityCompileError> {
    let first_budget = remaining_nodes / 2;
    let (selected, active) = match solve_at_fixed_setback(domains, first_budget) {
        Ok(positions) => (positions, None),
        Err(error) => {
            let spent = match error {
                CityCompileError::Packing {
                    issue:
                        CityPackingIssue::CoupledSearch {
                            issue: CoupledPackingIssue::SearchBudget { explored, .. },
                            ..
                        },
                    ..
                } => explored.min(first_budget),
                _ => first_budget,
            };
            let seated = setback::seat(domains)?;
            let positions = solve_at_fixed_setback(&seated, remaining_nodes - spent)?;
            (positions, Some(seated))
        }
    };
    let active = active.as_deref().unwrap_or(domains);
    Ok(selected
        .into_iter()
        .map(|selected| PropertyTranslation {
            domain: selected.domain,
            displacement: active[selected.domain.index()].delta_at(selected.position),
        })
        .collect())
}

fn solve_at_fixed_setback(
    domains: &[PlacementDomain],
    remaining_nodes: usize,
) -> Result<Vec<FrontageSelection>, CityCompileError> {
    let failure = |issue| CityCompileError::Packing {
        property: domains[0].owner,
        issue: CityPackingIssue::CoupledSearch {
            block: domains[0].frontage.block.id.0,
            members: domains.iter().map(|domain| domain.owner).collect(),
            issue,
        },
    };
    if remaining_nodes == 0 {
        return Err(failure(CoupledPackingIssue::SearchBudget {
            explored: 0,
            maximum: remaining_nodes,
        }));
    }
    #[cfg(test)]
    {
        let mut unrestricted = Model::new(domains);
        unrestricted.append_geometry().map_err(failure)?;
        unrestricted.record();
    }
    let rows = orderings::rows(domains);
    let mut explored = 0;
    for selected in orderings::Combinations::new(&rows) {
        explored += 1;
        if explored >= remaining_nodes {
            return Err(failure(CoupledPackingIssue::SearchBudget {
                explored,
                maximum: remaining_nodes,
            }));
        }
        let Some(narrowed) = orderings::constrain(domains, &selected) else {
            continue;
        };
        let mut model = Model::new(&narrowed);
        model.append_geometry().map_err(failure)?;
        model.problem = orderings::apply(&model, &selected);
        let maximum = remaining_nodes
            .saturating_sub(explored)
            .min(MAX_ROW_ORDER_SEARCH_NODES);
        let search = search::Search::new(&model, maximum);
        let counted = search.solve_counted();
        explored += counted.explored_nodes.count();
        if let Ok(positions) = counted.outcome
            && verification::validate(domains, &positions).is_ok()
        {
            return Ok(positions.into_selections());
        }
        if explored >= remaining_nodes {
            return Err(failure(CoupledPackingIssue::SearchBudget {
                explored,
                maximum: remaining_nodes,
            }));
        }
    }
    Err(failure(CoupledPackingIssue::SearchBudget {
        explored,
        maximum: remaining_nodes,
    }))
}
