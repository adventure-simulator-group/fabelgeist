use adventuresim_procedural_textures::{
    PROCEDURAL_TEXTURE_CATALOGUE, ProceduralTextureAssets, ProceduralTextureResidency,
};
use bevy::prelude::*;

use super::ClientStartupTiming;

#[derive(Resource)]
pub(crate) struct LazyProceduralTextures;

pub(super) fn setup_procedural_texture_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut residency: ResMut<ProceduralTextureResidency>,
    lazy: Option<Res<LazyProceduralTextures>>,
    startup: Option<Res<ClientStartupTiming>>,
) {
    let started = web_time::Instant::now();
    commands.insert_resource(ProceduralTextureAssets::reserve(
        &mut residency,
        &mut images,
    ));
    if lazy.is_none() {
        residency.request(
            &asset_server,
            PROCEDURAL_TEXTURE_CATALOGUE.iter().map(|recipe| recipe.id),
        );
    }
    info!(
        elapsed_ms = started.elapsed().as_millis(),
        "Requested baked procedural texture assets"
    );
    if let Some(startup) = startup {
        startup.mark("baked procedural texture assets requested");
    }
}
