//! Sampling outside a finite compute grid.

/// How a compute operation selects a sample beyond a grid boundary.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BoundaryMode {
    /// Use the nearest sample on the grid's edge.
    #[default]
    Clamp,
    /// Continue at the opposite edge of the grid.
    Wrap,
}
