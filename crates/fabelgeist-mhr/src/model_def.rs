//! The Momentum model-definition transform and its structured text admission.
mod error;
mod identity;
mod lexical;
mod limit;
mod sections;
mod sets_limits;
mod terms;
use crate::character::Skeleton;
pub use error::{DefinitionLayout, DefinitionNumberRole, ModelDefinitionError};
use fabelgeist_fs::FileText;
use identity::BlendShapeOrdinal;
pub use identity::{
    BlendShapeParameterCount, JointParameterChannel, JointParameterRow, ModelParameterCount,
    ModelParameterIndex, ModelParameterName, ParameterSetName,
};
pub use lexical::{DefinitionLine, RejectedDefinitionToken};
pub use limit::{ParameterBounds, ParameterBoundsError, SolverLimitWeight};
use sections::DefinitionSections;
use std::collections::HashMap;
use terms::TransformBuilder;

/// Inclusive bounds on one model parameter. Solver weight does not affect clamping.
#[derive(Debug, Clone, Copy)]
pub struct ParameterLimit {
    parameter: ModelParameterIndex,
    bounds: ParameterBounds,
    weight: SolverLimitWeight,
}
impl ParameterLimit {
    pub fn parameter(&self) -> ModelParameterIndex {
        self.parameter
    }
    pub fn bounds(&self) -> ParameterBounds {
        self.bounds
    }
    pub fn weight(&self) -> SolverLimitWeight {
        self.weight
    }
}
/// Row-major map: joint parameters = transform * model parameters + offsets.
#[derive(Debug, Default, Clone)]
pub struct ParameterTransform {
    pub(crate) names: Vec<ModelParameterName>,
    pub(crate) transform: Vec<f32>,
    pub(crate) offsets: Vec<f32>,
    pub(crate) active_joint_parameters: Vec<bool>,
    pub(crate) parameter_sets: HashMap<ParameterSetName, Vec<bool>>,
    pub(crate) limits: Vec<ParameterLimit>,
    pub(crate) num_joint_parameters: usize,
}
impl ParameterTransform {
    pub fn from_definition(
        text: &FileText,
        skeleton: &Skeleton,
    ) -> Result<Self, ModelDefinitionError> {
        let sections = DefinitionSections::try_from(text)?;
        let mut builder = TransformBuilder::from_skeleton(skeleton)?;
        builder.read_transform(&sections.transform)?;
        let mut transform = builder.finish()?;
        transform.read_sets(&sections.sets);
        transform.read_limits(&sections.limits)?;
        Ok(transform)
    }
    pub fn names(&self) -> impl Iterator<Item = &ModelParameterName> {
        self.names.iter()
    }
    pub fn limits(&self) -> impl Iterator<Item = &ParameterLimit> {
        self.limits.iter()
    }
    pub fn num_parameters(&self) -> ModelParameterCount {
        ModelParameterCount::from(self.names.len())
    }
    pub fn parameter_index(&self, name: &ModelParameterName) -> Option<ModelParameterIndex> {
        self.names
            .iter()
            .position(|n: &ModelParameterName| -> bool { n == name })
            .map(ModelParameterIndex)
    }
    /// Row `joint * 7 + channel` of the transform matrix.
    pub fn row(&self, joint_parameter: JointParameterRow) -> &[f32] {
        let stride = usize::from(self.num_parameters());
        &self.transform[joint_parameter.0 * stride..(joint_parameter.0 + 1) * stride]
    }
    pub fn apply_limits(&self, parameters: &mut [f32]) {
        for limit in &self.limits {
            if let Some(value) = parameters.get_mut(limit.parameter.0) {
                *value = value.clamp(limit.bounds.minimum, limit.bounds.maximum);
            }
        }
    }
    /// Adds blend-shape columns that do not drive joint channels.
    pub fn append_blend_shapes(
        &mut self,
        count: BlendShapeParameterCount,
    ) -> Result<(), ModelDefinitionError> {
        let old_columns = usize::from(self.num_parameters());
        let new_columns =
            old_columns
                .checked_add(count.0)
                .ok_or(ModelDefinitionError::LayoutOverflow(
                    DefinitionLayout::ParameterColumns,
                ))?;
        let entries = self.num_joint_parameters.checked_mul(new_columns).ok_or(
            ModelDefinitionError::LayoutOverflow(DefinitionLayout::DenseTransform),
        )?;
        let mut dense = vec![0.0; entries];
        for row in 0..self.num_joint_parameters {
            let source = &self.transform[row * old_columns..(row + 1) * old_columns];
            dense[row * new_columns..row * new_columns + old_columns].copy_from_slice(source);
        }
        self.transform = dense;
        for index in 0..count.0 {
            self.names
                .push(ModelParameterName::blend_shape(BlendShapeOrdinal(index)));
        }
        for set in self.parameter_sets.values_mut() {
            set.resize(new_columns, false);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
