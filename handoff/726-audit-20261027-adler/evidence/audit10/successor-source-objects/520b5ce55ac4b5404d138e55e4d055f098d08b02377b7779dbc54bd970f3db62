//! A rejected property identifies the exact attempted support strategy.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SupportGradingAttempt {
    NotSelected,
    SingleBuildingFloorAndEntrances,
    Compound(CourtTreatment),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SupportConstraint {
    MemberBinding,
    ThresholdBinding,
    ThresholdBearing,
    GateBinding,
    Reservation,
    MeshIndexCapacity,
    Bearing,
    AccessGrade,
    StairGoing,
    StairClearance,
    CutFill,
    SurfaceCoverage,
    BoundaryCoverage,
    SurfaceOverlap,
    SourceSample,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SupportBoundary {
    PropertyReservation,
    GateLanding,
    CourtLanding,
    StreetLanding,
    FrontBearing,
    RearBearing,
    GeographicSurface,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum SupportConstructionError {
    Surface(SupportSurfaceIssue),
    Floor(super::surface::FloorBearingConstructionError),
    FramedGeometry(adventuresim_building_generator::spatial_geometry::GeometryError),
    Geometry(SupportGeometryIssue),
    Door(adventuresim_building_generator::DoorError),
    Boundary(crate::city_layout::BoundaryGeometryError),
}

/// Whether a measurement must reach, remain below, or match its bound.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SupportBound {
    Minimum,
    Maximum,
    Exact,
}

/// Rejection identifies the immutable property, members and precise shortfall.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SupportDiagnostic {
    pub entrance: Option<adventuresim_building_generator::BuildingEntranceId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub construction_failure: Option<Box<SupportConstructionError>>,
    pub property_id: CityPropertyId,
    pub member_building_ids: Vec<crate::scene_input::SceneBuildingId>,
    pub constraint: SupportConstraint,
    pub boundary: SupportBoundary,
    pub location_metres: SupportDiagnosticLocation,
    pub violation: SupportViolation,
    pub attempted_treatment: Box<SupportGradingAttempt>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SupportDiagnosticUnit {
    Metres,
    SquareMetres,
    Count,
}

impl SupportDiagnostic {
    pub(super) fn floor_geometry(
        property: &CityCompound,
        cause: super::surface::FloorBearingConstructionError,
    ) -> Self {
        let mut diagnostic = Self::new(
            property,
            SupportConstraint::Bearing,
            SupportBoundary::PropertyReservation,
            property.plot.centre_metres(),
            1.0,
            0.0,
        );
        diagnostic.construction_failure = Some(Box::new(SupportConstructionError::Floor(cause)));
        diagnostic
    }
    pub(in crate::city_layout) fn gate_position(
        property: &CityCompound,
        cause: adventuresim_building_generator::spatial_geometry::GeometryError,
    ) -> Self {
        let mut diagnostic = Self::new(
            property,
            SupportConstraint::GateBinding,
            SupportBoundary::GateLanding,
            property.boundary.gate.centre_metres,
            1.0,
            0.0,
        );
        diagnostic.construction_failure =
            Some(Box::new(SupportConstructionError::FramedGeometry(cause)));
        diagnostic
    }
    pub(super) fn boundary_construction(
        property: &CityCompound,
        cause: crate::city_layout::BoundaryGeometryError,
    ) -> Self {
        let mut diagnostic = Self::new(
            property,
            SupportConstraint::GateBinding,
            SupportBoundary::GateLanding,
            property.boundary.gate.centre_metres,
            1.0,
            0.0,
        );
        diagnostic.construction_failure = Some(Box::new(SupportConstructionError::Boundary(cause)));
        diagnostic
    }
    pub(super) fn gate_construction(
        property: &CityCompound,
        cause: adventuresim_building_generator::DoorError,
    ) -> Self {
        let mut diagnostic = Self::new(
            property,
            SupportConstraint::GateBinding,
            SupportBoundary::GateLanding,
            property.boundary.gate.centre_metres,
            1.0,
            0.0,
        );
        diagnostic.construction_failure = Some(Box::new(SupportConstructionError::Door(cause)));
        diagnostic
    }
    pub(super) fn for_mesh(
        mesh: &PropertySupportMesh,
        constraint: SupportConstraint,
        location: Vec2,
        measured: f32,
        permitted: f32,
    ) -> Self {
        Self {
            entrance: None,
            construction_failure: None,
            property_id: mesh.property_id,
            member_building_ids: mesh.member_building_ids.clone(),
            constraint,
            boundary: SupportBoundary::PropertyReservation,
            location_metres: SupportDiagnosticLocation::from_attempt_metres(location),
            violation: SupportViolation::maximum(
                constraint.diagnostic_unit(),
                f64::from(measured),
                f64::from(permitted),
            ),
            attempted_treatment: Box::new(SupportGradingAttempt::NotSelected),
        }
    }

    pub(super) fn new(
        property: &CityCompound,
        constraint: SupportConstraint,
        boundary: SupportBoundary,
        location: Vec2,
        measured: f32,
        permitted: f32,
    ) -> Self {
        Self {
            entrance: None,
            construction_failure: None,
            property_id: property.id,
            member_building_ids: vec![property.front_building_id, property.rear_building_id],
            constraint,
            boundary,
            location_metres: SupportDiagnosticLocation::from_attempt_metres(location),
            violation: SupportViolation::maximum(
                constraint.diagnostic_unit(),
                f64::from(measured),
                f64::from(permitted),
            ),
            attempted_treatment: Box::new(SupportGradingAttempt::NotSelected),
        }
    }
}

impl SupportConstraint {
    pub(super) fn diagnostic_unit(self) -> SupportDiagnosticUnit {
        match self {
            SupportConstraint::SurfaceCoverage | SupportConstraint::SurfaceOverlap => {
                SupportDiagnosticUnit::SquareMetres
            }
            SupportConstraint::MeshIndexCapacity
            | SupportConstraint::MemberBinding
            | SupportConstraint::ThresholdBinding
            | SupportConstraint::GateBinding
            | SupportConstraint::Reservation
            | SupportConstraint::SourceSample => SupportDiagnosticUnit::Count,
            _ => SupportDiagnosticUnit::Metres,
        }
    }
}

impl SupportDiagnostic {
    pub(crate) fn framed_geometry(
        property: &CityCompound,
        point: Vec2,
        cause: adventuresim_building_generator::spatial_geometry::GeometryError,
    ) -> Self {
        let mut error = Self::new(
            property,
            SupportConstraint::Reservation,
            SupportBoundary::PropertyReservation,
            point,
            1.0,
            0.0,
        );
        error.construction_failure =
            Some(Box::new(SupportConstructionError::FramedGeometry(cause)));
        error
    }
}
