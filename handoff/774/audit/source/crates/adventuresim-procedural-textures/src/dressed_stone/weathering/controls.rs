//! Artist controls and canonical recipe defaults.
use super::*;

crate::parameters::parameter_block! {
    pub struct Parameters {
        face_detail_aspect_1: f32 = 0.65;
        face_detail_aspect_2: f32 = 0.70;
        damaged_edge_probability: f32 = DAMAGED_EDGE_PROBABILITY;
        chip_half_width_metres: [f32; 2] = CHIP_HALF_WIDTH_METRES;
        chip_depth_metres: [f32; 2] = CHIP_DEPTH_METRES;
        bevel_width_metres: [f32; 2] = BEVEL_WIDTH_METRES;
        pore_cell_metres: f32 = PORE_CELL_METRES;
        pore_probability: f32 = PORE_PROBABILITY;
        pore_radius_metres: [f32; 2] = PORE_RADIUS_METRES;
        mineral_patch_metres: f32 = MINERAL_PATCH_METRES;
        stone_grain_metres: f32 = STONE_GRAIN_METRES;
        bevel_width_variation: f32 = BEVEL_WIDTH_VARIATION;
        fracture_shoulder_ratio: f32 = FRACTURE_SHOULDER_RATIO;
        fracture_facets: [(f32, f32); 3] = FRACTURE_FACETS;
        edge_breakup_metres: f32 = EDGE_BREAKUP_METRES;
        spall_width_metres: f32 = SPALL_WIDTH_METRES;
        corner_fracture_probability: f32 = CORNER_FRACTURE_PROBABILITY;
        corner_fracture_metres: [f32; 2] = CORNER_FRACTURE_METRES;
    }
}
