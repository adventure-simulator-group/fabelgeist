use super::*;
use crate::{BlendShapes, Character, Mesh, Skeleton, SkinWeights};
use fabelgeist_numpy_storage::{NpyDecodeError, NpzArrayError};
use fabelgeist_storage::StorageView;

/// Authored NumPy records for admission and deterministic sparse layout checks.
#[derive(Clone, Copy)]
enum ArrayFixture {
    Basis,
    BasisRank,
    BasisAxes,
    BasisVertices,
    BasisComponents,
    Indices,
    IndexCount,
    NegativeRow,
    NegativeColumn,
    HighRow,
    HighColumn,
    Weights,
    OneWeight,
    EmptyWeights,
    EmptyIndices,
    FloatingIndices,
}
impl ArrayFixture {
    fn array(self) -> NpyArray {
        let (dtype, shape, payload) = match self {
            Self::Basis => ("|u1", "24, 1, 3", vec![0u8; 72]),
            Self::BasisRank => ("|u1", "72,", vec![0; 72]),
            Self::BasisAxes => ("|u1", "24, 1, 2", vec![0; 48]),
            Self::BasisVertices => ("|u1", "24, 2, 3", vec![0; 144]),
            Self::BasisComponents => ("|u1", "23, 1, 3", vec![0; 69]),
            Self::Weights | Self::OneWeight | Self::EmptyWeights => {
                let weights = match self {
                    Self::Weights => vec![1.0f32, -2.0, 4.0],
                    Self::OneWeight => vec![1.0],
                    Self::EmptyWeights => Vec::new(),
                    _ => unreachable!(),
                };
                let shape = weights.len().to_string();
                let mut payload = Vec::new();
                for weight in weights {
                    payload.extend_from_slice(&weight.to_le_bytes());
                }
                return NpyRecordFixture {
                    dtype: "<f4",
                    shape,
                    payload: FileContents::from(payload),
                }
                .decode();
            }
            Self::FloatingIndices => {
                let mut payload = Vec::new();
                for value in [0.9f64, -0.9, 23.7, 2.7, 2.1, 5.9] {
                    payload.extend_from_slice(&value.to_le_bytes());
                }
                return NpyRecordFixture {
                    dtype: "<f8",
                    shape: "6,".to_owned(),
                    payload: FileContents::from(payload),
                }
                .decode();
            }
            _ => {
                let indices = match self {
                    Self::Indices => vec![0i64, 0, 23, 2, 2, 5],
                    Self::IndexCount => vec![0, 2, 3],
                    Self::NegativeRow => vec![-1, 0],
                    Self::NegativeColumn => vec![0, -1],
                    Self::HighRow => vec![24, 0],
                    Self::HighColumn => vec![0, 6],
                    Self::EmptyIndices => Vec::new(),
                    _ => unreachable!(),
                };
                let shape = indices.len().to_string();
                let mut payload = Vec::new();
                for index in indices {
                    payload.extend_from_slice(&index.to_le_bytes());
                }
                return NpyRecordFixture {
                    dtype: "<i8",
                    shape,
                    payload: FileContents::from(payload),
                }
                .decode();
            }
        };
        NpyRecordFixture {
            dtype,
            shape: shape.to_owned(),
            payload: FileContents::from(payload),
        }
        .decode()
    }
}
#[test]
fn empty_sparse_arrays_produce_a_shaped_zero_activation_matrix() {
    let network = CorrectiveNetworkDimensions::try_from(rig()).unwrap();
    let data = TensorData::from(
        AdmittedSparseActivation::from_arrays(
            &ArrayFixture::EmptyIndices.array(),
            &ArrayFixture::EmptyWeights.array(),
            network,
        )
        .unwrap(),
    );
    assert_eq!(data.shape, burn::tensor::Shape::new([6, 24]));
    assert_eq!(data.as_slice::<f32>().unwrap(), [0.0; 144]);
}
#[test]
fn floating_sparse_coordinates_keep_cast_before_bound_admission() {
    let network = CorrectiveNetworkDimensions::try_from(rig()).unwrap();
    let integer = AdmittedSparseActivation::from_arrays(
        &ArrayFixture::Indices.array(),
        &ArrayFixture::Weights.array(),
        network,
    )
    .unwrap();
    let floating = AdmittedSparseActivation::from_arrays(
        &ArrayFixture::FloatingIndices.array(),
        &ArrayFixture::Weights.array(),
        network,
    )
    .unwrap();
    let integer = TensorData::from(integer);
    let floating = TensorData::from(floating);
    assert_eq!(integer.shape, floating.shape);
    assert_eq!(
        integer.as_slice::<f32>().unwrap(),
        floating.as_slice::<f32>().unwrap()
    );
}
/// A complete authored serialized array record, consumed only by its decoder.
struct NpyRecordFixture {
    dtype: &'static str,
    shape: String,
    payload: FileContents,
}
impl NpyRecordFixture {
    // Exact authored NumPy serialization boundary; no application primitive ports.
    fn decode(self) -> NpyArray {
        let Self {
            dtype,
            shape,
            payload,
        } = self;
        let header =
            format!("{{'descr': '{dtype}', 'fortran_order': False, 'shape': ({shape}), }}\n");
        let mut bytes = b"\x93NUMPY\x01\x00".to_vec();
        bytes.extend_from_slice(&u16::try_from(header.len()).unwrap().to_le_bytes());
        bytes.extend_from_slice(header.as_bytes());
        bytes.extend_from_slice(payload.as_ref());
        NpyArray::from_view(StorageView::from(bytes.as_slice())).unwrap()
    }
}
fn rig() -> CorrectiveRigTopology {
    CorrectiveRigTopology::from(&Character {
        skeleton: Skeleton {
            names: vec!["root".into(), "offset".into(), "posed".into()],
            ..Default::default()
        },
        mesh: Mesh {
            vertices: vec![[0.0; 3]],
            ..Default::default()
        },
        skin_weights: SkinWeights::default(),
        inverse_bind_pose: Vec::new(),
        blend_shapes: BlendShapes::default(),
    })
}
fn empty_archive() -> Npz {
    Npz::from_bytes(FileContents::from(vec![
        0x50, 0x4b, 0x05, 0x06, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]))
    .unwrap()
}
#[test]
fn basis_admission_distinguishes_rank_axes_vertices_and_hidden_components() {
    for fixture in [ArrayFixture::BasisRank, ArrayFixture::BasisAxes] {
        assert!(matches!(
            CorrectiveBasisDimensions::try_from(&fixture.array()),
            Err(CorrectiveDecodeError::BasisShape(_))
        ));
    }
    let dimensions =
        CorrectiveBasisDimensions::try_from(&ArrayFixture::BasisVertices.array()).unwrap();
    assert!(matches!(
        dimensions.admit_vertices(rig()),
        Err(CorrectiveDecodeError::BasisVertices { .. })
    ));
    let network = CorrectiveNetworkDimensions::try_from(rig()).unwrap();
    let dimensions =
        CorrectiveBasisDimensions::try_from(&ArrayFixture::BasisComponents.array()).unwrap();
    dimensions.admit_vertices(rig()).unwrap();
    assert!(matches!(
        dimensions.admit_components(network),
        Err(CorrectiveDecodeError::BasisComponents { .. })
    ));
    let dimensions = CorrectiveBasisDimensions::try_from(&ArrayFixture::Basis.array()).unwrap();
    dimensions.admit_vertices(rig()).unwrap();
    dimensions.admit_components(network).unwrap();
}
#[test]
fn sparse_densification_keeps_transposition_and_last_duplicate_assignment() {
    let network = CorrectiveNetworkDimensions::try_from(rig()).unwrap();
    let admitted = AdmittedSparseActivation::from_arrays(
        &ArrayFixture::Indices.array(),
        &ArrayFixture::Weights.array(),
        network,
    )
    .unwrap();
    let data = TensorData::from(admitted);
    assert_eq!(data.shape, burn::tensor::Shape::new([6, 24]));
    let values = data.as_slice::<f32>().unwrap();
    assert_eq!(values.len(), 144);
    assert_eq!(values[48], -2.0);
    assert_eq!(values[143], 4.0);
    let mut expected = vec![0.0f32; 144];
    expected[48] = -2.0;
    expected[143] = 4.0;
    assert_eq!(values, expected);
}
#[test]
fn sparse_admission_rejects_count_mismatch_negative_and_upper_bound_coordinates() {
    let network = CorrectiveNetworkDimensions::try_from(rig()).unwrap();
    assert!(matches!(
        AdmittedSparseActivation::from_arrays(
            &ArrayFixture::IndexCount.array(),
            &ArrayFixture::OneWeight.array(),
            network
        ),
        Err(CorrectiveDecodeError::SparseLayout(_))
    ));
    for (fixture, expected_axis) in [
        (ArrayFixture::NegativeRow, SparseCoordinateAxis::HiddenRow),
        (
            ArrayFixture::NegativeColumn,
            SparseCoordinateAxis::FeatureColumn,
        ),
    ] {
        let failure = match AdmittedSparseActivation::from_arrays(
            &fixture.array(),
            &ArrayFixture::OneWeight.array(),
            network,
        ) {
            Err(error) => error,
            Ok(_) => panic!("negative coordinate admitted"),
        };
        assert!(
            std::error::Error::source(&failure)
                .unwrap()
                .is::<std::num::TryFromIntError>()
        );
        let diagnostic = failure.to_string();
        assert!(diagnostic.contains(match expected_axis {
            SparseCoordinateAxis::HiddenRow => "slot 0 at (-1, 0)",
            SparseCoordinateAxis::FeatureColumn => "slot 0 at (0, -1)",
        }));
        assert!(
            matches!(failure, CorrectiveDecodeError::SparseCoordinateEncoding { axis, .. } if axis==expected_axis)
        );
    }
    for fixture in [ArrayFixture::HighRow, ArrayFixture::HighColumn] {
        assert!(matches!(
            AdmittedSparseActivation::from_arrays(
                &fixture.array(),
                &ArrayFixture::OneWeight.array(),
                network
            ),
            Err(CorrectiveDecodeError::SparseCoordinate { .. })
        ));
    }
}
#[test]
fn corrective_archive_and_array_failures_keep_roles_and_concrete_causes() {
    for role in [
        CorrectiveArchiveRole::Activation,
        CorrectiveArchiveRole::Basis,
    ] {
        let failure = match role.admit(FileContents::from(vec![])) {
            Err(error) => error,
            Ok(_) => panic!("invalid archive admitted"),
        };
        assert!(
            std::error::Error::source(&failure)
                .unwrap()
                .is::<ZipReadError>()
        );
        assert!(
            matches!(failure, CorrectiveDecodeError::Archive { role: original, .. } if original==role)
        );
    }
    for role in [
        CorrectiveArrayRole::Basis,
        CorrectiveArrayRole::SparseIndices,
        CorrectiveArrayRole::SparseWeights,
    ] {
        let failure = match role.read(&empty_archive()) {
            Err(error) => error,
            Ok(_) => panic!("missing array admitted"),
        };
        assert!(
            std::error::Error::source(&failure)
                .unwrap()
                .is::<NpzArrayError>()
        );
        match failure {
            CorrectiveDecodeError::Array {
                role: original,
                source,
            } => {
                assert_eq!(original, role);
                assert!(matches!(*source, NpzArrayError::Missing(name) if name==role.name()));
            }
            _ => panic!("expected array failure"),
        }
    }
    let failure = CorrectiveDecodeError::Array {
        role: CorrectiveArrayRole::Basis,
        source: Box::new(NpzArrayError::Decode {
            name: CorrectiveArrayRole::Basis.name(),
            source: NpyDecodeError::Magic,
        }),
    };
    let array = std::error::Error::source(&failure).unwrap();
    assert!(
        std::error::Error::source(array)
            .unwrap()
            .is::<NpyDecodeError>()
    );
}
#[test]
fn absence_and_archive_admission_order_remain_independent_of_device_upload() {
    assert!(
        PoseCorrectives::from_archives(empty_archive(), empty_archive(), rig(), &Device::default())
            .unwrap()
            .is_none()
    );
    let failure = match PoseCorrectives::from_bytes(
        FileContents::from(vec![]),
        FileContents::from(vec![]),
        rig(),
        &Device::default(),
    ) {
        Err(error) => error,
        Ok(_) => panic!("invalid archives admitted"),
    };
    assert!(matches!(
        failure,
        CorrectiveDecodeError::Archive {
            role: CorrectiveArchiveRole::Activation,
            ..
        }
    ));
}
