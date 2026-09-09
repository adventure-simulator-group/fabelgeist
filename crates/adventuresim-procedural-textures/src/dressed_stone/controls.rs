//! Artist controls and canonical recipe defaults.
use super::*;

crate::parameters::parameter_block! {
    pub struct Parameters {
        tool_relief_gain: f32 = 2.5;
        sample_stonework_bed_joint_1: f32 = 0.010;
        sample_stonework_bed_joint_2: f32 = 0.004;
        sample_stonework_head_joint_1: f32 = 0.008;
        sample_stonework_head_joint_2: f32 = 0.005;
        sample_stonework_horizontal_wobble: f32 = 0.008;
        sample_stonework_vertical_wobble: f32 = 0.010;
        sample_stonework_antialias: f32 = 0.8;
        sample_stonework_planar_tilt_1: f32 = 0.030;
        sample_stonework_planar_tilt_2: f32 = 0.022;
        sample_stonework_broad: f32 = 0.005;
        sample_stonework_face_height: f32 = 0.71;
        tool_marks_base_angle_1: f32 = 0.30;
        tool_marks_base_angle_2: f32 = 1.90;
        tool_marks_center_x_1: f32 = 1.30;
        tool_marks_center_x_2: f32 = 0.65;
        tool_marks_center_y_1: f32 = 1.30;
        tool_marks_center_y_2: f32 = 0.65;
        tool_marks_angle: f32 = 0.54;
        tool_marks_half_length_1: f32 = 0.11;
        tool_marks_half_length_2: f32 = 0.30;
        tool_marks_width_1: f32 = 0.026;
        tool_marks_width_2: f32 = 0.036;
        tool_marks_taper: f32 = 0.28;
        tool_marks_gap_radius_1: f32 = 0.05;
        tool_marks_gap_radius_2: f32 = 0.10;
        tool_marks_interruption: f32 = 0.35;
        course_offset_alternating: f32 = 0.47;
        course_weights_weight_1: f32 = 0.91;
        course_weights_weight_2: f32 = 0.18;
        tile_metres: f32 = DRESSED_STONE_TILE_METRES;
        height_range_metres: f32 = DRESSED_STONE_HEIGHT_RANGE_METRES;
        courses: i32 = COURSES;
        max_blocks_per_course: usize = MAX_BLOCKS_PER_COURSE;
        min_blocks_per_course: usize = MIN_BLOCKS_PER_COURSE;
        bevel_relief: f32 = BEVEL_RELIEF;
        pore_relief: f32 = PORE_RELIEF;
        grain_relief: f32 = GRAIN_RELIEF;
        spall_relief: f32 = SPALL_RELIEF;
        face_relief: f32 = FACE_RELIEF;
        block_height_variation: f32 = BLOCK_HEIGHT_VARIATION;
        colors: MasonryColors<6> = DRESSED_STONE_COLORS;
        stone_roughness_palette: [u8; 3] = STONE_ROUGHNESS_PALETTE;
        mortar_roughness: u8 = MORTAR_ROUGHNESS;
    }
}
