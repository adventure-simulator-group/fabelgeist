//! Enclosures bind to accepted property triangles, without additional grading.
use super::foundations::GroundTriangle;
use super::*;
use crate::city_layout::{CityBoundaryMaterial, CityBoundaryMember};
use crate::scene_coordinates::GateRelative;
use adventuresim_building_generator::spatial_geometry::Position;
use bevy::math::{DVec2, Quat, Vec3Swizzles};
mod binding;
mod projection;
pub use binding::{BoundaryAdmissionError, BoundaryOwnerBinding, BoundarySupportProjection};

/// Closed prism topology shared by rendering and collision. Top precedes base.
const ENCLOSURE_CELL_FACES: [[u32; 3]; 8] = [
    [0, 2, 1],
    [3, 4, 5],
    [0, 1, 4],
    [0, 4, 3],
    [1, 2, 5],
    [1, 5, 4],
    [2, 0, 3],
    [2, 3, 5],
];

/// An exact finite masonry cell; its base follows the complete bearing polygon.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "BoundaryCellWire")]
pub struct BoundarySupportCell {
    positions_metres: [Position<GateRelative>; 6],
    material: CityBoundaryMaterial,
    element: BoundarySupportElement,
}

#[derive(Deserialize)]
struct BoundaryCellWire {
    positions_metres: [Position<GateRelative>; 6],
    material: CityBoundaryMaterial,
    element: BoundarySupportElement,
}
impl TryFrom<BoundaryCellWire> for BoundarySupportCell {
    type Error = SupportGeometryIssue;
    fn try_from(wire: BoundaryCellWire) -> Result<Self, Self::Error> {
        Self::new(wire.positions_metres, wire.material, wire.element)
    }
}
impl BoundarySupportCell {
    pub fn new(
        positions: [Position<GateRelative>; 6],
        material: CityBoundaryMaterial,
        element: BoundarySupportElement,
    ) -> Result<Self, SupportGeometryIssue> {
        super::admission::prism(&positions.map(Position::metres))?;
        Ok(Self {
            positions_metres: positions,
            material,
            element,
        })
    }
    /// Explicit mesh/physics port: scene X/Z, height relative to the bound gate.
    pub fn native_positions(&self) -> [Vec3; 6] {
        self.positions_metres.map(Position::metres)
    }
    pub fn positions(&self) -> &[Position<GateRelative>; 6] {
        &self.positions_metres
    }
    pub fn material(&self) -> CityBoundaryMaterial {
        self.material
    }
    pub fn element(&self) -> BoundarySupportElement {
        self.element
    }

    pub fn triangles(&self) -> impl Iterator<Item = [Vec3; 3]> + '_ {
        ENCLOSURE_CELL_FACES
            .map(|indices| indices.map(|i| self.positions_metres[i as usize].metres()))
            .into_iter()
    }
}

pub use crate::city_layout::compound::BoundarySupportElement;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BoundarySupportConstraint {
    Construction,
    OwnerBinding,
    Coverage,
    MissingGateDatum,
    AmbiguousGateDatum,
    PostHeadroom,
}

#[derive(Clone, Debug, PartialEq, Serialize, thiserror::Error)]
pub enum BoundaryConstructionError {
    #[error(transparent)]
    Binding(#[from] BoundaryAdmissionError),
    #[error(transparent)]
    Recipe(#[from] crate::city_layout::BoundaryGeometryError),
    #[error(transparent)]
    Cell(#[from] SupportGeometryIssue),
}

/// A rejected enclosure is never substituted by a level one or omitted.
#[derive(Clone, Debug, PartialEq, Serialize, thiserror::Error)]
#[error(
    "property {property_id:?}, members {member_building_ids:?}, {element:?}: {constraint:?} at {location_metres:?}, {violation:?}"
)]
pub struct BoundarySupportError {
    #[source]
    pub construction_failure: Option<Box<BoundaryConstructionError>>,
    pub property_id: CityPropertyId,
    pub member_building_ids: [crate::scene_input::SceneBuildingId; 2],
    pub element: BoundarySupportElement,
    pub constraint: BoundarySupportConstraint,
    pub location_metres: SupportDiagnosticLocation,
    pub violation: SupportViolation,
}

impl BoundarySupportError {
    pub(crate) fn binding(property: &CityCompound, cause: BoundaryAdmissionError) -> Self {
        let mut error = Self::new(
            property,
            BoundarySupportElement::Owner,
            BoundarySupportConstraint::OwnerBinding,
            property.plot.centre_metres(),
            1.0,
            0.0,
        );
        error.construction_failure = Some(Box::new(BoundaryConstructionError::Binding(cause)));
        error
    }

    pub(crate) fn construction(
        property: &CityCompound,
        cause: crate::city_layout::BoundaryGeometryError,
    ) -> Self {
        let mut error = Self::new(
            property,
            cause.element,
            BoundarySupportConstraint::Construction,
            property.plot.centre_metres(),
            1.0,
            0.0,
        );
        error.construction_failure = Some(Box::new(BoundaryConstructionError::Recipe(cause)));
        error
    }
    pub(super) fn cell(
        property: &CityCompound,
        element: BoundarySupportElement,
        cause: SupportGeometryIssue,
    ) -> Self {
        let mut error = Self::new(
            property,
            element,
            BoundarySupportConstraint::Construction,
            property.plot.centre_metres(),
            1.0,
            0.0,
        );
        error.construction_failure = Some(Box::new(BoundaryConstructionError::Cell(cause)));
        error
    }
    pub(crate) fn new(
        property: &CityCompound,
        element: BoundarySupportElement,
        constraint: BoundarySupportConstraint,
        location_metres: Vec2,
        measured: f64,
        permitted: f64,
    ) -> Self {
        Self {
            construction_failure: None,
            property_id: property.id,
            member_building_ids: [property.front_building_id, property.rear_building_id],
            element,
            constraint,
            location_metres: SupportDiagnosticLocation::from_attempt_metres(location_metres),
            violation: SupportViolation::maximum(
                match constraint {
                    BoundarySupportConstraint::Construction
                    | BoundarySupportConstraint::OwnerBinding
                    | BoundarySupportConstraint::MissingGateDatum => SupportDiagnosticUnit::Count,
                    BoundarySupportConstraint::Coverage => SupportDiagnosticUnit::SquareMetres,
                    BoundarySupportConstraint::AmbiguousGateDatum
                    | BoundarySupportConstraint::PostHeadroom => SupportDiagnosticUnit::Metres,
                },
                measured,
                permitted,
            ),
        }
    }
}

/// Scene-relative enclosure geometry. The scene parent is the accepted gate
/// datum. Bodies follow local terraces; post heads remain at the gate elevation,
/// with masonry plinths reaching the lower soil wherever a post crosses a step.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "BoundaryMeshWire")]
pub struct BoundarySupportMesh {
    pub(super) binding: BoundaryOwnerBinding,
    pub(super) cells: Vec<BoundarySupportCell>,
}

#[derive(Deserialize)]
struct BoundaryMeshWire {
    binding: BoundaryOwnerBinding,
    cells: Vec<BoundarySupportCell>,
}
impl TryFrom<BoundaryMeshWire> for BoundarySupportMesh {
    type Error = SupportGeometryIssue;
    fn try_from(wire: BoundaryMeshWire) -> Result<Self, Self::Error> {
        wire.binding.validate_cells(&wire.cells)?;
        Ok(Self {
            binding: wire.binding,
            cells: wire.cells,
        })
    }
}
impl BoundarySupportMesh {
    pub fn binding(&self) -> &BoundaryOwnerBinding {
        &self.binding
    }

    pub fn cells(&self) -> &[BoundarySupportCell] {
        &self.cells
    }
    pub fn collider(&self) -> Result<SupportCollision, SupportColliderError> {
        let mut parts = Vec::new();
        for (index, cell) in self.cells.iter().enumerate() {
            let points = cell.native_positions();
            let Some(faces) = super::admission::solid_faces(&points, ENCLOSURE_CELL_FACES) else {
                continue;
            };
            let centre = points.iter().copied().sum::<Vec3>() / 6.0;
            let vertices = points.iter().map(|p| *p - centre).collect();
            let Some(shape) = avian3d::parry::shape::SharedShape::convex_mesh(vertices, &faces)
            else {
                return Err(SupportColliderError::Boundary {
                    property: self.binding.property(),
                    members: self.binding.members().ids().to_vec(),
                    element: cell.element,
                    cell: index,
                    issue: SupportGeometryIssue::Collider,
                });
            };
            parts.push((
                centre,
                Quat::IDENTITY,
                avian3d::prelude::Collider::from(shape),
            ));
        }
        Ok(SupportCollision::compound(parts))
    }

    pub fn project(
        property: &CityCompound,
        foundation: &PropertyFoundationMesh,
        limits: SupportLimits,
        embedment: FoundationEmbedment,
    ) -> Result<BoundarySupportProjection, BoundarySupportError> {
        projection::project(property, foundation, limits, embedment)
    }
}
