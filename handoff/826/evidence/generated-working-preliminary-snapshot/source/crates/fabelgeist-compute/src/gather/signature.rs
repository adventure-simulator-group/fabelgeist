//! Authored-order Gather roles and native resource metadata.

use crate::prelude::*;
use crate::source_parameter::{NativeParameter, SourceParameter};
use fabelgeist_gpu::data::gpu::signature::{
    CoordinateParameter, CoordinateShape, ShaderParameter, parse_base_type,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GatherSignature {
    parameters: Vec<SourceParameter<GatherParameterRole>>,
    output_element_type: ResourceBaseType,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum GatherParameterRole {
    Index(CoordinateParameter),
    Resource(ShaderParameter),
    Other(ShaderParameter),
}

impl GatherSignature {
    pub fn parse(code: &str) -> Result<Self> {
        let re = regex::Regex::new(r"(?s)fn\s+gather\s*\(([^)]*)\)\s*(?:->\s*([^\{]+))?\{")?;
        let caps = re
            .captures(code)
            .ok_or_else(|| anyhow!("Compute shader must define a 'gather' function"))?;
        let parameters = NativeParameter::parse(&caps[1], "gather")?;
        let coordinate = parameters
            .iter()
            .find(|parameter| {
                parameter.name == "index"
                    && CoordinateShape::from_native_type(parameter.native_type).is_some()
            })
            .or_else(|| {
                parameters.iter().find(|parameter| {
                    CoordinateShape::from_native_type(parameter.native_type).is_some()
                })
            })
            .map(|parameter| parameter.name);
        let parameters = parameters
            .iter()
            .map(|parameter| {
                let role = if Some(parameter.name) == coordinate {
                    GatherParameterRole::Index(CoordinateParameter::from_native(
                        parameter.name,
                        parameter.native_type,
                    )?)
                } else {
                    let value = ShaderParameter::new(
                        parameter.name,
                        parse_base_type(parameter.native_type),
                    );
                    if parameter.is_resource() {
                        GatherParameterRole::Resource(value)
                    } else {
                        GatherParameterRole::Other(value)
                    }
                };
                Ok(SourceParameter::new(parameter.declaration, role))
            })
            .collect::<Result<Vec<_>>>()?;
        let output = caps
            .get(2)
            .ok_or_else(|| anyhow!("Gather function must return a type."))?;
        Ok(Self {
            parameters,
            output_element_type: parse_base_type(output.as_str().trim()),
        })
    }

    pub fn index(&self) -> Option<&CoordinateParameter> {
        self.parameters
            .iter()
            .find_map(|parameter| match parameter.role() {
                GatherParameterRole::Index(index) => Some(index),
                _ => None,
            })
    }

    pub fn input_element_type(&self) -> &ResourceBaseType {
        self.parameters
            .iter()
            .rev()
            .find_map(|parameter| match parameter.role() {
                GatherParameterRole::Resource(value) => Some(value.value_type()),
                _ => None,
            })
            .unwrap_or(&ResourceBaseType::F32)
    }

    pub fn output_element_type(&self) -> &ResourceBaseType {
        &self.output_element_type
    }

    pub(super) fn declarations(&self) -> impl Iterator<Item = &str> {
        self.parameters
            .iter()
            .filter(|parameter| !matches!(parameter.role(), GatherParameterRole::Resource(_)))
            .map(SourceParameter::declaration)
    }

    pub(super) fn call_arguments(&self) -> Vec<&str> {
        self.index().map(|_| "index").into_iter().collect()
    }
}
