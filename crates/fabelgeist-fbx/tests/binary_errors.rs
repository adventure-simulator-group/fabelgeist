//! Public behavior of authored native FBX fixtures, including failure precedence.
use fabelgeist_fbx::{FbxDecodeError, Prop, Scene, parse};
use std::{error::Error as _, io};

#[test]
fn header_classification_distinguishes_ascii_from_other_rejected_bytes() {
    for bytes in [
        include_bytes!("fixtures/ascii.fbx").as_slice(),
        include_bytes!("fixtures/bom-ascii.fbx").as_slice(),
    ] {
        let error = parse(bytes).unwrap_err();
        assert!(matches!(error, FbxDecodeError::AsciiInput));
        assert_eq!(
            error.to_string(),
            "this is an ASCII FBX file; re-export it as binary FBX"
        );
        assert!(error.source().is_none());
        assert!(matches!(
            Scene::parse(bytes),
            Err(FbxDecodeError::AsciiInput)
        ));
    }
    for bytes in [
        include_bytes!("fixtures/empty.fbx").as_slice(),
        include_bytes!("fixtures/bad-magic.fbx").as_slice(),
        include_bytes!("fixtures/short-binary-header.fbx").as_slice(),
    ] {
        let error = parse(bytes).unwrap_err();
        assert!(matches!(error, FbxDecodeError::NotBinaryInput));
        assert_eq!(error.to_string(), "not a binary FBX file");
        assert!(error.source().is_none());
    }
}

#[test]
fn truncated_scalar_and_name_retain_native_request_and_position() {
    let error = parse(include_bytes!("fixtures/truncated-scalar.fbx")).unwrap_err();
    assert!(matches!(
        error,
        FbxDecodeError::TruncatedRead {
            native_offset: 41,
            native_length: 8
        }
    ));
    assert_eq!(
        error.to_string(),
        "truncated FBX file: wanted 8 bytes at offset 41"
    );
    let error = parse(include_bytes!("fixtures/truncated-name.fbx")).unwrap_err();
    assert!(matches!(
        error,
        FbxDecodeError::TruncatedRead {
            native_offset: 40,
            native_length: 4
        }
    ));
    assert_eq!(
        error.to_string(),
        "truncated FBX file: wanted 4 bytes at offset 40"
    );
}

#[test]
fn array_short_rejects_raw_and_successfully_inflated_buffers() {
    let error = parse(include_bytes!("fixtures/array-short.fbx")).unwrap_err();
    assert!(matches!(
        error,
        FbxDecodeError::ArrayShort {
            native_bytes: 4,
            native_count: 2,
            native_width: 4
        }
    ));
    assert_eq!(
        error.to_string(),
        "FBX array property is short: 4 bytes for 2 x 4"
    );
    let error = parse(include_bytes!("fixtures/compressed-short.fbx")).unwrap_err();
    assert!(matches!(
        error,
        FbxDecodeError::ArrayShort {
            native_bytes: 8,
            native_count: 2,
            native_width: 8
        }
    ));
    assert_eq!(
        error.to_string(),
        "FBX array property is short: 8 bytes for 2 x 8"
    );
}

#[test]
fn inflate_failure_retains_original_io_cause_and_separate_context() {
    let error = Scene::parse(include_bytes!("fixtures/inflate.fbx"))
        .err()
        .unwrap();
    let FbxDecodeError::Inflate { source } = &error else {
        panic!("unexpected error: {error:?}");
    };
    assert_eq!(source.kind(), io::ErrorKind::InvalidInput);
    assert_eq!(source.to_string(), "corrupt deflate stream");
    assert_eq!(error.to_string(), "inflating FBX array property");
    let cause = error.source().unwrap().downcast_ref::<io::Error>().unwrap();
    assert!(std::ptr::eq(cause, source));
    assert!(cause.source().is_none());
}

#[test]
fn first_property_failure_keeps_native_unknown_tag_and_rejection_order() {
    let error = parse(include_bytes!("fixtures/unknown-tag.fbx")).unwrap_err();
    assert!(matches!(
        error,
        FbxDecodeError::PropertyTag { native_tag: 255 }
    ));
    assert_eq!(error.to_string(), "unknown FBX property type 'ÿ'");
    let error = parse(include_bytes!("fixtures/unknown-before-short.fbx")).unwrap_err();
    assert!(matches!(
        error,
        FbxDecodeError::PropertyTag { native_tag: b'?' }
    ));
    assert_eq!(error.to_string(), "unknown FBX property type '?'");
    let error = parse(include_bytes!("fixtures/short-before-unknown.fbx")).unwrap_err();
    assert!(matches!(
        error,
        FbxDecodeError::ArrayShort {
            native_bytes: 0,
            native_count: 2,
            native_width: 4
        }
    ));
    assert_eq!(
        error.to_string(),
        "FBX array property is short: 0 bytes for 2 x 4"
    );
}

#[test]
fn accepted_layouts_keep_float_bits_lossy_names_and_nonzero_encoding_policy() {
    // Literal byte fixtures are the native version/encoding construction port.
    for bytes in [
        include_bytes!("fixtures/version0.fbx").as_slice(),
        include_bytes!("fixtures/version7100.fbx").as_slice(),
        include_bytes!("fixtures/version7400.fbx").as_slice(),
        include_bytes!("fixtures/version7499.fbx").as_slice(),
        include_bytes!("fixtures/version7500.fbx").as_slice(),
        include_bytes!("fixtures/version7700.fbx").as_slice(),
        include_bytes!("fixtures/version-max.fbx").as_slice(),
    ] {
        let roots = parse(bytes).unwrap();
        assert_eq!(
            roots
                .iter()
                .map(|node| node.name.as_str())
                .collect::<Vec<_>>(),
            ["�\0\nLeaf", "Objects", "Connections"]
        );
        let props = &roots[0].props;
        assert!(matches!(props[0], Prop::I64(i64::MIN)));
        assert!(matches!(props[1], Prop::I64(i64::MAX)));
        assert!(matches!(props[2], Prop::F64(value) if value.to_bits() == 0x8000_0000_0000_0000));
        assert!(matches!(&props[3], Prop::Raw(bytes) if bytes == b"\xff\0raw"));
        assert!(matches!(&props[4], Prop::Str(bytes) if bytes == b"ns:\0\xff\x01label"));
        assert!(matches!(props[5], Prop::Bool(true)));
        let Prop::ArrF32(values) = &props[6] else {
            panic!("wrong property type");
        };
        assert_eq!(
            values
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            [0, 0x8000_0000, 1, 0x7f80_0000, 0xff80_0000, 0x7fc0_1234]
        );
        for index in [7, 14, 15, 16] {
            let Prop::ArrF64(values) = &props[index] else {
                panic!("wrong property type");
            };
            assert_eq!(
                values
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>(),
                [
                    0,
                    0x8000_0000_0000_0000,
                    1,
                    0x7ff0_0000_0000_0000,
                    0xfff0_0000_0000_0000,
                    0x7ff8_0000_0000_1234
                ]
            );
        }
        assert!(matches!(&props[8], Prop::ArrI32(values) if values == &[-1, -1]));
        assert!(matches!(&props[9], Prop::ArrI64(values) if values == &[i64::MIN]));
        assert!(matches!(&props[10], Prop::ArrBool(values) if values == &[0, 1, 2, 255]));
        assert!(matches!(props[11], Prop::I16(i16::MIN)));
        assert!(matches!(props[12], Prop::I32(i32::MAX)));
        assert!(matches!(props[13], Prop::F32(value) if value.to_bits() == 0x7fc0_abcd));
        assert!(matches!(&props[17], Prop::ArrF32(values) if values.is_empty()));
        assert!(
            matches!(&props[18], Prop::ArrF32(values) if values.len() == 1 && values[0].to_bits() == 0)
        );
        let scene = Scene::from_roots(roots);
        assert_eq!(
            scene
                .objects
                .iter()
                .map(|object| object.id)
                .collect::<Vec<_>>(),
            [1, 2, 1]
        );
        assert_eq!(scene.get(1).unwrap().qualified, "other:duplicate");
        assert_eq!(
            scene
                .children(99)
                .map(|object| object.id)
                .collect::<Vec<_>>(),
            [2, 1, 2]
        );
        assert_eq!(
            scene.children_with_property(99).next().unwrap().1,
            Some("d|X�")
        );
    }
}

#[test]
fn named_null_and_partial_trailing_header_keep_empty_scene_behavior() {
    for bytes in [
        include_bytes!("fixtures/named-null.fbx").as_slice(),
        include_bytes!("fixtures/partial-header.fbx").as_slice(),
    ] {
        assert!(parse(bytes).unwrap().is_empty());
        assert!(Scene::parse(bytes).unwrap().objects.is_empty());
    }
}
