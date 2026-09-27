//! Shared geometry permits independent operable leaves to instance in PBR passes.
use adventuresim_building_generator::{
    BuildingLodMaterial, ClosureState, WindowLeafKind, compile_window_leaf,
};
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Dimensions([u32; 3]);

impl From<Vec3> for Dimensions {
    fn from(size: Vec3) -> Self {
        Self(size.to_array().map(f32::to_bits))
    }
}

pub(super) struct LeafSurface {
    pub mesh: Handle<Mesh>,
    pub material: BuildingLodMaterial,
}

#[derive(Default, Resource)]
pub(super) struct ClosureMeshes {
    doors: HashMap<Dimensions, Handle<Mesh>>,
    glass: HashMap<Dimensions, Vec<LeafSurface>>,
    shutters: HashMap<Dimensions, Vec<LeafSurface>>,
}

impl ClosureMeshes {
    pub fn door(&mut self, size: Vec3, meshes: &mut Assets<Mesh>) -> Handle<Mesh> {
        self.doors
            .entry(size.into())
            .or_insert_with(|| meshes.add(Cuboid::from_size(size)))
            .clone()
    }

    pub fn window(
        &mut self,
        size: Vec3,
        kind: WindowLeafKind,
        meshes: &mut Assets<Mesh>,
    ) -> &[LeafSurface] {
        let cache = match kind {
            WindowLeafKind::LeadedGlass => &mut self.glass,
            WindowLeafKind::TimberShutter => &mut self.shutters,
        };
        cache.entry(size.into()).or_insert_with(|| {
            compile_window_leaf(size, kind, ClosureState::Operable)
                .iter()
                .map(|batch| LeafSurface {
                    mesh: meshes.add(super::recipe_mesh::recipe_mesh(batch, Vec3::ZERO)),
                    material: batch.material,
                })
                .collect()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_leaves_share_geometry_without_sharing_transforms() {
        let mut meshes = Assets::<Mesh>::default();
        let mut cache = ClosureMeshes::default();
        let size = Vec3::new(1.0, 2.0, 0.05);
        let first = cache.door(size, &mut meshes);
        let repeated = cache.door(size, &mut meshes);
        assert_eq!(first, repeated);
        assert_ne!(first, cache.door(size * 2.0, &mut meshes));
        let mut world = World::new();
        let closed = world.spawn((Mesh3d(first), Transform::IDENTITY)).id();
        let open = world.spawn((Mesh3d(repeated), Transform::IDENTITY)).id();
        world.get_mut::<Transform>(open).unwrap().rotate_y(1.0);
        assert_eq!(
            *world.get::<Transform>(closed).unwrap(),
            Transform::IDENTITY
        );
        assert_ne!(world.get::<Transform>(closed), world.get::<Transform>(open));
    }

    #[test]
    fn window_cache_keeps_glazing_material_parts_and_distinguishes_shutters() {
        let mut meshes = Assets::<Mesh>::default();
        let mut cache = ClosureMeshes::default();
        let size = Vec3::new(0.8, 1.2, 0.04);
        let expected =
            compile_window_leaf(size, WindowLeafKind::LeadedGlass, ClosureState::Operable);
        let first: Vec<_> = cache
            .window(size, WindowLeafKind::LeadedGlass, &mut meshes)
            .iter()
            .map(|part| (part.mesh.clone(), part.material))
            .collect();
        assert_eq!(first.len(), expected.len());
        for ((mesh, material), source) in first.iter().zip(expected) {
            assert_eq!(*material, source.material);
            assert_eq!(
                meshes.get(mesh).unwrap().indices().unwrap().len(),
                source.indices.len()
            );
        }
        let again = cache.window(size, WindowLeafKind::LeadedGlass, &mut meshes);
        assert!(
            first
                .iter()
                .zip(again)
                .all(|((mesh, _), part)| *mesh == part.mesh)
        );
        let shutter = cache.window(size, WindowLeafKind::TimberShutter, &mut meshes);
        assert!(
            shutter
                .iter()
                .all(|part| first.iter().all(|(mesh, _)| *mesh != part.mesh))
        );
    }
}
