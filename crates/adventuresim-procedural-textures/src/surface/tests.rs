use super::*;
use crate::{BakedRecipe, MapChannel, PixelEncoding, TextureParameters, TextureRecipeId};

#[test]
fn default_oak_height_matches_the_pre_september_reference() {
    // Independent samples from 12d529d4, before the September 1 graph rewrite.
    let reference = include_bytes!("fixtures/august-height.f32");
    let params = TextureParameters::default();
    for (index, bytes) in reference.chunks_exact(4).enumerate() {
        let expected = f32::from_le_bytes(bytes.try_into().unwrap());
        let u = ((index % 64) as f32 + 0.5) / 64.0;
        let v = ((index / 64) as f32 + 0.5) / 64.0;
        let actual = oak_bark_height(&params, u, v);
        assert!(
            (actual - expected).abs() < 1.0e-6,
            "({u}, {v}): {actual} != {expected}"
        );
    }
    assert_eq!(reference.len(), 64 * 64 * 4);
}

#[test]
fn committed_oak_bake_matches_the_recipe_and_shader_channel_layout() {
    let bytes = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/textures/procedural/oak-bark.ptex"
    ));
    let stored = BakedRecipe::from_compressed_bytes(bytes).unwrap();
    let generated = BakedRecipe::generate(TextureRecipeId::OakBark, &TextureParameters::default());
    let map = stored.map(MapChannel::HeightAo).unwrap();
    let expected = generated.map(MapChannel::HeightAo).unwrap();
    assert_eq!(stored.recipe, TextureRecipeId::OakBark);
    assert_eq!(stored.tile_metres, generated.tile_metres);
    assert_eq!(stored.height_range_metres, generated.height_range_metres);
    assert!(matches!(map.encoding, PixelEncoding::Rgba8));
    assert_eq!(map.mip_levels, 11);
    assert_eq!(map.size, expected.size);
    assert_eq!(map.sampler, expected.sampler);
    assert_eq!(map.bytes.len(), expected.bytes.len());
    // Allow last-bit platform math differences, never a different height field
    // or swapped channels. Check every mip, including the runtime's far tile.
    for (actual, expected) in map
        .bytes
        .chunks_exact(4)
        .zip(expected.bytes.chunks_exact(4))
    {
        assert!(
            (crate::decode_height_ao(actual) - crate::decode_height_ao(expected)).abs()
                < 2.0 / 65535.0,
            "rebake oak-bark after editing its recipe"
        );
        assert!(actual[2].abs_diff(expected[2]) <= 1, "rebake oak-bark AO");
    }
    let base = &map.bytes[..(map.size * map.size * 4) as usize];
    let ao: std::collections::BTreeSet<_> = base.chunks_exact(4).map(|p| p[2]).collect();
    assert!(ao.len() > 96);
    assert!(ao.first().is_some_and(|value| *value < 150));
    assert_eq!(ao.last(), Some(&255));
    assert!(base.chunks_exact(4).all(|p| p[3] == 255));
}

#[test]
fn oak_controls_and_seed_change_the_rendered_height_and_still_tile() {
    let original = TextureParameters::default();
    let mut edited = original.clone();
    edited.seed = 73;
    edited.surface.columns = 12;
    edited.surface.rows = 8;
    edited.surface.valley_width_min *= 1.25;
    edited.validate().unwrap();
    let mut changed = 0;
    for y in 0..32 {
        for x in 0..32 {
            let u = (x as f32 + 0.5) / 32.0;
            let v = (y as f32 + 0.5) / 32.0;
            let height = oak_bark_height(&edited, u, v);
            changed += usize::from((height - oak_bark_height(&original, u, v)).abs() > 0.001);
            assert!((height - oak_bark_height(&edited, u + 1.0, v)).abs() < 0.0001);
            assert!((height - oak_bark_height(&edited, u, v + 1.0)).abs() < 0.0001);
        }
    }
    assert!(changed > 900);
}
