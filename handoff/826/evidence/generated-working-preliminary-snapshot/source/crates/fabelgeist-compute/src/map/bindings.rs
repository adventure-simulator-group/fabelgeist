//! Contextual primary, resource and uniform bindings for one Map compilation.

use super::MapSignature;
use crate::prelude::*;
use fabelgeist_gpu::data::gpu::signature::ShaderParameter;
use std::collections::HashMap;

/// Borrowed partitions of the signature, selected from actual resource presence.
pub(super) struct MapBindings<'signature> {
    primary: Option<&'signature ShaderParameter>,
    uniforms: Vec<&'signature ShaderParameter>,
    resources: Vec<&'signature ShaderParameter>,
}

impl<'signature> MapBindings<'signature> {
    pub(super) fn new(
        signature: &'signature MapSignature,
        input: Option<&ResourceDescriptor>,
        secondary: &HashMap<String, ResourceDescriptor>,
    ) -> Self {
        let mut extra = signature.user_parameters();
        let primary = input.and_then(|_| extra.next());
        let (resources, uniforms) =
            extra.partition(|parameter| secondary.contains_key(parameter.name()));
        Self {
            primary,
            uniforms,
            resources,
        }
    }

    pub(super) fn primary(&self) -> Option<&ShaderParameter> {
        self.primary
    }

    pub(super) fn declarations(
        &self,
        code: &mut String,
        input: Option<&ResourceDescriptor>,
        output: &ResourceDescriptor,
        secondary: &HashMap<String, ResourceDescriptor>,
    ) -> Result<()> {
        if let Some(input) = input {
            code.push_str(&input.to_wgsl_input_binding(0, 0, "input"));
        }
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

    pub(super) fn fetches(
        &self,
        code: &mut String,
        input: Option<&ResourceDescriptor>,
        output: &ResourceDescriptor,
        secondary: &HashMap<String, ResourceDescriptor>,
    ) -> Result<TextureAddressing> {
        let mut addressing = TextureAddressing::Coordinates;
        if let Some(input) = input {
            let fetch = input.generate_fetch(
                "input",
                output,
                self.primary
                    .map(ShaderParameter::value_type)
                    .unwrap_or(&ResourceBaseType::F32),
            )?;
            addressing = addressing.combine(fetch.addressing());
            code.push_str(fetch.code());
        }
        for parameter in &self.resources {
            let name = parameter.name();
            let fetch = secondary[name].generate_fetch(name, output, parameter.value_type())?;
            addressing = addressing.combine(fetch.addressing());
            let fetch_code = fetch
                .code()
                .replace("_in_global_index", &format!("_in_global_index_{}", name))
                .replace(
                    "_fetch_out_tex_dim",
                    &format!("_fetch_out_tex_dim_{}", name),
                )
                .replace("in_tex_dim", &format!("in_tex_dim_{}", name))
                .replace("let in_val = ", &format!("let {} = ", name));
            code.push_str(&fetch_code);
        }
        Ok(addressing)
    }

    pub(super) fn unpack_uniforms(&self, code: &mut String) {
        for parameter in &self.uniforms {
            let name = parameter.name();
            code.push_str(&format!("    let {} = _params.{};\n", name, name));
        }
    }
}
