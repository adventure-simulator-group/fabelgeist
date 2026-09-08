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
        sample_t_1: f32 = 0.095;
        sample_t_2: f32 = 0.82;
        sample_petiole: f32 = 0.008;
        sample_height: f32 = 0.09;
        sample_midrib_width_1: f32 = 0.0065;
        sample_midrib_width_2: f32 = 0.0025;
        sample_transverse: f32 = 0.001;
        sample_blade_dome_1: f32 = 0.68;
        sample_blade_dome_2: f32 = 0.125;
        sample_central_fold_1: f32 = 0.055;
        sample_central_fold_2: f32 = 0.028;
        sample_vein_ridge_1: f32 = 0.014;
        sample_vein_ridge_2: f32 = 0.085;
        sample_height_1: f32 = 0.09;
        sample_height_2: f32 = 0.01;
        sample_height_3: f32 = 0.30;
        sample_underside_vein_relief_1: f32 = 0.019;
        sample_underside_vein_relief_2: f32 = 0.009;
        tissue_relief_broad_1: f32 = 8.0;
        tissue_relief_broad_2: f32 = 6.0;
        tissue_relief_broad_3: f32 = 0.8;
        tissue_relief_cross_1: f32 = 5.0;
        tissue_relief_cross_2: f32 = 9.0;
        tissue_relief_cross_3: f32 = 1.4;
        tooth_extension_shift_1: f32 = 0.003;
        tooth_extension_shift_2: f32 = 0.004;
        left_widths: [WidthLandmark; 8] = LEFT_WIDTHS;
        right_widths: [WidthLandmark; 8] = RIGHT_WIDTHS;
        left_veins: [HawthornVein; 3] = LEFT_VEINS;
        right_veins: [HawthornVein; 3] = RIGHT_VEINS;
    }
}
