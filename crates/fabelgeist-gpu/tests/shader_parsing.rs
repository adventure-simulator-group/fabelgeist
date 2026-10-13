use fabelgeist_gpu::data::gpu::shader::{ShaderParseError, ShaderParseResult, parse_naga};
use fabelgeist_gpu::prelude::{ComputePipeline, ComputeShader};
use serde::Deserialize;
use std::error::Error;

#[derive(Deserialize)]
enum Outcome {
    Admitted,
    Rejected,
}

#[derive(Deserialize)]
struct Fixture {
    label: String,
    source: String,
    outcome: Outcome,
    diagnostic: Option<String>,
}

fn fixtures() -> Vec<Fixture> {
    serde_json::from_str(include_str!("fixtures/shader_parsing.json")).unwrap()
}

#[test]
fn public_frontends_preserve_frozen_diagnostics_and_admission() {
    for fixture in fixtures() {
        let result: ShaderParseResult<wgpu::naga::Module> =
            parse_naga(&fixture.source, wgpu::naga::ShaderStage::Compute);
        match (&fixture.outcome, result) {
            (Outcome::Admitted, Ok(_)) => {}
            (Outcome::Rejected, Err(error)) => {
                assert_eq!(
                    Some(error.to_string()),
                    fixture.diagnostic,
                    "{}",
                    fixture.label
                );
                let cause = error.source().unwrap();
                match error {
                    ShaderParseError::Wgsl { .. } => {
                        assert!(cause.is::<wgpu::naga::front::wgsl::ParseError>())
                    }
                    ShaderParseError::Glsl { .. } => {
                        assert!(cause.is::<wgpu::naga::front::glsl::ParseErrors>())
                    }
                }
                assert!(cause.source().is_none());
            }
            _ => panic!("admission changed: {}", fixture.label),
        }
    }
}

#[test]
fn wgsl_diagnostic_and_native_cause_outlive_borrowed_source() {
    let error = {
        let source = String::from("// source excerpt\nconst value: f32 = ;\n");
        parse_naga(&source, wgpu::naga::ShaderStage::Compute).unwrap_err()
    };
    assert!(matches!(error, ShaderParseError::Wgsl { .. }));
    let diagnostic = error.to_string();
    assert!(diagnostic.contains("const value: f32 = ;"));
    assert!(
        error
            .source()
            .unwrap()
            .is::<wgpu::naga::front::wgsl::ParseError>()
    );
    let contextual = anyhow::Error::from(error).context("shader caller context");
    assert_eq!(contextual.to_string(), "shader caller context");
    assert!(
        contextual
            .root_cause()
            .is::<wgpu::naga::front::wgsl::ParseError>()
    );
    assert!(contextual.downcast_ref::<ShaderParseError>().is_some());
    assert!(contextual.downcast_ref::<String>().is_none());
    assert_eq!(contextual.chain().count(), 3);
}

#[test]
fn public_interface_preserves_complete_prefix_and_frontend_root() {
    for fixture in fixtures()
        .into_iter()
        .filter(|fixture| matches!(fixture.outcome, Outcome::Rejected))
    {
        let pipeline = ComputePipeline {
            shader: ComputeShader {
                code: fixture.source.clone(),
                ..Default::default()
            },
            ..Default::default()
        };
        let error = pipeline.validate_interface().unwrap_err();
        assert_eq!(
            error.to_string(),
            format!(
                "Compute Shader Parse Error: {}",
                fixture.diagnostic.unwrap()
            )
        );
        let root = error.downcast_ref::<ShaderParseError>().unwrap();
        assert!(root.source().is_some());
        assert!(error.downcast_ref::<String>().is_some()); // presentation context
        assert!(
            error
                .chain()
                .last()
                .unwrap()
                .is::<wgpu::naga::front::wgsl::ParseError>()
                || error
                    .chain()
                    .last()
                    .unwrap()
                    .is::<wgpu::naga::front::glsl::ParseErrors>()
        );
    }
}

#[test]
fn frontend_admission_is_separate_from_interface_entry_validation() {
    let source = "fn helper() {}";
    assert!(parse_naga(source, wgpu::naga::ShaderStage::Compute).is_ok());
    let pipeline = ComputePipeline {
        shader: ComputeShader {
            code: source.to_owned(),
            ..Default::default()
        },
        ..Default::default()
    };
    let error = pipeline.validate_interface().unwrap_err();
    assert_eq!(error.to_string(), "Compute Shader missing entry point");
    assert!(error.downcast_ref::<ShaderParseError>().is_none());
}
