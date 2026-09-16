//! Construct heated masonry with measured contacts and explicit empty smoke paths.
use super::placement::Placement;
use crate::*;
use bevy::math::{Quat, Vec3};

pub(super) struct Assembly<'a> {
    pub plan: DomesticHeatingPlan,
    pub geometry: &'a mut ResolvedGeometry,
    pub placement: Placement,
}
impl Assembly<'_> {
    pub fn part(
        &mut self,
        kind: HeatingPartKind,
        material: BuildingLodMaterial,
        min: Vec3,
        max: Vec3,
    ) -> ResolvedItemId {
        let bounds = self.placement.bounds(min, max);
        self.absolute_part(kind, material, bounds)
    }
    pub fn absolute_part(
        &mut self,
        kind: HeatingPartKind,
        material: BuildingLodMaterial,
        bounds: ResolvedBounds,
    ) -> ResolvedItemId {
        self.oriented_part(
            kind,
            material,
            (bounds.min + bounds.max) * 0.5,
            bounds.max - bounds.min,
            Quat::IDENTITY,
        )
    }
    pub fn oriented_part(
        &mut self,
        kind: HeatingPartKind,
        material: BuildingLodMaterial,
        centre: Vec3,
        size: Vec3,
        rotation: Quat,
    ) -> ResolvedItemId {
        let slot = self.plan.parts.len() as u64 + 1;
        let id = self.id(1, slot);
        let node = StructuralNodeId(96_000_000 + slot);
        let (yaw, crossfall, longfall) = rotation.to_euler(bevy::math::EulerRot::YXZ);
        let solid = ResolvedSolid {
            id,
            owner: self.plan.owner,
            centre,
            size,
            yaw_radians: yaw,
            crossfall_radians: crossfall,
            longfall_radians: longfall,
            role: SolidRole::DomesticHeating,
            shape: ResolvedSolidShape::Cuboid,
            supported_by: vec![node],
        };
        let bounds = solid.cuboid_bounds();
        let contacts = self
            .geometry
            .solids
            .iter()
            .filter(|s| self.plan.parts.iter().any(|p| p.solid == s.id))
            .filter_map(|s| super::contact::measured(&solid, s).map(|b| (s.supported_by[0], b)))
            .collect::<Vec<_>>();
        self.geometry.structural_nodes.push(StructuralNode {
            id: node,
            owner: self.plan.owner,
            kind: StructuralNodeKind::WallBearing,
            position: Vec3::new(centre.x, bounds.min.y, centre.z),
            supported_by: contacts.iter().map(|(node, _)| *node).collect(),
            grounded: bounds.min.y <= 0.001,
        });
        let bearings = if bounds.min.y <= 0.001 {
            vec![ResolvedBounds {
                min: bounds.min,
                max: Vec3::new(bounds.max.x, bounds.min.y + 0.02, bounds.max.z),
            }]
        } else {
            contacts.iter().map(|(_, bounds)| *bounds).collect()
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
        id
    }
    pub fn passage(&mut self, kind: HeatingPassageKind, min: Vec3, max: Vec3) {
        let id = self.id(3, self.plan.passages.len() as u64 + 1);
        self.geometry.voids.push(ResolvedVoid {
            id,
            owner: self.plan.owner,
            bounds: self.placement.bounds(min, max),
            role: VoidRole::Passage,
            shape: ResolvedVoidShape::Box,
            subtracts_from: self.plan.owner,
        });
        self.plan.passages.push(HeatingPassage { kind, void: id });
    }
    pub fn id(&self, family: u64, slot: u64) -> ResolvedItemId {
        ResolvedItemId((family << 60) | (u64::from(self.plan.owner.0) << 32) | 0x0960_0000 | slot)
    }
}
