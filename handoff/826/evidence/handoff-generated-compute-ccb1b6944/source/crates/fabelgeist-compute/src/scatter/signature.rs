//! Ordered Scatter coordinates, input values and native writer metadata.

use crate::prelude::*;
use crate::source_error::{GeneratedFunction, GeneratedProgramError};
use crate::source_parameter::{NativeParameter, SourceParameter};
use fabelgeist_gpu::data::gpu::signature::{
    CoordinateParameter, CoordinateShape, ShaderParameter, parse_base_type,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ScatterSignature {
    parameters: Vec<SourceParameter<ScatterParameterRole>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum ScatterParameterRole {
    Index(CoordinateParameter),
    Resource(ShaderParameter),
    Value(ShaderParameter),
}

impl ScatterSignature {
    pub fn parse(code: &str) -> Result<Self> {
        let re = regex::Regex::new(r"(?s)fn\s+scatter\s*\(([^)]*)\)\s*\{")?;
        let caps = re
            .captures(code)
            .ok_or(GeneratedProgramError::MissingFunction {
                function: GeneratedFunction::Scatter,
            })?;
        let parameters = NativeParameter::parse(&caps[1], GeneratedFunction::Scatter)?;
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
                    ScatterParameterRole::Index(CoordinateParameter::from_native(
                        parameter.name,
                        parameter.native_type,
                    )?)
                } else {
                    let value = ShaderParameter::new(
                        parameter.name,
                        parse_base_type(parameter.native_type),
                    );
                    if parameter.is_resource() {
                        ScatterParameterRole::Resource(value)
                    } else {
                        ScatterParameterRole::Value(value)
                    }
                };
                Ok(SourceParameter::new(parameter.declaration, role))
            })
            .collect::<Result<Vec<_>>>()?;
        if !parameters
            .iter()
            .any(|parameter| matches!(parameter.role(), ScatterParameterRole::Value(_)))
        {
            return Err(GeneratedProgramError::MissingScatterValue.into());
        }
        Ok(Self { parameters })
    }

    pub fn index(&self) -> Option<&CoordinateParameter> {
        self.parameters
            .iter()
            .find_map(|parameter| match parameter.role() {
                ScatterParameterRole::Index(index) => Some(index),
                _ => None,
            })
    }

    pub fn input_element_type(&self) -> &ResourceBaseType {
        self.parameters
            .iter()
            .rev()
            .find_map(|parameter| match parameter.role() {
                ScatterParameterRole::Value(value) => Some(value.value_type()),
                _ => None,
            })
            .unwrap_or(&ResourceBaseType::F32)
    }

    pub fn output_element_type(&self) -> &ResourceBaseType {
        self.parameters
            .iter()
            .rev()
            .find_map(|parameter| match parameter.role() {
                ScatterParameterRole::Resource(value) => Some(value.value_type()),
                _ => None,
            })
            .unwrap_or(&ResourceBaseType::F32)
    }

    pub(super) fn declarations(&self) -> impl Iterator<Item = &str> {
        self.parameters
            .iter()
            .filter(|parameter| !matches!(parameter.role(), ScatterParameterRole::Resource(_)))
            .map(SourceParameter::declaration)
    }

    pub(super) fn call_arguments(&self) -> Vec<&str> {
        self.parameters
            .iter()
            .filter_map(|parameter| match parameter.role() {
                ScatterParameterRole::Index(_) => Some("index"),
                ScatterParameterRole::Value(_) => Some("in_val"),
                ScatterParameterRole::Resource(_) => None,
            })
            .collect()
    }
}
