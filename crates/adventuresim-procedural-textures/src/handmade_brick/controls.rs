//! Artist controls and canonical recipe defaults.
use super::*;

crate::parameters::parameter_block! {
    pub struct Parameters {
        pores: crate::stamps::Parameters = crate::stamps::Parameters { cells: [151,137], radius: [0.22,0.19], density:0.28, depth:0.017, ..Default::default() };
        mortar_grit: crate::stamps::Parameters = crate::stamps::Parameters { cells: [191,179], radius: [0.26,0.24], density:0.6, depth:0.025, roundness:0.2, ..Default::default() };
        sample_brickwork_antialias: f32 = 0.7;
        sample_brickwork_broad_cup_1: f32 = 0.33;
        sample_brickwork_broad_cup_2: f32 = 0.33;
        sample_brickwork_mortar_noise_1: f32 = 5.0;
        sample_brickwork_mortar_noise_2: f32 = 7.0;
        edge_chip_center_1: f32 = 1.5;
        edge_chip_center_2: f32 = 0.75;
        tile_metres: f32 = HANDMADE_BRICK_TILE_METRES;
        height_range_metres: f32 = HANDMADE_BRICK_HEIGHT_RANGE_METRES;
        colors: MasonryColors<5> = HANDMADE_BRICK_COLORS;
        brick_roughness: u8 = BRICK_ROUGHNESS;
        mortar_roughness: u8 = MORTAR_ROUGHNESS;
        courses: i32 = COURSES;
        bricks_per_course: i32 = BRICKS_PER_COURSE;
        horizontal_mortar_metres: f32 = HORIZONTAL_MORTAR_METRES;
        vertical_mortar_metres: f32 = VERTICAL_MORTAR_METRES;
        occlusion_strength: f32 = 2.8;
        minimum_visibility: f32 = 0.48;
        horizontal_jitter: f32 = 0.055;
        vertical_jitter: f32 = 0.045;
        width_minimum: f32 = 0.94;
        width_variation: f32 = 0.10;
        height_minimum: f32 = 0.92;
        height_variation: f32 = 0.13;
        cupping: f32 = 0.024;
        twist: f32 = 0.018;
        cup_aspect: f32 = 0.65;
        face_height: f32 = 0.73;
        face_noise_relief: f32 = 0.007;
        mortar_noise_relief: f32 = 0.008;
        mortar_height: f32 = 0.19;
        chip_absence_probability: f32 = 0.82;
        chip_half_width: f32 = 0.07;
        chip_width_variation: f32 = 0.10;
        chip_depth: f32 = 0.035;
        chip_depth_variation: f32 = 0.055;
        edge_bow_min: f32 = 0.004;
        edge_bow_variation: f32 = 0.008;
        broad_cross_frequency: f32 = 1.7;
        broad_long_frequency: f32 = 1.1;
        fine_cross_frequency: f32 = 3.1;
        fine_long_frequency: f32 = 2.3;
        broad_weight: f32 = 0.72;
        fine_weight: f32 = 0.28;
    }
}
