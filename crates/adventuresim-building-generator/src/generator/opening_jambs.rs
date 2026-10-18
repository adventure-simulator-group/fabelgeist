//! Architectural set-out and bearing admission shared by wall and church openings.
use super::*;
use crate::plan_geometry::ArchitecturalPlanPoint;
use crate::spatial_geometry::{Elevation, PlanDirection, PositiveLength};
pub(super) struct OpeningJambSetOut {
    nodes: [StructuralNodeId; 2],
    owner: GeometryOwnerId,
    bearing: StructuralNodeId,
    origin: ArchitecturalPlanPoint,
    tangent: PlanDirection<Architectural>,
    width: PositiveLength,
    base: Elevation<Architectural>,
}
impl OpeningJambSetOut {
    pub(super) fn from_metres(
        nodes: [StructuralNodeId; 2],
        owner: GeometryOwnerId,
        bearing: StructuralNodeId,
        origin: Vec2,
        tangent: Vec2,
        width: f32,
        base: f32,
    ) -> Result<Self, GenerationError> {
        let construct = || {
            Ok::<_, crate::spatial_geometry::GeometryError>(Self {
                nodes,
                owner,
                bearing,
                origin: ArchitecturalPlanPoint::from_metres(origin)?,
                tangent: PlanDirection::from_normalized(tangent)?,
                width: PositiveLength::from_metres(width)?,
                base: Elevation::from_metres(base)?,
            })
        };
        construct().map_err(|cause| {
            crate::StructuralNodeError {
                node: nodes[0],
                cause,
            }
            .into()
        })
    }
    pub(super) fn append_nodes(
        self,
        geometry: &mut ResolvedGeometry,
    ) -> Result<(), GenerationError> {
        let origin = self.origin.metres();
        let tangent = self.tangent.vector();
        let width = self.width.metres();
        for (side, node) in [-1.0_f32, 1.0].into_iter().zip(self.nodes) {
            geometry
                .structural_nodes
                .push(crate::StructuralNode::from_metres(
                    node,
                    self.owner,
                    StructuralNodeKind::OpeningJamb,
                    Vec3::new(
                        origin.x + tangent.x * side * width * 0.5,
                        self.base.metres(),
                        origin.y + tangent.y * side * width * 0.5,
                    ),
                    vec![self.bearing],
                    false,
                )?);
        }
        Ok(())
    }
}

/// Head and spandrel bearings keep the same architectural axis and node sequence.
pub(super) struct OpeningHeadSetOut {
    nodes: [StructuralNodeId; 2],
    owner: GeometryOwnerId,
    jambs: [StructuralNodeId; 2],
    origin: ArchitecturalPlanPoint,
    head: Elevation<Architectural>,
    spandrel: Elevation<Architectural>,
}
impl OpeningHeadSetOut {
    pub(super) fn from_metres(
        nodes: [StructuralNodeId; 2],
        owner: GeometryOwnerId,
        jambs: [StructuralNodeId; 2],
        origin: Vec2,
        head: f32,
        spandrel: f32,
    ) -> Result<Self, GenerationError> {
        let cause = |node, cause| crate::StructuralNodeError { node, cause };
        Ok(Self {
            nodes,
            owner,
            jambs,
            origin: ArchitecturalPlanPoint::from_metres(origin)
                .map_err(|error| cause(nodes[0], error))?,
            head: Elevation::from_metres(head).map_err(|error| cause(nodes[0], error))?,
            spandrel: Elevation::from_metres(spandrel).map_err(|error| cause(nodes[1], error))?,
        })
    }
    pub(super) fn nodes(self) -> Result<[crate::StructuralNode; 2], GenerationError> {
        let origin = self.origin.metres();
        Ok([
            crate::StructuralNode::from_metres(
                self.nodes[0],
                self.owner,
                StructuralNodeKind::OpeningHead,
                Vec3::new(origin.x, self.head.metres(), origin.y),
                self.jambs.to_vec(),
                false,
            )?,
            crate::StructuralNode::from_metres(
                self.nodes[1],
                self.owner,
                StructuralNodeKind::OpeningSpandrel,
                Vec3::new(origin.x, self.spandrel.metres(), origin.y),
                vec![self.nodes[0]],
                false,
            )?,
        ])
    }
}
