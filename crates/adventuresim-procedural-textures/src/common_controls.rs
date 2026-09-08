//! Artist controls and canonical recipe defaults.
use super::*;

crate::parameters::parameter_block! {
    pub struct Parameters {
        forest_soil_tile_metres: f32 = FOREST_SOIL_TILE_METRES;
        forest_soil_height_range_metres: f32 = FOREST_SOIL_HEIGHT_RANGE_METRES;
        forest_litter_tile_metres: f32 = FOREST_LITTER_TILE_METRES;
        forest_litter_height_range_metres: f32 = FOREST_LITTER_HEIGHT_RANGE_METRES;
    }
}
