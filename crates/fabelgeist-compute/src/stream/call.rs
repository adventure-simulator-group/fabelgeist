use super::StreamSignature;
use crate::prelude::*;
use fabelgeist_gpu::data::gpu::signature::CoordinateShape;

impl StreamSignature {
    pub(super) fn call_arguments(&self, input_res: &ResourceDescriptor) -> Result<Vec<String>> {
        let mut call_args = vec![];
        for (i, name) in self.param_names.iter().enumerate() {
            let ty = &self.stream_args[i].split(':').collect::<Vec<&str>>()[1].trim();
            if ty.contains("Resource<") || ty.contains("ptr<") || ty.contains("texture_") {
                continue;
            }

            if let Some(parameter) = self
                .index
                .as_ref()
                .filter(|parameter| parameter.name() == name)
            {
                let expression = match parameter.shape() {
                    CoordinateShape::Scalar => "_linear_index",
                    CoordinateShape::Plane => {
                        if matches!(input_res, ResourceDescriptor::Buffer(_)) {
                            return Err(anyhow::anyhow!("vec2 index not supported for Buffer"));
                        }
                        "_global_index.xy"
                    }
                    CoordinateShape::Volume => {
                        if !matches!(input_res, ResourceDescriptor::Texture3d(_)) {
                            return Err(anyhow::anyhow!("vec3 index only supported for Texture3d"));
                        }
                        "_global_index"
                    }
                };
                call_args.push(expression.to_string());
            } else if name == "offset" {
                call_args.push("_offset".to_string());
            } else {
                call_args.push("in_val".to_string());
            }
        }
        Ok(call_args)
    }
}
