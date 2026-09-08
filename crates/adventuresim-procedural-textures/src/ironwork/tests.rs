use super::*;
use crate::{BakeResolution, BakedRecipe, MapChannel, TextureParameters, TextureRecipeId};

#[test]
fn forged_field_is_periodic_including_changed_controls_and_seeds() {
    for seed in [0, 31, 918] {
        let mut p = TextureParameters {
            seed,
            ..Default::default()
        };
        p.ironwork.hammer_cells = [9, 23];
        for i in 0..200 {
            let u = (i as f32 + 0.5) / 200.0;
            let v = u * 0.713;
            let a = field::sample(&p, u, v);
            for (du, dv) in [(1.0, 0.0), (0.0, 1.0), (-1.0, -1.0)] {
                let b = field::sample(&p, u + du, v + dv);
                assert!((a.height - b.height).abs() < 0.0001);
                assert!((a.cavity - b.cavity).abs() < 0.001);
            }
        }
    }
}

#[test]
fn impressions_and_scale_have_separate_relief_and_response() {
    let p = TextureParameters::default();
    let mut clean = p.clone();
    clean.ironwork.scale_depth = 0.0;
    clean.ironwork.pit_depth = 0.0;
    let mut facets = 0;
    let mut cavities = 0;
    for y in 0..100 {
        for x in 0..100 {
            let u = (x as f32 + 0.5) / 100.0;
            let v = (y as f32 + 0.5) / 100.0;
            let a = field::sample(&p, u, v);
            let b = field::sample(&clean, u, v);
            assert!(a.height <= b.height + f32::EPSILON);
            assert!((0.0..=1.0).contains(&a.height));
            if a.cavity > 0.5 {
                cavities += 1;
                assert!(a.height < b.height);
                assert!(response(&p, a, 0.0)[0] < 240);
            }
            if a.crown > 0.8 {
                facets += 1;
            }
        }
    }
    assert!(
        cavities > 100 && cavities < 2500,
        "cavity coverage {cavities}/10000"
    );
    assert!(facets > 300, "hammer rims {facets}/10000");
    let impression_metres = p.ironwork.tile_metres / p.ironwork.hammer_cells[0] as f32;
    assert!((0.025..=0.08).contains(&impression_metres));
}

#[test]
fn categorical_palette_and_metallicity_agree_and_mips_are_complete() {
    let mut p = TextureParameters {
        resolution: BakeResolution::Draft,
        ..Default::default()
    };
    p.ironwork.oxide_fraction = 0.58;
    let a = BakedRecipe::generate(TextureRecipeId::Ironwork, &p);
    let b = BakedRecipe::generate(TextureRecipeId::Ironwork, &p);
    assert_eq!(a.to_bytes(), b.to_bytes());
    for map in &a.maps {
        assert_eq!(map.mip_levels, map.size.ilog2() + 1);
        let pixels: usize = (0..map.mip_levels)
            .map(|l| (map.size >> l).pow(2) as usize)
            .sum();
        assert_eq!(map.bytes.len(), pixels * map.encoding.channels());
    }
    let color = a.map(MapChannel::Albedo).unwrap();
    let arm = a.map(MapChannel::Arm).unwrap();
    let mut endpoints = 0;
    for (rgb, response) in color
        .bytes
        .chunks_exact(4)
        .zip(arm.bytes.chunks_exact(4))
        .take(color.size.pow(2) as usize)
    {
        if rgb[..3] == p.ironwork.bare_srgb.0 {
            assert_eq!(response[2], 255);
            endpoints += 1;
        } else if rgb[..3] == p.ironwork.oxide_srgb.0 {
            assert_eq!(response[2], 0);
            endpoints += 1;
        }
    }
    assert!(endpoints as f32 / color.size.pow(2) as f32 > 0.96);
}
