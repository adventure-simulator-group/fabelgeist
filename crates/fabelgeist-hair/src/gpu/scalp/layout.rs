//! Surface-net shader grid: native metre coordinates and invocation extents.
use super::*;
use fabelgeist_gpu::data::gpu::parameters::PassParameters;
pub(super) struct ScalpGrid {
    lo: Vec3,
    cell: f32,
    nx: u32,
    ny: u32,
    nz: u32,
    j0: u32,
    keep_y: f32,
    pub(super) corners: u32,
    pub(super) cells: u32,
    pub(super) edges: u32,
}
impl ScalpGrid {
    pub(super) fn new(head: &Head, cell: f32) -> Self {
        // The bust's grid, as `Head::mesh` lays it, from the rows the scalp
        // can reach: a kept vertex is above `keep_y`, and within three
        // cells of its own (two Newton steps each less than a cell), and a
        // quad joins cells a row apart.
        let lo = Vec3::new(-0.24, head.base - 0.01, -0.16);
        let hi = Vec3::new(0.24, 0.14, 0.14);
        let cells_along = [hi.x - lo.x, hi.y - lo.y, hi.z - lo.z]
            .map(|extent| ((extent / cell).ceil() as u32).max(1));
        let keep_y = head.neck.a.y - 0.04;
        let j0 =
            (((keep_y - lo.y) / cell).floor() as i64 - 6).clamp(0, cells_along[1] as i64) as u32;
        let (nx, ny, nz) = (
            cells_along[0] + 1,
            cells_along[1] + 1 - j0,
            cells_along[2] + 1,
        );
        let corners = nx * ny * nz;
        let cells = (nx - 1) * (ny - 1) * (nz - 1);
        let edges = corners * 3;
        Self {
            lo,
            cell,
            nx,
            ny,
            nz,
            j0,
            keep_y,
            corners,
            cells,
            edges,
        }
    }
    pub(super) fn parameters(&self, head: &Head, count: u32) -> PassParameters {
        params! {
            "lo_x" => self.lo.x,
            "lo_y" => self.lo.y,
            "lo_z" => self.lo.z,
            "cell" => self.cell,
            "nx" => self.nx,
            "ny" => self.ny,
            "nz" => self.nz,
            "j0" => self.j0,
            "hairline_front" => head.hairline[0],
            "hairline_side" => head.hairline[1],
            "hairline_back" => head.hairline[2],
            "recession" => head.recession,
            "peak" => head.peak,
            "keep_y" => self.keep_y,
            "count" => count,
            "pad0" => 0u32,
        }
    }
}

/// Decode the surface-net bounds pass's six ordered IEEE-754 words.
pub(super) fn ordered_bounds(words: &[u32]) -> [Vec3; 2] {
    let float = |key: u32| {
        f32::from_bits(if key & 0x8000_0000 != 0 {
            key & 0x7FFF_FFFF
        } else {
            !key
        })
    };
    [
        Vec3::new(float(!words[0]), float(!words[1]), float(!words[2])),
        Vec3::new(float(words[3]), float(words[4]), float(words[5])),
    ]
}

pub(super) fn trimmed(
    mut buffer: Buffer,
    bytes: fabelgeist_gpu::data::gpu::buffer::BufferByteLength,
) -> Buffer {
    buffer.size = u64::from(bytes).max(16).into();
    buffer
}
