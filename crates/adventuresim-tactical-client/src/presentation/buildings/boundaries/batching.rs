//! Fixed enclosure geometry shares material draws within spatial cells.
use adventuresim_tactical_core::prelude::CityBoundaryMember;
use bevy::prelude::*;
use std::collections::HashMap;

// Keep culling spatially useful without retaining an entity for every wall cap.
const BOUNDARY_BATCH_METRES: f32 = 128.0;

pub(super) struct BoundaryMesh {
    pub mesh: Mesh,
    pub material: Handle<StandardMaterial>,
    pub origin: Vec3,
}

#[derive(Default)]
pub(super) struct BoundaryBatches {
    meshes: HashMap<(IVec2, AssetId<StandardMaterial>), BoundaryMesh>,
    pub members: usize,
}

impl BoundaryBatches {
    pub fn insert(
        &mut self,
        member: CityBoundaryMember,
        elevation: f32,
        material: Handle<StandardMaterial>,
    ) {
        let centre = member.centre_metres + Vec3::Y * elevation;
        let cell = (centre.xz() / BOUNDARY_BATCH_METRES).floor().as_ivec2();
        let origin = Vec3::new(cell.x as f32, 0.0, cell.y as f32) * BOUNDARY_BATCH_METRES;
        let mesh = crate::presentation::recipe_mesh::metric_cuboid(member.size_metres)
            .transformed_by(
                Transform::from_translation(centre - origin)
                    .with_rotation(Quat::from_rotation_y(member.yaw_radians)),
            );
        match self.meshes.entry((cell, material.id())) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                entry
                    .get_mut()
                    .mesh
                    .merge(&mesh)
                    .expect("boundary cuboids share indexed vertex attributes");
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(BoundaryMesh {
                    mesh,
                    material,
                    origin,
                });
            }
        }
        self.members += 1;
    }

    pub fn finish(self) -> impl ExactSizeIterator<Item = BoundaryMesh> {
        self.meshes.into_values()
    }
}

#[cfg(test)]
mod tests;
