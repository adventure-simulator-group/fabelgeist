//! Coupled placement rejection is distinct from an unsupported terrain slope.
use super::*;

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoupledPackingIssue {
    Garden {
        property: CityPropertyId,
        issue: gardens::GardenIssue,
    },
    SolverRejected,
    InvalidModel,
    NumericalFailure,
    SearchBudget {
        explored: usize,
        maximum: usize,
    },
    OutsideDomain {
        property: CityPropertyId,
        displacement_metres: f64,
        permitted: FrontageInterval,
    },
    Overlap {
        first: CityPropertyId,
        second: CityPropertyId,
    },
}
