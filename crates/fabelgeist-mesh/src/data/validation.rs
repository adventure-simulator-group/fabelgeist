use crate::{
    DrawVertexCount, DrawVertexIndex, DrawVertexMembership, MeshAttribute, MeshAttributeLength,
    MeshValidationError, PrimitiveTopology, SkinWeightViolation,
};

use super::MeshData;

impl MeshData {
    pub fn validate(&self) -> Result<(), MeshValidationError> {
        let expected = MeshAttributeLength::from(self.positions.len());
        let vertices = DrawVertexCount::from_attribute_length(expected)?;
        for value in self.positions.iter().flatten() {
            if !value.is_finite() {
                return Err(MeshValidationError::NonFinitePosition);
            }
        }
        for (attribute, actual) in [
            (MeshAttribute::Normals, self.normals.len()),
            (MeshAttribute::TextureCoordinates, self.tex_coords.len()),
        ] {
            if actual != 0 && MeshAttributeLength::from(actual) != expected {
                return Err(MeshValidationError::AttributeLength {
                    attribute,
                    expected,
                    actual: MeshAttributeLength::from(actual),
                });
            }
        }
        for value in self.normals.iter().flatten() {
            if !value.is_finite() {
                return Err(MeshValidationError::NonFiniteAttribute {
                    attribute: MeshAttribute::Normals,
                });
            }
        }
        for value in self.tex_coords.iter().flatten() {
            if !value.is_finite() {
                return Err(MeshValidationError::NonFiniteAttribute {
                    attribute: MeshAttribute::TextureCoordinates,
                });
            }
        }
        self.validate_skin(expected)?;
        self.validate_indices(vertices)
    }

    fn validate_skin(&self, expected: MeshAttributeLength) -> Result<(), MeshValidationError> {
        match (&self.joints, &self.weights) {
            (Some(_), None) => {
                return Err(MeshValidationError::SkinPair {
                    missing: MeshAttribute::Weights,
                });
            }
            (None, Some(_)) => {
                return Err(MeshValidationError::SkinPair {
                    missing: MeshAttribute::Joints,
                });
            }
            _ => {}
        }
        if let Some(joints) = &self.joints {
            let actual = MeshAttributeLength::from(joints.len());
            if actual != expected {
                return Err(MeshValidationError::AttributeLength {
                    attribute: MeshAttribute::Joints,
                    expected,
                    actual,
                });
            }
        }
        if let Some(weights) = &self.weights {
            let actual = MeshAttributeLength::from(weights.len());
            if actual != expected {
                return Err(MeshValidationError::SkinWeights {
                    violation: SkinWeightViolation::Length { expected, actual },
                });
            }
            for value in weights.iter().flatten() {
                let violation = if !value.is_finite() {
                    Some(SkinWeightViolation::NonFinite)
                } else if *value < 0.0 {
                    Some(SkinWeightViolation::Negative)
                } else {
                    None
                };
                if let Some(violation) = violation {
                    return Err(MeshValidationError::SkinWeights { violation });
                }
            }
        }
        Ok(())
    }

    fn validate_indices(&self, vertices: DrawVertexCount) -> Result<(), MeshValidationError> {
        if let Some(indices) = &self.indices {
            for &word in indices {
                let index = DrawVertexIndex::from(word);
                if index.membership(vertices) == DrawVertexMembership::Outside {
                    return Err(MeshValidationError::IndexOutOfBounds { index, vertices });
                }
            }
        }
        let elements = match &self.indices {
            Some(indices) => indices.len(),
            None => self.positions.len(),
        };
        let count = MeshAttributeLength::from(elements);
        match self.topology {
            PrimitiveTopology::TriangleList if !elements.is_multiple_of(3) => {
                Err(MeshValidationError::TriangleListCount { actual: count })
            }
            PrimitiveTopology::LineList if !elements.is_multiple_of(2) => {
                Err(MeshValidationError::LineListCount { actual: count })
            }
            _ => Ok(()),
        }
    }
}
