use super::*;
use std::error::Error;

#[test]
fn scalar_and_zero_axes_have_distinct_rank_and_element_counts() {
    let scalar = NpyShape::try_from(Vec::new()).unwrap();
    assert_eq!(scalar.rank(), NpyRank::from(0));
    assert_eq!(scalar.element_count(), NpyElementCount::from(1));
    assert_eq!(scalar.element_count().occupancy(), NpyOccupancy::Nonempty);
    assert_eq!(scalar.tensor_layout::<0>().unwrap().dimensions, [0usize; 0]);
    let dimensions = vec![
        NpyDimension::from(usize::MAX),
        NpyDimension::from(usize::MAX),
        NpyDimension::from(0),
    ];
    let empty = NpyShape::try_from(dimensions.clone()).unwrap();
    assert_eq!(empty.dimensions(), dimensions);
    assert_eq!(empty.rank(), NpyRank::from(3));
    assert_eq!(empty.element_count(), NpyElementCount::from(0));
    assert_eq!(empty.element_count().occupancy(), NpyOccupancy::Empty);
    assert_eq!(
        empty.element_count().payload_length(Dtype::F64).unwrap(),
        NpyPayloadLength(0)
    );
}

#[test]
fn shape_payload_and_rank_failures_retain_their_distinct_context() {
    let dimensions = vec![NpyDimension::from(usize::MAX), NpyDimension::from(2)];
    let error = NpyShape::try_from(dimensions.clone()).unwrap_err();
    assert!(
        matches!(error, NpyLayoutError::ElementCountOverflow { dimensions: actual } if actual == dimensions)
    );
    let wide = NpyShape::try_from(vec![NpyDimension::from(usize::MAX)]).unwrap();
    let error = wide.element_count().payload_length(Dtype::F64).unwrap_err();
    assert!(
        matches!(error, NpyLayoutError::PayloadLengthOverflow { elements, width } if elements == NpyElementCount::from(usize::MAX) && width == NpyElementWidth::Word64)
    );
    assert_eq!(
        wide.element_count().payload_length(Dtype::U8).unwrap(),
        NpyPayloadLength(usize::MAX)
    );
    let shape = NpyShape::try_from(vec![NpyDimension::from(2), NpyDimension::from(3)]).unwrap();
    assert_eq!(shape.tensor_layout::<2>().unwrap().dimensions, [2, 3]);
    let error = match shape.tensor_layout::<1>() {
        Err(error) => error,
        Ok(_) => panic!("wrong rank was admitted"),
    };
    assert!(
        matches!(&error, NpyLayoutError::Rank { expected, actual, .. } if *expected == NpyRank::from(1) && *actual == NpyRank::from(2))
    );
    assert!(
        error
            .source()
            .unwrap()
            .is::<std::array::TryFromSliceError>()
    );
    assert_eq!(error.to_string(), "array has 2 dimensions, expected 1");
    let length = shape.element_count().payload_length(Dtype::F32).unwrap();
    let error = length.admit(&[0; 23]).unwrap_err();
    assert!(
        matches!(error, NpyLayoutError::TruncatedPayload { expected, available } if expected == NpyPayloadLength(24) && available == NpyPayloadLength(23))
    );
    assert_eq!(length.admit(&[7; 25]).unwrap(), &[7; 24]);
    assert_eq!(format!("{wide:?}"), format!("[{}]", usize::MAX));
}

#[test]
fn array_admission_retains_checked_shape_and_payload_consistency() {
    use super::super::{NpyArray, tests::encode};
    let array = NpyArray::from_bytes(&encode("<f4", "1,", &1.5f32.to_le_bytes())).unwrap();
    assert_eq!(array.element_count(), NpyElementCount::from(1));
    assert_eq!(array.dtype(), Dtype::F32);
    assert_eq!(array.dtype().storage_width(), NpyElementWidth::Word32);
    assert_eq!(array.to_f32(), [1.5]);
    assert_eq!(array.bytes(), &1.5f32.to_le_bytes());
    let dimensions = format!("{}, {}, 0", usize::MAX, usize::MAX);
    let empty = NpyArray::from_bytes(&encode("<f8", &dimensions, &[])).unwrap();
    assert_eq!(empty.occupancy(), NpyOccupancy::Empty);
    assert_eq!(empty.shape().rank(), NpyRank::from(3));
    assert!(empty.to_f32().is_empty());
    assert!(empty.bytes().is_empty());

    let dimensions = format!("{}, 2", usize::MAX);
    let error = match NpyArray::from_bytes(&encode("<f4", &dimensions, &[])) {
        Err(error) => error,
        Ok(_) => panic!("overflowing shape was admitted"),
    };
    assert!(matches!(
        error.downcast_ref::<NpyLayoutError>(),
        Some(NpyLayoutError::ElementCountOverflow { .. })
    ));
    let error = match NpyArray::from_bytes(&encode("<f8", &format!("{},", usize::MAX), &[])) {
        Err(error) => error,
        Ok(_) => panic!("overflowing payload was admitted"),
    };
    assert!(matches!(
        error.downcast_ref::<NpyLayoutError>(),
        Some(NpyLayoutError::PayloadLengthOverflow { .. })
    ));
}

#[test]
fn header_admission_still_precedes_shape_product_and_payload_checks() {
    use super::super::{NpyArray, tests::encode};
    let dimensions = format!("{}, 2", usize::MAX);
    let error = match NpyArray::from_bytes(&encode("<c8", &dimensions, &[])) {
        Err(error) => error,
        Ok(_) => panic!("unsupported dtype was admitted"),
    };
    assert!(error.to_string().starts_with("unsupported NumPy dtype"));
    let mut encoded = encode("<f4", &dimensions, &[]);
    let start = encoded
        .windows(5)
        .position(|window| window == b"False")
        .unwrap();
    encoded[start..start + 5].copy_from_slice(b"True ");
    let error = match NpyArray::from_bytes(&encoded) {
        Err(error) => error,
        Ok(_) => panic!("Fortran ordering was admitted"),
    };
    assert_eq!(
        error.to_string(),
        "Fortran-ordered .npy arrays are not supported"
    );
}

#[test]
fn every_stored_width_retains_element_cardinality_and_casts() {
    use super::super::{NpyArray, tests::encode};
    let double = [1.5f64, -2.25]
        .into_iter()
        .flat_map(f64::to_le_bytes)
        .collect::<Vec<_>>();
    let integer = [-7i32, 9]
        .into_iter()
        .flat_map(i32::to_le_bytes)
        .collect::<Vec<_>>();
    let cases = [
        ("<f8", double, vec![1.5, -2.25], vec![1, -2]),
        ("<i4", integer, vec![-7.0, 9.0], vec![-7, 9]),
        ("|u1", vec![0, 255], vec![0.0, 255.0], vec![0, 255]),
        ("|b1", vec![0, 255], vec![0.0, 255.0], vec![0, 255]),
    ];
    for (encoding, payload, floats, integers) in cases {
        let array = NpyArray::from_bytes(&encode(encoding, "2,", &payload)).unwrap();
        assert_eq!(array.element_count(), NpyElementCount::from(2));
        assert_eq!(array.to_f32(), floats, "{encoding}");
        assert_eq!(array.to_i64(), integers, "{encoding}");
        assert_eq!(array.bytes(), payload);
    }
}
