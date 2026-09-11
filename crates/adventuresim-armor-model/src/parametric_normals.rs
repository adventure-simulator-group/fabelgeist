//! Carrier normals for creases whose triangle areas vary with flute sampling.
use super::{PartMesh, add, cross, dot, subtract};
use crate::GenerateError;

impl PartMesh {
    pub(super) fn angle_weighted_normals(&self) -> Result<Vec<[f32; 3]>, GenerateError> {
        self.normals()?;
        let mut sums = vec![[0.0; 3]; self.positions.len()];
        for face in self.indices.as_chunks::<3>().0 {
            let points = face.map(|i| self.positions[i as usize]);
            let normal = unit(cross(
                subtract(points[1], points[0]),
                subtract(points[2], points[0]),
            ))?;
            for corner in 0..3 {
                let a = unit(subtract(points[(corner + 1) % 3], points[corner]))?;
                let b = unit(subtract(points[(corner + 2) % 3], points[corner]))?;
                let angle = dot(a, b).clamp(-1.0, 1.0).acos();
                let index = face[corner] as usize;
                sums[index] = add(sums[index], normal.map(|v| v * angle));
            }
        }
        sums.into_iter().map(unit).collect()
    }
}

fn unit(vector: [f32; 3]) -> Result<[f32; 3], GenerateError> {
    let length = dot(vector, vector).sqrt();
    if !length.is_finite() || length <= 1e-12 {
        return Err(GenerateError::InvalidSurface);
    }
    Ok(vector.map(|v| v / length))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crease_direction_is_invariant_when_one_face_is_refined() {
        let coarse = PartMesh {
            positions: vec![[0.0; 3], [2.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            indices: vec![0, 1, 2, 0, 3, 1],
            ..Default::default()
        };
        let mut refined = coarse.clone();
        refined.positions.push([0.2, 0.0, 0.0]);
        refined.indices = vec![0, 4, 2, 4, 1, 2, 0, 3, 1];
        let a = coarse.angle_weighted_normals().unwrap()[0];
        let b = refined.angle_weighted_normals().unwrap()[0];
        for axis in 0..3 {
            assert!((a[axis] - b[axis]).abs() < 1e-6);
        }
        assert!((a[1] - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-6);
        assert!((a[2] - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-6);
    }
}
