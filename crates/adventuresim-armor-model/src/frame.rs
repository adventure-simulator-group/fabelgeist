//! Anatomical placement of an armor part, and how its plates meet at their
//! edges.

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

#[derive(Clone, Copy, Debug)]
pub enum BoundaryNormals {
    Smooth,
    /// Duplicate return-wall vertices so plate surfaces keep their own normals.
    Separate,
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
