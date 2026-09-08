use super::*;
use crate::{BakeResolution, BakedRecipe, MapChannel, TextureParameters, TextureRecipeId};
use bevy::math::Vec2;
#[test]
fn presets_share_a_continuous_shape_space() {
    let a = LeafShape::default();
    let b = LeafShape::preset("beech").unwrap();
    assert_eq!(a.interpolate(&b, 0.0), a);
    assert_eq!(a.interpolate(&b, 1.0), b);
    let mask = |shape: &LeafShape| {
        let k = kernel::Kernel::from(uniform::LeafUniform::new(shape, 128, 7));
        (0..64 * 64)
            .map(|i| {
                k.class(
                    (Vec2::new((i % 64) as f32 + 0.5, (i / 64) as f32 + 0.5) / 64.0
                        - Vec2::splat(0.5))
                        * 2.0,
                ) > 0
            })
            .collect::<Vec<_>>()
    };
    let mut previous = mask(&a);
    let first = previous.clone();
    for step in 1..=20 {
        let next = mask(&a.interpolate(&b, step as f32 / 20.0));
        let changed = previous.iter().zip(&next).filter(|(a, b)| a != b).count();
        assert!(
            changed < 180,
            "shape popped at step {step}: {changed} texels"
        );
        previous = next;
    }
    assert_ne!(first, previous);
}
#[test]
fn every_reference_preset_has_finite_editable_controls_and_visible_anatomy() {
    for name in LeafShape::preset_names() {
        let shape = LeafShape::preset(name).unwrap();
        let value = serde_json::to_value(&shape).unwrap();
        for (field, v) in value.as_object().unwrap() {
            let (min, max) = parameter_range(field).unwrap();
            let n = v.as_f64().unwrap() as f32;
            assert!((min..=max).contains(&n), "{name}/{field}: {n}");
        }
        let k = kernel::Kernel::from(uniform::LeafUniform::new(&shape, 128, 1));
        let mut classes = [0; 3];
        for y in 0..64 {
            for x in 0..64 {
                classes[k.class(
                    (Vec2::new(x as f32 + 0.5, y as f32 + 0.5) / 64.0 - Vec2::splat(0.5)) * 2.0,
                ) as usize] += 1;
            }
        }
        assert!(
            classes.iter().all(|n| *n > 0),
            "{name} misses anatomy: {classes:?}"
        );
    }
}
#[test]
fn shared_leaf_bake_is_repeatable_antialiased_and_material_complete() {
    let params = TextureParameters {
        resolution: BakeResolution::Draft,
        ..Default::default()
    };
    let before = BakedRecipe::generate(TextureRecipeId::WhiteOakLeaf, &params);
    let after = BakedRecipe::generate(TextureRecipeId::WhiteOakLeaf, &params);
    for (a, b) in before.maps.iter().zip(&after.maps) {
        assert_eq!(a.bytes, b.bytes);
        assert_eq!(a.mip_levels, 8);
    }
    let alpha = before.map(MapChannel::Opacity).unwrap();
    assert!(
        alpha.bytes[..128 * 128 * 4]
            .chunks_exact(4)
            .any(|p| p[0] > 0 && p[0] < 255)
    );
    let arm = before.map(MapChannel::Arm).unwrap();
    assert!(arm.bytes.chunks_exact(4).all(|p| p[2] == 0));
    assert_ne!(
        before.map(MapChannel::FrontNormal).unwrap().bytes,
        before.map(MapChannel::BackNormal).unwrap().bytes
    );
}
#[test]
fn fractional_lobe_counts_do_not_pop_at_integer_boundaries() {
    let a = LeafShape {
        lobe_frequency: 4.999,
        ..Default::default()
    };
    let mut b = a.clone();
    b.lobe_frequency = 5.001;
    let ka = kernel::Kernel::from(uniform::LeafUniform::new(&a, 256, 1));
    let kb = kernel::Kernel::from(uniform::LeafUniform::new(&b, 256, 1));
    let mut changed = 0;
    for y in 0..128 {
        for x in 0..128 {
            let p = (Vec2::new(x as f32 + 0.5, y as f32 + 0.5) / 128.0 - Vec2::splat(0.5)) * 2.0;
            changed += usize::from((ka.class(p) > 0) != (kb.class(p) > 0));
        }
    }
    assert!(
        changed < 15,
        "integer count crossing changed {changed} texels"
    );
}
