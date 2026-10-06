//! Construct heated masonry with measured contacts and explicit empty smoke paths.
use super::floor_bearings::BearingContact;
use super::placement::Placement;
use crate::spatial_geometry::{CuboidDimensions, Position, RigidRotation};
use crate::*;
use bevy::math::Vec3;

pub(super) struct Assembly<'a> {
    pub plan: DomesticHeatingPlan,
    pub geometry: &'a mut ResolvedGeometry,
    pub placement: Placement,
}
impl<'a> Assembly<'a> {
    pub fn new(
        geometry: &'a mut ResolvedGeometry,
        placement: Placement,
        owner: GeometryOwnerId,
        programme: DomesticHeatingProgramme,
    ) -> Result<Self, crate::GenerationError> {
        Ok(Self {
            geometry,
            placement,
            plan: DomesticHeatingPlan {
                programme,
                owner,
                kitchen: HeatingRoom {
                    storey_level: placement.site.storey_level,
                    room_id: placement.site.kitchen,
                },
                heated_room: HeatingRoom {
                    storey_level: placement.site.storey_level,
                    room_id: placement.site.stube,
                },
                fire_wall: placement.site.wall,
                centre_metres: placement.site.centre,
                kitchen_axis: placement.site.kitchen_axis,
                floor_height_metres: placement.site.floor_height,
                floors: vec![],
                ground_support: StructuralNodeId(0),
                parts: vec![],
                passages: vec![],
                operating_space: placement.site.operating_space()?,
                roof: HeatingRoofPenetration {
                    roof: placement.roof,
                    face: placement.face,
                    cutout_index: 0,
                    edges: vec![],
                    flashing: vec![],
                },
            },
        })
    }
    pub fn part(
        &mut self,
        kind: HeatingPartKind,
        material: BuildingLodMaterial,
        min: crate::spatial_geometry::Displacement<super::placement::HearthLocal>,
        max: crate::spatial_geometry::Displacement<super::placement::HearthLocal>,
    ) -> Result<ResolvedItemId, crate::GenerationError> {
        let bounds = self.placement.site.bounds(min, max)?;
        self.absolute_part(kind, material, bounds)
    }
    pub fn absolute_part(
        &mut self,
        kind: HeatingPartKind,
        material: BuildingLodMaterial,
        bounds: SpatialBounds<Architectural>,
    ) -> Result<ResolvedItemId, crate::GenerationError> {
        self.oriented_part(
            kind,
            material,
            Position::from_metres((bounds.min().metres() + bounds.max().metres()) * 0.5)?,
            CuboidDimensions::from_metres(bounds.max().metres() - bounds.min().metres())?,
            RigidRotation::IDENTITY,
        )
    }
    pub fn oriented_part(
        &mut self,
        kind: HeatingPartKind,
        material: BuildingLodMaterial,
        centre: Position<Architectural>,
        size: CuboidDimensions,
        rotation: RigidRotation,
    ) -> Result<ResolvedItemId, crate::GenerationError> {
        let slot = self.plan.parts.len() as u64 + 1;
        let id = self.id(1, slot);
        let node = StructuralNodeId(96_000_000 + slot);
        let (yaw, crossfall, longfall) = rotation.quaternion().to_euler(bevy::math::EulerRot::YXZ);
        let solid = crate::ResolvedSolid::new(
            crate::CollisionCuboid::<crate::Architectural>::from_metres(
                id,
                centre.metres(),
                size.metres(),
                yaw,
                crossfall,
                longfall,
            )?,
            self.plan.owner,
            SolidRole::DomesticHeating,
            ResolvedSolidShape::Cuboid,
            vec![node],
        );
        let bounds = solid.cuboid_bounds()?;
        let mut contacts = Vec::new();
        for source in self.geometry.solids.iter().filter(|s| {
            self.plan.parts.iter().any(|p| p.solid == s.id)
                || (kind == HeatingPartKind::FloorClosure && s.role == SolidRole::FrameFloor)
        }) {
            if let Some(bounds) = super::contact::measured(&solid, source)? {
                let node = source
                    .supported_by
                    .first()
                    .copied()
                    .ok_or(HeatingConstructionError::MissingBearing { solid: source.id })?;
                contacts.push(BearingContact { node, bounds });
            }
        }
        self.geometry
            .structural_nodes
            .push(crate::StructuralNode::from_metres(
                node,
                self.plan.owner,
                StructuralNodeKind::WallBearing,
                Vec3::new(
                    centre.metres().x,
                    bounds.min().metres().y,
                    centre.metres().z,
                ),
                contacts.iter().map(|contact| contact.node).collect(),
                bounds.min().metres().y <= 0.001,
            )?);
        let bearings = if bounds.min().metres().y <= 0.001 {
            vec![SpatialBounds::<Architectural>::from_metres(
                bounds.min().metres(),
                Vec3::new(
                    bounds.max().metres().x,
                    bounds.min().metres().y + 0.02,
                    bounds.max().metres().z,
                ),
            )?]
        } else {
            contacts.iter().map(|contact| contact.bounds).collect()
        };
        for (index, bearing) in bearings.into_iter().enumerate() {
            self.geometry.support_interfaces.push(SupportInterface {
                id: self.id(4, (slot << 8) | index as u64),
                owner: self.plan.owner,
                node,
                bounds: bearing,
            });
        }
        self.geometry.solids.push(solid);
        self.plan.parts.push(HeatingPart {
            solid: id,
            kind,
            material,
        });
        Ok(id)
    }
    pub fn passage(
        &mut self,
        kind: HeatingPassageKind,
        min: crate::spatial_geometry::Displacement<super::placement::HearthLocal>,
        max: crate::spatial_geometry::Displacement<super::placement::HearthLocal>,
    ) -> Result<(), crate::GenerationError> {
        let id = self.id(3, self.plan.passages.len() as u64 + 1);
        self.geometry.voids.push(ResolvedVoid {
            id,
            owner: self.plan.owner,
            bounds: self.placement.site.bounds(min, max)?,
            role: VoidRole::Passage,
            shape: ResolvedVoidShape::Box,
            subtracts_from: self.plan.owner,
        });
        self.plan.passages.push(HeatingPassage { kind, void: id });
        Ok(())
    }
    pub fn bearing(&self, id: ResolvedItemId) -> Result<StructuralNodeId, crate::GenerationError> {
        let solid = self
            .geometry
            .solids
            .iter()
            .find(|solid| solid.id == id)
            .ok_or(HeatingConstructionError::MissingSolid { solid: id })?;
        Ok(solid
            .supported_by
            .first()
            .copied()
            .ok_or(HeatingConstructionError::MissingBearing { solid: id })?)
    }
    pub fn id(&self, family: u64, slot: u64) -> ResolvedItemId {
        ResolvedItemId((family << 60) | (u64::from(self.plan.owner.0) << 32) | 0x0960_0000 | slot)
    }
}
