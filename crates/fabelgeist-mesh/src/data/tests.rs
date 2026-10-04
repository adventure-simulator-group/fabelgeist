use super::*;
use crate::DrawVertexCount;
use fabelgeist_gpu::prelude::WgpuContext;

#[tokio::test]
async fn gpu_roundtrip_handles_empty_optional_and_skinned_attributes() {
    let context = WgpuContext::new().await.unwrap();
    let position_only = MeshData {
        positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        ..Default::default()
    };
    let skinned = MeshData {
        normals: vec![[0.0, 0.0, 1.0]; 3],
        tex_coords: vec![[0.0; 2]; 3],
        indices: Some(vec![0, 1, 2]),
        joints: Some(vec![[0; 4]; 3]),
        weights: Some(vec![[1.0, 0.0, 0.0, 0.0]; 3]),
        ..position_only.clone()
    };
    for mesh in [
        MeshData::default(),
        MeshData {
            indices: Some(vec![]),
            ..Default::default()
        },
        position_only,
        skinned,
    ] {
        let gpu = mesh.upload(&context).unwrap();
        assert_eq!(gpu.readback(&context).await.unwrap(), mesh);
    }
}
#[test]
fn rejects_invalid_attributes_and_indices() {
    let mut mesh = MeshData {
        positions: vec![[0.0; 3]; 3],
        indices: Some(vec![0, 1, 3]),
        ..Default::default()
    };
    assert!(mesh.validate().is_err());
    mesh.indices = Some(vec![0, 1, 2]);
    mesh.normals = vec![[0.0; 3]];
    assert!(mesh.validate().is_err());
}
#[test]
fn strip_alternates_winding() {
    let mesh = MeshData {
        positions: vec![[0.0; 3]; 4],
        topology: PrimitiveTopology::TriangleStrip,
        ..Default::default()
    };
    assert_eq!(mesh.triangles().unwrap(), vec![[0, 1, 2], [2, 1, 3]]);
}

mod contracts;
