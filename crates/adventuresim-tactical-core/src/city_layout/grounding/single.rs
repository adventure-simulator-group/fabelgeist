//! Single-building floors retain their reservation and bound every ground door.
use super::*;
use crate::city_layout::{CitySingleProperty, CityStreetPatch, SinglePropertyGradingPolicy};
use crate::scene_input::BuildingOrientation;
mod apron;
mod floor;
#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DoorwaySupportBinding {
    pub entrance: adventuresim_building_generator::BuildingEntranceId,
    pub support: adventuresim_building_generator::BuildingEntranceSupport,
    pub threshold_metres: Vec2,
    pub outward: Vec2,
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
    pub bearing_outline: Vec<Vec2>,
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
        let mut surface = PropertySupportSurface {
            mesh: PropertySupportMesh {
                property_id: self.property.id,
                member_building_ids: vec![self.property.building_id],
                positions: Vec::new(),
                support_triangles: Vec::new(),
                retaining_triangles: Vec::new(),
                contact_tolerance_metres: self.policy.limits.contact_tolerance_metres,
            },
            regions: vec![self.bearing],
            clipping_outlines: vec![self.bearing_outline.iter().map(|p| p.as_dvec2()).collect()],
            limits: self.policy.limits,
            treatment: SupportGradingAttempt::SingleBuildingFloorAndEntrances,
        };
        if !self.bearing.is_valid()
            || !self.property.plot.is_valid()
            || self.bearing_outline.len() < 3
            || self.bearing_outline.iter().any(|p| !p.is_finite())
            || planar::signed_area(&surface.clipping_outlines[0]) <= f64::EPSILON
        {
            return Err(surface.rejection(
                SupportConstraint::Reservation,
                SupportBoundary::PropertyReservation,
                self.bearing.centre_metres,
                1.0,
                0.0,
            ));
        }
        if let Some(point) = self
            .bearing_outline
            .iter()
            .find(|point| !self.property.plot.contains(**point))
        {
            let local = self
                .property
                .plot
                .orientation
                .world_to_local(*point - self.property.plot.centre_metres)
                .abs();
            let excess = (local - self.property.plot.dimensions_metres * 0.5).max_element();
            return Err(surface.rejection(
                SupportConstraint::Bearing,
                SupportBoundary::PropertyReservation,
                *point,
                excess,
                CityPlotBounds::COORDINATE_TOLERANCE_METRES as f32,
            ));
        }
        let mut thresholds = self.thresholds.to_vec();
        thresholds.sort_by_key(|threshold| threshold.entrance);
        if thresholds.is_empty()
            || thresholds
                .windows(2)
                .any(|pair| pair[0].entrance == pair[1].entrance)
        {
            return Err(surface.rejection(
                SupportConstraint::ThresholdBinding,
                SupportBoundary::FrontBearing,
                self.bearing.centre_metres,
                thresholds.len() as f32,
                1.0,
            ));
        }
        let constructed: Vec<_> = thresholds
            .iter()
            .filter(|entry| {
                entry.support
                    == adventuresim_building_generator::BuildingEntranceSupport::ArchitecturalFloor
            })
            .collect();
        let aprons = constructed
            .iter()
            .map(|door| apron::prepare(&self, **door, &surface))
            .collect::<Result<Vec<_>, _>>()?;
        let (threshold, elevation) = floor::select(&self, &constructed, &aprons, &surface)?;
        surface.mesh.convex_floor(&self.bearing_outline, elevation);
        for prepared in aprons {
            prepared.append_to(&self, elevation, &mut surface)?;
        }
        surface.validate_source_controls(self.geographic)?;
        Ok(SingleBuildingSupportPlan {
            property: self.property,
            floor: MemberSupport {
                building_id: self.property.building_id,
                contact: self.bearing,
                court_threshold_metres: threshold,
                elevation,
            },
            thresholds,
            surface,
        })
    }
}
