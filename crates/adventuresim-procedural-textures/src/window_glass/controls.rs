//! Artist controls and canonical recipe defaults.
use super::*;

crate::parameters::parameter_block! {
    pub struct Parameters {
        transmitted_color: [u8; 3] = [214,226,217];
        absorption_variation: f32 = 3.0;
        optical_normal_gain: f32 = 14.0;
        tile_metres: f32 = WINDOW_GLASS_TILE_METRES;
        thickness_variation_metres: f32 = WINDOW_GLASS_THICKNESS_VARIATION_METRES;
        sample_glass_absorption: f32 = 4.0;
        material_contract: WindowGlassMaterialContract = WINDOW_GLASS_MATERIAL_CONTRACT;
        patches: Vec<(f32, f32, f32, f32, f32, f32)> = PATCHES.to_vec();
        bubbles: Vec<(f32, f32, f32, f32)> = BUBBLES.to_vec();
        broad_relief: f32 = 0.15;
        bubble_relief: f32 = 0.035;
        broad_thickness: f32 = 0.17;
        striation_thickness: f32 = 0.60;
        bubble_thickness: f32 = 0.22;
        base_roughness: f32 = 0.11;
        bubble_roughness: f32 = 0.055;
        minimum_roughness: f32 = 0.08;
        maximum_roughness: f32 = 0.24;
        striation_bend: f32 = 0.27;
        horizontal_warp: f32 = 0.11;
        vertical_warp: f32 = 0.16;
        horizontal_weight: f32 = 0.42;
        vertical_weight: f32 = 0.31;
        cross_weight: f32 = 0.12;
        diagonal_weight: f32 = 0.08;
    }
}
