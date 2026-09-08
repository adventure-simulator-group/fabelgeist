//! Artist controls and canonical recipe defaults.
use super::*;

crate::parameters::parameter_block! {
    pub struct Parameters {
        generate_ao_1: f32 = 233.0;
        generate_ao_2: f32 = 32.0;
        generate_ao_3: f32 = 248.0;
        generate_ao_4: f32 = 222.0;
        generate_ao_5: f32 = 38.0;
        generate_ao_6: f32 = 222.0;
        generate_ao_7: f32 = 244.0;
        sample_t_1: f32 = 0.075;
        sample_t_2: f32 = 0.85;
        sample_petiole: f32 = 0.0085;
        sample_height: f32 = 0.085;
        sample_midrib_width_1: f32 = 0.0065;
        sample_midrib_width_2: f32 = 0.0025;
        sample_link_progress_1: f32 = 0.38;
        sample_link_progress_2: f32 = 0.45;
        sample_link_x: f32 = 0.018;
        sample_transverse: f32 = 0.001;
        sample_blade_dome_1: f32 = 0.76;
        sample_blade_dome_2: f32 = 0.125;
        sample_vein_ridge_1: f32 = 0.012;
        sample_vein_ridge_2: f32 = 0.090;
        sample_height_1: f32 = 0.085;
        sample_height_2: f32 = 0.010;
        sample_height_3: f32 = 0.275;
        sample_underside_vein_relief_1: f32 = 0.017;
        sample_underside_vein_relief_2: f32 = 0.014;
        tissue_relief_broad_1: f32 = 8.0;
        tissue_relief_broad_2: f32 = 7.0;
        tissue_relief_broad_3: f32 = 0.8;
        tissue_relief_cross_1: f32 = 6.0;
        tissue_relief_cross_2: f32 = 9.0;
        tissue_relief_cross_3: f32 = 1.6;
        vein_target_x_bowed: f32 = 0.010;
        side_width_envelope_1: f32 = 0.355;
        side_width_envelope_2: f32 = 0.64;
        side_width_envelope_3: f32 = 1.08;
        side_width_envelope_4: f32 = 0.16;
        side_width_side_scale_1: f32 = 0.993;
        side_width_side_scale_2: f32 = 1.007;
        side_width_margin: f32 = 0.043;
        left_veins: [BeechVein; 8] = LEFT_VEINS;
        right_veins: [BeechVein; 8] = RIGHT_VEINS;
    }
}
