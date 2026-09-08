//! Artist controls and canonical recipe defaults.

crate::parameters::parameter_block! {
    pub struct Parameters {
        relief_t_1: f32 = 0.075;
        relief_t_2: f32 = 0.85;
        relief_axis_1: f32 = 0.026;
        relief_axis_2: f32 = 0.42;
        relief_axis_3: f32 = 0.004;
        relief_transverse: f32 = 0.19;
        relief_edge_curl_1: f32 = 2.4;
        relief_edge_curl_2: f32 = 0.055;
        relief_edge_curl_3: f32 = 0.018;
        relief_edge_curl_4: f32 = 0.8;
        relief_pucker_1: f32 = 23.0;
        relief_pucker_2: f32 = 17.0;
        relief_pucker_3: f32 = 11.0;
        relief_pucker_4: f32 = 29.0;
        relief_pucker_5: f32 = 0.6;
        relief_pucker_6: f32 = 0.006;
    }
}
