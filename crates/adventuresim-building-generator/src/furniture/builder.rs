use super::*;
use crate::{BUILDING_DETAIL_UV_METRES_PER_UNIT, BuildingLodMaterial, ResolvedItemId};
use bevy::math::{EulerRot, Quat, Vec2};

const GROUND_CONTACT_TOLERANCE_METRES: f32 = 0.001;

#[derive(Default)]
pub(crate) struct Builder {
    meshes: Vec<LodMesh>,
    construction_error: Option<crate::CollisionError>,
    pub(super) wood_state: FurnitureWoodState,
    colliders: Vec<CollisionCuboid<crate::furniture::FurnitureLocal>>,
    supports: Vec<Vec3>,
    clearances: Vec<FurnitureClearance>,
    #[cfg(test)]
    members: Vec<CollisionCuboid<crate::furniture::FurnitureLocal>>,
}

#[derive(Clone, Copy)]
pub(crate) enum CollisionPolicy {
    Solid,
    Decoration,
}

impl Builder {
    /// Preserve the caller's local origin for architectural components.
    pub(crate) fn into_meshes(self) -> Result<Vec<LodMesh>, crate::CollisionError> {
        if let Some(error) = self.construction_error {
            return Err(error);
        }
        Ok(self.meshes)
    }
    pub(super) fn mesh(&mut self, material: BuildingLodMaterial) -> &mut LodMesh {
        if let Some(index) = self
            .meshes
            .iter()
            .position(|mesh| mesh.material == material)
        {
            return &mut self.meshes[index];
        }
        self.meshes.push(LodMesh::new(material));
        let index = self.meshes.len() - 1;
        &mut self.meshes[index]
    }

    pub(crate) fn cuboid(
        &mut self,
        material: BuildingLodMaterial,
        centre: Vec3,
        size: Vec3,
        rotation: Quat,
        collision: CollisionPolicy,
    ) {
        self.member(material, centre, size, rotation, collision, None);
    }

    pub(super) fn wood_member(
        &mut self,
        material: BuildingLodMaterial,
        centre: Vec3,
        size: Vec3,
        wear: Option<super::finish::WearFace>,
    ) {
        self.member(
            material,
            centre,
            size,
            Quat::IDENTITY,
            CollisionPolicy::Solid,
            wear,
        );
    }

    fn member(
        &mut self,
        material: BuildingLodMaterial,
        centre: Vec3,
        size: Vec3,
        rotation: Quat,
        collision: CollisionPolicy,
        wear: Option<super::finish::WearFace>,
    ) {
        use crate::spatial_geometry::{CuboidDimensions, Position, RigidRotation};
        let admit = || {
            crate::CuboidCorners::from_pose(
                Position::<super::FurnitureLocal>::from_metres(centre)?,
                CuboidDimensions::from_metres(size)?,
                RigidRotation::from_quaternion(rotation)?,
            )
        };
        let corners = match admit() {
            Ok(corners) => corners,
            Err(cause) => {
                self.reject(crate::CollisionError {
                    source_id: ResolvedItemId(self.colliders.len() as u64 + 1),
                    cause,
                });
                return;
            }
        };
        // Mesh winding uses this permutation of the shared sign-coded topology.
        let corners = [0, 1, 3, 2, 4, 5, 7, 6].map(|index| corners.points()[index].metres());
        for (indices, normal) in [
            ([0, 1, 2, 3], -Vec3::Z),
            ([5, 4, 7, 6], Vec3::Z),
            ([4, 0, 3, 7], -Vec3::X),
            ([1, 5, 6, 2], Vec3::X),
            ([3, 2, 6, 7], Vec3::Y),
            ([4, 5, 1, 0], -Vec3::Y),
        ] {
            let positions = indices.map(|index| corners[index]);
            if matches!(
                material,
                BuildingLodMaterial::InteriorTimber
                    | BuildingLodMaterial::Timber
                    | BuildingLodMaterial::FurnitureWood(_)
            ) {
                super::finish::face(
                    self,
                    material,
                    positions,
                    (centre, size, rotation, normal),
                    wear,
                );
            } else {
                self.quad(material, positions, rotation * normal);
            }
        }

        for point in corners
            .into_iter()
            .filter(|point| point.y.abs() <= GROUND_CONTACT_TOLERANCE_METRES)
        {
            self.support(point);
        }
        let (yaw_radians, crossfall_radians, longfall_radians) = rotation.to_euler(EulerRot::YXZ);
        match CollisionCuboid::from_metres(
            ResolvedItemId(self.colliders.len() as u64 + 1),
            centre,
            size,
            yaw_radians,
            crossfall_radians,
            longfall_radians,
        ) {
            Ok(cuboid) => {
                #[cfg(test)]
                self.members.push(cuboid);
                if matches!(collision, CollisionPolicy::Solid) {
                    self.colliders.push(cuboid);
                }
            }
            Err(error) => self.reject(error),
        }
    }

    // Authored recipe assembly is transactional. A failed member never yields a
    // finished recipe; retain the first cause until the assembly boundary.
    fn reject(&mut self, error: crate::CollisionError) {
        self.construction_error.get_or_insert(error);
    }

    pub(super) fn timber(&mut self, centre: Vec3, size: Vec3) {
        self.cuboid(
            if self.wood_state == FurnitureWoodState::Painted {
                BuildingLodMaterial::FurnitureWood(FurnitureWoodSurface::Painted)
            } else {
                BuildingLodMaterial::InteriorTimber
            },
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
        let tangent = (positions[1] - positions[0]).normalize_or_zero();
        let bitangent = normal.cross(tangent);
        self.mesh(material).push_triangle(
            positions,
            normal,
            positions.map(|point| {
                Vec2::new(point.dot(tangent), point.dot(bitangent))
                    / BUILDING_DETAIL_UV_METRES_PER_UNIT
            }),
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
        match CollisionCuboid::from_metres(
            ResolvedItemId(self.colliders.len() as u64 + 1),
            centre,
            size,
            0.0,
            0.0,
            0.0,
        ) {
            Ok(cuboid) => self.colliders.push(cuboid),
            Err(error) => self.reject(error),
        }
    }

    pub(super) fn clearance(&mut self, kind: FurnitureClearanceKind, min: Vec3, max: Vec3) {
        match crate::spatial_geometry::SpatialBounds::from_metres(min, max) {
            Ok(bounds) => self.clearances.push(FurnitureClearance { kind, bounds }),
            Err(cause) => self.reject(crate::CollisionError {
                source_id: ResolvedItemId::default(),
                cause,
            }),
        }
    }

    pub(crate) fn finish(mut self) -> Result<FurnitureRecipe, crate::CollisionError> {
        if let Some(error) = self.construction_error {
            return Err(error);
        }
        let failure = |cause| crate::CollisionError {
            source_id: ResolvedItemId::default(),
            cause,
        };
        // Infinity sentinels belong only to the native vertex accumulator. They
        // cannot enter the admitted bounds owner, including an empty mesh set.
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        for vertex in self.meshes.iter().flat_map(|mesh| &mesh.vertices) {
            min = min.min(vertex.position);
            max = max.max(vertex.position);
        }
        let bounds = crate::spatial_geometry::SpatialBounds::<crate::furniture::FurnitureLocal>
            ::from_metres(min, max).map_err(failure)?;
        let centre = bounds.centre().map_err(failure)?.metres();
        let origin = Vec3::new(centre.x, 0.0, centre.z);
        for vertex in self.meshes.iter_mut().flat_map(|mesh| &mut mesh.vertices) {
            vertex.position -= origin;
        }
        for collider in &mut self.colliders {
            collider.centre =
                crate::spatial_geometry::Position::from_metres(collider.centre.metres() - origin)
                    .map_err(|cause| crate::CollisionError {
                    source_id: collider.source,
                    cause,
                })?;
        }
        for point in &mut self.supports {
            *point -= origin;
        }
        for clearance in &mut self.clearances {
            clearance.bounds = crate::spatial_geometry::SpatialBounds::from_metres(
                clearance.bounds.min().metres() - origin,
                clearance.bounds.max().metres() - origin,
            )
            .map_err(failure)?;
        }
        #[cfg(test)]
        for member in &mut self.members {
            member.centre =
                crate::spatial_geometry::Position::from_metres(member.centre.metres() - origin)
                    .map_err(failure)?;
        }
        Ok(FurnitureRecipe {
            meshes: self.meshes,
            colliders: self.colliders,
            bounds: crate::spatial_geometry::SpatialBounds::from_metres(min - origin, max - origin)
                .map_err(failure)?,
            support_points_metres: self
                .supports
                .into_iter()
                .map(crate::spatial_geometry::Position::from_metres)
                .collect::<Result<Vec<_>, _>>()
                .map_err(failure)?,
            clearances: self.clearances,
            #[cfg(test)]
            members: self.members,
        })
    }
}
