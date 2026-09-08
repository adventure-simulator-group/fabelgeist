use super::*;

#[test]
fn finite_stamps_repeat_with_negative_coordinates_and_vanish_when_disabled() {
    let params = TextureParameters::default();
    let mut layer = Parameters {
        density: 1.0,
        ..Default::default()
    };
    let mut total = 0.0;
    for y in 0..32 {
        for x in 0..32 {
            let uv = Vec2::new(x as f32 / 32.0, y as f32 / 32.0);
            let sample = layer.sample(&params, uv, 0x1234);
            for offset in [Vec2::X, -Vec2::Y, Vec2::ONE] {
                let repeated = layer.sample(&params, uv + offset, 0x1234);
                assert_eq!(sample.bowl, repeated.bowl);
                assert_eq!(sample.facet, repeated.facet);
            }
            assert!((0.0..=layer.depth).contains(&sample.bowl));
            total += sample.bowl;
        }
    }
    assert!(total > 1.0);
    layer.density = 0.0;
    assert_eq!(layer.sample(&params, Vec2::splat(0.2), 0x1234).bowl, 0.0);
}

#[test]
fn changing_the_seed_repositions_features_instead_of_only_recoloring_them() {
    let a = TextureParameters::default();
    let b = TextureParameters {
        seed: 47,
        ..Default::default()
    };
    let layer = Parameters::default();
    let difference = (0..256)
        .map(|i| {
            let uv = Vec2::new((i % 16) as f32 / 16.0, (i / 16) as f32 / 16.0);
            (layer.sample(&a, uv, 12).bowl - layer.sample(&b, uv, 12).bowl).abs()
        })
        .sum::<f32>();
    assert!(difference > 0.1);
}
