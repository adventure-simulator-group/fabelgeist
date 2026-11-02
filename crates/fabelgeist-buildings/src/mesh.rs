//! Triangles to draw buildings with: flat faces, each vertex carrying its
//! colour and what it is made of, so that a shader can lay courses of
//! masonry, rows of tiles and the grain of timber over it.

/// What a surface is made of.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum Material {
    /// Coursed limestone ashlar.
    Stone = 0,
    /// Lime plaster between a house's timbers.
    Plaster = 1,
    /// Oak framing, weathered dark.
    Timber = 2,
    /// Clay roof tiles.
    Tile = 3,
    /// Slate.
    Slate = 4,
    /// Planked doors and shutters.
    Wood = 5,
    /// A window, a doorway or an arrow slit: the dark inside.
    Opening = 6,
    /// Straw thatch.
    Thatch = 7,
}

impl Material {
    /// Its colour, sRGB.
    pub fn color(self) -> [f32; 3] {
        match self {
            Material::Stone => [0.56, 0.53, 0.47],
            Material::Plaster => [0.86, 0.82, 0.72],
            Material::Timber => [0.24, 0.17, 0.11],
            Material::Tile => [0.58, 0.27, 0.17],
            Material::Slate => [0.25, 0.27, 0.30],
            Material::Wood => [0.36, 0.24, 0.14],
            Material::Opening => [0.05, 0.045, 0.04],
            Material::Thatch => [0.60, 0.50, 0.30],
        }
    }
}

/// Indexed triangles, counter-clockwise from outside.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    /// sRGB.
    pub colors: Vec<[f32; 3]>,
    /// A [`Material`] per vertex.
    pub materials: Vec<u32>,
    pub indices: Vec<u32>,
}

pub(crate) fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub(crate) fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

pub(crate) fn normalize(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-12);
    [v[0] / l, v[1] / l, v[2] / l]
}

impl Mesh {
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    fn vertex(&mut self, p: [f32; 3], n: [f32; 3], color: [f32; 3], material: Material) -> u32 {
        self.positions.push(p);
        self.normals.push(n);
        self.colors.push(color);
        self.materials.push(material as u32);
        (self.positions.len() - 1) as u32
    }

    /// A flat polygon, its corners counter-clockwise from the side it faces,
    /// fanned from the first.
    pub fn polygon(&mut self, corners: &[[f32; 3]], material: Material, tint: f32) {
        if corners.len() < 3 {
            return;
        }
        let n = normalize(cross(
            sub(corners[1], corners[0]),
            sub(corners[2], corners[0]),
        ));
        let c = material.color().map(|v| (v * tint).min(1.0));
        let base = self.positions.len() as u32;
        for &p in corners {
            self.vertex(p, n, c, material);
        }
        for i in 1..corners.len() as u32 - 1 {
            self.indices
                .extend_from_slice(&[base, base + i, base + i + 1]);
        }
    }

    /// A strip of quads around a curved surface, smooth across: `rings[k]`
    /// and `rings[k + 1]` are consecutive rings with their normals.
    pub fn smooth_band(
        &mut self,
        lower: &[([f32; 3], [f32; 3])],
        upper: &[([f32; 3], [f32; 3])],
        material: Material,
        tint: f32,
    ) {
        let c = material.color().map(|v| (v * tint).min(1.0));
        let base = self.positions.len() as u32;
        let n = lower.len() as u32;
        for &(p, normal) in lower.iter().chain(upper) {
            self.vertex(p, normal, c, material);
        }
        for i in 0..n.saturating_sub(1) {
            let (a, b) = (base + i, base + i + 1);
            let (c, d) = (base + n + i, base + n + i + 1);
            self.indices.extend_from_slice(&[a, b, d, a, d, c]);
        }
    }

    pub fn append(&mut self, other: &Mesh) {
        let base = self.positions.len() as u32;
        self.positions.extend_from_slice(&other.positions);
        self.normals.extend_from_slice(&other.normals);
        self.colors.extend_from_slice(&other.colors);
        self.materials.extend_from_slice(&other.materials);
        self.indices.extend(other.indices.iter().map(|i| i + base));
    }
}
