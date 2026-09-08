//! Artist controls and canonical recipe defaults.
use super::*;

crate::parameters::parameter_block! {
    pub struct Parameters {
        generate_ao_1: f32 = 231.0;
        generate_ao_2: f32 = 35.0;
        generate_ao_3: f32 = 248.0;
        generate_ao_4: f32 = 218.0;
        generate_ao_5: f32 = 42.0;
        generate_ao_6: f32 = 218.0;
        generate_ao_7: f32 = 243.0;
        sample_t_1: f32 = 0.09;
        sample_t_2: f32 = 0.82;
        sample_petiole: f32 = 0.012;
        sample_midrib_width_1: f32 = 0.010;
        sample_midrib_width_2: f32 = 0.003;
        sample_transverse: f32 = 0.001;
        sample_blade_dome_1: f32 = 0.68;
        sample_blade_dome_2: f32 = 0.16;
        sample_vein_ridge_1: f32 = 0.014;
        sample_vein_ridge_2: f32 = 0.11;
        sample_height_1: f32 = 0.10;
        sample_height_2: f32 = 0.012;
        sample_height_3: f32 = 0.32;
        tissue_relief_broad_1: f32 = 10.0;
        tissue_relief_broad_2: f32 = 6.0;
        tissue_relief_broad_3: f32 = 0.9;
        tissue_relief_cross_1: f32 = 7.0;
        tissue_relief_cross_2: f32 = 11.0;
        tissue_relief_cross_3: f32 = 1.7;
        side_width_envelope_1: f32 = 0.72;
        side_width_envelope_2: f32 = 0.305;
        side_width_envelope_3: f32 = 0.158;
        side_width_envelope_4: f32 = 0.82;
        side_width_envelope_5: f32 = 0.72;
        side_width_envelope_6: f32 = 0.392;
        side_width_envelope_7: f32 = 0.28;
        side_width_envelope_8: f32 = 1.28;
        side_width_side_scale_1: f32 = 0.985;
        side_width_side_scale_2: f32 = 1.015;
        side_width_asymmetry_1: f32 = 1.08;
        side_width_asymmetry_2: f32 = 0.94;
        left_veins: [HazelVein; 8] = LEFT_VEINS;
        right_veins: [HazelVein; 8] = RIGHT_VEINS;
    }
}
