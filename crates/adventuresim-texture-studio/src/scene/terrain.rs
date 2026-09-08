//! Material-only interpretation of the terrain's packed height, AO, tone and coverage maps.
use super::*;
use adventuresim_procedural_textures::{BakedMap, PixelEncoding};

pub(super) fn litter(
    world: &mut World,
    assets: &mut SceneAssets,
    material: &mut StandardMaterial,
    map: &BakedMap,
    document: &Document,
) {
    let soil = Color::srgb_from_array(document.surface.pigment)
        .to_linear()
        .to_vec4()
        .xyz();
    let soil = soil.lerp(soil * Vec3::new(0.38, 0.40, 0.37), 0.76);
    let [dark, mid, pale] = document.surface.litter_colors_linear.map(Vec3::from_array);
    let mut color = map.clone();
    color.encoding = PixelEncoding::Srgb8;
    color.bytes.clear();
    let mut arm = map.clone();
    arm.encoding = PixelEncoding::Rgba8;
    arm.bytes.clear();
    for pixel in map.bytes.as_chunks::<4>().0 {
        let tone = pixel[2] as f32 / 255.0;
        let coverage = pixel[3] as f32 / 255.0;
        let leaf = dark
            .lerp(mid, smooth(0.05, 0.58, tone))
            .lerp(pale, smooth(0.62, 0.96, tone));
        let rgb = soil.lerp(leaf, coverage * 0.92);
        let srgb = Color::linear_rgb(rgb.x, rgb.y, rgb.z).to_srgba();
        color.bytes.extend_from_slice(&[
            (srgb.red * 255.0).round() as u8,
            (srgb.green * 255.0).round() as u8,
            (srgb.blue * 255.0).round() as u8,
            255,
        ]);
        let ao =
            1.0 - (1.0 - pixel[1] as f32 / 255.0) * coverage * 0.88 * document.surface.ao_strength;
        arm.bytes.extend_from_slice(&[
            (ao * 255.0).round() as u8,
            ((0.84 + coverage * 0.055) * 255.0).round() as u8,
            0,
            255,
        ]);
    }
    material.base_color_texture = Some(maps::upload(world, assets, &color, false));
    let arm = maps::upload(world, assets, &arm, false);
    material.occlusion_texture = Some(arm.clone());
    material.metallic_roughness_texture = Some(arm);
    material.metallic = 0.0;
}
fn smooth(low: f32, high: f32, value: f32) -> f32 {
    let t = ((value - low) / (high - low)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
