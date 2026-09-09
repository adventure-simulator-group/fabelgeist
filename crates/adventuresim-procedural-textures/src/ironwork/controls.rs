//! Artist controls for forged impressions, oxide islands and scale loss.
use super::*;

crate::parameters::parameter_block! {
    pub struct Parameters {
        tile_metres: f32 = IRONWORK_TILE_METRES;
        height_range_metres: f32 = IRONWORK_HEIGHT_RANGE_METRES;
        hammer_cells: [i32; 2] = [8, 12];
        hammer_size_variation: f32 = 0.55;
        hammer_depth_variation: f32 = 0.55;
        hammer_jitter: f32 = 0.85;
        hammer_radius: f32 = 0.70;
        hammer_angle: f32 = 1.3;
        hammer_depth: f32 = 0.28;
        hammer_tilt: f32 = 0.07;
        hammer_rim: f32 = 0.012;
        hammer_roundness: f32 = 0.72;
        body_cells: [i32; 2] = [4, 5];
        body_relief: f32 = 0.035;
        grain_cells: [i32; 2] = [157, 173];
        grain_relief: f32 = 0.004;
        scale_angle: f32 = std::f32::consts::PI;
        scale_edge_breakup: f32 = 0.18;
        scale_cells: [i32; 2] = [38, 43];
        scale_density: f32 = 0.26;
        scale_radius: f32 = 0.34;
        scale_depth: f32 = 0.045;
        scale_edge_width: f32 = 0.12;
        pit_cells: [i32; 2] = [83, 79];
        pit_density: f32 = 0.10;
        pit_radius: f32 = 0.28;
        pit_depth: f32 = 0.028;
        oxide_cells: [i32; 2] = [5, 4];
        oxide_fraction: f32 = 1.0;
        polish_fraction: f32 = 0.0;
        bare_srgb: crate::SrgbColor = crate::SrgbColor([150, 155, 160]);
        oxide_srgb: crate::SrgbColor = crate::SrgbColor([39, 41, 44]);
        bare_roughness: f32 = 0.62;
        oxide_roughness: f32 = 0.67;
        polish_roughness: f32 = 0.40;
        cavity_roughness: f32 = 0.87;
        cavity_occlusion: f32 = 0.22;
    }
}
