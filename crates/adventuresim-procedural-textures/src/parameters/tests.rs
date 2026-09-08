use crate::{
    BakeResolution, BakedRecipe, MapChannel, PROCEDURAL_TEXTURE_CATALOGUE, TextureParameters,
    TextureRecipeId,
};

fn draft() -> TextureParameters {
    TextureParameters {
        resolution: BakeResolution::Draft,
        ..Default::default()
    }
}

#[test]
fn compact_json_float_spellings_survive_the_worker_boundary() {
    let source = serde_json::to_string(&draft()).unwrap();
    let value = serde_json::from_str(&source).unwrap();
    TextureParameters::from_value(value).unwrap();
}

#[test]
fn every_recipe_bakes_independently_with_complete_transferable_mips() {
    let parameters = draft();
    parameters.validate().unwrap();
    for descriptor in PROCEDURAL_TEXTURE_CATALOGUE {
        let bake = BakedRecipe::generate(descriptor.id, &parameters);
        assert!(!bake.maps.is_empty(), "{}", descriptor.id.slug());
        let transferred = BakedRecipe::from_bytes(&bake.to_bytes()).unwrap();
        for (map, copy) in bake.maps.iter().zip(&transferred.maps) {
            assert_eq!(
                map.mip_levels,
                map.size.ilog2() + 1,
                "{} / {}",
                descriptor.id.slug(),
                map.channel.slug()
            );
            let texels: usize = (0..map.mip_levels)
                .map(|level| (map.size >> level).pow(2) as usize)
                .sum();
            assert_eq!(map.bytes.len(), texels * map.encoding.channels());
            assert_eq!(map.bytes, copy.bytes);
        }
    }
}

#[test]
fn leaf_palette_changes_preserve_species_shape_and_relief() {
    let mut parameters = draft();
    let before = BakedRecipe::generate(TextureRecipeId::WhiteOakLeaf, &parameters);
    parameters.leaf_colors.white_oak = parameters.leaf_colors.hazel;
    parameters.leaf_colors.white_oak.blade = [12, 200, 40];
    let after = BakedRecipe::generate(TextureRecipeId::WhiteOakLeaf, &parameters);
    assert_ne!(
        before.map(MapChannel::FrontAlbedo).unwrap().bytes,
        after.map(MapChannel::FrontAlbedo).unwrap().bytes
    );
    for channel in [
        MapChannel::Opacity,
        MapChannel::Height,
        MapChannel::FrontNormal,
        MapChannel::BackNormal,
    ] {
        assert_eq!(
            before.map(channel).unwrap().bytes,
            after.map(channel).unwrap().bytes,
            "{}",
            channel.slug()
        );
    }
}

#[test]
fn removing_knots_changes_both_growth_color_and_relief_deterministically() {
    let parameters = draft();
    let before = BakedRecipe::generate(TextureRecipeId::HewnOak, &parameters);
    let mut edited = serde_json::to_value(&parameters).unwrap();
    edited["hewn_oak_grain"]["knots"] = serde_json::json!([]);
    let edited = TextureParameters::from_value(edited).unwrap();
    let after = BakedRecipe::generate(TextureRecipeId::HewnOak, &edited);
    let repeated = BakedRecipe::generate(TextureRecipeId::HewnOak, &edited);
    for channel in [MapChannel::Albedo, MapChannel::Height, MapChannel::Normal] {
        assert_ne!(
            before.map(channel).unwrap().bytes,
            after.map(channel).unwrap().bytes,
            "{}",
            channel.slug()
        );
        assert_eq!(
            after.map(channel).unwrap().bytes,
            repeated.map(channel).unwrap().bytes
        );
    }
}

#[test]
fn imported_nested_typos_and_degenerate_features_are_rejected() {
    let mut value = serde_json::to_value(draft()).unwrap();
    value["hewn_oak_grain"]["knots"][0]["radiuss"] = 0.05.into();
    assert!(TextureParameters::from_value(value).is_err());
    let mut value = serde_json::to_value(draft()).unwrap();
    value["window_glass"]["bubbles"][0][2] = 0.0.into();
    assert!(TextureParameters::from_value(value).is_err());
}
