//! Artist controls and canonical recipe defaults.
use super::*;

crate::parameters::parameter_block! {
    pub struct Parameters {
        edge_width_variation: f32 = 0.4;
        edge_irregularity: f32 = 0.35;
        edge_width_metres: f32 = 0.012;
        course_warp: f32 = 0.012;
        fracture_count: i32 = 3;
        fracture_depth: f32 = 0.24;
        fracture_offset: f32 = 0.02;
        fracture_offset_variation: f32 = 0.45;
        pores: crate::stamps::Parameters = crate::stamps::Parameters { cells: [109, 97], radius: [0.34, 0.25], density: 0.24, depth: 0.025, ..Default::default() };
        mortar_grit: crate::stamps::Parameters = crate::stamps::Parameters { cells: [151, 137], radius: [0.35, 0.32], density: 0.7, depth: 0.015, ..Default::default() };
        colors: crate::MasonryColors<8> = crate::MasonryColors { units: [[99,98,89],[119,108,88],[88,91,86],[128,116,94],[105,99,82],[113,104,86],[93,96,91],[123,111,91]].map(crate::SrgbColor), mortar: crate::SrgbColor([145,142,133]) };
        stone_color_near_joint_1: f32 = 18.0;
        stone_color_near_joint_2: f32 = 7.0;
        stone_color_roughness_1: f32 = 221.0;
        stone_color_roughness_2: f32 = 7.0;
        sample_masonry_previous_interlocks_1: f32 = 0.91;
        sample_masonry_previous_interlocks_2: f32 = 0.72;
        sample_masonry_interlocks: f32 = 0.91;
        sample_masonry_joint_metres_1: f32 = 0.006;
        sample_masonry_joint_metres_2: f32 = 0.010;
        sample_masonry_contact_variation_x_1: f32 = 0.68;
        sample_masonry_contact_variation_x_2: f32 = 0.32;
        sample_masonry_contact_variation_x_3: f32 = 6.0;
        sample_masonry_contact_variation_y_1: f32 = 0.68;
        sample_masonry_contact_variation_y_2: f32 = 0.32;
        sample_masonry_contact_variation_y_3: f32 = 6.0;
        sample_masonry_top_extent: f32 = 1.38;
        sample_masonry_corner_cut_1: f32 = 1.12;
        sample_masonry_corner_cut_2: f32 = 1.10;
        sample_masonry_antialias: f32 = 0.8;
        sample_masonry_planar_tilt_1: f32 = 0.14;
        sample_masonry_planar_tilt_2: f32 = 0.10;
        sample_masonry_face_height_1: f32 = 0.70;
        sample_masonry_face_height_2: f32 = 0.16;
        sample_masonry_mortar_variation_1: f32 = 11.0;
        sample_masonry_mortar_variation_2: f32 = 7.0;
        sample_masonry_mortar_variation_3: f32 = 0.014;
        sample_masonry_mortar_height_1: f32 = 0.17;
        sample_masonry_mortar_height_2: f32 = 0.035;
        row_at_weight_1: f32 = 0.65;
        row_at_weight_2: f32 = 0.25;
        tile_metres: f32 = RUBBLE_MASONRY_TILE_METRES;
        height_range_metres: f32 = RUBBLE_MASONRY_HEIGHT_RANGE_METRES;
        rows: i32 = ROWS;
        min_stones_per_row: usize = 15;
        max_stones_per_row: usize = 21;
    }
}
