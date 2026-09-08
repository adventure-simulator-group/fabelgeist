//! Artist controls and canonical recipe defaults.
use super::*;

crate::parameters::parameter_block! {
    pub struct Parameters {
        generate_white_oak_leaf_textures_ao_1: f32 = 232.0;
        generate_white_oak_leaf_textures_ao_2: f32 = 32.0;
        generate_white_oak_leaf_textures_ao_3: f32 = 248.0;
        generate_white_oak_leaf_textures_ao_4: f32 = 220.0;
        generate_white_oak_leaf_textures_ao_5: f32 = 34.0;
        generate_white_oak_leaf_textures_ao_6: f32 = 1.5;
        generate_white_oak_leaf_textures_ao_7: f32 = 216.0;
        generate_white_oak_leaf_textures_ao_8: f32 = 242.0;
        white_oak_sample_t_1: f32 = 0.075;
        white_oak_sample_t_2: f32 = 0.85;
        white_oak_sample_axis_1: f32 = 0.026;
        white_oak_sample_axis_2: f32 = 0.42;
        white_oak_sample_axis_3: f32 = 0.004;
        white_oak_sample_petiole: f32 = 0.008;
        white_oak_sample_midrib_width_1: f32 = 0.0035;
        white_oak_sample_midrib_width_2: f32 = 0.003;
        white_oak_sample_fork_progress_1: f32 = 0.48;
        white_oak_sample_fork_progress_2: f32 = 0.42;
        white_oak_sample_fork_x: f32 = 0.024;
        white_oak_sample_transverse: f32 = 0.001;
        white_oak_sample_blade_dome_1: f32 = 0.72;
        white_oak_sample_blade_dome_2: f32 = 0.17;
        white_oak_sample_vein_ridge_1: f32 = 0.012;
        white_oak_sample_vein_ridge_2: f32 = 0.10;
        white_oak_sample_height_1: f32 = 0.10;
        white_oak_sample_height_2: f32 = 0.004;
        white_oak_sample_height_3: f32 = 0.015;
        white_oak_sample_height_4: f32 = 0.32;
        white_oak_vein_x_eased: f32 = 0.78;
        white_oak_vein_x_inward_curve: f32 = 0.010;
        white_oak_tissue_mottle_broad_1: f32 = 8.0;
        white_oak_tissue_mottle_broad_2: f32 = 5.0;
        white_oak_tissue_mottle_broad_3: f32 = 0.7;
        white_oak_tissue_mottle_cross_1: f32 = 5.0;
        white_oak_tissue_mottle_cross_2: f32 = 9.0;
        white_oak_tissue_mottle_cross_3: f32 = 1.9;
        white_oak_tissue_mottle_fine_1: f32 = 19.0;
        white_oak_tissue_mottle_fine_2: f32 = 17.0;
        white_oak_tissue_mottle_fine_3: f32 = 0.3;
        white_oak_side_width_longitudinal_envelope: f32 = 0.58;
        white_oak_side_width_middle_breadth_1: f32 = 0.050;
        white_oak_side_width_middle_breadth_2: f32 = 0.024;
        white_oak_side_width_radius: f32 = 0.78;
        white_oak_side_width_profile: f32 = 0.72;
        white_oak_left_lobes: [WhiteOakLobe; 5] = WHITE_OAK_LEFT_LOBES;
        white_oak_right_lobes: [WhiteOakLobe; 5] = WHITE_OAK_RIGHT_LOBES;
    }
}
