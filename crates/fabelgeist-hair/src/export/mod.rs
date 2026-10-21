//! Reading and writing hair.
//!
//! * [`npz`]: NumPy arrays and archives, the DiffLocks dataset's format --
//!   a sample's `full_strands.npz` holds `positions` (strands x 256 x 3),
//!   `root_uv` (strands x 2) and `root_normal` (strands x 3), all `float32`.
//! * [`usc`]: the USC-HairSalon `.data` format most hair papers read and
//!   write -- a strand count, a point count, a point count per strand, then
//!   every point.
//! * [`ply`]: enough of PLY to read and write a scalp mesh.
//! * [`png`]: grey PNGs, for DiffLocks' density maps.

pub mod error;
pub mod npz;
pub mod ply;
pub mod png;
pub mod usc;

use std::path::Path;

use self::error::Result;

use crate::strands::Strands;

/// Write `strands` as DiffLocks writes a sample's strands: resampled to
/// `points` points each, in `full_strands.npz` layout.
pub fn write_difflocks_npz(path: &Path, strands: &Strands, points: usize) -> Result<()> {
    let strands = strands.resampled(points);
    let n = strands.len();
    let positions: Vec<f32> = strands
        .positions
        .iter()
        .flat_map(|p| p.to_array())
        .collect();
    let uv: Vec<f32> = strands.root_uv.iter().flatten().copied().collect();
    let normals: Vec<f32> = strands
        .root_normal
        .iter()
        .flat_map(|p| p.to_array())
        .collect();
    npz::write(
        path,
        &[
            ("positions", &[n, points, 3], &positions),
            ("root_uv", &[n, 2], &uv),
            ("root_normal", &[n, 3], &normals),
        ],
    )
}
