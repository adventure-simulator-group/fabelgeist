use super::*;
use crate::plan_geometry::ArchitecturalPlanPoint;
use crate::spatial_geometry::{
    CuboidDimensions, Elevation, PlanDirection, Position, PositiveLength, RigidRotation,
};
use crate::*;
pub(super) mod contact;

const WORKPLACE_OWNER: GeometryOwnerId = GeometryOwnerId(95_000);
const WORKPLACE_WALL_BASE: u64 = 95_000;
const BOARD_WALL_THICKNESS_METRES: f32 = 0.18;
const MASONRY_WALL_THICKNESS_METRES: f32 = 0.45;
const WORKPLACE_NODE_BASE: u64 = 95_000_000;
const BEARING_DEPTH_METRES: f32 = 0.04;

pub(super) struct Assembly<'a> {
    pub plan: WorkplacePlan,
    pub geometry: &'a mut ResolvedGeometry,
    pub walls: &'a mut Vec<WallAssembly>,
}

impl Assembly<'_> {
    /// Orient a fitted cuboid and rebuild its actual bearings before attaching later parts.
    pub fn orient_part(
        &mut self,
        id: ResolvedItemId,
        rotation: RigidRotation,
    ) -> Result<(), crate::GenerationError> {
        let solid = self.geometry.solids.iter_mut().find(|solid| solid.id == id);
        let Some(solid) = solid else {
            return Err(WorkplaceConstructionError::MissingPartAuthority {
                id,
                role: PartAuthority::Solid,
            }
            .into());
        };
        let rotation = rotation.quaternion();
        let (yaw, crossfall, longfall) = rotation.to_euler(bevy::math::EulerRot::YXZ);
        solid.yaw_radians = crate::spatial_geometry::Radians::new(yaw)?;
        solid.crossfall_radians = crate::spatial_geometry::Radians::new(crossfall)?;
        solid.longfall_radians = crate::spatial_geometry::Radians::new(longfall)?;
        let solid = solid.clone();
        let Some(node_id) = solid.supported_by.first().copied() else {
            return Err(WorkplaceConstructionError::MissingPartAuthority {
                id,
                role: PartAuthority::Bearing,
            }
            .into());
        };
        let contacts = self
            .geometry
            .solids
            .iter()
            .filter(|other| other.id != id && contact::touches(&solid, other, BEARING_DEPTH_METRES))
            .flat_map(|other| other.supported_by.iter().copied())
            .collect();
        let bounds = contact::bounds(&solid)?;
        let node = self
            .geometry
            .structural_nodes
            .iter_mut()
            .find(|node| node.id == node_id);
        let Some(node) = node else {
            return Err(WorkplaceConstructionError::MissingPartAuthority {
                id,
                role: PartAuthority::Bearing,
            }
            .into());
        };
        node.supported_by = contacts;
        node.position = Position::<crate::Architectural>::from_metres(Vec3::new(
            solid.centre.metres().x,
            bounds.min().metres().y,
            solid.centre.metres().z,
        ))?;
        node.grounded = bounds.min().metres().y <= BEARING_DEPTH_METRES;
        let interface = self
            .geometry
            .support_interfaces
            .iter_mut()
            .find(|interface| interface.node == node_id);
        let Some(interface) = interface else {
            return Err(WorkplaceConstructionError::MissingPartAuthority {
                id,
                role: PartAuthority::Interface,
            }
            .into());
        };
        interface.bounds = SpatialBounds::<Architectural>::from_metres(
            bounds.min().metres(),
            Vec3::new(
                bounds.max().metres().x,
                bounds.min().metres().y + BEARING_DEPTH_METRES,
                bounds.max().metres().z,
            ),
        )?;

        Ok(())
    }
    /// Every part owns a bearing node and records contact with existing support geometry.
    pub fn part(
        &mut self,
        feature: WorkplaceFeature,
        material: WorkplaceMaterial,
        centre: Position<Architectural>,
        size: CuboidDimensions,
        silhouette: WorkplacePartVisibility,
    ) -> Result<ResolvedItemId, crate::GenerationError> {
        let centre = centre.metres();
        let size = size.metres();
        let slot = self.plan.parts.len() as u64 + 1;
        let id = ResolvedItemId((1_u64 << 60) | (u64::from(WORKPLACE_OWNER.0) << 32) | slot);
        let node = StructuralNodeId(WORKPLACE_NODE_BASE + slot);
        let bottom = centre.y - size.y * 0.5;
        let solid = crate::ResolvedSolid::new(
            crate::CollisionCuboid::<crate::Architectural>::from_metres(
                id, centre, size, 0.0, 0.0, 0.0,
            )?,
            WORKPLACE_OWNER,
            SolidRole::WorkplacePart,
            ResolvedSolidShape::Cuboid,
            vec![node],
        );
        let supported_by = self
            .geometry
            .solids
            .iter()
            .filter(|other| contact::touches(&solid, other, BEARING_DEPTH_METRES))
            .flat_map(|other| other.supported_by.iter().copied())
            .collect();
        self.geometry
            .structural_nodes
            .push(crate::StructuralNode::from_metres(
                node,
                WORKPLACE_OWNER,
                StructuralNodeKind::WallBearing,
                Vec3::new(centre.x, bottom, centre.z),
                supported_by,
                bottom <= BEARING_DEPTH_METRES,
            )?);
        self.geometry.solids.push(solid);
        self.geometry.support_interfaces.push(SupportInterface {
            id: ResolvedItemId((4_u64 << 60) | (u64::from(WORKPLACE_OWNER.0) << 32) | slot),
            owner: WORKPLACE_OWNER,
            node,
            bounds: SpatialBounds::<Architectural>::from_metres(
                centre - size * 0.5,
                Vec3::new(
                    centre.x + size.x * 0.5,
                    bottom + BEARING_DEPTH_METRES,
                    centre.z + size.z * 0.5,
                ),
            )?,
        });
        self.plan.parts.push(WorkplacePart {
            solid: id,
            feature,
            material,
            silhouette,
        });
        Ok(id)
    }

    pub fn wall(
        &mut self,
        start: ArchitecturalPlanPoint,
        end: ArchitecturalPlanPoint,
        outward: PlanDirection<Architectural>,
        base: Elevation<Architectural>,
        height: PositiveLength,
        construction: WallConstruction,
    ) -> Result<(), crate::GenerationError> {
        let start = start.metres();
        let end = end.metres();
        let outward = outward.vector();
        let base = base.metres();
        let height = height.metres();
        let tangent = if end.x == start.x {
            Vec2::Y * (end.y - start.y).signum()
        } else {
            Vec2::X * (end.x - start.x).signum()
        };
        let length = start.distance(end);
        let thickness = if construction == WallConstruction::TimberBoards {
            BOARD_WALL_THICKNESS_METRES
        } else {
            MASONRY_WALL_THICKNESS_METRES
        };
        let centre = (start + end) * 0.5;
        let size = if tangent.x.abs() > 0.5 {
            Vec3::new(length, height, thickness)
        } else {
            Vec3::new(thickness, height, length)
        };
        let material = if construction == WallConstruction::TimberBoards {
            WorkplaceMaterial::Timber
        } else {
            WorkplaceMaterial::Masonry
        };
        let solid = self.part(
            WorkplaceFeature::Wall,
            material,
            Position::<crate::Architectural>::from_metres(Vec3::new(
                centre.x,
                base + height * 0.5,
                centre.y,
            ))?,
            CuboidDimensions::from_metres(size)?,
            WorkplacePartVisibility::Silhouette,
        )?;
        let slot = self.plan.walls.len();
        let index = u32::try_from(slot)
            .map_err(|cause| WorkplaceConstructionError::WallIdentity { slot, cause })?;
        let id = WallAssemblyId(WORKPLACE_WALL_BASE + u64::from(index));
        let Some(support_node) = self
            .geometry
            .solids
            .iter()
            .find(|part| part.id == solid)
            .and_then(|part| part.supported_by.first())
            .copied()
        else {
            return Err(WorkplaceConstructionError::MissingPartAuthority {
                id: solid,
                role: PartAuthority::Bearing,
            }
            .into());
        };
        self.walls.push(WallAssembly {
            id,
            owner: WORKPLACE_OWNER,
            source: WallSourceId::WorkplaceWall { index },
            material: if construction == WallConstruction::TimberBoards {
                WallMaterialClass::TimberInfill
            } else {
                WallMaterialClass::CivilianMasonry
            },
            storey_level: 0,
            frame: WallLocalFrame {
                origin: centre,
                tangent,
                outward,
                inside_room: Some(0),
                outside_room: None,
            },
            radial_frame: None,
            length_metres: length,
            height_metres: height,
            base_elevation_metres: base,
            thickness_metres: thickness,
            structural_role: WallStructuralRole::LoadBearing,
            support_node,
            host_solids: vec![solid],
            opening_ids: vec![],
            replaced_by_owner: None,
        });
        self.plan.walls.push(id);

        Ok(())
    }

    pub fn passage(
        &mut self,
        purpose: WorkplacePassagePurpose,
        min: Position<Architectural>,
        max: Position<Architectural>,
    ) -> Result<(), GenerationError> {
        let slot = self.plan.passages.len();
        let id = WorkplacePassageId(
            u32::try_from(slot)
                .map_err(|cause| WorkplaceConstructionError::PassageIdentity { slot, cause })?,
        );
        let bounds = crate::spatial_geometry::SpatialBounds::new(min, max)
            .and_then(ClearanceVolume::new)
            .map_err(|cause| WorkplaceConstructionError::Passage { id, cause })?;
        self.plan.passages.push(WorkplacePassage {
            id,
            purpose,
            bounds,
        });
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum WallConstruction {
    TimberBoards,
    Masonry,
}

#[derive(Clone, Debug, thiserror::Error, Eq, PartialEq)]
pub enum WorkplaceConstructionError {
    #[error("workplace wall ordinal {slot} exceeds its serialized identity: {cause}")]
    WallIdentity {
        slot: usize,
        #[source]
        cause: std::num::TryFromIntError,
    },
    #[error("workplace part {id:?}: {cause}")]
    Part {
        id: ResolvedItemId,
        #[source]
        cause: crate::spatial_geometry::GeometryError,
    },
    #[error("workplace passage ordinal {slot} exceeds its serialized identity: {cause}")]
    PassageIdentity {
        slot: usize,
        #[source]
        cause: std::num::TryFromIntError,
    },
    #[error("workplace passage {id:?}: {cause}")]
    Passage {
        id: WorkplacePassageId,
        #[source]
        cause: crate::spatial_geometry::GeometryError,
    },
    #[error("workplace part {id:?} has no {role:?}")]
    MissingPartAuthority {
        id: ResolvedItemId,
        role: PartAuthority,
    },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PartAuthority {
    Solid,
    Bearing,
    Interface,
}

pub(crate) fn resolve_workplace(
    program: &BuildingProgram,
    walls: &mut Vec<WallAssembly>,
    geometry: &mut ResolvedGeometry,
) -> Result<Option<WorkplacePlan>, GenerationError> {
    let Some(kind) = program.workplace_kind() else {
        return Ok(None);
    };
    let size = program
        .service_size
        .ok_or(GenerationError::InvalidWorkplaceProgram)?;
    let mut assembly = Assembly {
        plan: WorkplacePlan {
            kind,
            size,
            plot_dimensions_metres: crate::spatial_geometry::PlanDimensions::from_metres(
                program.plot_dimensions_metres(),
            )?,
            walls: vec![],
            parts: vec![],
            passages: vec![],
        },
        geometry,
        walls,
    };
    super::envelope::build_envelope(&mut assembly, program)?;
    super::equipment::fit_workplace(&mut assembly, program)?;
    Ok(Some(assembly.plan))
}
