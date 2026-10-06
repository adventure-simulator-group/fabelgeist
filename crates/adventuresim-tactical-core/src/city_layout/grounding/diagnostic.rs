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
    Door(adventuresim_building_generator::DoorError),
    Boundary(crate::city_layout::BoundaryGeometryError),
}

/// Rejection identifies the immutable property, members and precise shortfall.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SupportDiagnostic {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub construction_failure: Option<SupportConstructionError>,
    pub property_id: CityPropertyId,
    pub member_building_ids: Vec<u64>,
    pub constraint: SupportConstraint,
    pub boundary: SupportBoundary,
    pub location_metres: Vec2,
    pub measured: f32,
    pub permitted: f32,
    pub shortfall: f32,
    pub unit: SupportDiagnosticUnit,
    pub attempted_treatment: SupportGradingAttempt,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SupportDiagnosticUnit {
    Metres,
    SquareMetres,
    Count,
}

impl SupportDiagnostic {
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
        diagnostic.construction_failure = Some(SupportConstructionError::Boundary(cause));
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
        diagnostic.construction_failure = Some(SupportConstructionError::Door(cause));
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
            construction_failure: None,
            property_id: mesh.property_id,
            member_building_ids: mesh.member_building_ids.clone(),
            constraint,
            boundary: SupportBoundary::PropertyReservation,
            location_metres: location,
            measured,
            permitted,
            shortfall: (measured - permitted).max(0.0),
            unit: constraint.diagnostic_unit(),
            attempted_treatment: SupportGradingAttempt::NotSelected,
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
            construction_failure: None,
            property_id: property.id,
            member_building_ids: vec![property.front_building_id, property.rear_building_id],
            constraint,
            boundary,
            location_metres: location,
            measured,
            permitted,
            shortfall: (measured - permitted).max(0.0),
            unit: constraint.diagnostic_unit(),
            attempted_treatment: SupportGradingAttempt::NotSelected,
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
