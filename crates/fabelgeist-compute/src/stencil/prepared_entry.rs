//! Rewrite a parsed stencil entry for its reflected resource parameters.
use super::{StencilDefinition, StencilSignature, sanitize_type_name};
use fabelgeist_gpu::prelude::{PassParameterName, ResourceDescriptor};
use std::collections::HashMap;

pub(super) struct PreparedStencilEntry(String);
#[derive(Debug)]
pub(super) enum StencilRewriteError {
    Neighborhood(regex::Error),
    Signature(regex::Error),
}
impl std::fmt::Display for StencilRewriteError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Neighborhood(cause) | Self::Signature(cause) => cause.fmt(formatter),
        }
    }
}
impl std::error::Error for StencilRewriteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Neighborhood(cause) | Self::Signature(cause) => Some(cause),
        }
    }
}
impl PreparedStencilEntry {
    pub fn new(
        definition: &StencilDefinition,
        sig: &StencilSignature,
        secondary_resources: &HashMap<PassParameterName, ResourceDescriptor>,
    ) -> Result<Self, StencilRewriteError> {
        let mut preprocessed_user_code = definition.code.clone();

        if sig.is_neighborhood {
            let re_neighbors = regex::Regex::new(
                r"Neighbors([1-4])D\s*<\s*([a-zA-Z0-9_]+(?:\s*<\s*[a-zA-Z0-9_]+\s*>)?)\s*>",
            )
            .map_err(StencilRewriteError::Neighborhood)?;
            preprocessed_user_code = re_neighbors
                .replace_all(
                    &preprocessed_user_code,
                    |caps: &regex::Captures| -> String {
                        let d = &caps[1];
                        let ty = &caps[2];
                        let ty_sanitized = sanitize_type_name(ty);
                        format!("Neighbors{}D_{}", d, ty_sanitized)
                    },
                )
                .into_owned();
        } else {
            let mut new_args = Vec::new();
            for arg in &sig.param_names {
                let is_resource = arg == "input"
                    || secondary_resources.contains_key(&PassParameterName::from(arg.as_str()));

                if is_resource {
                    continue;
                }

                // Keep index, size, or uniform params in the signature
                if arg == "index" {
                    new_args.push(format!(
                        "index: {}",
                        sig.index_type.as_deref().unwrap_or("u32")
                    ));
                } else if arg == "size" {
                    new_args.push(format!(
                        "size: {}",
                        sig.size_type.as_deref().unwrap_or("u32")
                    ));
                } else if let Some((_, ty)) = sig.user_params.iter().find(|(n, _)| n == arg) {
                    new_args.push(format!("{}: {}", arg, ty.as_str()));
                }
            }

            let re_sig = regex::Regex::new(r"(?s)fn\s+stencil\s*\(([^)]*)\)")
                .map_err(StencilRewriteError::Signature)?;
            preprocessed_user_code = re_sig
                .replace(
                    &preprocessed_user_code,
                    format!("fn stencil({})", new_args.join(", ")),
                )
                .to_string();
        }

        Ok(Self(preprocessed_user_code))
    }
}
impl std::fmt::Display for PreparedStencilEntry {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}
