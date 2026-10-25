//! Exact neighborhood source protocol and its admitted native payload.

use crate::prelude::*;
use fabelgeist_gpu::data::gpu::signature::{ShaderParameter, parse_base_type};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct NeighborhoodParameter {
    parameter: ShaderParameter,
    dimension: NeighborhoodDimension,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum NeighborhoodDimension {
    One,
    Two,
    Three,
    Four,
}

impl NeighborhoodParameter {
    pub(super) fn from_native(name: &str, native_type: &str) -> Result<Option<Self>> {
        let native_type = native_type.trim();
        let outer = native_type
            .split_once('<')
            .map(|(outer, _)| outer.trim())
            .unwrap_or(native_type);
        let dimension = match outer {
            "Neighbors1D" => NeighborhoodDimension::One,
            "Neighbors2D" => NeighborhoodDimension::Two,
            "Neighbors3D" => NeighborhoodDimension::Three,
            "Neighbors4D" => NeighborhoodDimension::Four,
            _ => {
                if outer
                    .strip_prefix("Neighbors")
                    .and_then(|dimension| dimension.strip_suffix('D'))
                    .is_some_and(|dimension| {
                        !dimension.is_empty()
                            && dimension
                                .chars()
                                .all(|character| character.is_ascii_digit())
                    })
                {
                    return Err(anyhow!("Unsupported neighborhood type: {native_type}"));
                }
                return Ok(None);
            }
        };
        let inner = native_type
            .strip_suffix('>')
            .and_then(|body| body.split_once('<'))
            .map(|(_, inner)| inner.trim())
            .ok_or_else(|| anyhow!("Invalid neighborhood type: {native_type}"))?;
        Ok(Some(Self {
            parameter: ShaderParameter::new(name, parse_base_type(inner)),
            dimension,
        }))
    }

    pub(super) fn name(&self) -> &str {
        self.parameter.name()
    }

    pub(super) fn element_type(&self) -> &ResourceBaseType {
        self.parameter.value_type()
    }

    pub(super) fn dimension(&self) -> NeighborhoodDimension {
        self.dimension
    }
}
