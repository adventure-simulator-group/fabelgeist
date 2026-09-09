//! Asynchronous loading of checked-in texture recipes, without running generators.

mod handles;
#[cfg(test)]
mod tests;

use bevy::{
    asset::{AssetLoader, LoadContext, RenderAssetUsages, io::Reader},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension},
};

use crate::{BakedMap, BakedRecipe, TextureRecipeId};

pub struct BakedTexturesPlugin;

impl Plugin for BakedTexturesPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<RuntimeTextureRecipe>()
            .init_asset_loader::<RuntimeTextureLoader>();
    }
}

#[derive(Asset, TypePath)]
struct RuntimeTextureRecipe {
    #[dependency]
    maps: Vec<Handle<Image>>,
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
            .map(|map| {
                load_context.add_labeled_asset(map.channel.slug().to_owned(), Image::from(map))
            })
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
