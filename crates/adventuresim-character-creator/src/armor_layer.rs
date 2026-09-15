//! Explicit surfaces worn underneath an independently generated armor piece.
pub struct ArmorLayerSurface<'a> {
    /// Relief omitted from this smooth supporting surface.
    pub relief: adventuresim_armor_model::Millimeters,
    pub positions: &'a [[f32; 3]],
    pub faces: &'a [[u32; 3]],
}
