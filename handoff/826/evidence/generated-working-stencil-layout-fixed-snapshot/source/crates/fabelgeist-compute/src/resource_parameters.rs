//! Native texture binding and uniform adapters for generated fluid recipes.

use crate::source_error::{
    FluidOperation, GeneratedBinding, GeneratedProgramError, GeneratedProgramResult,
};
use fabelgeist_gpu::data::gpu::ResourceDescriptor;
use fabelgeist_gpu::data::gpu::parameters::{PassParameter, PassParameters};
use fabelgeist_gpu::data::gpu::resource::GpuResource;
use fabelgeist_gpu::data::vector::Vec4;

impl GeneratedBinding {
    pub(crate) fn texture_descriptor(
        self,
        operation: FluidOperation,
        resource: &GpuResource,
    ) -> GeneratedProgramResult<ResourceDescriptor> {
        match resource {
            GpuResource::Texture2d(texture) => Ok(ResourceDescriptor::Texture2d(texture.format())),
            GpuResource::Texture3d(texture) => Ok(ResourceDescriptor::Texture3d(texture.format())),
            _ => Err(GeneratedProgramError::ExpectedTexture {
                operation,
                binding: self,
                actual: resource.resource_type(),
            }),
        }
    }

    pub(crate) fn insert_texture(
        self,
        operation: FluidOperation,
        parameters: &mut PassParameters,
        resource: &GpuResource,
    ) -> GeneratedProgramResult<()> {
        let parameter = match resource {
            GpuResource::Texture2d(texture) => PassParameter::Texture2d(texture.clone()),
            GpuResource::Texture3d(texture) => PassParameter::Texture3d(texture.clone()),
            _ => {
                return Err(GeneratedProgramError::ExpectedTexture {
                    operation,
                    binding: self,
                    actual: resource.resource_type(),
                });
            }
        };
        parameters.insert(self.native_name(), parameter);
        Ok(())
    }

    /// Native uniform coordinates: x/y/z are texel extents; unused lanes are1.
    pub(crate) fn texture_size(
        self,
        operation: FluidOperation,
        resource: &GpuResource,
    ) -> GeneratedProgramResult<Vec4> {
        match resource {
            GpuResource::Texture2d(texture) => Ok(Vec4::new(
                texture.extent().width as f32,
                texture.extent().height as f32,
                1.0,
                1.0,
            )),
            GpuResource::Texture3d(texture) => Ok(Vec4::new(
                texture.extent().width as f32,
                texture.extent().height as f32,
                texture.extent().depth_or_array_layers as f32,
                1.0,
            )),
            _ => Err(GeneratedProgramError::ExpectedTexture {
                operation,
                binding: self,
                actual: resource.resource_type(),
            }),
        }
    }

    fn native_name(self) -> &'static str {
        match self {
            Self::Input => "input",
            Self::Output => "output",
            Self::Velocity => "velocity",
            Self::Quantity => "quantity",
            Self::Pressure => "pressure",
        }
    }
}
