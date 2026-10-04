use super::*;
use crate::character::fixture::{RigCase, RigFixture};
use crate::character::{PolygonCornerCount, RigArrayCount, RigMeshVertexOrdinal};
use fabelgeist_rig::RigJointName;

#[test]
fn a_small_binary_rig_preserves_geometry_uvs_joint_order_and_weights() {
    let rig = Character::from_fbx_bytes(&RigFixture::from_case(RigCase::Valid).bytes()).unwrap();
    assert_eq!(rig.skeleton.names, [RigJointName::ROOT]);
    assert_eq!(rig.skeleton.parents, [-1]);
    assert_eq!(
        rig.mesh.vertices,
        [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]
    );
    assert_eq!(rig.mesh.faces, [[0, 1, 2]]);
    assert_eq!(rig.mesh.texcoords, [[0.0, 0.75], [1.0, 0.75], [0.0, 0.25]]);
    assert_eq!(rig.mesh.texcoord_faces, [[0, 1, 2]]);
    assert_eq!(
        rig.skin_weights.weight[0],
        [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
    );
    assert_eq!(
        rig.inverse_bind_pose[0],
        [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0]
    );
}

#[test]
fn binary_failures_keep_the_concrete_decoder_source() {
    let error = Character::from_fbx_bytes(&fabelgeist_fs::FileContents::default())
        .err()
        .unwrap();
    assert!(matches!(
        &error,
        CharacterDecodeError::Fbx(fabelgeist_fbx::FbxDecodeError::Format(_))
    ));
    assert!(
        std::error::Error::source(&error)
            .unwrap()
            .is::<fabelgeist_fbx::FbxDecodeError>()
    );
}

#[test]
fn missing_rig_roles_and_coordinate_lengths_are_structured() {
    assert!(matches!(
        RigFixture::from_case(RigCase::NoJoints).character(),
        Err(CharacterDecodeError::NoJoints)
    ));
    assert!(matches!(
        RigFixture::from_case(RigCase::MissingMesh).character(),
        Err(CharacterDecodeError::MissingMesh)
    ));
    let fixture = RigFixture::from_case(RigCase::MissingGeometry);
    let scene = fixture.scene();
    match fixture.character().err().unwrap() {
        CharacterDecodeError::MissingGeometry(context) => {
            assert_eq!(
                context,
                RigObjectContext::from(scene.get(fabelgeist_fbx::FbxObjectId::from(2)).unwrap())
            )
        }
        other => panic!("wrong error: {other:?}"),
    }
    assert!(matches!(
        RigFixture::from_case(RigCase::MissingVertices).character(),
        Err(CharacterDecodeError::MissingVertices(_))
    ));
    assert!(matches!(
        RigFixture::from_case(RigCase::MissingPolygons).character(),
        Err(CharacterDecodeError::MissingPolygons(_))
    ));
    match RigFixture::from_case(RigCase::IncompletePosition)
        .character()
        .err()
        .unwrap()
    {
        CharacterDecodeError::IncompletePosition { values, .. } => {
            assert_eq!(values, RigArrayCount::from(4))
        }
        other => panic!("wrong error: {other:?}"),
    }
    match RigFixture::from_case(RigCase::IncompleteUv)
        .character()
        .err()
        .unwrap()
    {
        CharacterDecodeError::IncompleteTexcoord { values, .. } => {
            assert_eq!(values, RigArrayCount::from(3))
        }
        other => panic!("wrong error: {other:?}"),
    }
}

#[test]
fn polygon_failures_retain_corner_counts_and_existing_diagnostics() {
    let short = RigFixture::from_case(RigCase::ShortPolygon)
        .character()
        .err()
        .unwrap();
    assert!(
        matches!(&short, CharacterDecodeError::PolygonTooShort(count) if *count == PolygonCornerCount::from(2))
    );
    assert_eq!(
        short.to_string(),
        "invalid face with 2 indices; expected at least 3"
    );
    let trailing = RigFixture::from_case(RigCase::TrailingPolygon)
        .character()
        .err()
        .unwrap();
    assert!(
        matches!(&trailing, CharacterDecodeError::TrailingPolygon(count) if *count == PolygonCornerCount::from(3))
    );
    assert_eq!(
        trailing.to_string(),
        "trailing polygon indices without a terminator"
    );
}

#[test]
fn skin_failures_retain_object_provenance_lengths_and_native_vertex_slots() {
    assert!(matches!(
        RigFixture::from_case(RigCase::MissingSkin).character(),
        Err(CharacterDecodeError::MissingSkin(_))
    ));
    assert!(matches!(
        RigFixture::from_case(RigCase::UnknownBone).character(),
        Err(CharacterDecodeError::UnknownBone(_))
    ));
    let fixture = RigFixture::from_case(RigCase::SkinArrayMismatch);
    let scene = fixture.scene();
    match fixture.character().err().unwrap() {
        CharacterDecodeError::SkinArrayMismatch {
            cluster,
            indices,
            weights,
        } => {
            assert_eq!(
                cluster,
                RigObjectContext::from(scene.get(fabelgeist_fbx::FbxObjectId::from(5)).unwrap())
            );
            assert_eq!(indices, RigArrayCount::from(3));
            assert_eq!(weights, RigArrayCount::from(2));
        }
        other => panic!("wrong error: {other:?}"),
    }
    match RigFixture::from_case(RigCase::SkinVertexOutside)
        .character()
        .err()
        .unwrap()
    {
        CharacterDecodeError::SkinVertex {
            vertex, vertices, ..
        } => {
            assert_eq!(vertex, RigMeshVertexOrdinal::from(3));
            assert_eq!(vertices, crate::MeshVertexCount::from(3));
        }
        other => panic!("wrong error: {other:?}"),
    }
    assert!(
        matches!(RigFixture::from_case(RigCase::NoSkinWeights).character(), Err(CharacterDecodeError::NoSkinWeights(vertex)) if vertex == RigMeshVertexOrdinal::from(0))
    );
}

#[test]
fn skin_admission_keeps_the_existing_nan_behavior() {
    let rig = RigFixture::from_case(RigCase::NaNWeight)
        .character()
        .unwrap();
    assert!(rig.skin_weights.weight[0][0].is_nan());
    assert_eq!(rig.skin_weights.weight[1][0], 1.0);
}
