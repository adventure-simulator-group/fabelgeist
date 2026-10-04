use super::*;
use crate::PlaneGeometry;
use fabelgeist_gpu::prelude::{TextureFormat, UniformBytes, UniformPackingPolicy, Vec2};
use std::error::Error;

#[derive(serde::Deserialize)]
struct ScalarFixture {
    input: u64,
    scale: u32,
}

#[test]
fn native_scalar_admissions_preserve_original_extreme_words() {
    let fixtures: Vec<ScalarFixture> =
        serde_json::from_str(include_str!("../../../tests/fixtures/geometry_inputs.json")).unwrap();
    let member = fabelgeist_gpu::prelude::UniformMember {
        name: PassParameterName::from("strength"),
        offset: fabelgeist_gpu::prelude::BufferByteOffset::START,
        size: BufferByteLength::from(4u32),
    };
    for fixture in fixtures {
        let value = f64::from_bits(fixture.input);
        let mut parameters = PassParameters::new();
        DisplacementScale::from(value).bind(&mut parameters);
        let mut bytes = vec![0; u64::from(member.offset) as usize + 4];
        UniformBytes::from(bytes.as_mut_slice())
            .pack(
                std::slice::from_ref(&member),
                &parameters,
                UniformPackingPolicy::General,
            )
            .unwrap();
        assert_eq!(&bytes[bytes.len() - 4..], fixture.scale.to_le_bytes());
    }
}

#[test]
fn position_dispatch_retains_byte_narrowing_before_vertex_division() {
    #[derive(serde::Deserialize)]
    struct DispatchFixture {
        bytes: u64,
        groups: u32,
    }
    let fixtures: Vec<DispatchFixture> = serde_json::from_str(include_str!(
        "../../../tests/fixtures/displacement_dispatch.json"
    ))
    .unwrap();
    for fixture in fixtures {
        assert_eq!(
            DisplacementPositionSpan::from(BufferByteLength::from(fixture.bytes)).grid(),
            WorkgroupGrid::from([fixture.groups, 1, 1])
        );
    }
}

#[test]
fn singular_projection_keeps_default_inverse_and_error_preserves_pipeline_cause() {
    let singular = Mat4::from_cols_array_2d(&[[0.0; 4]; 4]);
    let mut parameters = PassParameters::new();
    DisplacementProjection::from(singular).bind(&mut parameters);
    assert!(
        matches!(parameters.get(&PassParameterName::from("projection")),
        Some(PassParameter::Mat4(value)) if *value == singular)
    );
    assert!(
        matches!(parameters.get(&PassParameterName::from("inv_projection")),
        Some(PassParameter::Mat4(value)) if *value == Mat4::default())
    );
    let failure = crate::MeshDisplacementError::from(
        fabelgeist_gpu::prelude::ComputePipelineError::MissingModule,
    );
    assert!(matches!(failure, crate::MeshDisplacementError::Pipeline(_)));
    assert!(
        failure
            .source()
            .unwrap()
            .is::<fabelgeist_gpu::prelude::ComputePipelineError>()
    );
    assert_eq!(
        failure.to_string(),
        "ComputePipeline: Shader Module missing"
    );
}

#[tokio::test]
async fn operation_uses_each_context_and_reuses_its_pipeline_for_dispatch() {
    #[derive(serde::Deserialize)]
    struct DisplacementFixture {
        scale: u64,
        positions: Vec<u32>,
    }
    let fixtures: Vec<DisplacementFixture> = serde_json::from_str(include_str!(
        "../../../tests/fixtures/displacement_positions.json"
    ))
    .unwrap();
    for _ in 0..2 {
        let context = WgpuContext::new().await.unwrap();
        let operation = MeshDisplacement::new(&context).unwrap();
        for fixture in &fixtures {
            let mesh = PlaneGeometry::default().upload(&context).unwrap();
            let depth =
                Texture2d::create(&context, Vec2::new(1.0, 1.0), TextureFormat::Rgba32Float)
                    .unwrap();
            depth.write(&context, &[0.25f32, 0.0, 0.0, 1.0]).unwrap();
            let output = operation
                .apply(
                    mesh,
                    depth,
                    DisplacementProjection::from(Mat4::default()),
                    DisplacementScale::from(f64::from_bits(fixture.scale)),
                )
                .unwrap();
            let positions = output.positions.read::<f32>(&context).await.unwrap();
            let bits: Vec<_> = positions.into_iter().map(f32::to_bits).collect();
            assert_eq!(bits, fixture.positions);
        }
    }
}
