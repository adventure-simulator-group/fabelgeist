//! Measured physical packing preserves the selected roster and its programmes.
use super::*;
use crate::scene_coordinates::{PlanDisplacement, ScenePlanPoint, ScenePlanPolygon};
use bevy::math::DVec2;
use bevy::math::Dir2;
pub use budget::{ExploredSearchNodes, SearchBudget};
pub(super) use context::CityPackingContext;
pub use coordinates::FrontageDisplacement;
pub use coupled_issue::CoupledPackingIssue;
pub use intervals::{FrontageInterval, FrontageIntervalError};
use std::collections::BTreeMap;
mod budget;
mod context;
mod coordinates;
mod coupled_issue;
mod intervals;
mod projection;
mod solver;

/// Elevated projections constrain building pairs; floor contacts constrain land.
#[derive(Clone, Debug)]
pub(in crate::city_layout) struct MeasuredBuildingEnvelope {
    pub building: crate::scene_input::SceneBuildingId,
    pub body: CityPlotBounds,
    pub bearing_outline: ScenePlanPolygon,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CityPackingIssue {
    CoupledSearch {
        block: BlockId,
        members: Vec<CityPropertyId>,
        issue: CoupledPackingIssue,
    },
    MissingFrontage,
    MissingMember {
        building: crate::scene_input::SceneBuildingId,
    },
    InvalidBearing {
        building: crate::scene_input::SceneBuildingId,
        issue: adventuresim_building_generator::plan_geometry::PlanGeometryError,
    },
    MissingBearing {
        building: crate::scene_input::SceneBuildingId,
    },
    BearingOutsidePlot {
        building: crate::scene_input::SceneBuildingId,
        minimum_local_metres: Vec2,
        maximum_local_metres: Vec2,
        plot_half_dimensions_metres: Vec2,
    },
    BuildingOverlap {
        first: crate::scene_input::SceneBuildingId,
        second: crate::scene_input::SceneBuildingId,
    },
    SearchBudget {
        block: BlockId,
        explored: ExploredSearchNodes,
        maximum: SearchBudget,
        members: Vec<CityPropertyId>,
    },
    NoFreeFrontage {
        block: BlockId,
        envelope: CityPlotBounds,
        available_displacement_metres: Option<FrontageInterval>,
        blocking_properties: Vec<CityPropertyId>,
    },
    GardenBlocked {
        building: crate::scene_input::SceneBuildingId,
    },
    Garden {
        issue: gardens::GardenIssue,
    },
}

impl CompiledCityLayout {
    /// Solve translations against actual envelopes and complete private parcels
    /// after programmes are fixed. No lot, member or programme can be replaced.
    pub(super) fn pack_properties(
        &mut self,
        context: &CityPackingContext,
        envelopes: &[MeasuredBuildingEnvelope],
    ) -> CityCompileResult<()> {
        let envelope_map: BTreeMap<_, _> = envelopes
            .iter()
            .map(|envelope| (envelope.building, envelope.clone()))
            .collect();
        let translations = solver::solve(self, context, &envelope_map)?;
        #[cfg(test)]
        let original_gardens = self.gardens.clone();
        let building_translations = projection::apply(self, context, &translations)?;
        let validation = projection::validate_gardens(self, envelopes, &building_translations);
        #[cfg(test)]
        tests::record_garden_failure(&original_gardens, self, &translations, &validation);
        validation?;
        Ok(())
    }
}
#[cfg(test)]
mod tests;
