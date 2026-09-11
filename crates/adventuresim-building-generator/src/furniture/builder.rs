use super::*;
use crate::{BUILDING_DETAIL_UV_METRES_PER_UNIT, BuildingLodMaterial, ResolvedItemId};
use bevy::math::{EulerRot, Quat, Vec2};

const GROUND_CONTACT_TOLERANCE_METRES: f32 = 0.001;

#[derive(Default)]
pub(super) struct Builder {
    meshes: Vec<LodMesh>,
    colliders: Vec<CollisionCuboid>,
    supports: Vec<Vec3>,
    clearances: Vec<FurnitureClearance>,
    #[cfg(test)]
    members: Vec<CollisionCuboid>,
}

#[derive(Clone, Copy)]
pub(super) enum CollisionPolicy {
    Solid,
    Decoration,
}

impl Builder {
    pub(super) fn mesh(&mut self, material: BuildingLodMaterial) -> &mut LodMesh {
        if let Some(index) = self
            .meshes
            .iter()
            .position(|mesh| mesh.material == material)
        {
            return &mut self.meshes[index];
        }
        self.meshes.push(LodMesh::new(material));
        self.meshes.last_mut().unwrap()
    }

    pub(super) fn cuboid(
        &mut self,
        material: BuildingLodMaterial,
        centre: Vec3,
        size: Vec3,
        rotation: Quat,
        collision: CollisionPolicy,
    ) {
        let half = size * 0.5;
        let corners = [
            Vec3::new(-half.x, -half.y, -half.z),
            Vec3::new(half.x, -half.y, -half.z),
            Vec3::new(half.x, half.y, -half.z),
            Vec3::new(-half.x, half.y, -half.z),
            Vec3::new(-half.x, -half.y, half.z),
            Vec3::new(half.x, -half.y, half.z),
            Vec3::new(half.x, half.y, half.z),
            Vec3::new(-half.x, half.y, half.z),
        ]
        .map(|point| centre + rotation * point);
        for (indices, normal) in [
            ([0, 1, 2, 3], -Vec3::Z),
            ([5, 4, 7, 6], Vec3::Z),
            ([4, 0, 3, 7], -Vec3::X),
            ([1, 5, 6, 2], Vec3::X),
            ([3, 2, 6, 7], Vec3::Y),
            ([4, 5, 1, 0], -Vec3::Y),
        ] {
            let positions = indices.map(|index| corners[index]);
            self.quad(material, positions, rotation * normal);
        }
        for point in corners
            .into_iter()
            .filter(|point| point.y.abs() <= GROUND_CONTACT_TOLERANCE_METRES)
        {
            self.support(point);
        }
        let (yaw_radians, crossfall_radians, longfall_radians) = rotation.to_euler(EulerRot::YXZ);
        let cuboid = CollisionCuboid {
            source: ResolvedItemId(self.colliders.len() as u64 + 1),
            centre,
            size,
            yaw_radians,
            crossfall_radians,
            longfall_radians,
        };
        #[cfg(test)]
        self.members.push(cuboid);
        if matches!(collision, CollisionPolicy::Solid) {
            self.colliders.push(cuboid);
        }
    }

    pub(super) fn timber(&mut self, centre: Vec3, size: Vec3) {
        self.cuboid(
            BuildingLodMaterial::InteriorTimber,
            centre,
            size,
            Quat::IDENTITY,
            CollisionPolicy::Solid,
        );
    }

    pub(super) fn quad(
        &mut self,
        material: BuildingLodMaterial,
        positions: [Vec3; 4],
        normal: Vec3,
    ) {
        let tangent = (positions[1] - positions[0]).normalize_or_zero();
        let up = normal.cross(tangent);
        let uvs = positions.map(|point| {
            Vec2::new(point.dot(tangent), point.dot(up)) / BUILDING_DETAIL_UV_METRES_PER_UNIT
        });
        self.mesh(material).push_quad(positions, normal, uvs);
    }

    pub(super) fn triangle(
        &mut self,
        material: BuildingLodMaterial,
        positions: [Vec3; 3],
        normal: Vec3,
    ) {
        self.mesh(material).push_triangle(
            positions,
            normal,
            positions.map(|point| Vec2::new(point.x, point.z) / BUILDING_DETAIL_UV_METRES_PER_UNIT),
        );
    }

    pub(super) fn support(&mut self, point: Vec3) {
        if !self
            .supports
            .iter()
            .any(|existing| existing.distance(point) <= GROUND_CONTACT_TOLERANCE_METRES)
        {
            self.supports.push(point);
        }
    }

    pub(super) fn collider(&mut self, centre: Vec3, size: Vec3) {
        self.colliders.push(CollisionCuboid {
            source: ResolvedItemId(self.colliders.len() as u64 + 1),
            centre,
            size,
            yaw_radians: 0.0,
            crossfall_radians: 0.0,
            longfall_radians: 0.0,
        });
    }

    pub(super) fn clearance(&mut self, kind: FurnitureClearanceKind, min: Vec3, max: Vec3) {
        self.clearances.push(FurnitureClearance {
            kind,
            bounds: CollisionBounds { min, max },
        });
    }

    pub(super) fn finish(mut self) -> FurnitureRecipe {
        let mut bounds = CollisionBounds {
            min: Vec3::splat(f32::INFINITY),
            max: Vec3::splat(f32::NEG_INFINITY),
        };
        for vertex in self.meshes.iter().flat_map(|mesh| &mesh.vertices) {
            bounds.min = bounds.min.min(vertex.position);
            bounds.max = bounds.max.max(vertex.position);
        }
        let centre = (bounds.min + bounds.max) * 0.5;
        let origin = Vec3::new(centre.x, 0.0, centre.z);
        for vertex in self.meshes.iter_mut().flat_map(|mesh| &mut mesh.vertices) {
            vertex.position -= origin;
        }
        for collider in &mut self.colliders {
            collider.centre -= origin;
        }
        for point in &mut self.supports {
            *point -= origin;
        }
        for clearance in &mut self.clearances {
            clearance.bounds.min -= origin;
            clearance.bounds.max -= origin;
        }
        #[cfg(test)]
        for member in &mut self.members {
            member.centre -= origin;
        }
        bounds.min -= origin;
        bounds.max -= origin;
        FurnitureRecipe {
            meshes: self.meshes,
            colliders: self.colliders,
            bounds,
            support_points_metres: self.supports,
            clearances: self.clearances,
            #[cfg(test)]
            members: self.members,
        }
    }
}
