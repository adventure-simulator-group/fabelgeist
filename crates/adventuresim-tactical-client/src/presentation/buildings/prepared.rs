//! Load compiled city meshes without running the building generator in Wasm.
use super::*;
use adventuresim_building_generator::prepared::{PreparedBuilding, recipe_key};
use bevy::asset::{AssetLoader, LoadContext, LoadState, io::Reader};

const MAX_CONCURRENT_FACADE_LOADS: usize = 4;

#[derive(Asset, TypePath)]
pub(super) struct PreparedCityAsset(CompiledBuildingLevels);

#[derive(Default, TypePath)]
pub(super) struct PreparedCityLoader;

impl AssetLoader for PreparedCityLoader {
    type Asset = PreparedCityAsset;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        context: &mut LoadContext<'_>,
    ) -> std::io::Result<Self::Asset> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let data: PreparedBuilding = postcard::from_bytes(&bytes)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
        let mut batches = |level, source: Vec<LodMesh>| {
            source
                .into_iter()
                .enumerate()
                .map(|(index, batch)| {
                    let mut mesh = recipe_mesh(&batch, data.local_origin);
                    mesh.asset_usage = RenderAssetUsages::RENDER_WORLD;
                    CompiledBuildingBatch {
                        material: batch.material,
                        triangles: batch.indices.len() / 3,
                        mesh: context.add_labeled_asset(format!("lod{level}/{index}"), mesh),
                    }
                })
                .collect()
        };
        let detail = if !data.lod0.is_empty() {
            BuildingDetail::Static
        } else if !data.lod1.is_empty() {
            BuildingDetail::Facade
        } else {
            BuildingDetail::Shell
        };
        Ok(PreparedCityAsset(CompiledBuildingLevels {
            facade_openings: Default::default(),
            interior: None,
            program: data.program,
            detail,
            local_origin: data.local_origin,
            floor_offset_metres: data.floor_offset_metres,
            sign_sites: data.sign_sites,
            lod0: batches(0, data.lod0),
            lod1: batches(1, data.lod1),
            lod2: batches(2, data.lod2),
        }))
    }

    fn extensions(&self) -> &[&str] {
        &["building"]
    }
}

#[derive(Default, Resource)]
pub(in crate::presentation) struct PreparedCityAssets {
    handles: Vec<(BuildingProgram, BuildingDetail, Handle<PreparedCityAsset>)>,
}

impl PreparedCityAssets {
    pub(super) fn get(
        &mut self,
        server: &AssetServer,
        assets: &Assets<PreparedCityAsset>,
        program: &BuildingProgram,
        detail: BuildingDetail,
    ) -> Result<Option<CompiledBuildingLevels>> {
        let handle = if let Some((_, _, handle)) = self
            .handles
            .iter()
            .find(|(recipe, level, _)| recipe == program && *level == detail)
        {
            handle
        } else {
            if matches!(detail, BuildingDetail::Facade | BuildingDetail::Shell)
                && self
                    .handles
                    .iter()
                    .filter(|(_, level, handle)| {
                        *level == detail
                            && matches!(
                                server.load_state(handle.id()),
                                LoadState::Loading | LoadState::NotLoaded
                            )
                    })
                    .count()
                    >= MAX_CONCURRENT_FACADE_LOADS
            {
                return Ok(None);
            }
            let suffix = match detail {
                BuildingDetail::Shell => "overview",
                BuildingDetail::Facade => "facade",
                BuildingDetail::Static => "detail",
                BuildingDetail::Dynamic => {
                    return Err("prepared city assets have static openings".into());
                }
            };
            let handle = server.load(format!(
                "art-demo/buildings/{}.{suffix}.building",
                recipe_key(program)
            ));
            self.handles.push((program.clone(), detail, handle));
            &self.handles.last().expect("just inserted").2
        };
        if let Some(LoadState::Failed(error)) = server.get_load_state(handle.id()) {
            return Err(format!("prepared city recipe failed: {error}").into());
        }
        Ok(assets.get(handle).map(|asset| asset.0.clone()))
    }

    pub(super) fn retain_streamed(
        &mut self,
        facades: &[(Entity, f32, BuildingProgram)],
        details: &[(Entity, f32, BuildingProgram)],
    ) {
        self.handles.retain(|(program, detail, _)| {
            matches!(detail, BuildingDetail::Shell)
                || (*detail == BuildingDetail::Facade
                    && facades.iter().any(|(_, _, recipe)| recipe == program))
                || (*detail == BuildingDetail::Static
                    && details.iter().any(|(_, _, recipe)| recipe == program))
        });
    }

    pub(in crate::presentation) fn release_details(&mut self) {
        self.handles
            .retain(|(_, detail, _)| *detail == BuildingDetail::Shell);
    }
}
