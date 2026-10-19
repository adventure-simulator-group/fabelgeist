//! Consume validated native meshes into owned ordinary material assets.

use anyhow::{Context, Result, ensure};
use bevy::{
    camera::primitives::{Aabb, MeshAabb},
    prelude::*,
    render::storage::ShaderBuffer,
};

use super::{
    GrownFullAssets, GrownLeafMaterial, GrownPartAsset, GrownVertexEncoding, GrownWoodMaterial,
};
use crate::prepared::GrownFullGeometry;

impl GrownFullAssets {
    /// Framework boundary: moves every mesh; it does not duplicate a scene cache.
    pub fn install(
        geometry: GrownFullGeometry,
        encoding: GrownVertexEncoding,
        world: &mut World,
    ) -> Result<Self> {
        ensure!(
            world.contains_resource::<Assets<GrownWoodMaterial>>()
                && world.contains_resource::<Assets<GrownLeafMaterial>>()
                && world.contains_resource::<Assets<Mesh>>()
                && world.contains_resource::<Assets<ShaderBuffer>>(),
            "grown material plugin and renderer must be installed"
        );
        let bank = geometry.bank().clone();
        let source = geometry.source();
        let variant = geometry.variant();
        let library = bank.library();
        let model = library
            .models
            .get(variant.index())
            .context("grown material model")?;
        let low = Vec3::from_array(model.bounds.low.map(|value| value.metres()));
        let high = Vec3::from_array(model.bounds.high.map(|value| value.metres()));
        let source_bounds = Aabb::from_min_max(low, high);
        let sources = super::source::SourceMaterials::from_model(&bank, variant, encoding, world)?;
        let mut parts = Vec::new();
        for part in geometry.into_parts() {
            let role = part.role();
            let records = part.records();
            let mut mesh = part.into_mesh();
            let bounds = match encoding {
                GrownVertexEncoding::Address => {
                    // Every reference channel is ignored; culling uses the source envelope.
                    let count = mesh.count_vertices();
                    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0_f32; 3]; count]);
                    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0_f32; 3]; count]);
                    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0_f32; 2]; count]);
                    source_bounds
                }
                GrownVertexEncoding::Expanded => {
                    mesh.compute_aabb().context("grown mesh bounds")?
                }
            };
            let material = sources.material(role)?;
            let mesh = world.resource_mut::<Assets<Mesh>>().add(mesh);
            parts.push(GrownPartAsset {
                role,
                records,
                mesh,
                material,
                bounds,
            });
        }
        Ok(Self {
            bank,
            source,
            variant,
            encoding,
            parts,
            buffers: sources.buffers,
        })
    }
}
