//! Inspect the four affine coefficients consumed by Bevy's actual vertex shader.
use bevy::{mesh::VertexAttributeValues, prelude::*};
use serde::Serialize;

const PRIMARY_WEIGHT_SUM_TOLERANCE: f32 = 1e-4;

#[derive(Serialize)]
pub(super) struct WeightSummary {
    pub vertices: usize,
    pub minimum_sum: Option<f32>,
    pub maximum_sum: Option<f32>,
    pub invalid_vertices: usize,
}

impl WeightSummary {
    pub fn from_mesh(mesh: &Mesh) -> Option<Self> {
        let VertexAttributeValues::Float32x4(weights) =
            mesh.attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT)?
        else {
            return None;
        };
        let mut result = Self {
            vertices: weights.len(),
            minimum_sum: None,
            maximum_sum: None,
            invalid_vertices: 0,
        };
        for weights in weights {
            let sum = weights.iter().sum::<f32>();
            if sum.is_finite() {
                result.minimum_sum = Some(result.minimum_sum.map_or(sum, |old| old.min(sum)));
                result.maximum_sum = Some(result.maximum_sum.map_or(sum, |old| old.max(sum)));
            }
            if weights
                .iter()
                .any(|weight| !weight.is_finite() || *weight < 0.0)
                || (sum - 1.0).abs() > PRIMARY_WEIGHT_SUM_TOLERANCE
            {
                result.invalid_vertices += 1;
            }
        }
        Some(result)
    }

    pub fn valid(&self) -> bool {
        self.vertices > 0 && self.invalid_vertices == 0
    }
}
