//! Context-selected Stencil resource bindings and uniform source emission.

use super::signature::{StencilParameterRole, StencilSignature};
use crate::prelude::*;
use fabelgeist_gpu::data::gpu::signature::ShaderParameter;
use std::collections::HashMap;

pub(super) struct StencilBindings<'signature> {
    uniforms: Vec<&'signature ShaderParameter>,
    resources: Vec<&'signature ShaderParameter>,
}

impl<'signature> StencilBindings<'signature> {
    pub(super) fn new(
        signature: &'signature StencilSignature,
        secondary: &HashMap<String, ResourceDescriptor>,
    ) -> Self {
        let (resources, uniforms) = signature
            .user_parameters()
            .partition(|parameter| secondary.contains_key(parameter.name()));
        Self {
            uniforms,
            resources,
        }
    }

    pub(super) fn declarations(
        &self,
        code: &mut String,
        input: &ResourceDescriptor,
        output: &ResourceDescriptor,
        secondary: &HashMap<String, ResourceDescriptor>,
    ) -> Result<()> {
        code.push_str(&input.to_wgsl_input_binding(0, 0, "input"));
        code.push_str(&output.to_wgsl_output_binding(0, 1, "output")?);
        let mut binding = 2;
        for parameter in &self.resources {
            let name = parameter.name();
            code.push_str(&secondary[name].to_wgsl_input_binding(0, binding, name));
            binding += 1;
        }
        if !self.uniforms.is_empty() {
            code.push_str("\nstruct Parameters {\n");
            for parameter in &self.uniforms {
                code.push_str(&format!(
                    "    {}: {},\n",
                    parameter.name(),
                    parameter.value_type().as_str()
                ));
            }
            code.push_str("};\n");
            code.push_str(&format!(
                "@group(0) @binding({}) var<uniform> _params: Parameters;\n",
                binding
            ));
        }
        Ok(())
    }

    pub(super) fn unpack(&self, code: &mut String, signature: &StencilSignature) {
        for parameter in &self.uniforms {
            let name = parameter.name();
            code.push_str(&format!("    let {} = _params.{};\n", name, name));
        }
        if signature.neighborhood().is_some() {
            for parameter in signature.user_parameters() {
                if !matches!(
                    parameter.value_type(),
                    ResourceBaseType::Texture2d(_) | ResourceBaseType::Texture3d(_)
                ) {
                    let name = parameter.name();
                    code.push_str(&format!("    let _val_{} = _params.{};\n", name, name));
                }
            }
        }
    }
}

pub(super) fn preprocess(
    source: &str,
    signature: &StencilSignature,
    secondary: &HashMap<String, ResourceDescriptor>,
) -> Result<String> {
    if signature.neighborhood().is_some() {
        let re = regex::Regex::new(
            r"Neighbors([1-4])D\s*<\s*([a-zA-Z0-9_]+(?:\s*<\s*[a-zA-Z0-9_]+\s*>)?)\s*>",
        )?;
        return Ok(re
            .replace_all(source, |caps: &regex::Captures| {
                let dimension = &caps[1];
                let ty = super::neighborhood::sanitize_type_name(&caps[2]);
                format!("Neighbors{dimension}D_{ty}")
            })
            .into_owned());
    }
    let arguments = signature
        .parameters()
        .iter()
        .filter_map(|parameter| match parameter.role() {
            StencilParameterRole::Index(coordinate) | StencilParameterRole::Size(coordinate) => {
                Some(format!(
                    "{}: {}",
                    coordinate.name(),
                    coordinate.shape().value_type().as_str()
                ))
            }
            StencilParameterRole::Value(value) if !secondary.contains_key(value.name()) => {
                Some(format!("{}: {}", value.name(), value.value_type().as_str()))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let re = regex::Regex::new(r"(?s)fn\s+stencil\s*\(([^)]*)\)")?;
    Ok(re
        .replace(source, format!("fn stencil({})", arguments.join(", ")))
        .into_owned())
}

pub(super) fn call_arguments(
    signature: &StencilSignature,
    secondary: &HashMap<String, ResourceDescriptor>,
) -> Vec<String> {
    signature
        .parameters()
        .iter()
        .filter_map(|parameter| match parameter.role() {
            StencilParameterRole::Index(_) => Some("index".to_owned()),
            StencilParameterRole::Size(_) => Some("size".to_owned()),
            StencilParameterRole::Center => Some("center".to_owned()),
            StencilParameterRole::Neighborhood(_) => Some("neighbors".to_owned()),
            StencilParameterRole::Value(value) => {
                let name = value.name();
                if signature.neighborhood().is_some() {
                    if matches!(
                        value.value_type(),
                        ResourceBaseType::Texture2d(_) | ResourceBaseType::Texture3d(_)
                    ) {
                        Some(name.to_owned())
                    } else {
                        Some(format!("_val_{name}"))
                    }
                } else if !secondary.contains_key(name) {
                    Some(name.to_owned())
                } else {
                    None
                }
            }
            StencilParameterRole::Input(_) => None,
        })
        .collect()
}

/// Collect supplied resource descriptors, using authored roles to project
/// buffer element types.
pub(super) fn secondary_resources(
    signature: &StencilSignature,
    parameters: &fabelgeist_gpu::data::gpu::parameters::PassParameters,
) -> HashMap<String, ResourceDescriptor> {
    let mut secondary_resources = HashMap::new();
    for (name, val) in &parameters.parameters {
        match val {
            fabelgeist_gpu::data::gpu::parameters::PassParameter::Buffer(_) => {
                if let Some(parameter) = signature
                    .user_parameters()
                    .find(|parameter| parameter.name() == name)
                {
                    secondary_resources.insert(
                        name.clone(),
                        ResourceDescriptor::Buffer(parameter.value_type().clone()),
                    );
                }
            }
            fabelgeist_gpu::data::gpu::parameters::PassParameter::Texture2d(t) => {
                secondary_resources.insert(name.clone(), ResourceDescriptor::Texture2d(t.format()));
            }
            fabelgeist_gpu::data::gpu::parameters::PassParameter::Texture3d(t) => {
                secondary_resources.insert(name.clone(), ResourceDescriptor::Texture3d(t.format()));
            }
            _ => {}
        }
    }

    secondary_resources
}
