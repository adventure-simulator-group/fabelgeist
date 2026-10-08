use fabelgeist_numpy_storage::{
    Dtype, NpyArray,
    npy::{DecodeResult, NpyDecodeError, NpyHeaderField, NpyReadError},
};
use std::{error::Error, io, num::ParseIntError, path::Path, str::Utf8Error};

fn rejected<T>(result: DecodeResult<T>) -> NpyDecodeError {
    result
        .err()
        .expect("fixture must reject at its public decoder boundary")
}

#[test]
fn magic_and_extent_failures_remain_distinct() {
    let error = rejected(NpyArray::from_bytes(include_bytes!(
        "fixtures/npy-errors/wrong-magic.npy"
    )));
    assert!(matches!(error, NpyDecodeError::Magic));
    assert_eq!(error.to_string(), "not a .npy array");
    assert!(error.source().is_none());
    let error = rejected(NpyArray::from_bytes(include_bytes!(
        "fixtures/npy-errors/truncated-header.npy"
    )));
    assert!(matches!(error, NpyDecodeError::TruncatedHeader));
    assert_eq!(error.to_string(), "truncated .npy header");
    let error = rejected(NpyArray::from_bytes(include_bytes!(
        "fixtures/npy-errors/truncated-payload.npy"
    )));
    assert!(matches!(error, NpyDecodeError::TruncatedPayload));
    assert_eq!(error.to_string(), "truncated .npy payload");
}

#[test]
fn missing_fields_and_descriptor_keep_their_rejected_native_roles() {
    let error = rejected(NpyArray::from_bytes(include_bytes!(
        "fixtures/npy-errors/missing-descr.npy"
    )));
    assert!(matches!(
        error,
        NpyDecodeError::MissingField(NpyHeaderField::Dtype)
    ));
    assert_eq!(error.to_string(), "missing 'descr' in .npy header");
    let error = rejected(NpyArray::from_bytes(include_bytes!(
        "fixtures/npy-errors/missing-shape.npy"
    )));
    assert!(matches!(
        error,
        NpyDecodeError::MissingField(NpyHeaderField::Shape)
    ));
    assert_eq!(error.to_string(), "missing 'shape' in .npy header");
    let error = rejected(NpyArray::from_bytes(include_bytes!(
        "fixtures/npy-errors/unsupported-dtype.npy"
    )));
    assert!(matches!(&error, NpyDecodeError::Dtype { native_descr } if native_descr == ">f4"));
    assert_eq!(error.to_string(), "unsupported NumPy dtype \">f4\"");
    assert!(error.source().is_none());
    let error = rejected(NpyArray::from_bytes(include_bytes!(
        "fixtures/npy-errors/fortran.npy"
    )));
    assert!(matches!(error, NpyDecodeError::FortranOrder));
    assert_eq!(
        error.to_string(),
        "Fortran-ordered .npy arrays are not supported"
    );
}

#[test]
fn rejected_encoding_and_shape_preserve_original_standard_causes() {
    let error = rejected(NpyArray::from_bytes(include_bytes!(
        "fixtures/npy-errors/header-nonutf8.npy"
    )));
    let NpyDecodeError::HeaderEncoding(source) = &error else {
        panic!("wrong classification: {error:?}")
    };
    assert_eq!(source.valid_up_to(), 0);
    assert_eq!(source.error_len(), Some(1));
    assert!(std::ptr::eq(
        source,
        error.source().unwrap().downcast_ref::<Utf8Error>().unwrap()
    ));
    assert_eq!(error.to_string(), "non-UTF-8 .npy header");
    let error = rejected(NpyArray::from_bytes(include_bytes!(
        "fixtures/npy-errors/shape-token.npy"
    )));
    let NpyDecodeError::ShapeDimension {
        native_token,
        source,
    } = &error
    else {
        panic!("wrong classification: {error:?}")
    };
    assert_eq!(native_token, "-1");
    assert_eq!(*source.kind(), std::num::IntErrorKind::InvalidDigit);
    assert!(std::ptr::eq(
        source,
        error
            .source()
            .unwrap()
            .downcast_ref::<ParseIntError>()
            .unwrap()
    ));
    assert_eq!(error.to_string(), "bad .npy shape");
    let outer: anyhow::Error = error.into();
    assert!(outer.downcast_ref::<NpyDecodeError>().is_some());
    assert!(outer.downcast_ref::<ParseIntError>().is_none());
    assert!(
        outer
            .chain()
            .any(|cause| cause.downcast_ref::<ParseIntError>().is_some())
    );
}

#[test]
fn native_file_read_and_decode_have_different_owning_causes() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/npy-errors");
    let missing = fixtures.join("not-present.npy");
    let error = NpyArray::read(&missing).err().unwrap();
    let NpyReadError::Read {
        native_path,
        source,
    } = &error
    else {
        panic!("wrong classification: {error:?}")
    };
    assert_eq!(native_path, &missing);
    assert_eq!(source.kind(), io::ErrorKind::NotFound);
    assert_eq!(error.to_string(), format!("reading {}", missing.display()));
    assert!(std::ptr::eq(
        source,
        error.source().unwrap().downcast_ref::<io::Error>().unwrap()
    ));
    let bad = fixtures.join("shape-token.npy");
    let error = NpyArray::read(&bad).err().unwrap();
    let NpyReadError::Decode {
        native_path,
        source,
    } = &error
    else {
        panic!("wrong classification: {error:?}")
    };
    assert_eq!(native_path, &bad);
    assert!(matches!(source, NpyDecodeError::ShapeDimension { .. }));
    assert_eq!(error.to_string(), format!("parsing {}", bad.display()));
    assert!(std::ptr::eq(
        source,
        error
            .source()
            .unwrap()
            .downcast_ref::<NpyDecodeError>()
            .unwrap()
    ));
    assert!(source.source().unwrap().is::<ParseIntError>());
}

#[test]
fn associated_admission_keeps_values_and_public_reinterpretation_behavior() {
    let mut array =
        NpyArray::from_bytes(include_bytes!("fixtures/npy-errors/valid-f32.npy")).unwrap();
    assert_eq!(array.dtype, Dtype::F32);
    assert_eq!(array.shape, [6]);
    assert_eq!(
        array
            .to_f32()
            .iter()
            .map(|v| v.to_bits())
            .collect::<Vec<_>>(),
        [0, 0x8000_0000, 1, 0x7f80_0000, 0xff80_0000, 0x7fc0_1234]
    );
    let original_bytes = array.bytes().to_vec();
    array.dtype = Dtype::I64;
    array.shape = vec![9];
    assert_eq!(array.len(), 9);
    assert_eq!(array.bytes(), original_bytes);
    assert_eq!(array.to_i64().len(), 3);
    assert_eq!(array.to_bool().len(), original_bytes.len());
    // Authored native NPY fixture: five byte-valued elements become one
    // complete little-endian i32 word after current public reinterpretation.
    let header = b"{'descr': '|u1', 'shape': (5,), }";
    let mut native = b"\x93NUMPY\x01\x00".to_vec();
    native.extend_from_slice(&(header.len() as u16).to_le_bytes());
    native.extend_from_slice(header);
    native.extend_from_slice(&[0, 1, 2, 3, 4]);
    let mut partial = NpyArray::from_bytes(&native).unwrap();
    partial.dtype = Dtype::I32;
    assert_eq!(partial.to_i64(), [0x0302_0100]);
    assert_eq!(partial.to_bool(), [false, true, true, true, true]);
    let integer =
        NpyArray::from_bytes(include_bytes!("fixtures/npy-errors/valid-i64.npy")).unwrap();
    assert_eq!(
        integer.to_i64(),
        [
            0,
            -1,
            i64::MIN,
            i64::MAX,
            9_007_199_254_740_993,
            -9_007_199_254_740_993
        ]
    );
}

#[test]
fn rank_rejection_keeps_the_native_array_port_before_tensor_creation() {
    let array = NpyArray::from_bytes(include_bytes!("fixtures/npy-errors/valid-f32.npy")).unwrap();
    let device = Default::default();
    let float = array.to_tensor::<2>(&device).err().unwrap();
    let integer = array.to_int_tensor::<2>(&device).err().unwrap();
    for error in [float, integer] {
        assert_eq!(error.to_string(), "array has 1 dimensions, expected 2");
        assert!(
            error
                .downcast_ref::<std::array::TryFromSliceError>()
                .is_some()
        );
        assert!(error.downcast_ref::<NpyDecodeError>().is_none());
    }
}
