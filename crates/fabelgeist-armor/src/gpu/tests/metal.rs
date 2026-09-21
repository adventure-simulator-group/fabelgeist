//! Metal maps baked on the device.

use super::gpu;
use crate::engraving::{
    Engraving, Relief,
    tests::{groove_png, png},
};
use crate::gpu::metal::bake;
use crate::material::{Metal, MetalTextures};

const FLAT_NORMAL: [u8; 4] = [127, 127, 255, 255];

/// An unscratched metal, so that only the engraving shapes the maps.
fn polished() -> Metal {
    Metal {
        scratch_density: 0,
        ..Metal::default()
    }
}

/// Bake `metal` with `engraving` decoded from `image`.
fn engraved(metal: &Metal, engraving: &Engraving, image: &[u8], size: u32) -> MetalTextures {
    let image = engraving.decode(image).unwrap();
    bake(gpu(), metal, size, Some((engraving, &image))).unwrap()
}

fn pixel(map: &[u8], size: u32, x: u32, y: u32) -> &[u8] {
    let at = ((y * size + x) * 4) as usize;
    &map[at..at + 4]
}

#[test]
fn scratches_are_deterministic_and_change_normal_and_roughness() {
    let mut m = Metal::default();
    let a = gpu().textures(&m, 64).unwrap();
    let b = gpu().textures(&m, 64).unwrap();
    assert_eq!(a.size, 64);
    assert_eq!(a.normal.len(), 64 * 64 * 4);
    assert_eq!(a.metal_roughness.len(), 64 * 64 * 4);
    assert_eq!(a.normal, b.normal);
    assert_eq!(a.metal_roughness, b.metal_roughness);
    assert!(a.depth.is_none());
    m.seed += 1;
    assert_ne!(a.normal, gpu().textures(&m, 64).unwrap().normal);
    let smooth = gpu().textures(&polished(), 64).unwrap();
    assert!(
        smooth
            .normal
            .as_chunks::<4>()
            .0
            .iter()
            .all(|p| *p == FLAT_NORMAL)
    );
    assert_ne!(a.normal, smooth.normal);
    assert_ne!(a.metal_roughness, smooth.metal_roughness);
}

#[test]
fn texture_sizes_and_engraving_images_are_checked() {
    let metal = Metal::default();
    assert!(gpu().textures(&metal, 16).is_err());
    assert!(gpu().textures(&metal, u32::MAX).is_err());
    let full = gpu().textures(&metal, Metal::TEXTURE_SIZE).unwrap();
    assert_eq!(full.normal.len(), (Metal::TEXTURE_SIZE.pow(2) * 4) as usize);
    let engraved = Metal {
        engraving: Some(Engraving::new("missing.png")),
        ..Metal::default()
    };
    assert!(gpu().textures(&engraved, 64).is_err());
    let invalid = Metal {
        roughness: f32::NAN,
        ..Metal::default()
    };
    assert!(gpu().textures(&invalid, 64).is_err());
}

#[test]
fn a_groove_cuts_below_the_surface_and_tilts_the_normal_at_its_walls() {
    let size = 64;
    let mut engraving = Engraving::new("groove.png");
    engraving.relief = Relief::Height {
        depth: Relief::MAX_DEPTH,
    };
    let baked = engraved(&polished(), &engraving, &groove_png(), size);
    let normal = |x| pixel(&baked.normal, size, x, 0);
    // Descending into the groove the surface falls to the right: normal
    // tilts right. Climbing out it rises to the right: normal tilts left.
    assert!(
        normal(24)[0] > 140 && normal(40)[0] < 115,
        "{:?} {:?}",
        normal(24),
        normal(40)
    );
    assert_eq!(normal(0), FLAT_NORMAL);
    let depth = baked.depth.expect("a height map has depth");
    assert!((depth.uv_scale - Relief::MAX_DEPTH * Metal::TILES_PER_METRE).abs() < 1e-6);
    let cut = |x| pixel(&depth.pixels, size, x, 0)[0];
    assert!(cut(32) > 250 && cut(0) < 3);
    // The floor of the cut is rougher than the polished surface.
    let roughness = |x| pixel(&baked.metal_roughness, size, x, 0)[1];
    assert!(roughness(32) > roughness(0) + 60);
}

#[test]
fn a_normal_map_tilts_with_its_turn_and_has_no_depth() {
    let size = 32;
    // A normal tilted left encodes a surface rising to the right.
    let image = png(4, 4, |_, _| [64, 128, 255]);
    let mut engraving = Engraving::new("tilt.png");
    engraving.relief = Relief::Normal { strength: 1.0 };
    let slope = -(64.0f32 / 255.0 * 2.0 - 1.0);
    let lean = slope / (slope * slope + 1.0).sqrt();
    let byte = |component: f32| ((component * 0.5 + 0.5) * 255.0) as u8;
    let tilted = byte(-lean);
    let near = |a: u8, b: u8| a.abs_diff(b) <= 2;
    let baked = engraved(&polished(), &engraving, &image, size);
    assert!(baked.depth.is_none());
    assert!(
        baked
            .normal
            .as_chunks::<4>()
            .0
            .iter()
            .all(|p| near(p[0], tilted) && near(p[1], 127))
    );
    engraving.rotation = std::f32::consts::FRAC_PI_2;
    let turned = engraved(&polished(), &engraving, &image, size);
    // Turned a quarter, the surface rises down the image instead.
    let raised = byte(lean);
    assert!(
        turned
            .normal
            .as_chunks::<4>()
            .0
            .iter()
            .all(|p| near(p[0], 127) && near(p[1], raised))
    );
}

#[test]
fn tiling_repeats_the_engraving() {
    let size = 32;
    let mut engraving = Engraving::new("groove.png");
    engraving.tiles = 2.0;
    let baked = engraved(&polished(), &engraving, &groove_png(), size);
    let depth = baked.depth.expect("a height map has depth");
    let row = (0..size)
        .map(|x| pixel(&depth.pixels, size, x, 0)[0])
        .collect::<Vec<_>>();
    let grooves = row.windows(2).filter(|w| w[0] < 128 && w[1] >= 128).count();
    assert_eq!(grooves, 2, "{row:?}");
}
