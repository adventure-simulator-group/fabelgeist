//! One native deinterleave path for the shared extraction vertex format.

use super::{ExtractedVertices, ExtractionError, VertexOutputLayout};
use crate::prelude::*;
use fabelgeist_gpu::prelude::{Buffer, BufferDefinition, BufferLabel, BufferUse, GpuResource};

pub(crate) struct AttributeLabels {
    pub(crate) positions: BufferLabel,
    pub(crate) normals: BufferLabel,
}

impl VertexOutputLayout {
    pub(crate) fn project(
        self,
        context: &WgpuContext,
        pipeline: &ResourcePipeline,
        vertices: Buffer,
        labels: AttributeLabels,
    ) -> Result<ExtractedVertices> {
        let expected = self.storage_bytes(pipeline.storage_stride()?);
        if vertices.size != expected {
            return Err(ExtractionError::VertexStorage {
                expected,
                actual: vertices.size,
            }
            .into());
        }
        let positions = Buffer::new(
            context,
            self.packed_attribute_bytes(),
            BufferDefinition::storage()
                .with_label(labels.positions)
                .with_usage(BufferUse::CopySource)
                .with_usage(BufferUse::CopyDestination)
                .with_usage(BufferUse::Vertex),
        )?;
        let normals = Buffer::new(
            context,
            self.packed_attribute_bytes(),
            BufferDefinition::storage()
                .with_label(labels.normals)
                .with_usage(BufferUse::CopySource)
                .with_usage(BufferUse::CopyDestination)
                .with_usage(BufferUse::Vertex),
        )?;
        let workgroups = pipeline.workgroups(&GpuResource::Buffer(vertices.clone()))?;
        let mut parameters = PassParameters::new();
        parameters.insert("vertices", vertices);
        parameters.insert("out_positions", positions.clone());
        parameters.insert("out_normals", normals.clone());
        ComputePass::dispatch(context, pipeline.pipeline().clone(), parameters, workgroups)?;
        Ok(ExtractedVertices::new(self.vertices, positions, normals))
    }
}
