//! Coupled placement rejection is distinct from an unsupported terrain slope.
use super::*;

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoupledPackingIssue {
    Garden {
        property: CityPropertyId,
        issue: gardens::GardenIssue,
    },
    InvalidGeometry {
        property: CityPropertyId,
        issue: adventuresim_building_generator::plan_geometry::PlanGeometryError,
    },
    SolverRejected,
    InvalidModel,
    NumericalFailure,
    SearchBudget {
        explored: ExploredSearchNodes,
        maximum: SearchBudget,
    },
    OutsideDomain {
        property: CityPropertyId,
        displacement_metres: FrontageDisplacement,
        permitted: FrontageInterval,
    },
    Overlap {
        first: CityPropertyId,
        second: CityPropertyId,
    },
}
