//! Single-building floors retain their reservation and bound every ground door.
use super::*;
use crate::city_layout::{CitySingleProperty, CityStreetPatch, SinglePropertyGradingPolicy};
use crate::scene_input::BuildingOrientation;
mod apron;
mod bearing;
mod floor;
#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DoorwaySupportBinding {
    pub entrance: adventuresim_building_generator::BuildingEntranceId,
    pub support: adventuresim_building_generator::BuildingEntranceSupport,
    pub threshold_metres: crate::scene_coordinates::ScenePlanPoint,
    pub outward: adventuresim_building_generator::spatial_geometry::PlanDirection<
        crate::scene_coordinates::Scene,
    >,
}

#[derive(Clone, Debug)]
pub struct SingleBuildingSupportPlan {
    pub property: CitySingleProperty,
    pub floor: MemberSupport,
    pub thresholds: Vec<DoorwaySupportBinding>,
    pub surface: PropertySupportSurface,
}

pub struct SingleBuildingSupportRequest<'a> {
    pub property: CitySingleProperty,
    pub bearing: CityPlotBounds,
    pub bearing_region: FloorRegion,
    pub thresholds: &'a [DoorwaySupportBinding],
    pub geographic: &'a GeographicSurface,
    pub streets: &'a [CityStreetPatch],
    pub policy: SinglePropertyGradingPolicy,
}

impl SingleBuildingSupportRequest<'_> {
    /// Select the architectural floor from an exact ground doorway, then prove
    /// complete source intersections and all doorway approaches. Other courts
    /// and gardens retain natural terrain; this does not level the whole plot.
    pub fn select(self) -> Result<SingleBuildingSupportPlan, SupportDiagnostic> {
        let mut surface = PropertySupportSurface::for_single_bearing(&self)?;
        let thresholds = self.thresholds.to_vec();
        if thresholds.is_empty() {
            let mut error = surface.rejection(
                SupportConstraint::ThresholdBinding,
                SupportBoundary::FrontBearing,
                self.bearing.centre_metres(),
                0.0,
                1.0,
            );
            error.violation = error.violation.with_bound(SupportBound::Minimum);
            return Err(error);
        }
        if let Some(duplicate) = thresholds.iter().enumerate().find_map(|(index, entry)| {
            thresholds[..index]
                .iter()
                .any(|previous| previous.entrance == entry.entrance)
                .then_some(entry)
        }) {
            let mut error = surface.rejection(
                SupportConstraint::ThresholdBinding,
                SupportBoundary::FrontBearing,
                duplicate.threshold_metres.metres(),
                2.0,
                1.0,
            );
            error.violation = error.violation.with_bound(SupportBound::Exact);
            error.entrance = Some(duplicate.entrance);
            return Err(error);
        }
        let mut constructed: Vec<_> = thresholds
            .iter()
            .filter(|entry| {
                entry.support
                    == adventuresim_building_generator::BuildingEntranceSupport::ArchitecturalFloor
            })
            .collect();
        // Retain the admitted entrance order while preserving the established
        // lowest-identity floor selection and canonical apron assembly order.
        constructed.sort_by_key(|entry| entry.entrance);
        let aprons = constructed
            .iter()
            .map(|door| apron::prepare(&self, **door, &surface))
            .collect::<Result<Vec<_>, _>>()?;
        let selected = floor::select(&self, &constructed, &aprons, &surface)?;
        let elevation = selected.elevation;
        surface.floor_bearings = vec![surface::FloorBearing {
            building: self.property.building_id,
            regions: vec![self.bearing_region.clone()],
            elevation,
        }];
        surface
            .mesh
            .convex_floor(self.bearing_region.outline(), elevation)?;
        for prepared in aprons {
            prepared.append_to(&self, elevation, &mut surface)?;
        }
        surface.validate_source_controls(self.geographic)?;
        Ok(SingleBuildingSupportPlan {
            property: self.property,
            floor: MemberSupport {
                building_id: self.property.building_id,
                contact: self.bearing,
                court_threshold_metres: selected.threshold,
                elevation,
            },
            thresholds,
            surface,
        })
    }
}
