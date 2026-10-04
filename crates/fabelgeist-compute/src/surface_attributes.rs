//! Positions and normals deinterleaved from a surface extraction's vertices.
use fabelgeist_gpu::prelude::BufferUse;
use fabelgeist_gpu::prelude::{
    Buffer, BufferByteLength, BufferCreationError, BufferDefinition, ComputePass, ComputePassError,
    ComputePipeline, PassParameters, WgpuContext, WorkgroupGrid,
};

pub(crate) enum SurfaceExtraction {
    MarchingCubes,
    DualContouring,
}
pub(crate) struct SurfaceAttributes {
    pub positions: Buffer,
    pub normals: Buffer,
}
#[derive(Debug)]
pub(crate) enum SurfaceAttributeError {
    Allocation(BufferCreationError),
    Dispatch(ComputePassError),
}
impl std::fmt::Display for SurfaceAttributeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Allocation(cause) => cause.fmt(formatter),
            Self::Dispatch(cause) => cause.fmt(formatter),
        }
    }
}
impl std::error::Error for SurfaceAttributeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Allocation(cause) => Some(cause),
            Self::Dispatch(cause) => Some(cause),
        }
    }
}
impl SurfaceAttributes {
    pub fn new(
        context: &WgpuContext,
        pipeline: &ComputePipeline,
        vertices: Buffer,
        coordinate_bytes: BufferByteLength,
        groups: WorkgroupGrid,
        extraction: SurfaceExtraction,
    ) -> Result<Self, SurfaceAttributeError> {
        let (position_label, normal_label) = match extraction {
            SurfaceExtraction::MarchingCubes => {
                ("marching_cubes_positions", "marching_cubes_normals")
            }
            SurfaceExtraction::DualContouring => {
                ("dual_contouring_positions", "dual_contouring_normals")
            }
        };
        let positions = Buffer::new(
            context,
            coordinate_bytes,
            BufferDefinition::storage()
                .with_label((position_label).into())
                .with_usage(BufferUse::CopySource)
                .with_usage(BufferUse::CopyDestination)
                .with_usage(BufferUse::Vertex),
        )
        .map_err(SurfaceAttributeError::Allocation)?;
        let normals = Buffer::new(
            context,
            coordinate_bytes,
            BufferDefinition::storage()
                .with_label((normal_label).into())
                .with_usage(BufferUse::CopySource)
                .with_usage(BufferUse::CopyDestination)
                .with_usage(BufferUse::Vertex),
        )
        .map_err(SurfaceAttributeError::Allocation)?;
        let parameters = PassParameters::from([
            ("vertices".into(), vertices.into()),
            ("out_positions".into(), positions.clone().into()),
            ("out_normals".into(), normals.clone().into()),
        ]);
        ComputePass::dispatch(context, pipeline.clone(), parameters, groups)
            .map_err(SurfaceAttributeError::Dispatch)?;
        Ok(Self { positions, normals })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fabelgeist_gpu::prelude::BufferUpload;

    #[tokio::test]
    async fn shared_deinterleaving_preserves_attribute_words_and_extents() {
        let context = WgpuContext::new().await.unwrap();
        let definition = crate::marching_cubes::MarchingCubesDefinition::new(&context).unwrap();
        let words = [
            -0.0f32, 2.0, 3.0, 1.0, 4.0, 5.0, 6.0, 0.0, 7.0, 8.0, 9.0, 1.0, 10.0, 11.0, 12.0, 0.0,
        ];
        for extraction in [
            SurfaceExtraction::MarchingCubes,
            SurfaceExtraction::DualContouring,
        ] {
            let vertices = Buffer::from_upload(
                &context,
                BufferUpload::from_elements(&words),
                BufferDefinition::storage(),
            )
            .unwrap();
            let attributes = SurfaceAttributes::new(
                &context,
                &definition.deinterleave_pipeline,
                vertices,
                BufferByteLength::from(24u32),
                WorkgroupGrid::from([1, 1, 1]),
                extraction,
            )
            .unwrap();
            let positions: Vec<f32> = attributes.positions.read(&context).await.unwrap();
            let normals: Vec<f32> = attributes.normals.read(&context).await.unwrap();
            let expected_positions = [-0.0f32, 2.0, 3.0, 7.0, 8.0, 9.0];
            let expected_normals = [4.0f32, 5.0, 6.0, 10.0, 11.0, 12.0];
            assert_eq!(
                bytemuck::cast_slice::<f32, u32>(&positions),
                bytemuck::cast_slice::<f32, u32>(&expected_positions)
            );
            assert_eq!(
                bytemuck::cast_slice::<f32, u32>(&normals),
                bytemuck::cast_slice::<f32, u32>(&expected_normals)
            );
            assert_eq!(attributes.positions.length(), 24u64.into());
            assert_eq!(attributes.normals.length(), 24u64.into());
        }
        assert!(matches!(
            Buffer::new(&context, (0u64).into(), BufferDefinition::storage()),
            Err(BufferCreationError::Empty)
        ));
    }
}
