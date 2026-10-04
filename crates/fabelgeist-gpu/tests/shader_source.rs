use fabelgeist_gpu::prelude::{
    ComputePipeline, ComputePipelineError, ComputeShader, ComputeShaderError,
    PreparedComputeShader, ShaderLanguage, ShaderParseError, ShaderSource,
};
use std::error::Error;

#[test]
fn binding_marker_inspection_preserves_generator_spelling() {
    use fabelgeist_gpu::prelude::{ShaderBindingMarker, ShaderBindingName};
    let binding = ShaderBindingName::from("positions");
    for (text, expected) in [
        (
            "var<storage, read> positions: array<f32>;",
            ShaderBindingMarker::Present,
        ),
        (
            "var<storage, read>  positions: array<f32>;",
            ShaderBindingMarker::Absent,
        ),
        (
            "// > positions: marker in a comment",
            ShaderBindingMarker::Present,
        ),
        (
            "var<storage, read> normals: array<f32>;",
            ShaderBindingMarker::Absent,
        ),
    ] {
        assert_eq!(ShaderSource::from(text).binding_marker(&binding), expected);
    }
}

#[tokio::test]
async fn source_constructor_retains_shader_and_reflection_failure_stages() {
    use fabelgeist_gpu::prelude::{ShaderEntryPoint, WgpuContext};
    let context = WgpuContext::new().await.unwrap();
    let error = ComputePipeline::from_source(&context, ShaderSource::from("fn {"))
        .err()
        .unwrap();
    assert!(matches!(
        error,
        ComputePipelineError::Shader(ComputeShaderError::Parse(_))
    ));
    assert!(
        error
            .source()
            .unwrap()
            .downcast_ref::<ComputeShaderError>()
            .is_some()
    );
    let missing = ComputePipeline::from_source(&context, ShaderSource::from("fn helper() {}"));
    assert!(matches!(
        missing,
        Err(ComputePipelineError::MissingComputeEntryPoint)
    ));
    let source = ShaderSource::from("@compute @workgroup_size(1) fn selected() {}");
    let pipeline = ComputePipeline::from_source(&context, source.clone()).unwrap();
    assert_eq!(pipeline.shader.code, source);
    assert_eq!(
        pipeline.reflection.as_ref().unwrap().compute_entry_point,
        ShaderEntryPoint::from("selected")
    );
    assert!(pipeline.pipeline.is_some());
}

#[test]
fn preparation_preserves_frozen_source_conversion_and_error_diagnostics() {
    // Produced by the original parser and preparation code before migration.
    // Pin emitted WGSL and complete diagnostic spans, including Unicode text.
    let cases: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/shader_source.json")).unwrap();
    for case in cases.as_array().unwrap() {
        let source = ShaderSource::from(case["source"].as_str().unwrap());
        match PreparedComputeShader::new(source.clone()) {
            Ok(prepared) => {
                assert!(case.get("error").is_none(), "{case}");
                assert_eq!(prepared.source(), &source);
                assert_eq!(
                    serde_json::json!(<&str>::from(prepared.gpu_source())),
                    case["prepared"]
                );
                assert_eq!(prepared.gpu_source().language(), ShaderLanguage::Wgsl);
            }
            Err(error) => {
                assert!(case.get("prepared").is_none(), "{case}");
                assert_eq!(serde_json::json!(error.to_string()), case["error"]);
                assert!(error.source().is_some());
            }
        }
    }
}

#[test]
fn language_detection_keeps_the_exact_marker_policy() {
    assert_eq!(
        ShaderSource::from("// #version occurs in a comment").language(),
        ShaderLanguage::Glsl
    );
    assert_eq!(
        ShaderSource::from("#VERSION 450").language(),
        ShaderLanguage::Wgsl
    );
    assert_eq!(
        ShaderSource::from(" λ\n@compute fn main() {}").language(),
        ShaderLanguage::Wgsl
    );
}

#[test]
fn parse_failures_retain_source_requested_stage_and_provider_cause() {
    let source = ShaderSource::from("fn {");
    let error = source
        .parse(wgpu::naga::ShaderStage::Fragment)
        .err()
        .unwrap();
    assert!(
        error
            .source()
            .unwrap()
            .downcast_ref::<wgpu::naga::front::wgsl::ParseError>()
            .is_some()
    );
    match error {
        ShaderParseError::Wgsl {
            source: retained,
            stage,
            ..
        } => {
            assert_eq!(retained, source);
            assert_eq!(stage, wgpu::naga::ShaderStage::Fragment);
        }
        ShaderParseError::Glsl { .. } => panic!("WGSL entered the GLSL parser"),
    }
    let source = ShaderSource::from("#version 450\ninvalid");
    let error = source
        .parse(wgpu::naga::ShaderStage::Compute)
        .err()
        .unwrap();
    assert!(
        error
            .source()
            .unwrap()
            .downcast_ref::<wgpu::naga::front::glsl::ParseErrors>()
            .is_some()
    );
    match error {
        ShaderParseError::Glsl {
            source: retained,
            stage,
            ..
        } => {
            assert_eq!(retained, source);
            assert_eq!(stage, wgpu::naga::ShaderStage::Compute);
        }
        ShaderParseError::Wgsl { .. } => panic!("GLSL entered the WGSL parser"),
    }
}

#[test]
fn parse_validation_and_missing_compute_entry_are_separate_contracts() {
    let parse = PreparedComputeShader::new(ShaderSource::from("fn {"))
        .err()
        .unwrap();
    assert!(matches!(parse, ComputeShaderError::Parse(_)));
    let source = ShaderSource::from("@compute @workgroup_size(0) fn main() {}");
    let validation = PreparedComputeShader::new(source.clone()).err().unwrap();
    assert!(
        validation
            .source()
            .unwrap()
            .downcast_ref::<wgpu::naga::WithSpan<wgpu::naga::valid::ValidationError>>()
            .is_some()
    );
    assert!(
        matches!(validation, ComputeShaderError::Validation { source: ref retained, .. } if retained == &source)
    );

    let helper = ShaderSource::from("fn helper() {}");
    PreparedComputeShader::new(helper.clone()).unwrap();
    let pipeline = ComputePipeline {
        shader: ComputeShader {
            code: helper,
            module: None,
        },
        ..Default::default()
    };
    assert!(matches!(
        pipeline.validate_interface(),
        Err(ComputePipelineError::MissingComputeEntryPoint)
    ));
}
