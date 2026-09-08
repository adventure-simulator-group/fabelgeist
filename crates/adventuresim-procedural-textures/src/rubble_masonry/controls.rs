//! Artist controls and canonical recipe defaults.
use super::*;

crate::parameters::parameter_block! {
    pub struct Parameters {
        stone_color_shade: f32 = 0.55;
        stone_color_face_shift_1: f32 = 0.70;
        stone_color_face_shift_2: f32 = 25.0;
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
        sample_masonry_corner_cut_1: f32 = 1.16;
        sample_masonry_corner_cut_2: f32 = 0.36;
        sample_masonry_corner_bias: f32 = 0.22;
        sample_masonry_antialias: f32 = 0.8;
        sample_masonry_face_variation_1: f32 = 1.35;
        sample_masonry_face_variation_2: f32 = 0.77;
        sample_masonry_face_variation_3: f32 = 0.006;
        sample_masonry_face_variation_4: f32 = 2.7;
        sample_masonry_face_variation_5: f32 = 1.9;
        sample_masonry_face_variation_6: f32 = 0.63;
        sample_masonry_face_variation_7: f32 = 0.003;
        sample_masonry_planar_tilt_1: f32 = 0.025;
        sample_masonry_planar_tilt_2: f32 = 0.018;
        sample_masonry_face_height_1: f32 = 0.70;
        sample_masonry_face_height_2: f32 = 0.10;
        sample_masonry_mortar_variation_1: f32 = 11.0;
        sample_masonry_mortar_variation_2: f32 = 7.0;
        sample_masonry_mortar_variation_3: f32 = 0.014;
        sample_masonry_mortar_height_1: f32 = 0.17;
        sample_masonry_mortar_height_2: f32 = 0.035;
        row_pitch_total_1: f32 = 0.84;
        row_pitch_total_2: f32 = 0.32;
        row_at_weight_1: f32 = 0.84;
        row_at_weight_2: f32 = 0.32;
        tile_metres: f32 = RUBBLE_MASONRY_TILE_METRES;
        height_range_metres: f32 = RUBBLE_MASONRY_HEIGHT_RANGE_METRES;
        rows: i32 = ROWS;
        min_stones_per_row: usize = 15;
        max_stones_per_row: usize = 21;
    }
}
