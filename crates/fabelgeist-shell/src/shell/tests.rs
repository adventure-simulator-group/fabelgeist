use super::*;
use crate::surface_contact::{SurfaceProjectionError, SurfaceProjectionStage};
use crate::{ShellMaterialError, ShellMaterialParameter, ShellMesh, ShellMeshError};
use fabelgeist_xpbd::{ParticleError, ParticleInputCount};
use std::error::Error;

fn material() -> ShellMaterial {
    ShellMaterial {
        stretch_compliance: 0.0,
        bend_compliance: 0.0,
        seam_compliance: 0.0,
        thickness: 0.003,
        friction: 0.0,
        damping: 0.0,
    }
}
fn mesh() -> ShellMesh {
    ShellMesh::new(
        vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        ],
        vec![[0, 1, 2]],
        0.2.into(),
    )
    .unwrap()
}

#[tokio::test]
async fn construction_admits_mesh_then_material_before_device_resources() {
    let context = WgpuContext::new().await.expect("device context");
    let cache = KernelCache::new();
    let mut invalid = material();
    invalid.thickness = -1.0;
    let error = Shell::new(&context, &cache, &ShellMesh::default(), invalid)
        .err()
        .expect("empty mesh");
    assert!(
        matches!(error, ShellBuildError::Mesh(ShellMeshError::ParticleCount { actual }) if actual == ParticleInputCount::from(0))
    );
    assert!(
        error
            .source()
            .unwrap()
            .downcast_ref::<ShellMeshError>()
            .is_some()
    );
    assert_eq!(error.to_string(), "invalid shell particle count");
    assert_eq!(cache.count().unwrap().to_string(), "0");
    let error = Shell::new(&context, &cache, &mesh(), invalid)
        .err()
        .expect("invalid material");
    assert!(matches!(
        error,
        ShellBuildError::Material(ShellMaterialError::InvalidParameter {
            parameter: ShellMaterialParameter::Thickness
        })
    ));
    assert!(
        error
            .source()
            .unwrap()
            .downcast_ref::<ShellMaterialError>()
            .is_some()
    );
    assert_eq!(cache.count().unwrap().to_string(), "0");
}

#[tokio::test]
async fn host_projection_retains_surface_or_outer_layer_readback_stage() {
    let context = WgpuContext::new().await.expect("device context");
    let cache = KernelCache::new();
    let mut shell = Shell::new(&context, &cache, &mesh(), material()).unwrap();
    shell.particles.positions = Buffer::from_upload(
        &context,
        BufferUpload::from_elements(&[0u32]),
        BufferDefinition::storage(),
    )
    .unwrap();
    let error = shell
        .project_host_contacts(&context, HostContactSchedule::default(), None)
        .await
        .unwrap_err();
    assert!(matches!(
        error,
        ShellProjectionError::SurfaceContact(SurfaceProjectionError::ParticleState {
            stage: SurfaceProjectionStage::PositionsRead,
            ..
        })
    ));
    assert!(
        error
            .source()
            .unwrap()
            .downcast_ref::<SurfaceProjectionError>()
            .is_some()
    );
    shell.outer_layer = Some(crate::outer_layer::OuterLayer::new(
        vec![
            Vec3::new(-1.0, -1.0, 0.0),
            Vec3::new(1.0, -1.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        ],
        vec![[0, 1, 2]],
        0.003,
        Vec3::new(0.0, 0.0, 1.0),
    ));
    let error = shell
        .project_host_contacts(&context, HostContactSchedule::default(), None)
        .await
        .unwrap_err();
    assert!(matches!(
        error,
        ShellProjectionError::ParticleState {
            stage: ShellProjectionStage::OuterLayer,
            ..
        }
    ));
    assert!(
        error
            .source()
            .unwrap()
            .downcast_ref::<ParticleError>()
            .is_some()
    );
}

#[tokio::test]
async fn finite_state_classification_and_exports_keep_readback_failures() {
    let context = WgpuContext::new().await.expect("device context");
    let cache = KernelCache::new();
    let geometry = mesh();
    let mut shell = Shell::new(&context, &cache, &geometry, material()).unwrap();
    assert_eq!(
        shell.position_validity(&context).await.unwrap(),
        ShellPositionValidity::Finite
    );
    let mut positions = geometry.positions.clone();
    positions[1].x = f32::NAN;
    shell
        .particles
        .write(&context, &positions, &geometry.inverse_masses())
        .unwrap();
    assert_eq!(
        shell.position_validity(&context).await.unwrap(),
        ShellPositionValidity::NonFinite
    );
    shell.particles.positions = Buffer::from_upload(
        &context,
        BufferUpload::from_elements(&[0u32]),
        BufferDefinition::storage(),
    )
    .unwrap();
    assert!(matches!(
        shell.position_validity(&context).await,
        Err(ParticleError::Readback { .. })
    ));
    assert!(matches!(
        shell.read_mesh(&context).await,
        Err(ParticleError::Readback { .. })
    ));
}
