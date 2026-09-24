//! Asynchronous loading of checked-in texture recipes, without running generators.

mod handles;
#[cfg(test)]
mod tests;

use bevy::{
    asset::{AssetLoader, LoadContext, RenderAssetUsages, io::Reader},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension},
};
use std::collections::{HashMap, HashSet};

use crate::{BakedMap, BakedRecipe, MapChannel, TextureRecipeId};

pub struct BakedTexturesPlugin;

impl Plugin for BakedTexturesPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<RuntimeTextureRecipe>()
            .init_asset_loader::<RuntimeTextureLoader>()
            .init_resource::<ProceduralTextureResidency>()
            .add_systems(Update, install_loaded_texture_recipes);
    }
}

#[derive(Asset, TypePath)]
struct RuntimeTextureRecipe {
    maps: Vec<(MapChannel, Image)>,
}

/// Owns one strong container handle per requested recipe and installs its maps
/// into the stable image handles used by materials.
#[derive(Resource, Default)]
pub struct ProceduralTextureResidency {
    requested: HashMap<TextureRecipeId, Handle<RuntimeTextureRecipe>>,
    installed: HashSet<TextureRecipeId>,
    destinations: HashMap<(TextureRecipeId, MapChannel), Handle<Image>>,
}

impl ProceduralTextureResidency {
    pub fn request(
        &mut self,
        server: &AssetServer,
        recipes: impl IntoIterator<Item = TextureRecipeId>,
    ) {
        for recipe in recipes {
            self.requested
                .entry(recipe)
                .or_insert_with(|| server.load(recipe.runtime_asset_path()));
        }
    }

    pub fn is_ready(&self, recipes: impl IntoIterator<Item = TextureRecipeId>) -> bool {
        recipes
            .into_iter()
            .all(|recipe| self.installed.contains(&recipe))
    }

    pub fn requested_container_count(&self) -> usize {
        self.requested.len()
    }

    fn destination(
        &mut self,
        recipe: TextureRecipeId,
        channel: MapChannel,
        images: &mut Assets<Image>,
    ) -> Handle<Image> {
        self.destinations
            .entry((recipe, channel))
            .or_insert_with(|| images.add(Image::default()))
            .clone()
    }
}

fn install_loaded_texture_recipes(
    recipes: Res<Assets<RuntimeTextureRecipe>>,
    mut residency: ResMut<ProceduralTextureResidency>,
    mut images: ResMut<Assets<Image>>,
) {
    let ready = residency
        .requested
        .iter()
        .filter_map(|(id, handle)| (!residency.installed.contains(id)).then_some((*id, handle)))
        .filter_map(|(id, handle)| recipes.get(handle).map(|recipe| (id, recipe.maps.clone())))
        .collect::<Vec<_>>();
    for (id, maps) in ready {
        for (channel, image) in maps {
            let handle = residency
                .destinations
                .get(&(id, channel))
                .expect("runtime map has a destination");
            images
                .insert(handle.id(), image)
                .expect("reserved runtime image handle exists");
        }
        residency.installed.insert(id);
    }
}

#[derive(Default, TypePath)]
struct RuntimeTextureLoader;

impl AssetLoader for RuntimeTextureLoader {
    type Asset = RuntimeTextureRecipe;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let bake = BakedRecipe::from_compressed_bytes(&bytes)?;
        if load_context
            .path()
            .path()
            .file_stem()
            .and_then(|name| name.to_str())
            != Some(bake.recipe.slug())
        {
            return Err(std::io::Error::other(
                "texture bake recipe does not match its filename",
            ));
        }
        let maps = bake
            .maps
            .into_iter()
            .map(|map| (map.channel, Image::from(map)))
            .collect();
        Ok(RuntimeTextureRecipe { maps })
    }

    fn extensions(&self) -> &[&str] {
        &["ptex"]
    }
}

impl TextureRecipeId {
    /// Relative to Bevy's asset root, on native and web builds alike.
    pub fn runtime_asset_path(self) -> String {
        format!("textures/procedural/{}.ptex", self.slug())
    }
}

impl From<BakedMap> for Image {
    fn from(map: BakedMap) -> Self {
        let mut image = Image::new_uninit(
            Extent3d {
                width: map.size,
                height: map.size,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            map.encoding.texture_format(),
            RenderAssetUsages::RENDER_WORLD,
        );
        image.texture_descriptor.mip_level_count = map.mip_levels;
        image.data = Some(map.bytes);
        image.sampler = map.sampler;
        image
    }
}
