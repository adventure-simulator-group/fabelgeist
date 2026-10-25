//! One authored-order Stencil schema, including optional neighborhood center.

use super::neighborhood::NeighborhoodParameter;
use crate::prelude::*;
use crate::source_parameter::{NativeParameter, SourceParameter};
use fabelgeist_gpu::data::gpu::signature::{CoordinateParameter, ShaderParameter, parse_base_type};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StencilSignature {
    parameters: Vec<SourceParameter<StencilParameterRole>>,
    output_element_type: ResourceBaseType,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) enum StencilParameterRole {
    Index(CoordinateParameter),
    Size(CoordinateParameter),
    Input(ShaderParameter),
    Center(ShaderParameter),
    Neighborhood(NeighborhoodParameter),
    Value(ShaderParameter),
}

impl StencilSignature {
    pub fn parse(code: &str) -> Result<Self> {
        let re = regex::Regex::new(r"(?s)fn\s+stencil\s*\(([^)]*)\)\s*(?:->\s*([^\{]+))?\{")?;
        let caps = re
            .captures(code)
            .ok_or_else(|| anyhow!("Stencil shader must define a 'stencil' function"))?;
        let parameters = NativeParameter::parse(&caps[1], "stencil")?;
        let neighborhoods = parameters
            .iter()
            .map(|parameter| {
                NeighborhoodParameter::from_native(parameter.name, parameter.native_type)
            })
            .collect::<Result<Vec<_>>>()?;
        let neighborhood_count = neighborhoods.iter().flatten().count();
        if neighborhood_count > 1 {
            return Err(anyhow!("Stencil accepts one neighborhood parameter"));
        }
        let center = if neighborhood_count == 1 {
            Self::center_name(&parameters, &neighborhoods)?
        } else {
            None
        };
        let parameters = parameters
            .iter()
            .zip(neighborhoods)
            .map(|(parameter, neighborhood)| {
                let role = match parameter.name {
                    "index" => StencilParameterRole::Index(CoordinateParameter::from_native(
                        parameter.name,
                        parameter.native_type,
                    )?),
                    "size" => StencilParameterRole::Size(CoordinateParameter::from_native(
                        parameter.name,
                        parameter.native_type,
                    )?),
                    _ => {
                        if let Some(neighborhood) = neighborhood {
                            StencilParameterRole::Neighborhood(neighborhood)
                        } else {
                            let value = ShaderParameter::new(
                                parameter.name,
                                parse_base_type(parameter.native_type),
                            );
                            if Some(parameter.name) == center {
                                StencilParameterRole::Center(value)
                            } else if neighborhood_count == 0 && parameter.name == "input" {
                                StencilParameterRole::Input(value)
                            } else {
                                StencilParameterRole::Value(value)
                            }
                        }
                    }
                };
                Ok(SourceParameter::new(parameter.declaration, role))
            })
            .collect::<Result<Vec<_>>>()?;
        let output = caps
            .get(2)
            .ok_or_else(|| anyhow!("Stencil function must return a type."))?;
        Ok(Self {
            parameters,
            output_element_type: parse_base_type(output.as_str().trim()),
        })
    }

    pub fn index(&self) -> Option<&CoordinateParameter> {
        self.parameters
            .iter()
            .find_map(|parameter| match parameter.role() {
                StencilParameterRole::Index(index) => Some(index),
                _ => None,
            })
    }

    pub fn size(&self) -> Option<&CoordinateParameter> {
        self.parameters
            .iter()
            .find_map(|parameter| match parameter.role() {
                StencilParameterRole::Size(size) => Some(size),
                _ => None,
            })
    }

    pub fn input_element_type(&self) -> &ResourceBaseType {
        if let Some(neighborhood) = self.neighborhood() {
            return neighborhood.element_type();
        }
        self.parameters
            .iter()
            .rev()
            .find_map(|parameter| match parameter.role() {
                StencilParameterRole::Input(input) => Some(input.value_type()),
                _ => None,
            })
            .unwrap_or(&ResourceBaseType::F32)
    }

    pub fn output_element_type(&self) -> &ResourceBaseType {
        &self.output_element_type
    }

    pub fn user_parameters(&self) -> impl DoubleEndedIterator<Item = &ShaderParameter> {
        self.parameters
            .iter()
            .filter_map(|parameter| match parameter.role() {
                StencilParameterRole::Value(value) => Some(value),
                _ => None,
            })
    }

    pub(super) fn neighborhood(&self) -> Option<&NeighborhoodParameter> {
        self.parameters
            .iter()
            .find_map(|parameter| match parameter.role() {
                StencilParameterRole::Neighborhood(neighborhood) => Some(neighborhood),
                _ => None,
            })
    }

    pub(super) fn parameters(&self) -> &[SourceParameter<StencilParameterRole>] {
        &self.parameters
    }

    fn center_name<'source>(
        parameters: &[NativeParameter<'source>],
        neighborhoods: &[Option<NeighborhoodParameter>],
    ) -> Result<Option<&'source str>> {
        let explicit = parameters
            .iter()
            .zip(neighborhoods)
            .filter(|(parameter, neighborhood)| {
                neighborhood.is_none() && matches!(parameter.name, "center" | "val")
            })
            .map(|(parameter, _)| parameter.name)
            .collect::<Vec<_>>();
        if explicit.len() > 1 {
            return Err(anyhow!(
                "Stencil accepts one explicit center or val parameter"
            ));
        }
        if let Some(name) = explicit.first() {
            return Ok(Some(*name));
        }
        Ok(parameters
            .iter()
            .zip(neighborhoods)
            .find(|(parameter, neighborhood)| {
                neighborhood.is_none() && !matches!(parameter.name, "index" | "size")
            })
            .map(|(parameter, _)| parameter.name))
    }
}
