//! Ordered Stream roles, including optional input and writer metadata.

use crate::prelude::*;
use crate::source_parameter::{NativeParameter, SourceParameter};
use fabelgeist_gpu::data::gpu::signature::{
    CoordinateParameter, CoordinateShape, ShaderParameter, parse_base_type,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StreamSignature {
    parameters: Vec<SourceParameter<StreamParameterRole>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) enum StreamParameterRole {
    Index(CoordinateParameter),
    Offset(ShaderParameter),
    Resource(ShaderParameter),
    Value(ShaderParameter),
}

impl StreamSignature {
    pub fn parse(code: &str) -> Result<Self> {
        let re = regex::Regex::new(r"(?s)fn\s+stream\s*\(([^)]*)\)\s*\{")?;
        let caps = re.captures(code).ok_or_else(|| {
            anyhow!("Compute shader must define a 'stream' function: fn stream(...)")
        })?;
        let parameters = NativeParameter::parse(&caps[1], "stream")?;
        let coordinate = parameters
            .iter()
            .find(|parameter| {
                parameter.name == "index"
                    && CoordinateShape::from_native_type(parameter.native_type).is_some()
            })
            .or_else(|| {
                parameters.iter().find(|parameter| {
                    parameter.name != "offset"
                        && CoordinateShape::from_native_type(parameter.native_type).is_some()
                })
            })
            .map(|parameter| parameter.name);
        let parameters = parameters
            .iter()
            .map(|parameter| {
                let role = if parameter.name == "offset" {
                    if parameter.native_type != "u32" {
                        return Err(anyhow!("Offset must be u32"));
                    }
                    StreamParameterRole::Offset(ShaderParameter::new(
                        parameter.name,
                        ResourceBaseType::U32,
                    ))
                } else if Some(parameter.name) == coordinate {
                    StreamParameterRole::Index(CoordinateParameter::from_native(
                        parameter.name,
                        parameter.native_type,
                    )?)
                } else {
                    let value = ShaderParameter::new(
                        parameter.name,
                        parse_base_type(parameter.native_type),
                    );
                    if parameter.is_resource() {
                        StreamParameterRole::Resource(value)
                    } else {
                        StreamParameterRole::Value(value)
                    }
                };
                Ok(SourceParameter::new(parameter.declaration, role))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { parameters })
    }

    pub fn input_element_type(&self) -> &ResourceBaseType {
        self.roles()
            .rev()
            .find_map(|role| match role {
                StreamParameterRole::Value(value) => Some(value.value_type()),
                _ => None,
            })
            .unwrap_or(&ResourceBaseType::F32)
    }

    pub fn output_element_type(&self) -> &ResourceBaseType {
        self.roles()
            .rev()
            .find_map(|role| match role {
                StreamParameterRole::Resource(value) => Some(value.value_type()),
                _ => None,
            })
            .unwrap_or(&ResourceBaseType::F32)
    }

    pub fn has_input_value(&self) -> bool {
        self.roles()
            .any(|role| matches!(role, StreamParameterRole::Value(_)))
    }

    pub(super) fn roles(&self) -> impl DoubleEndedIterator<Item = &StreamParameterRole> {
        self.parameters.iter().map(SourceParameter::role)
    }

    pub(super) fn declarations(&self) -> impl Iterator<Item = &str> {
        self.parameters
            .iter()
            .filter(|parameter| !matches!(parameter.role(), StreamParameterRole::Resource(_)))
            .map(SourceParameter::declaration)
    }
}
