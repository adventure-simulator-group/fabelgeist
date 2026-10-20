//! Consume validated native meshes into owned ordinary material assets.

use anyhow::{Context, Result, ensure};
use bevy::{
    asset::RenderAssetUsages,
    camera::primitives::{Aabb, MeshAabb},
    prelude::*,
    render::storage::ShaderBuffer,
};
use std::sync::Arc;

use super::{
    GrownAssets, GrownBankAssets, GrownLeafMaterial, GrownPartAsset, GrownVertexEncoding,
    GrownWoodMaterial,
};
use crate::prepared::GrownGeometry;

impl GrownAssets {
    /// Framework boundary: moves every mesh; it does not duplicate a scene cache.
    pub fn install(
        geometry: GrownGeometry,
        bank_assets: Arc<GrownBankAssets>,
        encoding: GrownVertexEncoding,
        world: &mut World,
    ) -> Result<Self> {
        ensure!(
            bank_assets.world == world.id(),
            "grown bank assets belong to another world"
        );
        ensure!(
            bank_assets.bank().id() == geometry.bank().id(),
            "grown bank identity differs from geometry"
        );
        ensure!(
            world.contains_resource::<Assets<GrownWoodMaterial>>()
                && world.contains_resource::<Assets<GrownLeafMaterial>>()
                && world.contains_resource::<Assets<Mesh>>()
                && world.contains_resource::<Assets<ShaderBuffer>>(),
            "grown material plugin and renderer must be installed"
        );
        let source = geometry.source();
        let variant = geometry.variant();
        let level = geometry.level();
        let library = bank_assets.bank().library();
        let model = library
            .models
            .get(variant.index())
            .context("grown material model")?;
        let low = Vec3::from_array(model.bounds.low.map(|value| value.metres()));
        let high = Vec3::from_array(model.bounds.high.map(|value| value.metres()));
        let source_bounds = Aabb::from_min_max(low, high);
        let sources = super::source::SourceMaterials::from_geometry(
            &geometry,
            &bank_assets,
            encoding,
            world,
        )?;
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
            mesh.asset_usage = RenderAssetUsages::RENDER_WORLD;
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
            bank_assets,
            source,
            variant,
            level,
            encoding,
            parts,
        })
    }
}
