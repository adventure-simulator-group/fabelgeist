//! Exact owned surfaces shared by single buildings and compound properties.
use super::*;
mod floor;
mod validation;
pub use floor::{FloorBearing, FloorBearingConstructionError, FloorBearingIssue, FloorRegion};
pub use validation::{SupportSurfaceAdmissionError, SupportSurfaceIssue};

/// Accepted architectural support, with explicit ownership and clipping bounds.
/// This projection contains no recipe selection or tactical simulation state.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "SupportSurfaceWire")]
pub struct PropertySupportSurface {
    pub(super) mesh: PropertySupportMesh,
    pub(super) floor_bearings: Vec<FloorBearing>,
    pub(super) regions: Vec<CityPlotBounds>,
    pub(super) clipping_outlines: Vec<Vec<bevy::math::DVec2>>,
    pub(super) limits: SupportLimits,
    pub(super) treatment: SupportGradingAttempt,
}

#[derive(Deserialize)]
struct SupportSurfaceWire {
    mesh: PropertySupportMesh,
    floor_bearings: Vec<FloorBearing>,
    regions: Vec<CityPlotBounds>,
    clipping_outlines: Vec<Vec<bevy::math::DVec2>>,
    limits: SupportLimits,
    treatment: SupportGradingAttempt,
}

impl TryFrom<SupportSurfaceWire> for PropertySupportSurface {
    type Error = SupportSurfaceAdmissionError;
    fn try_from(wire: SupportSurfaceWire) -> Result<Self, Self::Error> {
        let surface = Self {
            mesh: wire.mesh,
            floor_bearings: wire.floor_bearings,
            regions: wire.regions,
            clipping_outlines: wire.clipping_outlines,
            limits: wire.limits,
            treatment: wire.treatment,
        };
        surface
            .validate_encoded()
            .map_err(|issue| SupportSurfaceAdmissionError::for_mesh(&surface.mesh, issue))?;
        Ok(surface)
    }
}

impl PropertySupportSurface {
    pub fn property_id(&self) -> CityPropertyId {
        self.mesh.property_id
    }

    pub fn member_building_ids(&self) -> &[crate::scene_input::SceneBuildingId] {
        &self.mesh.member_building_ids
    }

    pub fn support_regions(&self) -> &[CityPlotBounds] {
        &self.regions
    }

    /// Exact accepted single-building approaches, including workplace passages.
    /// Their first region is the bearing floor; the rest follow the sorted
    /// ground entrance identities. Compound routes retain their own contract.
    pub(crate) fn doorway_approaches(&self) -> &[CityPlotBounds] {
        match self.treatment {
            SupportGradingAttempt::SingleBuildingFloorAndEntrances => &self.regions[1..],
            SupportGradingAttempt::Compound(_) | SupportGradingAttempt::NotSelected => &[],
        }
    }

    pub fn contains(&self, point: crate::scene_coordinates::ScenePlanPoint) -> bool {
        let point = point.metres();
        self.clipping_outlines.iter().any(|outline| {
            (0..outline.len()).all(|i| {
                (outline[(i + 1) % outline.len()] - outline[i])
                    .perp_dot(point.as_dvec2() - outline[i])
                    >= 0.0
            })
        })
    }

    /// Test the complete circular ground envelope against exact owned outlines.
    /// A centre outside an approach can still obstruct its usable width.
    pub(crate) fn intersects_disc(&self, centre: Vec2, radius_metres: f32) -> bool {
        self.clipping_outlines.iter().any(|outline| {
            planar::distance_outside(centre.as_dvec2(), outline) <= f64::from(radius_metres)
        })
    }

    pub fn mesh(&self) -> &PropertySupportMesh {
        &self.mesh
    }

    pub fn foundations(
        &self,
        geographic: &GeographicSurface,
        embedment: FoundationEmbedment,
    ) -> Result<PropertyFoundationMesh, SupportDiagnostic> {
        foundations::compile(self, geographic, embedment)
    }

    /// Complete source coverage, displacement and boundary checks for selection.
    /// Closed foundation cells are materialized only by final composition.
    pub(super) fn validate_source_controls(
        &self,
        geographic: &GeographicSurface,
    ) -> Result<(), SupportDiagnostic> {
        foundations::validate_source_controls(self, geographic)
    }

    pub(super) fn rejection(
        &self,
        constraint: SupportConstraint,
        boundary: SupportBoundary,
        location_metres: Vec2,
        measured: f32,
        permitted: f32,
    ) -> SupportDiagnostic {
        SupportDiagnostic {
            entrance: None,
            construction_failure: None,
            property_id: self.property_id(),
            member_building_ids: self.mesh.member_building_ids.clone(),
            constraint,
            boundary,
            location_metres: SupportDiagnosticLocation::from_attempt_metres(location_metres),
            violation: SupportViolation::maximum(
                constraint.diagnostic_unit(),
                f64::from(measured),
                f64::from(permitted),
            ),
            attempted_treatment: Box::new(self.treatment),
        }
    }
}

impl CompoundSupportPlan {
    /// Freeze the accepted surface after floor, court, gate and doorway checks.
    /// Every property shares this representation when composing city terrain.
    pub fn support_surface(&self) -> Result<PropertySupportSurface, SupportDiagnostic> {
        Ok(PropertySupportSurface {
            mesh: self.mesh()?,
            floor_bearings: self
                .member_support()
                .into_iter()
                .map(|member| FloorBearing::from_member(self, member))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|cause| SupportDiagnostic::floor_geometry(&self.property, cause))?,
            regions: self.support_regions(),
            clipping_outlines: self.source_clipping_outlines(),
            limits: self.limits,
            treatment: SupportGradingAttempt::Compound(self.treatment),
        })
    }
}
