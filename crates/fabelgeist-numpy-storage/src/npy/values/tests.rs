use super::*;
use crate::npy::NpyDimension;
use crate::npy::fixture::{FixtureShape, NpyFixture};
use fabelgeist_fs::FileContents;

enum CastFixture {
    Float32,
    Float64,
    Integer32,
    Integer64,
    Byte,
    Boolean,
}
impl CastFixture {
    fn array(self) -> NpyArray {
        let mut payload = Vec::new();
        let (dtype, shape) = match self {
            Self::Float32 => {
                for value in [
                    0.0f32,
                    -0.0,
                    1.9,
                    -1.9,
                    f32::from_bits(0x7fc1_2345),
                    f32::INFINITY,
                    f32::NEG_INFINITY,
                    f32::MAX,
                ] {
                    payload.extend_from_slice(&value.to_le_bytes());
                }
                (Dtype::F32, NpyDimension::from(8))
            }
            Self::Float64 => {
                for value in [
                    16_777_217.0f64,
                    -16_777_217.0,
                    f64::MAX,
                    f64::NAN,
                    f64::INFINITY,
                    f64::NEG_INFINITY,
                ] {
                    payload.extend_from_slice(&value.to_le_bytes());
                }
                (Dtype::F64, NpyDimension::from(6))
            }
            Self::Integer32 => {
                for value in [i32::MIN, 0, i32::MAX] {
                    payload.extend_from_slice(&value.to_le_bytes());
                }
                (Dtype::I32, NpyDimension::from(3))
            }
            Self::Integer64 => {
                for value in [i64::MIN, -9_007_199_254_740_993, i64::MAX] {
                    payload.extend_from_slice(&value.to_le_bytes());
                }
                (Dtype::I64, NpyDimension::from(3))
            }
            Self::Byte | Self::Boolean => {
                payload.extend_from_slice(&[0, 1, 2, 255]);
                (
                    if matches!(self, Self::Byte) {
                        Dtype::U8
                    } else {
                        Dtype::Bool
                    },
                    NpyDimension::from(4),
                )
            }
        };
        let fixture = NpyFixture::from_parts(
            dtype.into(),
            FixtureShape::from(vec![shape]),
            FileContents::from(payload),
        )
        .unwrap();
        NpyArray::from_view(fixture.view()).unwrap()
    }
}

#[test]
fn floating_storage_preserves_signed_zero_nan_bits_and_saturating_integer_casts() {
    let array = CastFixture::Float32.array();
    let words = StoredValues::from(&array);
    assert_eq!(words.size_hint(), (8, Some(8)));
    let floats = array.floating_values();
    assert_eq!(floats.element_count(), array.element_count());
    assert_eq!(f32::from(floats.values()[0]).to_bits(), 0);
    assert_eq!(f32::from(floats.values()[1]).to_bits(), 0x8000_0000);
    assert_eq!(f32::from(floats.values()[4]).to_bits(), 0x7fc1_2345);
    assert_eq!(
        Vec::<i64>::from(array.integer_values()),
        [0, 0, 1, -1, 0, i64::MAX, i64::MIN, i64::MAX]
    );
}

#[test]
fn float64_integer_conversion_keeps_precision_before_f32_rounding() {
    let array = CastFixture::Float64.array();
    let floats = array.floating_values();
    assert_eq!(f32::from(floats.values()[0]), 16_777_216.0);
    assert_eq!(f32::from(floats.values()[1]), -16_777_216.0);
    assert_eq!(f32::from(floats.values()[2]), f32::INFINITY);
    assert!(f32::from(floats.values()[3]).is_nan());
    assert_eq!(
        Vec::<i64>::from(array.integer_values()),
        [16_777_217, -16_777_217, i64::MAX, 0, i64::MAX, i64::MIN]
    );
}

#[test]
fn signed_words_and_noncanonical_bool_bytes_keep_existing_numeric_values() {
    let integers32 = CastFixture::Integer32.array().floating_values();
    assert_eq!(f32::from(integers32.values()[0]).to_bits(), 0xcf00_0000);
    assert_eq!(f32::from(integers32.values()[2]).to_bits(), 0x4f00_0000);
    let integers64 = CastFixture::Integer64.array().floating_values();
    assert_eq!(f32::from(integers64.values()[0]).to_bits(), 0xdf00_0000);
    assert_eq!(f32::from(integers64.values()[1]).to_bits(), 0xda00_0000);
    assert_eq!(f32::from(integers64.values()[2]).to_bits(), 0x5f00_0000);
    assert_eq!(
        Vec::<i64>::from(CastFixture::Integer32.array().integer_values()),
        [i64::from(i32::MIN), 0, i64::from(i32::MAX)]
    );
    assert_eq!(
        Vec::<i64>::from(CastFixture::Integer64.array().integer_values()),
        [i64::MIN, -9_007_199_254_740_993, i64::MAX]
    );
    for fixture in [CastFixture::Byte, CastFixture::Boolean] {
        let array = fixture.array();
        assert_eq!(
            Vec::<f32>::from(array.floating_values()),
            [0.0, 1.0, 2.0, 255.0]
        );
        assert_eq!(Vec::<i64>::from(array.integer_values()), [0, 1, 2, 255]);
        assert_eq!(
            array.byte_states().states(),
            [
                NpyByteState::Zero,
                NpyByteState::Nonzero,
                NpyByteState::Nonzero,
                NpyByteState::Nonzero,
            ]
        );
    }
}

#[test]
fn byte_states_classify_payload_bytes_independently_of_logical_elements() {
    let array = CastFixture::Float32.array();
    let states = array.byte_states();
    assert_eq!(states.byte_length(), array.payload().length());
    assert_eq!(
        &states.states()[..8],
        [
            NpyByteState::Zero,
            NpyByteState::Zero,
            NpyByteState::Zero,
            NpyByteState::Zero,
            NpyByteState::Zero,
            NpyByteState::Zero,
            NpyByteState::Zero,
            NpyByteState::Nonzero,
        ]
    );
    assert_eq!(array.element_count(), NpyElementCount::from(8));
    assert_eq!(states.byte_length(), StorageByteLength::from(32usize));
}

#[test]
fn value_ordinals_and_block_products_reject_overflow_and_outside_access() {
    let values = CastFixture::Byte.array().integer_values();
    assert_eq!(
        values.value(NpyElementOrdinal::from(3)),
        Some(NpyIntegerValue::from(255))
    );
    assert_eq!(values.value(NpyElementOrdinal::from(4)), None);
    assert_eq!(values.value(NpyElementOrdinal::from(usize::MAX)), None);
    assert_eq!(
        NpyElementOrdinal::from(usize::MAX).advance(NpyElementCount::from(1)),
        None
    );
    assert_eq!(
        NpyElementCount::from(usize::MAX).repeated(NpyDimension::from(2)),
        None
    );
    assert_eq!(
        NpyElementCount::from(0).repeated(NpyDimension::from(usize::MAX)),
        Some(NpyElementCount::from(0))
    );
    assert!(usize::try_from(NpyIntegerValue::from(-1)).is_err());
    assert_eq!(usize::try_from(NpyIntegerValue::from(255)).unwrap(), 255);
}
