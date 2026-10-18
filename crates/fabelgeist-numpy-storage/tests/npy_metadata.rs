use std::error::Error;

use burn::tensor::Device;
use fabelgeist_numpy_storage::{
    Dtype, NpyArray, NpyDimension, NpyElementCount, NpyElementWidth, NpyLayoutError, NpyOccupancy,
    NpyRank, NpyShape,
};

fn encoded(dtype: &str, dimensions: &str, payload: &[u8]) -> Vec<u8> {
    let header =
        format!("{{'descr': '{dtype}', 'fortran_order': False, 'shape': ({dimensions}), }}\n");
    let mut bytes = b"\x93NUMPY\x01\x00".to_vec();
    bytes.extend(u16::try_from(header.len()).unwrap().to_le_bytes());
    bytes.extend(header.bytes());
    bytes.extend(payload);
    bytes
}

#[test]
fn decoded_queries_keep_scalar_rank_count_and_storage_width_distinct() {
    let scalar = NpyArray::from_bytes(&encoded("<f4", "", &1.5f32.to_le_bytes())).unwrap();
    assert_eq!(scalar.shape().rank(), NpyRank::from(0));
    assert_eq!(scalar.element_count(), NpyElementCount::from(1));
    assert_eq!(scalar.occupancy(), NpyOccupancy::Nonempty);
    assert_eq!(scalar.dtype(), Dtype::F32);
    assert_eq!(scalar.dtype().storage_width(), NpyElementWidth::Word32);
    assert_eq!(scalar.bytes(), &1.5f32.to_le_bytes());
    assert_eq!(
        Vec::<f32>::from(fabelgeist_numpy_storage::NpyFloatValues::from(&scalar)),
        [1.5]
    );
    assert_eq!(format!("{:?}", scalar.shape()), "[]");
    let path = std::env::temp_dir().join(format!("npy-layout-{}.npy", std::process::id()));
    std::fs::write(&path, encoded("<f4", "", &1.5f32.to_le_bytes())).unwrap();
    let read = NpyArray::read(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    assert_eq!(read.shape(), scalar.shape());
    assert_eq!(read.bytes(), scalar.bytes());
}

#[test]
fn zero_axes_precede_product_overflow_and_preserve_the_authored_shape() {
    for position in 0..3 {
        let mut axes = [usize::MAX; 3];
        axes[position] = 0;
        let dimensions = axes.map(NpyDimension::from).to_vec();
        let shape = NpyShape::try_from(dimensions.clone()).unwrap();
        assert_eq!(shape.dimensions(), dimensions);
        assert_eq!(shape.rank(), NpyRank::from(3));
        assert_eq!(shape.element_count(), NpyElementCount::from(0));
        assert_eq!(shape.element_count().occupancy(), NpyOccupancy::Empty);
        let text = format!("{}, {}, {}", axes[0], axes[1], axes[2]);
        let array = NpyArray::from_bytes(&encoded("<f8", &text, &[])).unwrap();
        assert_eq!(array.shape(), &shape);
        assert!(array.bytes().is_empty());
    }
}

#[test]
fn layout_failures_keep_rejected_dimensions_counts_widths_and_lengths() {
    let dimensions = vec![NpyDimension::from(usize::MAX), NpyDimension::from(2)];
    let error = NpyShape::try_from(dimensions.clone()).unwrap_err();
    assert!(
        matches!(error, NpyLayoutError::ElementCountOverflow { dimensions: actual } if actual == dimensions)
    );
    let error = NpyElementCount::from(usize::MAX)
        .payload_length(Dtype::F64)
        .unwrap_err();
    assert!(
        matches!(error, NpyLayoutError::PayloadLengthOverflow { elements, width } if elements == NpyElementCount::from(usize::MAX) && width == NpyElementWidth::Word64)
    );
    let error = NpyArray::from_bytes(&encoded("<f4", "2,", &[0; 7]))
        .err()
        .unwrap();
    let layout = match &error {
        fabelgeist_numpy_storage::NpyDecodeError::Layout(layout) => Some(layout),
        _ => None,
    }
    .unwrap();
    assert!(
        matches!(layout, NpyLayoutError::TruncatedPayload { expected, available } if expected.to_string() == "8" && available.to_string() == "7")
    );
    assert_eq!(error.to_string(), "truncated .npy payload");
    assert!(layout.source().is_none());
}

#[test]
fn rank_failure_occurs_before_device_upload_and_keeps_its_native_cause() {
    let array = NpyArray::from_bytes(&encoded("|u1", "", &[7])).unwrap();
    let error = array.to_tensor::<1>(&Device::default()).err().unwrap();
    assert!(
        matches!(&error, NpyLayoutError::Rank { expected, actual, .. } if *expected == NpyRank::from(1) && *actual == NpyRank::from(0))
    );
    assert_eq!(error.to_string(), "array has 0 dimensions, expected 1");
    assert!(
        error
            .source()
            .unwrap()
            .is::<std::array::TryFromSliceError>()
    );
    let integer = array.to_int_tensor::<1>(&Device::default()).err().unwrap();
    assert_eq!(integer.to_string(), error.to_string());
}

#[test]
fn header_priority_and_trailing_payload_policy_remain_observable() {
    let axes = format!("{}, 2", usize::MAX);
    let error = NpyArray::from_bytes(&encoded("<c8", &axes, &[]))
        .err()
        .unwrap();
    assert_eq!(error.to_string(), "unsupported NumPy dtype \"<c8\"");
    let mut bytes = encoded("<f4", &axes, &[]);
    let position = bytes.windows(5).position(|v| v == b"False").unwrap();
    bytes[position..position + 5].copy_from_slice(b"True ");
    let error = NpyArray::from_bytes(&bytes).err().unwrap();
    assert_eq!(
        error.to_string(),
        "Fortran-ordered .npy arrays are not supported"
    );
    let array = NpyArray::from_bytes(&encoded("|u1", "2,", &[0, 255, 19])).unwrap();
    assert_eq!(array.bytes(), &[0, 255]);
    assert_eq!(
        Vec::<bool>::from(fabelgeist_numpy_storage::NpyByteStates::from(&array)),
        [false, true]
    );
}

#[test]
fn stored_values_keep_float_bits_integer_casts_and_byte_boolean_semantics() {
    let words = [0.0f32, -0.0, f32::INFINITY, f32::from_bits(0x7fc1_2345)];
    let payload = words
        .into_iter()
        .flat_map(f32::to_le_bytes)
        .collect::<Vec<_>>();
    let array = NpyArray::from_bytes(&encoded("<f4", "2, 2", &payload)).unwrap();
    assert_eq!(array.shape().dimensions(), [NpyDimension::from(2); 2]);
    assert_eq!(array.element_count(), NpyElementCount::from(4));
    assert_eq!(
        Vec::<f32>::from(fabelgeist_numpy_storage::NpyFloatValues::from(&array))
            .iter()
            .map(|v| v.to_bits())
            .collect::<Vec<_>>(),
        words.map(f32::to_bits)
    );
    assert_eq!(
        Vec::<i64>::from(fabelgeist_numpy_storage::NpyIntegerValues::from(&array)),
        [0, 0, i64::MAX, 0]
    );
    assert_eq!(
        Vec::<bool>::from(fabelgeist_numpy_storage::NpyByteStates::from(&array)),
        payload.iter().map(|v| *v != 0).collect::<Vec<_>>()
    );
}
