use adventuresim_procedural_textures::ProceduralTextureAssets;
use bevy::prelude::*;

use super::ClientStartupTiming;

pub(super) fn setup_procedural_texture_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    startup: Option<Res<ClientStartupTiming>>,
) {
    let started = web_time::Instant::now();
    commands.insert_resource(ProceduralTextureAssets::load(&asset_server, &mut images));
    info!(
        elapsed_ms = started.elapsed().as_millis(),
        "Requested baked procedural texture assets"
    );
    if let Some(startup) = startup {
        startup.mark("baked procedural texture assets requested");
    }
}
