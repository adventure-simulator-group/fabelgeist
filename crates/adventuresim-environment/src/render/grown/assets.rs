//! Consume checked encoded meshes into their matching ordinary material assets.

use anyhow::{Context, Result, ensure};
use bevy::{
    asset::RenderAssetUsages,
    camera::primitives::{Aabb, MeshAabb},
    prelude::*,
    render::storage::ShaderBuffer,
};
use std::sync::Arc;

use super::{GrownAssets, GrownBankAssets, GrownLeafMaterial, GrownPartAsset, GrownWoodMaterial};
use crate::prepared::{GrownGeometry, GrownVertexEncoding};

impl GrownAssets {
    /// Read-only native asset-store join; successful draws are verified separately.
    pub fn validate_materials(
        &self,
        wood: &Assets<GrownWoodMaterial>,
        leaves: &Assets<GrownLeafMaterial>,
    ) -> Result<()> {
        use super::{GrownBufferRole, GrownMaterialHandle};
        let shape = self.level.shape(&self.bank().library().detail);
        let shape = UVec4::new(shape.sides, shape.spans, shape.blades, 0);
        let buffers = &self.bank_assets.buffers;
        let species = super::source::buffer_handle(buffers, GrownBufferRole::Species)?;
        for part in self.parts() {
            match part.material() {
                GrownMaterialHandle::Wood(handle) => {
                    let actual = wood.get(handle).context("grown wood material removed")?;
                    ensure!(
                        actual.shape == shape
                            && actual.encoding == self.encoding
                            && actual.records
                                == super::source::buffer_handle(buffers, GrownBufferRole::Wood)?
                            && actual.species == species
                            && actual.plants
                                == super::source::buffer_handle(buffers, GrownBufferRole::Plants)?,
                        "grown wood source/shape binding changed"
                    );
                }
                GrownMaterialHandle::Leaves(handle) => {
                    let actual = leaves.get(handle).context("grown leaf material removed")?;
                    ensure!(
                        actual.shape == shape
                            && actual.encoding == self.encoding
                            && actual.records
                                == super::source::buffer_handle(buffers, GrownBufferRole::Leaves)?
                            && actual.species == species,
                        "grown leaf source/shape binding changed"
                    );
                }
            }
        }
        Ok(())
    }

    /// Check the complete target before a catalogue inserts its first asset.
    pub fn validate_target(world: &World) -> Result<()> {
        ensure!(
            world.contains_resource::<Assets<GrownWoodMaterial>>()
                && world.contains_resource::<Assets<GrownLeafMaterial>>()
                && world.contains_resource::<Assets<Mesh>>()
                && world.contains_resource::<Assets<ShaderBuffer>>(),
            "grown material plugin and renderer must be installed"
        );
        Ok(())
    }

    /// Framework boundary: moves every mesh; it does not duplicate a scene cache.
    pub fn install(
        geometry: GrownGeometry,
        bank_assets: Arc<GrownBankAssets>,
        world: &mut World,
    ) -> Result<Self> {
        bank_assets.validate_world(world)?;
        ensure!(
            bank_assets.bank().id() == geometry.bank().id(),
            "grown bank identity differs from geometry"
        );
        Self::validate_target(world)?;
        let encoding = geometry.encoding();
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
        let sources =
            super::source::SourceMaterials::from_geometry(&geometry, &bank_assets, world)?;
        let mut parts = Vec::new();
        for part in geometry.into_parts() {
            let role = part.role();
            let records = part.records();
            let mut mesh = part.into_mesh();
            let bounds = match encoding {
                GrownVertexEncoding::Address => source_bounds,
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
