//! Owned, bounded support surfaces for generated merchant properties.
//!
//! Planning retains the complete horizontal reservation and member identities.
//! A higher gate platform and a lower building floor are separate surfaces;
//! a retaining face joins them without inventing a shared heightfield datum.
//! This compiler does not mutate geographic terrain or publish scene products.

use super::{CityAccessSegment, CityCompound, CityPlotBounds, CityPropertyId};
use bevy::math::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

pub(super) mod access;
mod compound;
mod single;
mod surface;
pub use single::{DoorwaySupportBinding, SingleBuildingSupportPlan, SingleBuildingSupportRequest};
pub use surface::{
    FloorBearing, FloorBearingConstructionError, FloorBearingIssue, FloorRegion,
    PropertySupportSurface, SupportSurfaceAdmissionError, SupportSurfaceIssue,
};
mod admission;
mod collider;
pub use collider::{SupportColliderError, SupportCollision};
mod members;
pub use admission::SupportGeometryIssue;
pub use members::{PropertyMemberIssue, PropertyMembers};
mod boundaries;
mod diagnostic;
mod location;
mod measurement;
pub use location::{SceneCoordinateAttempt, SupportDiagnosticLocation};
pub use measurement::{
    BoundViolation, DiagnosticArea, DiagnosticCount, DiagnosticMetres, GeographicGrade,
    SupportViolation,
};
pub(crate) mod enclosure;
mod entry;
pub use enclosure::{
    BoundarySupportCell, BoundarySupportConstraint, BoundarySupportElement, BoundarySupportError,
    BoundarySupportMesh, BoundarySupportProjection,
};
mod foundations;
mod mesh;
mod planar;
mod policy;
pub use policy::{SupportGrade, SupportLimits};
mod profile;
mod surface_hit;
pub use diagnostic::{
    SupportBound, SupportBoundary, SupportConstraint, SupportConstructionError, SupportDiagnostic,
    SupportDiagnosticUnit, SupportGradingAttempt,
};
pub use surface_hit::SurfaceHit;
mod query;
pub use query::{SupportCeiling, SupportQuery};
mod selection;
pub use selection::CompoundSupportRequest;
mod stairs;
#[cfg(test)]
mod tests;
pub use foundations::{
    BoundedPropertyTerrain, BoundedSettlementTerrain, FoundationEmbedment, GeographicHeightControl,
    GeographicHeightRange, GeographicRegionMeasurement, GeographicSurface,
    GeographicSurfaceComparison, PropertyFoundationMesh, SettlementSupportError,
    SurfaceDifferenceControl,
};
pub use mesh::PropertySupportMesh;
use mesh::SupportFaceRole;
use profile::{FloorBasisRole, ProfileCoordinate, ProfilePoint, SupportProfile};
use stairs::CourtStair;
pub use stairs::CourtStairLimits;

/// Finite architectural floor or support elevation in scene metres.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, bevy::reflect::Reflect)]
#[reflect(opaque)]
#[serde(transparent)]
pub struct SupportElevation(f32);

impl SupportElevation {
    pub const ZERO: Self = Self(0.0);
    pub fn from_metres(metres: f32) -> Option<Self> {
        metres.is_finite().then_some(Self(metres))
    }
    pub fn metres(self) -> f32 {
        self.0
    }
}

impl<'de> Deserialize<'de> for SupportElevation {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let metres = f32::deserialize(deserializer)?;
        Self::from_metres(metres)
            .ok_or_else(|| serde::de::Error::custom("support elevation must be finite"))
    }
}

/// A spatial query can encounter both levels of a retaining boundary.
/// Consumers must use their bound floor, route or gate to select support.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(transparent)]
pub struct SurfaceElevations(Vec<SupportElevation>);

impl SurfaceElevations {
    pub fn from_elevation(elevation: SupportElevation) -> Self {
        Self(vec![elevation])
    }

    pub fn iter(&self) -> impl Iterator<Item = SupportElevation> + '_ {
        self.0.iter().copied()
    }
}

/// Exact bearing reservation, court threshold and selected member floor.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct MemberSupport {
    pub building_id: crate::scene_input::SceneBuildingId,
    pub contact: CityPlotBounds,
    pub court_threshold_metres: crate::scene_coordinates::ScenePlanPoint,
    pub elevation: SupportElevation,
}

/// Chosen support levels at explicitly bound access endpoints. Geographic
/// observations remain separate; selecting a level does not prove bounded grading.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct CompoundSupportLevels {
    pub front: MemberSupport,
    pub rear: MemberSupport,
    pub court: SupportElevation,
    pub gate: SupportElevation,
    pub street: SupportElevation,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CourtTreatment {
    Level,
    Terraced(CourtStairLimits),
}

/// Complete property top surface, stepped court and independently seated gate.
/// Two surfaces can meet at different elevations across a retaining face.
#[derive(Clone, Debug)]
pub struct CompoundSupportPlan {
    property: CityCompound,
    levels: CompoundSupportLevels,
    limits: SupportLimits,
    passage: CityAccessSegment,
    passage_region: CityPlotBounds,
    split_frontage_metres: f32,
    court_profile: SupportProfile,
    passage_profile: SupportProfile,
    court_stairs: Vec<CourtStair>,
    treatment: CourtTreatment,
    street_entry: Option<entry::StreetEntryApron>,
}

impl CompoundSupportPlan {
    pub fn compile(
        property: &CityCompound,
        levels: CompoundSupportLevels,
        limits: SupportLimits,
        court: CourtTreatment,
    ) -> Result<Self, SupportDiagnostic> {
        compound::compile(property, levels, limits, court).map_err(|mut error| {
            error.attempted_treatment = Box::new(SupportGradingAttempt::Compound(court));
            error
        })
    }

    pub fn property_id(&self) -> CityPropertyId {
        self.property.id
    }

    pub fn member_support(&self) -> [MemberSupport; 2] {
        [self.levels.front, self.levels.rear]
    }

    pub fn gate_elevation(&self) -> SupportElevation {
        self.levels.gate
    }

    pub fn court_elevation(&self) -> SupportElevation {
        self.levels.court
    }

    pub fn elevations_at(
        &self,
        scene_point: crate::scene_coordinates::ScenePlanPoint,
    ) -> SurfaceElevations {
        let point = scene_point.metres();
        if !self.contains(scene_point) {
            return SurfaceElevations::default();
        }
        if let Some(entry) = &self.street_entry
            && entry.support_region.contains(point)
        {
            return entry.mesh.elevations_at(scene_point);
        }
        let local = self
            .property
            .plot
            .orientation()
            .world_to_local(point - self.property.plot.centre_metres());
        let side = self.property.boundary.gate.hinge.opposite().sign();
        let offset = (local.x - self.split_frontage_metres) * side;
        let mut heights = Vec::new();
        if self.property.plot.contains(point)
            && offset <= self.limits.contact_tolerance_metres.metres()
            && let Some(height) = self.main_height(local)
        {
            heights.push(height);
        }
        if offset >= -self.limits.contact_tolerance_metres.metres()
            || !self.property.plot.contains(point)
        {
            let distance = (point - self.passage.start_metres())
                .dot((self.passage.end_metres() - self.passage.start_metres()).normalize());
            if let Some(coordinate) = ProfileCoordinate::from_metres(distance) {
                heights.push(self.passage_profile.height_at(coordinate));
            }
        }
        SurfaceElevations(heights)
    }

    pub fn reservation(&self) -> CityPlotBounds {
        self.property.plot
    }

    /// Complete plot plus its already-bound street approach. Grading owns no
    /// arbitrary margin or neighbouring property outside these regions.
    pub fn support_regions(&self) -> Vec<CityPlotBounds> {
        let mut regions = vec![self.property.plot, self.passage_region];
        if let Some(entry) = &self.street_entry {
            regions.push(entry.reservation);
        }
        regions
    }

    /// The short street approach is already an owned access reservation.
    /// It can extend beyond the plot; unrelated terrain is not part of this core.
    pub fn contains(&self, point: crate::scene_coordinates::ScenePlanPoint) -> bool {
        let point = point.metres();
        if self
            .street_entry
            .as_ref()
            .is_some_and(|entry| entry.reservation.contains(point))
        {
            return true;
        }
        if self.property.plot.contains(point) {
            return true;
        }
        let delta = self.passage.end_metres() - self.passage.start_metres();
        let length = delta.length();
        let offset = point - self.passage.start_metres();
        let distance = offset.dot(delta / length);
        (0.0..=length).contains(&distance)
            && offset.perp_dot(delta / length).abs() <= self.passage.half_width_metres()
    }

    /// Planar support triangles and internal retaining faces. Perimeter grading,
    /// source-triangle clipping and actor/gate collision remain separate checks.
    pub fn mesh(&self) -> Result<PropertySupportMesh, SupportDiagnostic> {
        let attach_treatment = |mut error: SupportDiagnostic| {
            error.attempted_treatment = Box::new(SupportGradingAttempt::Compound(self.treatment));
            error
        };
        let mut mesh = PropertySupportMesh::from_compound_plan(self).map_err(attach_treatment)?;
        if let Some(entry) = &self.street_entry {
            mesh.append(&entry.mesh).map_err(attach_treatment)?;
        }
        Ok(mesh)
    }

    /// Compile closed foundation cells against complete source triangles.
    /// This does not replace geographic triangles or enlarge owned regions.
    pub fn foundations(
        &self,
        geographic: &GeographicSurface,
        embedment: FoundationEmbedment,
    ) -> Result<PropertyFoundationMesh, SupportDiagnostic> {
        foundations::compile(&self.support_surface()?, geographic, embedment)
    }

    pub fn stair_flights(
        &self,
    ) -> impl Iterator<Item = &adventuresim_building_generator::AccessStairFlight> {
        self.court_stairs.iter().map(|stair| &stair.flight)
    }

    fn main_height(&self, local: Vec2) -> Option<SupportElevation> {
        if let Some(height) = self
            .court_stairs
            .iter()
            .find_map(|stair| stair.height_at(local))
        {
            SupportElevation::from_metres(height)
        } else {
            Some(
                self.court_profile
                    .height_at(ProfileCoordinate::from_metres(local.y)?),
            )
        }
    }

    /// A private unit translation evaluates the affine response at unchanged
    /// triangle locations. It is never installed or accepted as a support plan.
    fn floor_translation_basis(&self) -> Result<Self, SupportDiagnostic> {
        let mut basis = self.clone();
        basis.levels.front.elevation.0 += 1.0;
        basis.levels.rear.elevation.0 += 1.0;
        basis.levels.court.0 += 1.0;
        let invalid = |error: profile::ProfileError| {
            SupportDiagnostic::new(
                &self.property,
                SupportConstraint::Reservation,
                SupportBoundary::CourtLanding,
                self.property.plot.centre_metres()
                    + self
                        .property
                        .plot
                        .orientation()
                        .local_to_world(Vec2::Y * error.coordinate().metres()),
                1.0,
                0.0,
            )
        };
        basis.court_profile = basis
            .court_profile
            .translated_for_floor_basis(FloorBasisRole::Court)
            .map_err(invalid)?;
        basis.passage_profile = basis
            .passage_profile
            .translated_for_floor_basis(FloorBasisRole::PassageTerminalLanding)
            .map_err(invalid)?;
        for stair in &mut basis.court_stairs {
            stair.shift_floor(1.0);
        }
        Ok(basis)
    }

    /// Diagnose a selected point-query elevation. Complete cut/fill acceptance
    /// uses each actual support triangle at every geographic intersection;
    /// containment roundoff at an edge must not omit its displacement.
    pub fn validate_displacement_at(
        &self,
        scene_point: crate::scene_coordinates::ScenePlanPoint,
        geographic_height: SupportElevation,
    ) -> Result<(), SupportDiagnostic> {
        let point = scene_point.metres();
        let displacement = self
            .elevations_at(scene_point)
            .iter()
            .map(|support| (support.metres() - geographic_height.metres()).abs())
            .fold(0.0, f32::max);
        if displacement > self.limits.maximum_displacement_metres.metres() {
            let mut error = SupportDiagnostic::new(
                &self.property,
                SupportConstraint::CutFill,
                SupportBoundary::GeographicSurface,
                point,
                displacement,
                self.limits.maximum_displacement_metres.metres(),
            );
            error.attempted_treatment = Box::new(SupportGradingAttempt::Compound(self.treatment));
            return Err(error);
        }
        Ok(())
    }
}
