//! The outer face of a plate as rows along it and columns across it.

/// The outer face of one plate, as the grid its carrier was laid out on:
/// `rows` rows along the plate, each `columns` vertices across it, closed
/// around when `cyclic`. A construction laid over the plate, such as scales,
/// is placed in the grid's own coordinates, so it follows the plate through
/// every body morph.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SurfaceGrid {
    pub rows: u32,
    pub columns: u32,
    pub cyclic: bool,
    /// The piece's vertex at each row and column, row by row.
    pub vertices: Vec<u32>,
}

impl SurfaceGrid {
    /// The piece's vertex at `row` and `column`.
    pub fn vertex(&self, row: u32, column: u32) -> u32 {
        self.vertices[(row * self.columns + column) as usize]
    }
}
