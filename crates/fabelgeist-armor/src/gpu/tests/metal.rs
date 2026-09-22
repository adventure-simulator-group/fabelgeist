//! Metal maps baked on the device.

use super::gpu;
use crate::engraving::{
    Cut, Engraving, Relief,
    tests::{groove_png, png},
};
use crate::gpu::metal::bake;
use crate::material::{Metal, MetalTextures};

const FLAT_NORMAL: [u8; 4] = [127, 127, 255, 255];

/// A flawless metal, so that only the engraving shapes the maps.
fn polished() -> Metal {
    Metal {
        scratch_density: 0,
        waviness: 0.0,
        grain: 0.0,
        smudge: 0.0,
        ..Metal::default()
    }
}

/// Bake `metal` with `engraving` decoded from `image`.
fn engraved(metal: &Metal, engraving: &Engraving, image: &[u8], size: u32) -> MetalTextures {
    let image = Cut::Image(engraving.decode(image).unwrap());
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
fn the_finish_tilts_and_roughens_the_surface_and_still_tiles() {
    let size = 64;
    let finished = Metal {
        waviness: Metal::MAX_WAVINESS,
        grain: 1.0,
        smudge: 0.3,
        ..polished()
    };
    let a = gpu().textures(&finished, size).unwrap();
    assert_eq!(a.normal, gpu().textures(&finished, size).unwrap().normal);
    let tilted = a
        .normal
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| **p != FLAT_NORMAL);
    assert!(tilted.count() > (size * size / 2) as usize);
    let roughness = |x, y| i32::from(pixel(&a.metal_roughness, size, x, y)[1]);
    let values = (0..size).flat_map(|y| (0..size).map(move |x| (x, y)));
    let (low, high) = values.fold((255, 0), |(low, high), (x, y)| {
        (low.min(roughness(x, y)), high.max(roughness(x, y)))
    });
    assert!(high - low > 25, "gloss varies: {low}..{high}");
    // Opposite edges of the tile continue each other: neighbours across the
    // seam differ no more than neighbours inside it.
    let step = |x0, x1| (roughness(x0, 7) - roughness(x1, 7)).abs();
    let inside = (1..size).map(|x| step(x - 1, x)).max().unwrap();
    assert!(step(size - 1, 0) <= inside + 2);
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

/// The share of each depth row that is cut, top to bottom, for `ornament`
/// filling the tile once.
fn ornament_rows(ornament: crate::ornament::Ornament, size: u32) -> (MetalTextures, Vec<f32>) {
    let engraving = Engraving::ornament(ornament);
    let baked = gpu()
        .textures(
            &Metal {
                engraving: Some(engraving),
                ..polished()
            },
            size,
        )
        .unwrap();
    let depth = baked.depth.as_ref().expect("an ornament is cut as heights");
    let rows = (0..size)
        .map(|y| {
            (0..size)
                .map(|x| f32::from(pixel(&depth.pixels, size, x, y)[0]) / 255.0)
                .sum::<f32>()
                / size as f32
        })
        .collect();
    (baked, rows)
}

#[test]
fn every_ornament_motif_cuts_part_of_its_cell() {
    use crate::ornament::{Motif, Ornament};
    let size = 128;
    for motif in Motif::ALL {
        let ornament = Ornament {
            motif,
            fillets: false,
            ..Ornament::default()
        };
        let (a, rows) = ornament_rows(ornament.clone(), size);
        let (b, _) = ornament_rows(ornament, size);
        assert_eq!(a.normal, b.normal, "{motif:?} is deterministic");
        let cut = rows.iter().sum::<f32>() / size as f32;
        assert!((0.03..0.6).contains(&cut), "{motif:?} cuts {cut}");
        // Without fillets the cell's long sides stay untouched.
        assert!(
            rows[0] < 0.01 && rows[size as usize - 1] < 0.01,
            "{motif:?}"
        );
    }
}

#[test]
fn fillets_run_along_both_sides_of_the_cell() {
    use crate::ornament::{Motif, Ornament};
    let size = 128;
    let ornament = Ornament {
        motif: Motif::Beads { radius: 0.3 },
        line: 0.06,
        ..Ornament::default()
    };
    let (_, rows) = ornament_rows(ornament, size);
    // Fillets centred one line in from each side are cut the whole way along.
    let at = (0.06 * size as f32) as usize;
    assert!(
        rows[at] > 0.95 && rows[size as usize - 1 - at] > 0.95,
        "{rows:?}"
    );
    // Between a fillet and the beads the surface is untouched.
    assert!(rows[(0.16 * size as f32) as usize] < 0.05, "{rows:?}");
}
