//! The ground plants grow on: a height map and a soil map (Sec. 6.1).

/// A regular height field over `[origin, origin + size]` in `x`/`z`, with a
/// soil mask that blocks growth (water, rock, roads).
#[derive(Clone, Debug, PartialEq)]
pub struct Terrain {
    pub origin: [f32; 2],
    pub size: [f32; 2],
    /// Samples along `x` and `z`; at least two each.
    pub resolution: [usize; 2],
    /// Row-major, `z` outer.
    pub heights: Vec<f32>,
    /// `true` where nothing may grow.
    pub blocked: Vec<bool>,
}

impl Terrain {
    /// Level ground centred on the origin.
    pub fn flat(width: f32, depth: f32) -> Self {
        Self::from_fn(
            [-width * 0.5, -depth * 0.5],
            [width, depth],
            [2, 2],
            |_, _| 0.0,
        )
    }

    /// Sample `height(x, z)` on a grid.
    pub fn from_fn(
        origin: [f32; 2],
        size: [f32; 2],
        resolution: [usize; 2],
        height: impl Fn(f32, f32) -> f32,
    ) -> Self {
        let resolution = [resolution[0].max(2), resolution[1].max(2)];
        let mut heights = Vec::with_capacity(resolution[0] * resolution[1]);
        for j in 0..resolution[1] {
            for i in 0..resolution[0] {
                let x = origin[0] + size[0] * i as f32 / (resolution[0] - 1) as f32;
                let z = origin[1] + size[1] * j as f32 / (resolution[1] - 1) as f32;
                heights.push(height(x, z));
            }
        }
        Self {
            origin,
            size,
            resolution,
            blocked: vec![false; heights.len()],
            heights,
        }
    }

    /// Block the soil wherever `blocked(x, z, height)` holds.
    pub fn with_soil(mut self, blocked: impl Fn(f32, f32, f32) -> bool) -> Self {
        for j in 0..self.resolution[1] {
            for i in 0..self.resolution[0] {
                let [x, z] = self.sample_position(i, j);
                let index = j * self.resolution[0] + i;
                self.blocked[index] = blocked(x, z, self.heights[index]);
            }
        }
        self
    }

    fn sample_position(&self, i: usize, j: usize) -> [f32; 2] {
        [
            self.origin[0] + self.size[0] * i as f32 / (self.resolution[0] - 1) as f32,
            self.origin[1] + self.size[1] * j as f32 / (self.resolution[1] - 1) as f32,
        ]
    }

    pub fn contains(&self, x: f32, z: f32) -> bool {
        x >= self.origin[0]
            && z >= self.origin[1]
            && x <= self.origin[0] + self.size[0]
            && z <= self.origin[1] + self.size[1]
    }

    fn grid(&self, x: f32, z: f32) -> (usize, usize, f32, f32) {
        let u =
            ((x - self.origin[0]) / self.size[0]).clamp(0.0, 1.0) * (self.resolution[0] - 1) as f32;
        let v =
            ((z - self.origin[1]) / self.size[1]).clamp(0.0, 1.0) * (self.resolution[1] - 1) as f32;
        let i = (u.floor() as usize).min(self.resolution[0] - 2);
        let j = (v.floor() as usize).min(self.resolution[1] - 2);
        (i, j, u - i as f32, v - j as f32)
    }

    /// Bilinear height at `(x, z)`, clamped to the edges.
    pub fn height_at(&self, x: f32, z: f32) -> f32 {
        let (i, j, fu, fv) = self.grid(x, z);
        let w = self.resolution[0];
        let h00 = self.heights[j * w + i];
        let h10 = self.heights[j * w + i + 1];
        let h01 = self.heights[(j + 1) * w + i];
        let h11 = self.heights[(j + 1) * w + i + 1];
        let a = h00 + (h10 - h00) * fu;
        let b = h01 + (h11 - h01) * fu;
        a + (b - a) * fv
    }

    /// Whether a plant may take root at `(x, z)`: inside the terrain, on open
    /// soil (the nearest soil sample decides).
    pub fn is_open(&self, x: f32, z: f32) -> bool {
        if !self.contains(x, z) {
            return false;
        }
        let (i, j, fu, fv) = self.grid(x, z);
        let i = if fu > 0.5 { i + 1 } else { i };
        let j = if fv > 0.5 { j + 1 } else { j };
        !self.blocked[j * self.resolution[0] + i]
    }

    /// Lowest and highest sample.
    pub fn height_range(&self) -> (f32, f32) {
        self.heights
            .iter()
            .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), &h| {
                (lo.min(h), hi.max(h))
            })
    }

    /// The terrain as a triangle mesh.
    pub fn mesh(&self) -> fabelgeist_mesh::MeshData {
        let [nx, nz] = self.resolution;
        let mut positions = Vec::with_capacity(nx * nz);
        let mut normals = Vec::with_capacity(nx * nz);
        let mut tex_coords = Vec::with_capacity(nx * nz);
        let dx = self.size[0] / (nx - 1) as f32;
        let dz = self.size[1] / (nz - 1) as f32;
        for j in 0..nz {
            for i in 0..nx {
                let [x, z] = self.sample_position(i, j);
                let h = |a: usize, b: usize| self.heights[b * nx + a];
                let hx = h((i + 1).min(nx - 1), j) - h(i.saturating_sub(1), j);
                let hz = h(i, (j + 1).min(nz - 1)) - h(i, j.saturating_sub(1));
                let n = [-hx / (2.0 * dx), 1.0, -hz / (2.0 * dz)];
                let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
                positions.push([x, h(i, j), z]);
                normals.push([n[0] / l, n[1] / l, n[2] / l]);
                tex_coords.push([i as f32 / (nx - 1) as f32, j as f32 / (nz - 1) as f32]);
            }
        }
        let mut indices = Vec::with_capacity((nx - 1) * (nz - 1) * 6);
        for j in 0..nz - 1 {
            for i in 0..nx - 1 {
                let a = (j * nx + i) as u32;
                let b = a + 1;
                let c = a + nx as u32;
                let d = c + 1;
                indices.extend_from_slice(&[a, c, b, b, c, d]);
            }
        }
        fabelgeist_mesh::MeshData {
            positions,
            normals,
            tex_coords,
            indices: Some(indices),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bilinear_heights_and_soil() {
        let terrain = Terrain::from_fn([0.0, 0.0], [10.0, 10.0], [11, 11], |x, z| x + 2.0 * z)
            .with_soil(|x, _, _| x > 8.0);
        assert!((terrain.height_at(3.25, 4.5) - (3.25 + 9.0)).abs() < 1e-4);
        assert!(terrain.is_open(1.0, 1.0));
        assert!(!terrain.is_open(9.5, 1.0));
        assert!(!terrain.is_open(-1.0, 1.0));
    }
}
