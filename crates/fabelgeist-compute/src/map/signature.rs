//! Authored-order Map roles; primary input is selected by the actual call.

use crate::prelude::*;
use crate::source_error::{GeneratedFunction, GeneratedProgramError};
use crate::source_parameter::{NativeParameter, SourceParameter};
use fabelgeist_gpu::data::gpu::signature::{CoordinateParameter, ShaderParameter, parse_base_type};

/// One immutable ordered signature admitted from a native map declaration.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MapSignature {
    parameters: Vec<SourceParameter<MapParameterRole>>,
    output_element_type: ResourceBaseType,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum MapParameterRole {
    Index(CoordinateParameter),
    Size(CoordinateParameter),
    Value(ShaderParameter),
}

impl MapSignature {
    pub fn parse(code: &str) -> Result<Self> {
        let re = regex::Regex::new(r"(?s)fn\s+map\s*\(([^)]*)\)\s*(?:->\s*([^\{]+))?\{")?;
        let caps = re
            .captures(code)
            .ok_or(GeneratedProgramError::MissingFunction {
                function: GeneratedFunction::Map,
            })?;
        let parameters = NativeParameter::parse(&caps[1], GeneratedFunction::Map)?
            .into_iter()
            .map(|parameter| {
                let role = match parameter.name {
                    "index" => MapParameterRole::Index(CoordinateParameter::from_native(
                        parameter.name,
                        parameter.native_type,
                    )?),
                    "size" => MapParameterRole::Size(CoordinateParameter::from_native(
                        parameter.name,
                        parameter.native_type,
                    )?),
                    _ => MapParameterRole::Value(ShaderParameter::new(
                        parameter.name,
                        parse_base_type(parameter.native_type),
                    )),
                };
                Ok(SourceParameter::new(parameter.declaration, role))
            })
            .collect::<Result<Vec<_>>>()?;
        let output = caps.get(2).ok_or(GeneratedProgramError::MissingReturn {
            function: GeneratedFunction::Map,
        })?;
        Ok(Self {
            parameters,
            output_element_type: parse_base_type(output.as_str().trim()),
        })
    }

    pub fn index(&self) -> Option<&CoordinateParameter> {
        self.parameters
            .iter()
            .find_map(|parameter| match parameter.role() {
                MapParameterRole::Index(index) => Some(index),
                _ => None,
            })
    }

    pub fn size(&self) -> Option<&CoordinateParameter> {
        self.parameters
            .iter()
            .find_map(|parameter| match parameter.role() {
                MapParameterRole::Size(size) => Some(size),
                _ => None,
            })
    }

    pub fn user_parameters(&self) -> impl DoubleEndedIterator<Item = &ShaderParameter> {
        self.parameters
            .iter()
            .filter_map(|parameter| match parameter.role() {
                MapParameterRole::Value(value) => Some(value),
                _ => None,
            })
    }

    pub fn output_element_type(&self) -> &ResourceBaseType {
        &self.output_element_type
    }

    /// Original trimmed declarations, including native alias and wrapper spelling.
    pub fn declarations(&self) -> impl Iterator<Item = &str> {
        self.parameters.iter().map(SourceParameter::declaration)
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.parameters
            .iter()
            .map(|parameter| parameter.role().name())
    }

    pub(super) fn call_arguments(&self, primary: Option<&ShaderParameter>) -> Vec<String> {
        self.parameters
            .iter()
            .map(|parameter| match parameter.role() {
                MapParameterRole::Index(_) => "index".to_string(),
                MapParameterRole::Size(_) => "size".to_string(),
                MapParameterRole::Value(value)
                    if primary.map(ShaderParameter::name) == Some(value.name()) =>
                {
                    "in_val".to_string()
                }
                MapParameterRole::Value(value) => value.name().to_owned(),
            })
            .collect()
    }
}

impl MapParameterRole {
    fn name(&self) -> &str {
        match self {
            Self::Index(parameter) | Self::Size(parameter) => parameter.name(),
            Self::Value(parameter) => parameter.name(),
        }
    }
}
