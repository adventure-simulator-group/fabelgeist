//! Recipe components retain the role of an offset independently of their dimensions.
use crate::spatial_geometry::{Architectural, CuboidDimensions, Displacement};

pub(super) struct RecipeComponent {
    pub offset: Displacement<Architectural>,
    pub dimensions: CuboidDimensions,
}
