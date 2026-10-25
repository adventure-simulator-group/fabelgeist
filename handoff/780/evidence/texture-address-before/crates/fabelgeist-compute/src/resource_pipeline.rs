//! A generated resource program and its controlling native layout.

use crate::kernel::WorkgroupDeclarationError;
use crate::prelude::*;
use fabelgeist_gpu::data::gpu::resource::{GpuResource, ResourceType};
use fabelgeist_gpu::prelude::{BufferElementCount, BufferElementStride};

/// A cached pipeline with the layout and workgroup shape of the same source.
#[derive(Clone, Debug)]
pub struct ResourcePipeline {
    pipeline: ComputePipeline,
    layout: ResourceLayout,
    shape: WorkgroupShape,
}

/// The resource that determines the generated program's dispatch population.
#[derive(Clone, Copy, Debug)]
pub enum DispatchResource {
    Input,
    Output,
}

#[derive(Clone, Copy, Debug)]
enum ResourceLayout {
    Buffer(BufferElementStride),
    Texture2d,
    Texture3d,
}

/// The generated program and supplied controlling resource do not agree.
#[derive(Debug, thiserror::Error)]
pub enum ResourcePipelineError {
    #[error("generated program has no {resource:?} binding")]
    MissingBinding { resource: DispatchResource },
    #[error("generated {resource:?} binding has no supported resource layout")]
    UnsupportedLayout { resource: DispatchResource },
    #[error("resource program expects {expected:?}, received {actual:?}")]
    ResourceKind {
        expected: ResourceType,
        actual: ResourceType,
    },
}

impl ResourcePipeline {
    pub(crate) fn new(
        context: &WgpuContext,
        source: String,
        resource: DispatchResource,
    ) -> Result<Self> {
        let module = fabelgeist_gpu::data::gpu::shader::parse_naga(
            &source,
            wgpu::naga::ShaderStage::Compute,
        )?;
        let entry = module.entry_points.iter()
            .find(|entry| entry.stage == wgpu::naga::ShaderStage::Compute)
            .ok_or(ComputePipelineError::MissingEntryPoint)?;
        let shape = WorkgroupShape::try_from(entry.workgroup_size)
            .map_err(|cause| WorkgroupDeclarationError::at_entry_point(&entry.name, cause))?;
        let layout = resource.layout(&module)?;
        let shader = ComputeShader::new(context, source)?;
        let pipeline = ComputePipeline::new(context, shader)?;
        Ok(Self { pipeline, layout, shape })
    }

    pub fn pipeline(&self) -> &ComputePipeline {
        &self.pipeline
    }

    /// Cover the controlling resource with the actual native declaration.
    /// Texture grids use their independent axes, without a flat-volume cap.
    /// Flat-address validity within the program is a separate admission.
    pub fn workgroups(&self, resource: &GpuResource) -> Result<WorkgroupGrid> {
        let axes = match (self.layout, resource) {
            (ResourceLayout::Buffer(stride), GpuResource::Buffer(buffer)) => {
                let elements = BufferElementCount::from_extent(buffer.size, stride)?;
                return Ok(self.shape.covering_x(elements.into()));
            }
            (ResourceLayout::Texture2d, GpuResource::Texture2d(texture)) => {
                let extent = texture.extent();
                [extent.width.into(), extent.height.into(), 1.into()]
            }
            (ResourceLayout::Texture3d, GpuResource::Texture3d(texture)) => {
                let extent = texture.extent();
                [extent.width.into(), extent.height.into(), extent.depth_or_array_layers.into()]
            }
            (expected, actual) => return Err(ResourcePipelineError::ResourceKind {
                expected: expected.resource_type(),
                actual: actual.resource_type(),
            }.into()),
        };
        Ok(self.shape.covering(axes))
    }
}

impl DispatchResource {
    fn layout(self, module: &wgpu::naga::Module) -> Result<ResourceLayout> {
        // Existing generated WGSL global-name protocol; classify at this
        // native reflection boundary rather than carrying naked byte metadata.
        let name = match self {
            Self::Input => "input",
            Self::Output => "output",
        };
        let (_, variable) = module.global_variables.iter()
            .find(|(_, variable)| variable.name.as_deref() == Some(name))
            .ok_or(ResourcePipelineError::MissingBinding { resource: self })?;
        match &module.types[variable.ty].inner {
            wgpu::naga::TypeInner::Array { stride, .. } => {
                Ok(ResourceLayout::Buffer(BufferElementStride::try_from(*stride)?))
            }
            wgpu::naga::TypeInner::Image {
                dim: wgpu::naga::ImageDimension::D2, arrayed: false, ..
            } => Ok(ResourceLayout::Texture2d),
            wgpu::naga::TypeInner::Image {
                dim: wgpu::naga::ImageDimension::D3, arrayed: false, ..
            } => Ok(ResourceLayout::Texture3d),
            _ => Err(ResourcePipelineError::UnsupportedLayout { resource: self }.into()),
        }
    }
}

impl ResourceLayout {
    fn resource_type(self) -> ResourceType {
        match self {
            Self::Buffer(_) => ResourceType::Buffer,
            Self::Texture2d => ResourceType::Texture2d,
            Self::Texture3d => ResourceType::Texture3d,
        }
    }
}
