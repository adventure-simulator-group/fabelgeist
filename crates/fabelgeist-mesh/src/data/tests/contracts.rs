use super::*;
use crate::{
    DrawVertexIndex, MeshAttribute, MeshAttributeLength, MeshTriangleError, MeshValidationError,
    SkinWeightViolation,
};

#[derive(serde::Deserialize)]
struct ContractFixture {
    name: String,
    mesh: MeshWire,
    validation_error: Option<String>,
    triangle_error: Option<String>,
    triangles: Option<Vec<[u32; 3]>>,
}
#[derive(serde::Deserialize)]
struct MeshWire {
    positions: Vec<[u32; 3]>,
    normals: Vec<[u32; 3]>,
    tex_coords: Vec<[u32; 2]>,
    indices: Option<Vec<u32>>,
    joints: Option<Vec<[u32; 4]>>,
    weights: Option<Vec<[u32; 4]>>,
    topology: PrimitiveTopology,
}
impl MeshWire {
    fn into_mesh(self) -> MeshData {
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut tex_coords = Vec::new();
        for row in self.positions {
            positions.push(row.map(f32::from_bits));
        }
        for row in self.normals {
            normals.push(row.map(f32::from_bits));
        }
        for row in self.tex_coords {
            tex_coords.push(row.map(f32::from_bits));
        }
        let weights = match self.weights {
            Some(rows) => {
                let mut decoded = Vec::new();
                for row in rows {
                    decoded.push(row.map(f32::from_bits));
                }
                Some(decoded)
            }
            None => None,
        };
        MeshData {
            positions,
            normals,
            tex_coords,
            weights,
            indices: self.indices,
            joints: self.joints,
            topology: self.topology,
            ..Default::default()
        }
    }
}
#[test]
fn original_admission_order_diagnostics_and_triangle_results() {
    let fixtures: Vec<ContractFixture> =
        serde_json::from_str(include_str!("../../../tests/fixtures/mesh_contracts.json")).unwrap();
    assert_eq!(fixtures.len(), 47);
    for fixture in fixtures {
        let mesh = fixture.mesh.into_mesh();
        let error = match mesh.validate() {
            Ok(()) => None,
            Err(error) => Some(error.to_string()),
        };
        assert_eq!(error, fixture.validation_error, "{}", fixture.name);
        match mesh.triangles() {
            Ok(triangles) => {
                assert!(fixture.triangle_error.is_none(), "{}", fixture.name);
                assert_eq!(Some(triangles), fixture.triangles, "{}", fixture.name);
            }
            Err(error) => {
                assert!(fixture.triangles.is_none(), "{}", fixture.name);
                assert_eq!(
                    Some(error.to_string()),
                    fixture.triangle_error,
                    "{}",
                    fixture.name
                );
            }
        }
    }
}
#[test]
fn classification_retains_attribute_lengths_skin_reason_and_rejected_draw_address() {
    let mut mesh = MeshData {
        positions: vec![[0.0; 3]; 3],
        normals: vec![[0.0; 3]],
        ..Default::default()
    };
    assert_eq!(
        mesh.validate(),
        Err(MeshValidationError::AttributeLength {
            attribute: MeshAttribute::Normals,
            expected: MeshAttributeLength::from(3),
            actual: MeshAttributeLength::from(1)
        })
    );
    mesh.normals.clear();
    mesh.joints = Some(vec![[0; 4]; 3]);
    assert_eq!(
        mesh.validate(),
        Err(MeshValidationError::SkinPair {
            missing: MeshAttribute::Weights
        })
    );
    mesh.weights = Some(vec![[0.0; 4]; 2]);
    assert_eq!(
        mesh.validate(),
        Err(MeshValidationError::SkinWeights {
            violation: SkinWeightViolation::Length {
                expected: 3usize.into(),
                actual: 2usize.into()
            }
        })
    );
    mesh.weights = Some(vec![[-1.0, 0.0, 0.0, 0.0]; 3]);
    assert_eq!(
        mesh.validate(),
        Err(MeshValidationError::SkinWeights {
            violation: SkinWeightViolation::Negative
        })
    );
    mesh.weights = Some(vec![[f32::NAN, 0.0, 0.0, 0.0]; 3]);
    assert_eq!(
        mesh.validate(),
        Err(MeshValidationError::SkinWeights {
            violation: SkinWeightViolation::NonFinite
        })
    );
    mesh.weights = Some(vec![[1.0; 4]; 3]);
    mesh.indices = Some(vec![u32::MAX]);
    assert_eq!(
        mesh.validate(),
        Err(MeshValidationError::IndexOutOfBounds {
            index: DrawVertexIndex::from(u32::MAX),
            vertices: DrawVertexCount::from(3)
        })
    );
    mesh.indices = None;
    mesh.topology = PrimitiveTopology::PointList;
    assert_eq!(
        mesh.triangles(),
        Err(MeshTriangleError::UnsupportedTopology {
            topology: PrimitiveTopology::PointList
        })
    );
}
#[test]
fn draw_count_admits_empty_and_maximum_but_rejects_host_overflow() {
    assert_eq!(
        DrawVertexCount::from_attribute_length(0usize.into()).unwrap(),
        DrawVertexCount::default()
    );
    assert_eq!(
        DrawVertexCount::from_attribute_length((u32::MAX as usize).into()).unwrap(),
        DrawVertexCount::from(u32::MAX)
    );
    if let Some(overflow) = (u32::MAX as usize).checked_add(1) {
        assert_eq!(
            DrawVertexCount::from_attribute_length(overflow.into()),
            Err(MeshValidationError::TooManyVertices {
                actual: overflow.into()
            })
        );
    }
}
#[tokio::test]
async fn readback_retains_each_attribute_role_and_concrete_cause() {
    use fabelgeist_gpu::prelude::ReadbackError;
    use std::error::Error;
    let context = WgpuContext::new().await.unwrap();
    let mesh = MeshData {
        positions: vec![[0.0; 3]; 3],
        normals: vec![[0.0; 3]; 3],
        tex_coords: vec![[0.0; 2]; 3],
        indices: Some(vec![0, 1, 2]),
        joints: Some(vec![[0; 4]; 3]),
        weights: Some(vec![[1.0, 0.0, 0.0, 0.0]; 3]),
        ..Default::default()
    };
    for attribute in [
        MeshAttribute::Positions,
        MeshAttribute::Normals,
        MeshAttribute::TextureCoordinates,
        MeshAttribute::Indices,
        MeshAttribute::Joints,
        MeshAttribute::Weights,
    ] {
        let mut gpu = mesh.upload(&context).unwrap();
        let buffer = match attribute {
            MeshAttribute::Positions => &mut gpu.positions,
            MeshAttribute::Normals => &mut gpu.normals,
            MeshAttribute::TextureCoordinates => &mut gpu.tex_coords,
            MeshAttribute::Indices => gpu.indices.as_mut().unwrap(),
            MeshAttribute::Joints => gpu.joints.as_mut().unwrap(),
            MeshAttribute::Weights => gpu.weights.as_mut().unwrap(),
        };
        *buffer = buffer.clone().with_logical_length(1u64.into()).unwrap();
        let error = gpu.readback(&context).await.unwrap_err();
        assert_eq!(error.attribute, attribute);
        assert!(matches!(error.source, ReadbackError::PartialElement { .. }));
        assert!(
            error
                .source()
                .unwrap()
                .downcast_ref::<ReadbackError>()
                .is_some()
        );
    }
}
