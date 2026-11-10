//! Parser for the "Momentum Model Definition V1.0" text format
//! (`compact_v6_1.model`), which maps model parameters onto joint parameters.

use crate::character::Skeleton;
use anyhow::{Result, bail};
use fabelgeist_rig::RigJointName;
use std::collections::HashMap;

pub use layout::{
    JointChannel, JointParameterRow, JointParameterTable, ModelParameterColumn, ModelParameterName,
    ModelParameterRow, ModelParameterTable, ParameterMatrix, ParameterSetName,
};

mod layout;

/// One contribution to a typed joint-channel row and model-parameter column.
#[derive(Clone, Copy)]
struct Contribution {
    row: JointParameterRow,
    column: ModelParameterColumn,
    weight: f32,
}

struct ParsedTransform {
    transform: ParameterTransform,
    contributions: Vec<Contribution>,
}

/// Inclusive bounds on one model parameter.
#[derive(Debug, Clone, Copy)]
pub struct ParameterLimit {
    pub parameter: ModelParameterColumn,
    pub min: f32,
    pub max: f32,
    /// How strongly a solver should enforce the bound; 1.0 unless the file says
    /// otherwise. Clamping ignores it.
    pub weight: f32,
}

/// The linear map from model parameters to joint parameters.
///
/// `joint_parameters = transform * model_parameters + offsets`, with
/// `transform` stored row-major as `[num_joints * 7, names.len()]`.
#[derive(Debug, Default, Clone)]
pub struct ParameterTransform {
    /// Model parameter names, in the order the `.model` file introduces them.
    pub names: ModelParameterTable<ModelParameterName>,
    pub transform: ParameterMatrix,
    pub offsets: JointParameterTable<f32>,
    /// Joint channels that any parameter drives.
    pub active_joint_parameters: JointParameterTable<bool>,
    /// Named parameter subsets from `[ParameterSets]`.
    pub parameter_sets: HashMap<ParameterSetName, ModelParameterTable<bool>>,
    pub limits: Vec<ParameterLimit>,
}

impl ParameterTransform {
    pub fn num_parameters(&self) -> usize {
        self.names.len()
    }

    pub fn parameter_index(&self, name: &ModelParameterName) -> Option<ModelParameterColumn> {
        self.names.column(name)
    }

    /// Coefficients for one joint channel, retaining model-column identity.
    pub fn row(&self, joint_parameter: JointParameterRow) -> ModelParameterRow<'_> {
        self.transform.row(joint_parameter)
    }

    /// Clamps model parameters to the `[Limits]` section's `minmax` bounds.
    pub fn apply_limits(&self, parameters: &mut [f32]) {
        for limit in &self.limits {
            if let Some(value) = parameters.get_mut(limit.parameter.native_index()) {
                *value = value.clamp(limit.min, limit.max);
            }
        }
    }
}

/// Parses a `.model` file against a skeleton, producing the parameter transform.
pub fn parse_model_definition(text: &str, skeleton: &Skeleton) -> Result<ParameterTransform> {
    let sections = split_sections(text)?;
    let empty = Vec::new();
    let ParsedTransform {
        mut transform,
        contributions,
    } = parse_parameter_transform(
        sections.get("ParameterTransform").unwrap_or(&empty),
        skeleton,
    )?;

    // Densify once the parameter count is final.
    let columns = transform.num_parameters();
    transform.transform = ParameterMatrix::zeros(skeleton.len(), columns);
    for contribution in contributions {
        if contribution.weight != 0.0 {
            transform
                .transform
                .add(contribution.row, contribution.column, contribution.weight);
        }
    }

    if let Some(lines) = sections.get("ParameterSets") {
        parse_parameter_sets(lines, &mut transform);
    }
    if let Some(lines) = sections.get("Limits") {
        parse_limits(lines, &mut transform);
    }

    Ok(transform)
}

/// Appends one column per blend-shape coefficient, as momentum's
/// `Character::withBlendShape` does. The new columns drive no joint.
pub fn append_blend_shape_parameters(transform: &mut ParameterTransform, count: usize) {
    let old_columns = transform.num_parameters();
    let new_columns = old_columns + count;
    transform.transform.append_columns(new_columns);
    for index in 0..count {
        transform
            .names
            .push(ModelParameterName::from(format!("blend_{index}")));
    }
    for set in transform.parameter_sets.values_mut() {
        set.resize(new_columns, false);
    }
}

/// Splits on any of `delimiters`, dropping empty fragments, and trims each token.
fn tokenize<'a>(text: &'a str, delimiters: &str) -> Vec<&'a str> {
    text.split(|c| delimiters.contains(c))
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .collect()
}

fn strip_comment(line: &str) -> &str {
    match line.find('#') {
        Some(pos) => line[..pos].trim(),
        None => line.trim(),
    }
}

/// Splits the file into its `[Section]` blocks.
fn split_sections(text: &str) -> Result<HashMap<String, Vec<String>>> {
    let mut lines = text.lines();
    let mut saw_header = false;
    for line in lines.by_ref() {
        let line = strip_comment(line);
        if line.is_empty() {
            continue;
        }
        if line == "Momentum Model Definition V1.0" {
            saw_header = true;
            break;
        }
        bail!("invalid model definition file; got {line:?}");
    }
    if !saw_header {
        bail!("invalid model definition file; missing the version header");
    }

    let mut sections: HashMap<String, Vec<String>> = HashMap::new();
    let mut current: Option<String> = None;
    for line in lines {
        let line = strip_comment(line);
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            current = Some(line[1..line.len() - 1].to_string());
            sections
                .entry(line[1..line.len() - 1].to_string())
                .or_default();
            continue;
        }
        if let Some(section) = &current {
            sections
                .entry(section.clone())
                .or_default()
                .push(line.to_string());
        }
    }
    Ok(sections)
}

/// Parses one right-hand side, e.g. `1.0 * spine0_rx + 0.22 * spine_lean0`.
fn parse_expression(
    triplets: &mut Vec<Contribution>,
    transform: &mut ParameterTransform,
    skeleton: &Skeleton,
    expression: &str,
    row: JointParameterRow,
    line: &str,
) -> Result<()> {
    for term in tokenize(expression, "+") {
        let factors = tokenize(term, "*");
        if factors.len() == 1 {
            // A bare constant is an offset on this joint channel.
            let Ok(weight) = factors[0].parse::<f32>() else {
                continue;
            };
            transform.offsets[row] = weight;
            continue;
        }
        if factors.len() != 2 {
            continue;
        }

        let Ok(weight) = factors[0].parse::<f32>() else {
            bail!("could not parse weight in: {line}");
        };
        let name = factors[1];

        // The right side may name either a model parameter or a joint channel
        // defined earlier in the file, in which case its terms are copied.
        let parameter_name = ModelParameterName::from(name);
        let parameter = transform.parameter_index(&parameter_name);
        let reference = match name.split_once('.') {
            Some((joint_name, channel)) => {
                let joint_name = RigJointName::from(joint_name);
                skeleton.joint_index(&joint_name).and_then(|joint| {
                    JointChannel::from_model_token(channel)
                        .map(|channel| JointParameterRow::new(joint, channel))
                })
            }
            None => None,
        };

        match (parameter, reference) {
            (Some(column), _) => triplets.push(Contribution {
                row,
                column,
                weight,
            }),
            (None, Some(reference)) => {
                for index in 0..triplets.len() {
                    let source = triplets[index];
                    if source.row == reference {
                        triplets.push(Contribution {
                            row,
                            column: source.column,
                            weight: source.weight * weight,
                        });
                    }
                }
            }
            (None, None) => {
                let column = transform.names.push(parameter_name);
                triplets.push(Contribution {
                    row,
                    column,
                    weight,
                });
            }
        }
    }
    Ok(())
}

fn parse_parameter_transform(lines: &[String], skeleton: &Skeleton) -> Result<ParsedTransform> {
    let mut transform = ParameterTransform {
        offsets: JointParameterTable::filled(skeleton.len(), 0.0),
        active_joint_parameters: JointParameterTable::filled(skeleton.len(), false),
        ..Default::default()
    };
    let mut triplets: Vec<Contribution> = Vec::new();

    for line in lines {
        let Some((left, right)) = line.split_once('=') else {
            continue;
        };
        let Some((joint_name, channel_name)) = left.trim().split_once('.') else {
            bail!("unknown joint name in expression: {line}");
        };
        // The .model file is raw text; admit its exact label after the
        // format's whitespace handling, then retain it for rig lookup.
        let joint_name = RigJointName::from(joint_name.trim());
        let Some(joint) = skeleton.joint_index(&joint_name) else {
            bail!("unknown joint name in expression: {line}");
        };
        let Some(channel) = JointChannel::from_model_token(channel_name.trim()) else {
            bail!("unknown channel name in expression: {line}");
        };

        let row = JointParameterRow::new(joint, channel);
        transform.active_joint_parameters[row] = true;
        parse_expression(&mut triplets, &mut transform, skeleton, right, row, line)?;
    }

    Ok(ParsedTransform {
        transform,
        contributions: triplets,
    })
}

fn parse_parameter_sets(lines: &[String], transform: &mut ParameterTransform) {
    for line in lines {
        let tokens = tokenize(line, " \t");
        if tokens.len() < 2 || tokens[0] != "parameterset" {
            continue;
        }
        let mut set = ModelParameterTable::filled(transform.num_parameters(), false);
        for name in &tokens[2..] {
            if let Some(index) = transform.parameter_index(&ModelParameterName::from(*name)) {
                set[index] = true;
            }
        }
        transform
            .parameter_sets
            .insert(ParameterSetName::from(tokens[1]), set);
    }
}

fn parse_limits(lines: &[String], transform: &mut ParameterTransform) {
    for line in lines {
        let tokens = tokenize(line, " \t");
        // `limit <parameter> minmax [min, max]`; momentum's other limit kinds
        // (minmaxjoint, linear, ellipsoid, halfplane) are unused by MHR.
        if tokens.len() < 4 || tokens[0] != "limit" || tokens[2] != "minmax" {
            continue;
        }
        let Some(parameter) = transform.parameter_index(&ModelParameterName::from(tokens[1]))
        else {
            continue;
        };
        // `[min, max]`, optionally followed by a weight.
        let (Some(open), Some(close)) = (line.find('['), line.find(']')) else {
            continue;
        };
        let bounds: Vec<f32> = tokenize(&line[open + 1..close], ",")
            .iter()
            .filter_map(|t| t.parse::<f32>().ok())
            .collect();
        if bounds.len() == 2 {
            transform.limits.push(ParameterLimit {
                parameter,
                min: bounds[0],
                max: bounds[1],
                weight: line[close + 1..].trim().parse().unwrap_or(1.0),
            });
        }
    }
}

#[cfg(test)]
mod tests;
