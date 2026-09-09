use std::time::{Duration, Instant};

use bevy::asset::{AssetPlugin, LoadState};

use super::*;
use crate::{PROCEDURAL_TEXTURE_CATALOGUE, ProceduralTextureAssets};

fn requested_images(textures: &ProceduralTextureAssets) -> Vec<Handle<Image>> {
    let mut handles = vec![
        textures.oak_bark.height_ao.clone(),
        textures.forest_soil.height_ao.clone(),
        textures.forest_soil.litter_surface.clone(),
        textures.forest_soil.litter_normal.clone(),
        textures.window_glass.transmittance.clone(),
        textures.window_glass.optical_normal_gl.clone(),
        textures.window_glass.thickness_roughness.clone(),
        textures.crenellation_mask.clone(),
    ];
    for leaf in [
        &textures.oak_leaf,
        &textures.dry_oak_leaf,
        &textures.hazel_leaf,
        &textures.blackthorn_leaf,
        &textures.hawthorn_leaf,
        &textures.beech_leaf,
    ] {
        handles.extend([
            leaf.opacity.clone(),
            leaf.front_albedo.clone(),
            leaf.back_albedo.clone(),
            leaf.front_normal.clone(),
            leaf.back_normal.clone(),
            leaf.height.clone(),
            leaf.arm.clone(),
        ]);
    }
    for surface in [
        &textures.rock,
        &textures.lime_plaster,
        &textures.hewn_oak,
        &textures.wattle_and_daub,
        &textures.handmade_brick,
        &textures.rubble_masonry,
        &textures.dressed_stone,
        &textures.clay_roof_tile,
        &textures.slate_roof,
        &textures.timber_shingle,
        &textures.plank_floor,
        &textures.lead_sheet,
        &textures.ironwork,
    ] {
        handles.extend([
            surface.albedo.clone(),
            surface.normal_gl.clone(),
            surface.height.clone(),
            surface.arm.clone(),
        ]);
    }
    handles
}

#[test]
fn committed_collection_loads_exact_mips_and_samplers_without_baking() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin {
            file_path: root.to_str().unwrap().to_owned(),
            ..Default::default()
        },
        BakedTexturesPlugin,
    ))
    .init_asset::<Image>();
    app.finish();
    app.cleanup();
    let server = app.world().resource::<AssetServer>().clone();
    let textures = ProceduralTextureAssets::load(
        &server,
        &mut app.world_mut().resource_mut::<Assets<Image>>(),
    );
    let handles = requested_images(&textures);
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        app.update();
        for handle in &handles {
            assert!(
                !matches!(server.load_state(handle.id()), LoadState::Failed(_)),
                "failed to load {:?}: {:?}",
                handle.path(),
                server.load_state(handle.id())
            );
        }
        if handles
            .iter()
            .all(|handle| server.is_loaded_with_dependencies(handle.id()))
        {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "committed texture loading timed out"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    // Verify the runtime bindings cover every committed recipe channel exactly once,
    // and that Image upload preserves all metadata and the complete pixel payload.
    let images = app.world().resource::<Assets<Image>>();
    let mut checked = 0;
    for descriptor in PROCEDURAL_TEXTURE_CATALOGUE {
        let bytes = std::fs::read(root.join(descriptor.id.runtime_asset_path())).unwrap();
        let bake = BakedRecipe::from_compressed_bytes(&bytes).unwrap();
        assert_eq!(bake.recipe, descriptor.id);
        for map in bake.maps {
            let path = format!(
                "{}#{}",
                descriptor.id.runtime_asset_path(),
                map.channel.slug()
            );
            let matches: Vec<_> = handles
                .iter()
                .filter(|handle| handle.path().unwrap().to_string() == path)
                .collect();
            assert_eq!(matches.len(), 1, "runtime binding for {path}");
            let image = images.get(matches[0]).unwrap();
            assert_eq!(image.width(), map.size);
            assert_eq!(image.height(), map.size);
            assert_eq!(image.texture_descriptor.mip_level_count, map.mip_levels);
            assert_eq!(map.mip_levels, map.size.ilog2() + 1);
            assert_eq!(
                image.texture_descriptor.format,
                map.encoding.texture_format()
            );
            assert_eq!(image.sampler, map.sampler);
            assert_eq!(image.data.as_ref().unwrap(), &map.bytes);
            checked += 1;
        }
    }
    assert_eq!(checked, handles.len());
    let blood = images.get(&textures.terrain_blood_mask).unwrap();
    assert!(blood.data.as_ref().unwrap().iter().all(|pixel| *pixel == 0));
    assert!(blood.asset_usage.contains(RenderAssetUsages::MAIN_WORLD));
}
