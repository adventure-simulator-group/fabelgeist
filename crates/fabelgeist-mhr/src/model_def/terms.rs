//! Ordered linear contributions, including references to earlier joint rows.
use super::identity::{JointParameterRow, ModelParameterIndex};
use super::lexical::{
    DefinitionLine, DefinitionLines, DefinitionToken, JointSpellingContext,
    RejectedDefinitionToken, TokenSeparator,
};
use super::{DefinitionLayout, DefinitionNumberRole, ModelDefinitionError, ParameterTransform};
use crate::character::{PARAMETERS_PER_JOINT, Skeleton};
use fabelgeist_rig::{RigJointName, RigJointOrdinal};
use std::collections::HashMap;

pub(super) struct SkeletonBindings(HashMap<RigJointName, RigJointOrdinal>);
impl From<&Skeleton> for SkeletonBindings {
    fn from(skeleton: &Skeleton) -> Self {
        let mut joints = HashMap::new();
        // Preserve the original position lookup's first duplicate-name match.
        for (index, name) in skeleton.names.iter().enumerate() {
            joints
                .entry(name.clone())
                .or_insert(RigJointOrdinal::from(index));
        }
        Self(joints)
    }
}
impl SkeletonBindings {
    fn target(
        &self,
        token: DefinitionToken<'_>,
        line: &DefinitionLine,
    ) -> Result<JointParameterRow, ModelDefinitionError> {
        let (name, channel) = token
            .joint_channel(JointSpellingContext::Target)
            .ok_or_else(|| -> ModelDefinitionError {
                ModelDefinitionError::InvalidTarget(line.clone())
            })?;
        let joint = self.0.get(&name).ok_or_else(|| -> ModelDefinitionError {
            ModelDefinitionError::UnknownJoint {
                line: line.clone(),
                joint: name,
            }
        })?;
        let channel = channel.channel().ok_or_else(|| -> ModelDefinitionError {
            ModelDefinitionError::UnknownChannel {
                line: line.clone(),
                token: RejectedDefinitionToken::from(channel),
            }
        })?;
        Ok(channel.row(*joint))
    }
    fn reference(&self, token: DefinitionToken<'_>) -> Option<JointParameterRow> {
        let (joint, channel) = token.joint_channel(JointSpellingContext::Reference)?;
        Some(channel.channel()?.row(*self.0.get(&joint)?))
    }
}
#[derive(Clone, Copy)]
pub(super) struct ExpressionWeight(pub(super) f32);
impl ExpressionWeight {
    pub(super) fn parse(
        token: DefinitionToken<'_>,
        line: &DefinitionLine,
        role: DefinitionNumberRole,
    ) -> Result<Self, ModelDefinitionError> {
        token.0.parse::<f32>().map(Self).map_err(
            |source: std::num::ParseFloatError| -> ModelDefinitionError {
                ModelDefinitionError::Number {
                    line: line.clone(),
                    role,
                    token: RejectedDefinitionToken::from(token),
                    source,
                }
            },
        )
    }
    fn multiplied(self, other: Self) -> Self {
        Self(self.0 * other.0)
    }
}
#[derive(Clone, Copy)]
pub(super) struct ParameterContribution {
    row: JointParameterRow,
    column: ModelParameterIndex,
    weight: ExpressionWeight,
}
pub(super) struct TransformBuilder {
    pub(super) transform: ParameterTransform,
    contributions: Vec<ParameterContribution>,
    bindings: SkeletonBindings,
}
impl TransformBuilder {
    pub(super) fn from_skeleton(skeleton: &Skeleton) -> Result<Self, ModelDefinitionError> {
        let rows = skeleton.len().checked_mul(PARAMETERS_PER_JOINT).ok_or(
            ModelDefinitionError::LayoutOverflow(DefinitionLayout::JointRows),
        )?;
        Ok(Self {
            transform: ParameterTransform {
                offsets: vec![0.0; rows],
                active_joint_parameters: vec![false; rows],
                num_joint_parameters: rows,
                ..Default::default()
            },
            contributions: Vec::new(),
            bindings: SkeletonBindings::from(skeleton),
        })
    }
    pub(super) fn read_transform(
        &mut self,
        lines: &DefinitionLines,
    ) -> Result<(), ModelDefinitionError> {
        for line in &lines.0 {
            let Some((target, expression)) = line.assignment() else {
                continue;
            };
            let row = self.bindings.target(target, line)?;
            self.transform.active_joint_parameters[row.0] = true;
            self.read_expression(expression, row, line)?;
        }
        Ok(())
    }
    fn read_expression(
        &mut self,
        expression: DefinitionToken<'_>,
        row: JointParameterRow,
        line: &DefinitionLine,
    ) -> Result<(), ModelDefinitionError> {
        for term in expression.tokens(TokenSeparator::Terms).0 {
            let factors = term.tokens(TokenSeparator::Factors);
            match factors.0.as_slice() {
                [constant] => {
                    // Preserve the supported subset: nonnumeric bare terms are ignored.
                    if let Ok(weight) = constant.0.parse::<f32>() {
                        self.transform.offsets[row.0] = weight;
                    }
                }
                [weight, name] => {
                    let weight = ExpressionWeight::parse(
                        *weight,
                        line,
                        DefinitionNumberRole::ExpressionWeight,
                    )?;
                    self.append_term(*name, row, weight);
                }
                _ => (),
            }
        }
        Ok(())
    }
    fn append_term(
        &mut self,
        name: DefinitionToken<'_>,
        row: JointParameterRow,
        weight: ExpressionWeight,
    ) {
        let name_identity = name.parameter_name();
        if let Some(column) = self.transform.parameter_index(&name_identity) {
            self.contributions.push(ParameterContribution {
                row,
                column,
                weight,
            });
        } else if let Some(reference) = self.bindings.reference(name) {
            // Snapshot length prevents self references from growing their own loop.
            for index in 0..self.contributions.len() {
                let source = self.contributions[index];
                if source.row == reference {
                    self.contributions.push(ParameterContribution {
                        row,
                        column: source.column,
                        weight: source.weight.multiplied(weight),
                    });
                }
            }
        } else {
            let column = ModelParameterIndex(self.transform.names.len());
            self.transform.names.push(name_identity);
            self.contributions.push(ParameterContribution {
                row,
                column,
                weight,
            });
        }
    }
    pub(super) fn finish(mut self) -> Result<ParameterTransform, ModelDefinitionError> {
        let columns = usize::from(self.transform.num_parameters());
        let entries = self
            .transform
            .num_joint_parameters
            .checked_mul(columns)
            .ok_or(ModelDefinitionError::LayoutOverflow(
                DefinitionLayout::DenseTransform,
            ))?;
        self.transform.transform = vec![0.0; entries];
        for contribution in self.contributions {
            if contribution.weight.0 != 0.0 {
                self.transform.transform[contribution.row.0 * columns + contribution.column.0] +=
                    contribution.weight.0;
            }
        }
        Ok(self.transform)
    }
}
