//! `.npy` array decoding.

use fabelgeist_fs::NativeFile;
use fabelgeist_storage::{StorageByteLength, StorageByteOffset, StorageByteSpan, StorageView};
mod error;
#[cfg(test)]
pub(crate) mod fixture;
mod header;
mod metadata;
mod values;
use burn::tensor::{Device, Int, Tensor, TensorData};
pub use error::{
    NpyDecodeError, NpyHeaderField, NpyRank, NpyReadError, NpySection, NpyVersionEncoding,
    RejectedNpyValue,
};
use header::NpyHeader;
pub use metadata::{
    NpyDimension, NpyElementCount, NpyElementOrdinal, NpyElementWidth, NpyOccupancy, NpyShape,
};
pub use values::{
    NpyByteState, NpyByteStates, NpyFloatValue, NpyFloatValues, NpyIntegerValue, NpyIntegerValues,
};

/// The NumPy element types this crate decodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dtype {
    F32,
    F64,
    I32,
    I64,
    U8,
    Bool,
}

/// One admitted array with immutable shape, element type, and payload metadata.
///
/// ```compile_fail
/// use fabelgeist_numpy_storage::NpyArray;
/// fn replace_shape(array: &mut NpyArray) {
///     array.shape = array.shape.clone();
/// }
/// ```
///
/// ```compile_fail
/// use fabelgeist_numpy_storage::{Dtype, NpyArray};
/// fn reinterpret_dtype(array: &mut NpyArray) {
///     array.dtype = Dtype::U8;
/// }
/// ```
pub struct NpyArray {
    shape: NpyShape,
    dtype: Dtype,
    bytes: Vec<u8>,
}

impl NpyArray {
    pub fn shape(&self) -> &NpyShape {
        &self.shape
    }
    pub fn dtype(&self) -> Dtype {
        self.dtype
    }
    pub fn element_count(&self) -> NpyElementCount {
        self.shape.element_count()
    }
    pub fn occupancy(&self) -> NpyOccupancy {
        self.element_count().occupancy()
    }

    /// Immutable view of the element payload, exactly as stored.
    pub fn payload(&self) -> StorageView<'_> {
        StorageView::from(self.bytes.as_slice())
    }

    /// Ordered floating values cast from the stored element type.
    pub fn floating_values(&self) -> NpyFloatValues {
        NpyFloatValues::from(self)
    }
    /// Ordered signed integer values cast from the stored element type.
    pub fn integer_values(&self) -> NpyIntegerValues {
        NpyIntegerValues::from(self)
    }
    /// Zero/nonzero state of each payload byte, independent of element width.
    pub fn byte_states(&self) -> NpyByteStates {
        NpyByteStates::from(self)
    }

    /// Uploads the array to a device as a float tensor.
    pub fn to_tensor<const D: usize>(&self, device: &Device) -> Result<Tensor<D>, NpyDecodeError> {
        Ok(Tensor::from_data(
            TensorData::new(
                Vec::<f32>::from(self.floating_values()),
                self.shape.tensor_layout::<D>()?.dimensions,
            ),
            device,
        ))
    }

    /// Uploads the array to a device as an integer tensor.
    pub fn to_int_tensor<const D: usize>(
        &self,
        device: &Device,
    ) -> Result<Tensor<D, Int>, NpyDecodeError> {
        Ok(Tensor::from_data(
            TensorData::new(
                Vec::<i64>::from(self.integer_values()),
                self.shape.tensor_layout::<D>()?.dimensions,
            ),
            device,
        ))
    }
}

impl NpyArray {
    /// Admits a serialized array view without changing element values or order.
    pub fn from_view(view: StorageView<'_>) -> Result<Self, NpyDecodeError> {
        ArrayFileSections::from_view(view)?.decode()
    }
    /// Reads an array from an admitted native file address.
    pub fn read(file: &NativeFile) -> Result<Self, NpyReadError> {
        let contents =
            std::fs::read(file.as_ref()).map_err(|source: std::io::Error| -> NpyReadError {
                NpyReadError::Read {
                    file: file.clone(),
                    source,
                }
            })?;
        Self::from_view(StorageView::from(contents.as_slice())).map_err(
            |source: NpyDecodeError| -> NpyReadError {
                NpyReadError::Decode {
                    file: file.clone(),
                    source,
                }
            },
        )
    }
}

struct ArrayFileSections<'a> {
    header: StorageView<'a>,
    payload: StorageView<'a>,
}
impl<'a> ArrayFileSections<'a> {
    fn from_view(view: StorageView<'a>) -> Result<Self, NpyDecodeError> {
        const MAGIC: &[u8] = b"\x93NUMPY";
        if view.length() < StorageByteLength::from(6u64)
            || view
                .portion(StorageByteSpan {
                    offset: StorageByteOffset::default(),
                    length: StorageByteLength::from(6u64),
                })
                .expect("checked magic width")
                .as_ref()
                != MAGIC
        {
            return Err(NpyDecodeError::Magic);
        }
        let version = view
            .portion(StorageByteSpan {
                offset: StorageByteOffset::from(6u64),
                length: StorageByteLength::from(2u64),
            })
            .map_err(
                |source: fabelgeist_storage::StorageBoundsError| -> NpyDecodeError {
                    NpyDecodeError::Bounds {
                        section: NpySection::Version,
                        source,
                    }
                },
            )?;
        let (header_len, header_start) = match version.as_ref() {
            [1, 0] => (
                StorageByteLength::from(u64::from(
                    view.decode_u16(StorageByteOffset::from(8u64)).map_err(
                        |source: fabelgeist_storage::StorageBoundsError| -> NpyDecodeError {
                            NpyDecodeError::Bounds {
                                section: NpySection::HeaderLength,
                                source,
                            }
                        },
                    )?,
                )),
                StorageByteOffset::from(10u64),
            ),
            [2 | 3, 0] => (
                StorageByteLength::from(u64::from(
                    view.decode_u32(StorageByteOffset::from(8u64)).map_err(
                        |source: fabelgeist_storage::StorageBoundsError| -> NpyDecodeError {
                            NpyDecodeError::Bounds {
                                section: NpySection::HeaderLength,
                                source,
                            }
                        },
                    )?,
                )),
                StorageByteOffset::from(12u64),
            ),
            encoding => {
                return Err(NpyDecodeError::Version(NpyVersionEncoding::from(
                    <[u8; 2]>::try_from(encoding).expect("checked version width"),
                )));
            }
        };
        let header_bytes = view
            .portion(StorageByteSpan {
                offset: header_start,
                length: header_len,
            })
            .map_err(
                |source: fabelgeist_storage::StorageBoundsError| -> NpyDecodeError {
                    NpyDecodeError::Bounds {
                        section: NpySection::Header,
                        source,
                    }
                },
            )?;
        let start = header_start.advance(header_len).map_err(
            |source: fabelgeist_storage::StorageBoundsError| -> NpyDecodeError {
                NpyDecodeError::Bounds {
                    section: NpySection::Payload,
                    source,
                }
            },
        )?;
        let payload = view.tail_at(start).map_err(
            |source: fabelgeist_storage::StorageBoundsError| -> NpyDecodeError {
                NpyDecodeError::Bounds {
                    section: NpySection::Payload,
                    source,
                }
            },
        )?;
        Ok(Self {
            header: header_bytes,
            payload,
        })
    }
    fn decode(self) -> Result<NpyArray, NpyDecodeError> {
        let header = NpyHeader::try_from(&self.header)?;
        let dtype = header.dtype()?;
        header.admit_order()?;
        let shape = header.shape()?;
        let length = shape.element_count().payload_length(dtype)?;
        let available = self.payload.length();
        if length > available {
            return Err(NpyDecodeError::TruncatedPayload {
                expected: length,
                available,
            });
        }
        let payload = self
            .payload
            .portion(StorageByteSpan {
                offset: StorageByteOffset::default(),
                length,
            })
            .map_err(
                |source: fabelgeist_storage::StorageBoundsError| -> NpyDecodeError {
                    NpyDecodeError::Bounds {
                        section: NpySection::Payload,
                        source,
                    }
                },
            )?;
        Ok(NpyArray {
            shape,
            dtype,
            bytes: payload.as_ref().to_vec(),
        })
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use fabelgeist_fs::FileContents;
    use fixture::{FixtureDtype, FixtureShape, NpyFixture};

    #[test]
    fn parses_a_float_array() {
        let mut payload = Vec::new();
        for value in [1.0f32, -2.5, 3.25] {
            payload.extend_from_slice(&value.to_le_bytes());
        }
        let array = NpyArray::from_view(
            NpyFixture::from_parts(
                Dtype::F32.into(),
                FixtureShape::from(vec![NpyDimension::from(3)]),
                FileContents::from(payload),
            )
            .unwrap()
            .view(),
        )
        .unwrap();
        assert_eq!(array.shape().dimensions(), [NpyDimension::from(3)]);
        assert_eq!(array.dtype(), Dtype::F32);
        assert_eq!(Vec::<f32>::from(array.floating_values()), [1.0, -2.5, 3.25]);
    }

    #[test]
    fn parses_a_two_dimensional_int_array() {
        let mut payload = Vec::new();
        for value in [1i64, 2, 3, 4] {
            payload.extend_from_slice(&value.to_le_bytes());
        }
        let array = NpyArray::from_view(
            NpyFixture::from_parts(
                Dtype::I64.into(),
                FixtureShape::from(vec![NpyDimension::from(2), NpyDimension::from(2)]),
                FileContents::from(payload),
            )
            .unwrap()
            .view(),
        )
        .unwrap();
        assert_eq!(
            array.shape().dimensions(),
            [NpyDimension::from(2), NpyDimension::from(2)]
        );
        assert_eq!(Vec::<i64>::from(array.integer_values()), [1, 2, 3, 4]);
        assert_eq!(
            Vec::<f32>::from(array.floating_values()),
            [1.0, 2.0, 3.0, 4.0]
        );
    }

    #[test]
    fn parses_a_bool_array() {
        let array = NpyArray::from_view(
            NpyFixture::from_parts(
                Dtype::Bool.into(),
                FixtureShape::from(vec![NpyDimension::from(4)]),
                FileContents::from(vec![1, 0, 1, 1]),
            )
            .unwrap()
            .view(),
        )
        .unwrap();
        assert_eq!(
            array.byte_states().states(),
            [
                NpyByteState::Nonzero,
                NpyByteState::Zero,
                NpyByteState::Nonzero,
                NpyByteState::Nonzero
            ]
        );
    }

    #[test]
    fn parses_a_scalar_shape() {
        let array = NpyArray::from_view(
            NpyFixture::from_parts(
                Dtype::F32.into(),
                FixtureShape::from(vec![]),
                FileContents::from(1.5f32.to_le_bytes().to_vec()),
            )
            .unwrap()
            .view(),
        )
        .unwrap();
        assert_eq!(array.shape().rank(), NpyRank::from(0));
        assert_eq!(array.element_count(), NpyElementCount::from(1));
    }

    #[test]
    fn rejects_fortran_order_and_bad_dtypes() {
        let mut header = b"{'descr': '<f4', 'fortran_order': True, 'shape': (1,), }\n".to_vec();
        let mut bytes = b"\x93NUMPY\x01\x00".to_vec();
        bytes.extend_from_slice(&(header.len() as u16).to_le_bytes());
        bytes.append(&mut header);
        bytes.extend_from_slice(&1.0f32.to_le_bytes());
        assert!(NpyArray::from_view(StorageView::from(bytes.as_slice())).is_err());

        assert!(
            NpyArray::from_view(
                NpyFixture::from_parts(
                    FixtureDtype::Complex64,
                    FixtureShape::from(vec![NpyDimension::from(1)]),
                    FileContents::from(vec![0; 8])
                )
                .unwrap()
                .view()
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_a_truncated_payload() {
        assert!(
            NpyArray::from_view(
                NpyFixture::from_parts(
                    Dtype::F32.into(),
                    FixtureShape::from(vec![NpyDimension::from(4)]),
                    FileContents::from(vec![0; 8])
                )
                .unwrap()
                .view()
            )
            .is_err()
        );
    }
    #[test]
    fn malformed_versions_and_all_truncated_prefixes_are_structured_failures() {
        for version in [[1u8, 0], [2, 0], [3, 0]] {
            let mut encoded = Vec::<u8>::from(FileContents::from(
                NpyFixture::from_parts(
                    Dtype::F32.into(),
                    FixtureShape::from(vec![NpyDimension::from(1)]),
                    FileContents::from(1.5f32.to_le_bytes().to_vec()),
                )
                .unwrap(),
            ));
            encoded[6..8].copy_from_slice(&version);
            if version[0] != 1 {
                encoded.splice(10..10, [0, 0]);
            }
            let array = NpyArray::from_view(StorageView::from(encoded.as_slice())).unwrap();
            assert_eq!(Vec::<f32>::from(array.floating_values()), [1.5]);
            for end in 0..encoded.len() {
                assert!(
                    NpyArray::from_view(StorageView::from(&encoded[..end])).is_err(),
                    "version {version:?} prefix {end}"
                );
            }
        }
        for version in [[0u8, 0], [9, 0], [1, 1]] {
            let mut encoded = Vec::<u8>::from(FileContents::from(
                NpyFixture::from_parts(
                    Dtype::F32.into(),
                    FixtureShape::from(vec![NpyDimension::from(1)]),
                    FileContents::from(1.5f32.to_le_bytes().to_vec()),
                )
                .unwrap(),
            ));
            encoded[6..8].copy_from_slice(&version);
            assert!(matches!(
                NpyArray::from_view(StorageView::from(encoded.as_slice())),
                Err(NpyDecodeError::Version(_))
            ));
        }
    }
    #[test]
    fn shape_overflow_and_parse_causes_are_retained_without_panics() {
        let shape = FixtureShape::from(vec![NpyDimension::from(usize::MAX), NpyDimension::from(2)]);
        assert!(matches!(
            NpyArray::from_view(
                NpyFixture::from_parts(Dtype::F32.into(), shape, FileContents::default())
                    .unwrap()
                    .view()
            ),
            Err(NpyDecodeError::ElementCountOverflow)
        ));
        let shape = FixtureShape::BareDimension(NpyDimension::from(usize::MAX));
        assert!(matches!(
            NpyArray::from_view(
                NpyFixture::from_parts(Dtype::F32.into(), shape, FileContents::default())
                    .unwrap()
                    .view()
            ),
            Err(NpyDecodeError::PayloadLengthOverflow)
        ));
        let shape = FixtureShape::from(vec![
            NpyDimension::from(usize::MAX),
            NpyDimension::from(usize::MAX),
            NpyDimension::from(0),
        ]);
        let empty = NpyArray::from_view(
            NpyFixture::from_parts(Dtype::F32.into(), shape, FileContents::default())
                .unwrap()
                .view(),
        )
        .unwrap();
        assert_eq!(empty.element_count(), NpyElementCount::from(0));
        assert_eq!(empty.occupancy(), NpyOccupancy::Empty);
        let failure = match NpyArray::from_view(
            NpyFixture::from_parts(
                Dtype::F32.into(),
                FixtureShape::InvalidDimension,
                FileContents::default(),
            )
            .unwrap()
            .view(),
        ) {
            Err(error) => error,
            Ok(_) => panic!("invalid dimension accepted"),
        };
        assert!(
            std::error::Error::source(&failure)
                .unwrap()
                .is::<std::num::ParseIntError>()
        );
        assert!(matches!(failure, NpyDecodeError::ShapeDimension { .. }));
    }
    #[test]
    fn native_array_errors_keep_the_file_and_provider_or_decode_cause() {
        let directory = tempfile::tempdir().unwrap();
        let file = NativeFile::from(directory.path().join("array.npy"));
        let failure = match NpyArray::read(&file) {
            Err(error) => error,
            Ok(_) => panic!("missing array admitted"),
        };
        match failure {
            NpyReadError::Read {
                file: original,
                source,
            } => {
                assert_eq!(original.as_ref(), file.as_ref());
                assert_eq!(source.kind(), std::io::ErrorKind::NotFound);
            }
            _ => panic!("expected read failure"),
        }
        std::fs::write(file.as_ref(), b"invalid").unwrap();
        assert!(matches!(
            NpyArray::read(&file),
            Err(NpyReadError::Decode {
                source: NpyDecodeError::Magic,
                ..
            })
        ));
    }
    #[test]
    fn tensor_rank_admission_keeps_the_original_conversion_cause() {
        let array = NpyArray::from_view(
            NpyFixture::from_parts(
                Dtype::F32.into(),
                FixtureShape::from(vec![NpyDimension::from(1)]),
                FileContents::from(1.5f32.to_le_bytes().to_vec()),
            )
            .unwrap()
            .view(),
        )
        .unwrap();
        let failure = array.shape.tensor_layout::<2>().unwrap_err();
        match &failure {
            NpyDecodeError::Rank {
                expected, actual, ..
            } => {
                assert_eq!(*expected, NpyRank::from(2));
                assert_eq!(*actual, NpyRank::from(1));
            }
            _ => panic!("expected rank admission failure"),
        }
        assert!(
            std::error::Error::source(&failure)
                .unwrap()
                .is::<std::array::TryFromSliceError>()
        );
    }

    #[test]
    fn metadata_and_widening_retain_the_admitted_element_layout() {
        let mut payload = Vec::new();
        for value in [1.5f64, -2.5, 3.0] {
            payload.extend_from_slice(&value.to_le_bytes());
        }
        let array = NpyArray::from_view(
            NpyFixture::from_parts(
                Dtype::F64.into(),
                FixtureShape::from(vec![NpyDimension::from(3)]),
                FileContents::from(payload.clone()),
            )
            .unwrap()
            .view(),
        )
        .unwrap();
        assert_eq!(array.shape().dimensions(), [NpyDimension::from(3)]);
        assert_eq!(array.shape().rank(), NpyRank::from(1));
        assert_eq!(array.element_count(), NpyElementCount::from(3));
        assert_eq!(array.occupancy(), NpyOccupancy::Nonempty);
        assert_eq!(array.dtype(), Dtype::F64);
        assert_eq!(array.dtype().storage_width(), NpyElementWidth::Word64);
        assert_eq!(
            array.element_count().payload_length(array.dtype()).unwrap(),
            StorageByteLength::from(payload.len())
        );
        assert_eq!(array.payload().as_ref(), payload);
        assert_eq!(Vec::<f32>::from(array.floating_values()), [1.5, -2.5, 3.0]);
        assert_eq!(Vec::<i64>::from(array.integer_values()), [1, -2, 3]);
        assert_eq!(array.shape.tensor_layout::<1>().unwrap().dimensions, [3]);
    }

    #[test]
    fn header_admission_keeps_dtype_before_order_before_shape_failures() {
        let mut encoded = Vec::<u8>::from(FileContents::from(
            NpyFixture::from_parts(
                FixtureDtype::Complex64,
                FixtureShape::InvalidDimension,
                FileContents::default(),
            )
            .unwrap(),
        ));
        let false_at = encoded
            .windows(5)
            .position(|word: &[u8]| -> bool { word == b"False" })
            .unwrap();
        encoded[false_at..false_at + 5].copy_from_slice(b"True ");
        assert!(matches!(
            NpyArray::from_view(StorageView::from(encoded.as_slice())),
            Err(NpyDecodeError::Dtype(_))
        ));
        let mut encoded = Vec::<u8>::from(FileContents::from(
            NpyFixture::from_parts(
                Dtype::F32.into(),
                FixtureShape::InvalidDimension,
                FileContents::default(),
            )
            .unwrap(),
        ));
        let false_at = encoded
            .windows(5)
            .position(|word: &[u8]| -> bool { word == b"False" })
            .unwrap();
        encoded[false_at..false_at + 5].copy_from_slice(b"True ");
        assert!(matches!(
            NpyArray::from_view(StorageView::from(encoded.as_slice())),
            Err(NpyDecodeError::FortranOrder)
        ));
        let encoded = Vec::<u8>::from(FileContents::from(
            NpyFixture::from_parts(
                Dtype::F32.into(),
                FixtureShape::InvalidDimension,
                FileContents::default(),
            )
            .unwrap(),
        ));
        assert!(matches!(
            NpyArray::from_view(StorageView::from(encoded.as_slice())),
            Err(NpyDecodeError::ShapeDimension { .. })
        ));
    }
}
