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

/// Rejection identifies the immutable property, members and precise shortfall.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SupportDiagnostic {
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
    pub(super) fn new(
        property: &CityCompound,
        constraint: SupportConstraint,
        boundary: SupportBoundary,
        location: Vec2,
        measured: f32,
        permitted: f32,
    ) -> Self {
        Self {
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
            SupportConstraint::MemberBinding
            | SupportConstraint::ThresholdBinding
            | SupportConstraint::GateBinding
            | SupportConstraint::Reservation
            | SupportConstraint::SourceSample => SupportDiagnosticUnit::Count,
            _ => SupportDiagnosticUnit::Metres,
        }
    }
}
