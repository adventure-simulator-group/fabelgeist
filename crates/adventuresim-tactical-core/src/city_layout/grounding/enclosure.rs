//! Enclosures bind to accepted property triangles, without additional grading.
use super::foundations::GroundTriangle;
use super::*;
use crate::city_layout::{CityBoundaryMaterial, CityBoundaryMember};
use bevy::math::{DVec2, Quat, Vec3Swizzles};
mod projection;

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
pub struct BoundarySupportCell {
    pub positions_metres: [Vec3; 6],
    pub material: CityBoundaryMaterial,
    pub element: BoundarySupportElement,
}

impl BoundarySupportCell {
    pub fn triangles(&self) -> impl Iterator<Item = [Vec3; 3]> + '_ {
        ENCLOSURE_CELL_FACES
            .map(|indices| indices.map(|i| self.positions_metres[i as usize]))
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

/// A rejected enclosure is never substituted by a level one or omitted.
#[derive(Clone, Debug, PartialEq, Serialize, thiserror::Error)]
#[error(
    "property {property_id:?}, members {member_building_ids:?}, {element:?}: {constraint:?} at {location_metres:?}, measured {measured}, permitted {permitted}, shortfall {shortfall} ({unit:?})"
)]
pub struct BoundarySupportError {
    #[source]
    pub construction_failure: Option<crate::city_layout::BoundaryGeometryError>,
    pub property_id: CityPropertyId,
    pub member_building_ids: [u64; 2],
    pub element: BoundarySupportElement,
    pub constraint: BoundarySupportConstraint,
    pub location_metres: Vec2,
    pub measured: f64,
    pub permitted: f64,
    pub shortfall: f64,
    pub unit: SupportDiagnosticUnit,
}

impl BoundarySupportError {
    pub(crate) fn construction(
        property: &CityCompound,
        cause: crate::city_layout::BoundaryGeometryError,
    ) -> Self {
        let mut error = Self::new(
            property,
            cause.element,
            BoundarySupportConstraint::Construction,
            property.plot.centre_metres,
            1.0,
            0.0,
        );
        error.construction_failure = Some(cause);
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
            location_metres,
            measured,
            permitted,
            shortfall: (measured - permitted).max(0.0),
            unit: match constraint {
                BoundarySupportConstraint::Construction
                | BoundarySupportConstraint::OwnerBinding
                | BoundarySupportConstraint::MissingGateDatum => SupportDiagnosticUnit::Count,
                BoundarySupportConstraint::Coverage => SupportDiagnosticUnit::SquareMetres,
                BoundarySupportConstraint::AmbiguousGateDatum
                | BoundarySupportConstraint::PostHeadroom => SupportDiagnosticUnit::Metres,
            },
        }
    }
}

/// Scene-relative enclosure geometry. The scene parent is the accepted gate
/// datum. Bodies follow local terraces; post heads remain at the gate elevation,
/// with masonry plinths reaching the lower soil wherever a post crosses a step.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BoundarySupportMesh {
    pub cells: Vec<BoundarySupportCell>,
}

impl BoundarySupportMesh {
    pub fn collider(&self) -> avian3d::prelude::Collider {
        avian3d::prelude::Collider::compound(
            self.cells
                .iter()
                .map(|cell| {
                    let centre = cell.positions_metres.iter().copied().sum::<Vec3>() / 6.0;
                    let vertices = cell.positions_metres.iter().map(|p| *p - centre).collect();
                    let shape = avian3d::parry::shape::SharedShape::convex_mesh(
                        vertices,
                        &ENCLOSURE_CELL_FACES,
                    )
                    .expect("accepted enclosure cells are closed finite convex prisms");
                    (
                        centre,
                        Quat::IDENTITY,
                        avian3d::prelude::Collider::from(shape),
                    )
                })
                .collect(),
        )
    }

    pub fn project(
        property: &CityCompound,
        foundation: &PropertyFoundationMesh,
        limits: SupportLimits,
        embedment: FoundationEmbedment,
    ) -> Result<(Self, SupportElevation), BoundarySupportError> {
        projection::project(property, foundation, limits, embedment)
    }
}
