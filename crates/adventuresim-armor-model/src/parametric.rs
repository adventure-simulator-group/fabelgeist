//! Construction-independent geometry and anatomical placement for armor families.

use std::collections::BTreeMap;

use crate::GenerateError;

/// Anatomical placement, separate from a recipe's artistic controls.
/// Local coordinates are metres. Reflected frames are supported explicitly.
#[derive(Clone, Copy, Debug)]
pub struct PartFrame {
    pub origin: [f32; 3],
    pub axes: [[f32; 3]; 3],
    pub half_extents: [f32; 3],
}

impl PartFrame {
    pub fn point(&self, local: [f32; 3]) -> [f32; 3] {
        std::array::from_fn(|axis| {
            self.origin[axis] + (0..3).map(|i| self.axes[i][axis] * local[i]).sum::<f32>()
        })
    }

    pub fn validate(&self) -> Result<(), GenerateError> {
        let finite = self
            .origin
            .iter()
            .chain(self.axes.iter().flatten())
            .chain(self.half_extents.iter())
            .all(|v| v.is_finite());
        let orthogonal = (0..3).all(|i| {
            (0..3).all(|j| {
                (dot(self.axes[i], self.axes[j]) - if i == j { 1.0 } else { 0.0 }).abs() < 0.001
            })
        });
        if !finite || !orthogonal || self.half_extents.iter().any(|v| *v <= 0.0) {
            return Err(GenerateError::InvalidSurface);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct PartMesh {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    shells: Vec<ShellLayout>,
}

#[derive(Clone, Debug)]
struct ShellLayout {
    first_vertex: usize,
    vertex_count: usize,
    first_index: usize,
    index_count: usize,
    thickness: f32,
}

impl PartMesh {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append(&mut self, other: Self) {
        let offset = self.positions.len() as u32;
        let index_offset = self.indices.len();
        self.shells
            .extend(other.shells.into_iter().map(|mut shell| {
                shell.first_vertex += offset as usize;
                shell.first_index += index_offset;
                shell
            }));
        self.positions.extend(other.positions);
        self.indices
            .extend(other.indices.into_iter().map(|i| i + offset));
    }

    /// Refit authored carriers, then rebuild their inner walls at physical gauge.
    /// Carrier vertex correspondence and connectivity remain unchanged.
    pub fn refit_surfaces(
        &self,
        mut fit: impl FnMut(&mut [[f32; 3]]),
    ) -> Result<Self, GenerateError> {
        let mut result = Self::new();
        for shell in &self.shells {
            let mut positions = self.positions
                [shell.first_vertex..shell.first_vertex + shell.vertex_count]
                .to_vec();
            let indices = self.indices[shell.first_index..shell.first_index + shell.index_count]
                .iter()
                .map(|i| i - shell.first_vertex as u32)
                .collect();
            fit(&mut positions);
            result.append(Self::from_surface(positions, indices, shell.thickness)?);
        }
        Ok(result)
    }

    pub fn transformed(mut self, frame: &PartFrame) -> Self {
        self.positions.iter_mut().for_each(|p| *p = frame.point(*p));
        if dot(cross(frame.axes[0], frame.axes[1]), frame.axes[2]) < 0.0 {
            for triangle in self.indices.as_chunks_mut::<3>().0 {
                triangle.swap(1, 2);
            }
        }
        self
    }

    /// Area-weighted vertex normals, with finite, index and triangle checks.
    pub fn normals(&self) -> Result<Vec<[f32; 3]>, GenerateError> {
        if self.positions.is_empty()
            || self.indices.is_empty()
            || !self.indices.len().is_multiple_of(3)
            || self.positions.iter().flatten().any(|v| !v.is_finite())
            || self
                .indices
                .iter()
                .any(|i| *i as usize >= self.positions.len())
        {
            return Err(GenerateError::InvalidSurface);
        }
        let mut normals = vec![[0.0; 3]; self.positions.len()];
        for triangle in self.indices.as_chunks::<3>().0 {
            let [a, b, c] =
                [triangle[0], triangle[1], triangle[2]].map(|i| self.positions[i as usize]);
            let normal = cross(subtract(b, a), subtract(c, a));
            if dot(normal, normal) <= 1e-18 {
                return Err(GenerateError::Degenerate);
            }
            for i in triangle {
                normals[*i as usize] = add(normals[*i as usize], normal);
            }
        }
        normals
            .into_iter()
            .map(|n| {
                let magnitude = dot(n, n).sqrt();
                if magnitude <= 1e-12 {
                    Err(GenerateError::Degenerate)
                } else {
                    Ok(n.map(|v| v / magnitude))
                }
            })
            .collect()
    }

    /// Thicken an outward-wound carrier and close every boundary edge.
    /// The input must have shared indices along its intended seams.
    pub fn from_surface(
        positions: Vec<[f32; 3]>,
        indices: Vec<u32>,
        thickness: f32,
    ) -> Result<Self, GenerateError> {
        if !thickness.is_finite() || thickness <= 0.0 {
            return Err(GenerateError::InvalidSurface);
        }
        let layout = ShellLayout {
            first_vertex: 0,
            vertex_count: positions.len(),
            first_index: 0,
            index_count: indices.len(),
            thickness,
        };
        let mut mesh = Self {
            positions,
            indices,
            shells: vec![layout],
        };
        let normals = mesh.normals()?;
        let count = mesh.positions.len() as u32;
        let mut edges = BTreeMap::<(u32, u32), Vec<(u32, u32)>>::new();
        for triangle in mesh.indices.as_chunks::<3>().0 {
            for (a, b) in [
                (triangle[0], triangle[1]),
                (triangle[1], triangle[2]),
                (triangle[2], triangle[0]),
            ] {
                edges.entry((a.min(b), a.max(b))).or_default().push((a, b));
            }
        }
        if edges
            .values()
            .any(|e| e.len() > 2 || (e.len() == 2 && e[0] != (e[1].1, e[1].0)))
        {
            return Err(GenerateError::InvalidSurface);
        }
        let inner = mesh
            .positions
            .iter()
            .zip(normals)
            .map(|(p, n)| std::array::from_fn(|axis| p[axis] - thickness * n[axis]))
            .collect::<Vec<_>>();
        let inner_indices = mesh
            .indices
            .as_chunks::<3>()
            .0
            .iter()
            .flat_map(|t| [t[0] + count, t[2] + count, t[1] + count])
            .collect::<Vec<_>>();
        mesh.positions.extend(inner);
        mesh.indices.extend(inner_indices);
        for edge in edges.values().filter(|e| e.len() == 1) {
            let (a, b) = edge[0];
            mesh.indices
                .extend([b, a, a + count, b, a + count, b + count]);
        }
        mesh.normals()?;
        Ok(mesh)
    }
}

pub(crate) fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    (0..3).map(|i| a[i] * b[i]).sum()
}
pub(crate) fn subtract(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}
pub(crate) fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|i| a[i] + b[i])
}
pub(crate) fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
