//! Artist controls and canonical recipe defaults.
use super::*;

crate::parameters::parameter_block! {
    pub struct Parameters {
        generate_ao_1: f32 = 230.0;
        generate_ao_2: f32 = 36.0;
        generate_ao_3: f32 = 248.0;
        generate_ao_4: f32 = 220.0;
        generate_ao_5: f32 = 40.0;
        generate_ao_6: f32 = 220.0;
        generate_ao_7: f32 = 244.0;
        sample_t_1: f32 = 0.09;
        sample_t_2: f32 = 0.82;
        sample_petiole: f32 = 0.009;
        sample_height: f32 = 0.09;
        sample_midrib_width_1: f32 = 0.0075;
        sample_midrib_width_2: f32 = 0.0025;
        sample_transverse: f32 = 0.001;
        sample_blade_dome_1: f32 = 0.72;
        sample_blade_dome_2: f32 = 0.135;
        sample_vein_ridge_1: f32 = 0.013;
        sample_vein_ridge_2: f32 = 0.095;
        sample_height_1: f32 = 0.09;
        sample_height_2: f32 = 0.01;
        sample_height_3: f32 = 0.29;
        sample_underside_vein_relief_1: f32 = 0.018;
        sample_underside_vein_relief_2: f32 = 0.010;
        tissue_relief_broad_1: f32 = 9.0;
        tissue_relief_broad_2: f32 = 7.0;
        tissue_relief_broad_3: f32 = 0.6;
        tissue_relief_cross_1: f32 = 6.0;
        tissue_relief_cross_2: f32 = 10.0;
        tissue_relief_cross_3: f32 = 1.5;
        side_width_basal_1: f32 = 0.105;
        side_width_basal_2: f32 = 1.7;
        side_width_broad_lamina_1: f32 = 0.345;
        side_width_broad_lamina_2: f32 = 0.62;
        side_width_broad_lamina_3: f32 = 0.96;
        side_width_broad_lamina_4: f32 = 0.08;
        side_width_side_scale_1: f32 = 0.992;
        side_width_side_scale_2: f32 = 1.008;
        tooth_extension_amplitude_1: f32 = 0.010;
        tooth_extension_amplitude_2: f32 = 0.008;
        tooth_extension_amplitude_3: f32 = 0.009;
        tooth_extension_amplitude_4: f32 = 0.007;
        tooth_center_offset_1: f32 = 0.003;
        tooth_center_offset_2: f32 = 0.004;
        tooth_count: usize = TOOTH_COUNT;
        left_veins: [BlackthornVein; 6] = LEFT_VEINS;
        right_veins: [BlackthornVein; 6] = RIGHT_VEINS;
    }
}
