//! Artist controls and canonical recipe defaults.
use super::*;

crate::parameters::parameter_block! {
    pub struct Parameters {
        pores: crate::stamps::Parameters = crate::stamps::Parameters { cells: [113, 109], radius: [0.32, 0.28], density: 0.14, depth: 0.022, size_variation: 0.75, cluster_strength: 0.85, ..Default::default() };
        drag_marks: crate::stamps::Parameters = crate::stamps::Parameters { cells: [83, 41], radius: [0.12, 0.75], density: 0.3, depth: 0.018, angle_variation: 0.15, roundness: 0.45, ..Default::default() };
        palette: [[u8; 3]; 5] = [[139,58,41],[150,68,45],[129,55,40],[144,63,42],[136,61,46]];
        generate_clay_roof_tile_textures_ao: f32 = 0.32;
        color_and_roughness_roughness_1: f32 = 210.0;
        color_and_roughness_roughness_2: f32 = 7.0;
        color_and_roughness_roughness_3: f32 = 4.0;
        color_and_roughness_roughness_4: f32 = 11.0;
        sample_tiles_vertical_offset: f32 = 0.070;
        sample_tiles_yaw: f32 = 0.070;
        sample_tiles_tail_asymmetry: f32 = 0.12;
        sample_tiles_side_warp: f32 = 0.007;
        edge_width_metres: f32 = 0.006;
        sample_tiles_antialias: f32 = 0.75;
        sample_tiles_thickness: f32 = 0.024;
        sample_tiles_cup_1: f32 = 0.16;
        sample_tiles_cup_2: f32 = 0.35;
        sample_tiles_cup_3: f32 = 0.026;
        sample_tiles_twist: f32 = 0.030;
        sample_tiles_lip_1: f32 = 0.54;
        sample_tiles_lip_2: f32 = 0.19;
        sample_tiles_lip: f32 = 0.050;
        sample_tiles_face_height_1: f32 = 0.60;
        sample_tiles_face_height_2: f32 = 0.038;
        sample_tiles_face_height_3: f32 = 0.004;
        sample_tiles_under_variation: f32 = 0.12;
        sample_tiles_under_height_1: f32 = 0.555;
        sample_tiles_under_height_2: f32 = 0.004;
        sample_tiles_edge_proximity: f32 = 0.085;
        sample_tiles_lower_lip_contact_1: f32 = 0.58;
        sample_tiles_lower_lip_contact_2: f32 = 0.12;
        sample_tiles_lower_lip_contact_3: f32 = 0.94;
        sample_tiles_lower_lip_contact_4: f32 = 0.16;
        sample_tiles_contact_1: f32 = 0.24;
        sample_tiles_contact_2: f32 = 0.20;
        sample_tiles_contact_3: f32 = 0.05;
        sample_tiles_wear_segment: f32 = 7.0;
        sample_tiles_edge_wear_1: f32 = 0.78;
        sample_tiles_edge_wear_2: f32 = 0.22;
        sample_tiles_firing_cluster: f32 = 0.28;
        sample_tiles_firing: f32 = 0.38;
        face_variation_broad_1: f32 = 2.1;
        face_variation_broad_2: f32 = 0.8;
        face_variation_fine_1: f32 = 7.0;
        face_variation_fine_2: f32 = 1.7;
        tail_side_width_hand_width_1: f32 = 0.89;
        tail_side_width_hand_width_2: f32 = 0.095;
        tail_side_width_tail_start: f32 = 0.070;
        tail_side_width_roundness_1: f32 = 1.65;
        tail_side_width_roundness_2: f32 = 0.75;
        tile_metres: f32 = CLAY_ROOF_TILE_TILE_METRES;
        height_range_metres: f32 = CLAY_ROOF_TILE_HEIGHT_RANGE_METRES;
        courses: i32 = COURSES;
        tiles_per_course: i32 = TILES_PER_COURSE;
        tail_start: f32 = TAIL_START;
    }
}
