//! Coordinate-shell topology built on the host: vertex by vertex, rings
//! joined into strips.
//!
//! Each vertex is the coordinate a shape's WGSL turns into a position, so the
//! topology is complete before any position exists.

/// A carrier's vertex coordinates and triangles, before any position exists.
#[derive(Clone, Debug, Default)]
pub(crate) struct CoordTopology {
    pub coords: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
}

impl CoordTopology {
    pub(crate) fn vertex(&mut self, coord: [f32; 4]) -> u32 {
        self.coords.push(coord);
        self.coords.len() as u32 - 1
    }

    /// Join two rings of equal length with two triangles per segment,
    /// closing the strip when `periodic`.
    pub(crate) fn connect(&mut self, upper: &[u32], lower: &[u32], periodic: bool) {
        let count = if periodic {
            upper.len()
        } else {
            upper.len() - 1
        };
        for i in 0..count {
            let next = (i + 1) % upper.len();
            self.indices.extend([
                upper[i],
                lower[i],
                lower[next],
                upper[i],
                lower[next],
                upper[next],
            ]);
        }
    }
}
