use super::*;
use crate::test_utils::*;
use fabelgeist_gpu::prelude::BufferUpload;

pub async fn test_stencil_generalized<IN, OUT, S>(
    definition_code: &str,
    input_data: &[IN],
    expected_output: &[OUT],
    boundary_mode: BoundaryMode,
) -> Result<()>
where
    IN: bytemuck::NoUninit + bytemuck::AnyBitPattern + PartialEq + std::fmt::Debug + Default + Copy,
    OUT:
        bytemuck::NoUninit + bytemuck::AnyBitPattern + PartialEq + std::fmt::Debug + Default + Copy,
    S: bytemuck::Pod + std::fmt::Debug + Default + Copy + PartialEq,
{
    let definition =
        StencilDefinition::new_with_boundary(definition_code.to_string(), boundary_mode)?;
    let sig = StencilSignature::parse(definition_code)?;

    run_compute_test::<IN, OUT, S, _>(
        input_data,
        expected_output,
        sig.input_element_type().clone(),
        sig.output_element_type().clone(),
        |context, in_res, out_res| Stencil::execute(context, &definition, &in_res, &out_res),
    )
    .await
}

macro_rules! test_stencil {
    ($name:ident, $definition:expr, $input:expr, $output:expr, $s:ty) => {
        #[tokio::test]
        async fn $name() -> Result<()> {
            test_stencil_generalized::<_, _, $s>(
                $definition,
                &$input,
                &$output,
                BoundaryMode::Clamp,
            )
            .await
        }
    };
    ($name:ident, $definition:expr, $input:expr, $output:expr, $s:ty, $boundary:expr) => {
        #[tokio::test]
        async fn $name() -> Result<()> {
            test_stencil_generalized::<_, _, $s>($definition, &$input, &$output, $boundary).await
        }
    };
}

test_stencil!(
    test_1d_stencil,
    "fn stencil(center: f32, neighbors: Neighbors1D<f32>) -> f32 { return neighbors.left + neighbors.right - 2.0 * center; }",
    [1.0f32, 2.0f32, 4.0f32, 7.0f32],
    [1.0f32, 1.0f32, 1.0f32, -3.0f32],
    f32
);

test_stencil!(
    test_heat_diffusion_step,
    "fn stencil(val: f32, neighbors: Neighbors1D<f32>) -> f32 { let lap = neighbors.left + neighbors.right - 2.0 * val; return val + 0.2 * lap; }",
    [10.0f32, 20.0f32, 40.0f32],
    [12.0f32, 22.0f32, 36.0f32],
    f32
);

#[tokio::test]
async fn test_2d_stencil_texture() -> Result<()> {
    let context = WgpuContext::new().await.expect("Failed to init WGPU");
    let code = "fn stencil(center: f32, neighbors: Neighbors2D<f32>) -> f32 { return neighbors.left + neighbors.right + neighbors.top + neighbors.bottom - 4.0 * center; }";
    let definition = StencilDefinition::new(code.to_string())?;

    let size = Vec2::new(4.0, 4.0);
    let mut input_data = vec![0.0f32; 16];
    input_data[5] = 1.0;

    let input_texture = fabelgeist_gpu::data::gpu::texture::Texture2d::create(
        &context,
        size,
        fabelgeist_gpu::data::gpu::texture::TextureFormat::R32Float,
    )?;
    input_texture.write(&context, &input_data)?;

    let output_texture = fabelgeist_gpu::data::gpu::texture::Texture2d::create(
        &context,
        size,
        fabelgeist_gpu::data::gpu::texture::TextureFormat::R32Float,
    )?;

    Stencil::execute(
        &context,
        &definition,
        &GpuResource::Texture2d(input_texture),
        &GpuResource::Texture2d(output_texture.clone()),
    )?;

    let result = output_texture.read::<f32>(&context).await?;

    assert_eq!(result[5], -4.0);
    assert_eq!(result[4], 1.0);

    Ok(())
}

#[tokio::test]
async fn test_resource_based_stencil() -> Result<()> {
    let context = WgpuContext::new().await.expect("Failed to init WGPU");
    let code = "fn stencil(index: u32, input: Resource<f32>, weights: Resource<f32>) -> f32 { return input[index] * weights[0]; }";
    let definition = StencilDefinition::new(code.to_string())?;

    let input_data = vec![1.0f32, 2.0f32, 3.0f32, 4.0f32];
    let weights_data = vec![2.0f32];

    let input_buf = fabelgeist_gpu::data::gpu::buffer::Buffer::from_upload(
        &context,
        BufferUpload::from_elements(&input_data),
        fabelgeist_gpu::data::gpu::buffer::BufferDefinition::storage(),
    )?;
    let output_buf = fabelgeist_gpu::data::gpu::buffer::Buffer::new(
        &context,
        (16u64).into(),
        fabelgeist_gpu::data::gpu::buffer::BufferDefinition::storage(),
    )?;

    let weights_buf = fabelgeist_gpu::data::gpu::buffer::Buffer::from_upload(
        &context,
        BufferUpload::from_elements(&weights_data),
        fabelgeist_gpu::data::gpu::buffer::BufferDefinition::storage(),
    )?;

    let mut params = fabelgeist_gpu::data::gpu::parameters::PassParameters::new();
    params.insert("weights", weights_buf);

    Stencil::execute_with_parameters(
        &context,
        &definition,
        &GpuResource::Buffer(input_buf),
        &GpuResource::Buffer(output_buf.clone()),
        Some(params),
    )?;

    let result = output_buf.read::<f32>(&context).await?;
    assert_eq!(result, vec![2.0, 4.0, 6.0, 8.0]);

    Ok(())
}
