//! Artist controls and canonical recipe defaults.
use super::*;

crate::parameters::parameter_block! {
    pub struct Parameters {
        cleft: super::cleft::Parameters = super::cleft::Parameters::default();
        generate_slate_roof_textures_ao: f32 = 0.34;
        color_and_roughness_cool_shift: f32 = 5.0;
        color_and_roughness_color_1: f32 = 52.0;
        color_and_roughness_color_2: f32 = 6.0;
        color_and_roughness_color_3: f32 = 59.0;
        color_and_roughness_color_4: f32 = 7.0;
        color_and_roughness_color_5: f32 = 66.0;
        color_and_roughness_color_6: f32 = 8.0;
        color_and_roughness_roughness_1: f32 = 175.0;
        color_and_roughness_roughness_2: f32 = 10.0;
        color_and_roughness_roughness_3: f32 = 9.0;
        color_and_roughness_roughness_4: f32 = 5.0;
        sample_slate_left_boundary: f32 = 0.150;
        sample_slate_right_boundary: f32 = 0.150;
        sample_slate_side_joint: f32 = 0.040;
        sample_slate_active_phase: f32 = 0.48;
        sample_slate_piece_thickness: f32 = 0.012;
        sample_slate_plane_tilt_1: f32 = 0.015;
        sample_slate_plane_tilt_2: f32 = 0.012;
        sample_slate_lip_1: f32 = 0.10;
        sample_slate_lip_2: f32 = 0.10;
        sample_slate_lip: f32 = 0.10;
        sample_slate_base_1: f32 = 0.604;
        sample_slate_base_2: f32 = 0.37;
        sample_slate_lip_contact: f32 = 0.060;
        sample_slate_contact_1: f32 = 0.70;
        sample_slate_contact_2: f32 = 0.82;
        sample_slate_edge_band_1: f32 = 0.040;
        sample_slate_edge_band_2: f32 = 0.030;
        sample_slate_wear_cell: f32 = 9.0;
        sample_slate_edge_wear_1: f32 = 0.72;
        sample_slate_edge_wear_2: f32 = 0.28;
        lower_edge_heel_bias: f32 = 0.080;
        lower_edge_left_clip_1: f32 = 0.25;
        lower_edge_left_clip_2: f32 = 0.20;
        lower_edge_right_clip_1: f32 = 0.29;
        lower_edge_right_clip_2: f32 = 0.16;
        lower_edge_asymmetry_1: f32 = 0.045;
        lower_edge_asymmetry_2: f32 = 0.050;
        lower_edge_asymmetry_3: f32 = 0.025;
        lower_edge_asymmetry_4: f32 = 0.045;
        lower_edge_chip_segment: f32 = 7.0;
        lower_edge_chip_1: f32 = 0.86;
        lower_edge_chip_2: f32 = 0.14;
        lower_edge_chip_3: f32 = 0.024;
        piece_coordinates_stagger: f32 = 0.82;
        tile_metres: f32 = SLATE_ROOF_TILE_METRES;
        height_range_metres: f32 = SLATE_ROOF_HEIGHT_RANGE_METRES;
        courses: i32 = COURSES;
        pieces_per_course: i32 = PIECES_PER_COURSE;
        course_rise_per_repeat: i32 = COURSE_RISE_PER_REPEAT;
        course_face_end: f32 = COURSE_FACE_END;
    }
}
