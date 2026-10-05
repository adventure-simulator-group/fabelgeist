//! Column counts and identity-column extension of a model transform.

use super::ParameterTransform;

/// Number of named columns, including any appended identity blend shapes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelParameterCount(usize);

impl From<usize> for ModelParameterCount {
    fn from(count: usize) -> Self {
        Self(count)
    }
}

impl From<ModelParameterCount> for usize {
    fn from(count: ModelParameterCount) -> Self {
        count.0
    }
}

/// Number of identity columns to append; these columns drive no joint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlendShapeParameterCount(usize);

impl From<usize> for BlendShapeParameterCount {
    fn from(count: usize) -> Self {
        Self(count)
    }
}

impl ParameterTransform {
    /// Number of columns currently present in the transform.
    pub fn parameter_count(&self) -> ModelParameterCount {
        ModelParameterCount::from(self.names.len())
    }

    /// Appends one column per identity blend-shape coefficient, as Momentum's
    /// `Character::withBlendShape` does. The new columns drive no joint.
    pub fn append_blend_shape_parameters(&mut self, count: BlendShapeParameterCount) {
        let old_columns = usize::from(self.parameter_count());
        let new_columns = old_columns + count.0;
        let mut dense = vec![0.0; self.num_joint_parameters * new_columns];
        for row in 0..self.num_joint_parameters {
            let source = &self.transform[row * old_columns..(row + 1) * old_columns];
            dense[row * new_columns..row * new_columns + old_columns].copy_from_slice(source);
        }
        self.transform = dense;
        for index in 0..count.0 {
            self.names.push(format!("blend_{index}"));
        }
        for set in self.parameter_sets.values_mut() {
            set.resize(new_columns, false);
        }
    }
}
