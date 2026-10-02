//! Physical scale and authored plate/fissure controls. Widths are fractions of a tile.
use super::*;

crate::parameters::parameter_block! {
    pub struct Parameters {
        oak_bark_tile_metres: f32 = OAK_BARK_TILE_METRES;
        oak_bark_height_range_metres: f32 = OAK_BARK_HEIGHT_RANGE_METRES;
        columns: i32 = OAK_BARK_COLUMNS;
        rows: i32 = OAK_BARK_ROWS;
        fissure_width_min: f32 = OAK_BARK_FISSURE_WIDTH_MIN;
        fissure_width_span: f32 = OAK_BARK_FISSURE_WIDTH_SPAN;
        valley_width_min: f32 = OAK_BARK_VALLEY_WIDTH_MIN;
        valley_width_span: f32 = OAK_BARK_VALLEY_WIDTH_SPAN;
        crown_height_min: f32 = OAK_BARK_CROWN_HEIGHT_MIN;
        crown_height_span: f32 = OAK_BARK_CROWN_HEIGHT_SPAN;
    }
}
