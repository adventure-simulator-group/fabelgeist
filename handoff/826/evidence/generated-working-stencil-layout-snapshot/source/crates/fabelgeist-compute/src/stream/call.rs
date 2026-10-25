use super::StreamSignature;
use super::signature::StreamParameterRole;
use crate::prelude::*;
use crate::source_error::{GeneratedBinding, GeneratedFunction, GeneratedProgramError};
use fabelgeist_gpu::data::gpu::signature::CoordinateShape;

impl StreamSignature {
    pub(super) fn call_arguments(&self, input_res: &ResourceDescriptor) -> Result<Vec<String>> {
        let mut call_args = vec![];
        for role in self.roles() {
            if let StreamParameterRole::Index(parameter) = role {
                let expression = match parameter.shape() {
                    CoordinateShape::Scalar => "_linear_index",
                    CoordinateShape::Plane => {
                        if matches!(input_res, ResourceDescriptor::Buffer(_)) {
                            return Err(GeneratedProgramError::UnsupportedCoordinate {
                                function: GeneratedFunction::Stream,
                                shape: parameter.shape(),
                                binding: GeneratedBinding::Input,
                                actual: input_res.resource_type(),
                            }
                            .into());
                        }
                        "_global_index.xy"
                    }
                    CoordinateShape::Volume => {
                        if !matches!(input_res, ResourceDescriptor::Texture3d(_)) {
                            return Err(GeneratedProgramError::UnsupportedCoordinate {
                                function: GeneratedFunction::Stream,
                                shape: parameter.shape(),
                                binding: GeneratedBinding::Input,
                                actual: input_res.resource_type(),
                            }
                            .into());
                        }
                        "_global_index"
                    }
                };
                call_args.push(expression.to_string());
            } else {
                match role {
                    StreamParameterRole::Offset(_) => call_args.push("_offset".to_string()),
                    StreamParameterRole::Value(_) => call_args.push("in_val".to_string()),
                    StreamParameterRole::Resource(_) | StreamParameterRole::Index(_) => {}
                }
            }
        }
        Ok(call_args)
    }
}
