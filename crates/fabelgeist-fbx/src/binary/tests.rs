use super::*;
use crate::Prop;
use std::io::Write;

#[derive(Clone, Copy)]
enum FixtureProperties {
    Scalars,
    OneInteger16,
    ArrayKinds,
    PlainArray,
    CompressedArray,
    ExtraArray,
    ExtraPlainArray,
    ShortArray,
    UnknownEncoding,
    BrokenInflate,
    BrokenChecksum,
    UnknownTag,
}
#[derive(Clone, Copy)]
enum FixtureFault {
    EndOutside,
    EndBeforeHeader,
    PropertyCount,
    PropertyLength,
    UnconsumedPropertyByte,
}
#[derive(Clone, Copy)]
enum ChildScope {
    WithinParent,
    BeyondParent,
}
struct EncodedFixtureProperties {
    data: Vec<u8>,
    count: FbxPropertyCount,
}
impl EncodedFixtureProperties {
    fn from_case(kind: FixtureProperties) -> Self {
        match kind {
            FixtureProperties::Scalars => Self::scalars(),
            FixtureProperties::ArrayKinds => Self::array_kinds(),
            FixtureProperties::OneInteger16 => Self {
                data: [vec![b'Y'], i16::MIN.to_le_bytes().to_vec()].concat(),
                count: FbxPropertyCount::from(1),
            },
            FixtureProperties::UnknownTag => Self {
                data: vec![b'?'],
                count: FbxPropertyCount::from(1),
            },
            _ => Self::integer64_array(kind),
        }
    }
    fn scalars() -> Self {
        let mut payload = Vec::new();
        payload.push(b'Y');
        payload.extend_from_slice(&i16::MIN.to_le_bytes());
        payload.extend_from_slice(&[b'C', 2]);
        payload.push(b'I');
        payload.extend_from_slice(&i32::MAX.to_le_bytes());
        payload.push(b'L');
        payload.extend_from_slice(&i64::MIN.to_le_bytes());
        payload.push(b'F');
        payload.extend_from_slice(&(-0.0f32).to_le_bytes());
        payload.push(b'D');
        payload.extend_from_slice(&16_777_217.0f64.to_le_bytes());
        payload.push(b'S');
        payload.extend_from_slice(&3u32.to_le_bytes());
        payload.extend_from_slice(&[0xff, 0, 1]);
        payload.push(b'R');
        payload.extend_from_slice(&2u32.to_le_bytes());
        payload.extend_from_slice(&[9, 7]);
        Self {
            data: payload,
            count: FbxPropertyCount::from(8),
        }
    }
    fn integer64_array(kind: FixtureProperties) -> Self {
        let mut payload = Vec::new();
        let mut raw = Vec::new();
        raw.extend_from_slice(&i64::MIN.to_le_bytes());
        raw.extend_from_slice(&i64::MAX.to_le_bytes());
        if matches!(
            kind,
            FixtureProperties::ExtraArray | FixtureProperties::ExtraPlainArray
        ) {
            raw.extend_from_slice(&[9; 8]);
        }
        if matches!(kind, FixtureProperties::BrokenChecksum) {
            raw.extend_from_slice(&[9; 4096]);
        }
        if matches!(kind, FixtureProperties::ShortArray) {
            raw.truncate(8);
        }
        let compressed = matches!(
            kind,
            FixtureProperties::CompressedArray
                | FixtureProperties::UnknownEncoding
                | FixtureProperties::BrokenInflate
                | FixtureProperties::BrokenChecksum
                | FixtureProperties::ExtraArray
        );
        if compressed {
            let mut encoder =
                flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
            encoder.write_all(&raw).unwrap();
            raw = encoder.finish().unwrap();
        }
        if matches!(kind, FixtureProperties::BrokenInflate) {
            raw = vec![0xff, 0x00, 0xff];
        }
        if matches!(kind, FixtureProperties::BrokenChecksum) {
            *raw.last_mut().unwrap() ^= 0xff;
        }
        payload.push(b'l');
        payload.extend_from_slice(&2u32.to_le_bytes());
        let encoding = if matches!(kind, FixtureProperties::UnknownEncoding) {
            2u32
        } else if compressed {
            1
        } else {
            0
        };
        payload.extend_from_slice(&encoding.to_le_bytes());
        payload.extend_from_slice(&u32::try_from(raw.len()).unwrap().to_le_bytes());
        payload.extend_from_slice(&raw);
        Self {
            data: payload,
            count: FbxPropertyCount::from(1),
        }
    }
    fn array_kinds() -> Self {
        let arrays = [
            (
                b'f',
                [0x8000_0000u32.to_le_bytes(), 0x7fc0_1234u32.to_le_bytes()].concat(),
            ),
            (
                b'd',
                [16_777_217.0f64.to_le_bytes(), (-0.0f64).to_le_bytes()].concat(),
            ),
            (
                b'i',
                [i32::MIN.to_le_bytes(), i32::MAX.to_le_bytes()].concat(),
            ),
            (
                b'l',
                [i64::MIN.to_le_bytes(), i64::MAX.to_le_bytes()].concat(),
            ),
            (b'b', vec![2, 255]),
        ];
        let mut data = Vec::new();
        for (tag, raw) in arrays {
            data.push(tag);
            data.extend_from_slice(&2u32.to_le_bytes());
            data.extend_from_slice(&0u32.to_le_bytes());
            data.extend_from_slice(&u32::try_from(raw.len()).unwrap().to_le_bytes());
            data.extend_from_slice(&raw);
        }
        Self {
            data,
            count: FbxPropertyCount::from(5),
        }
    }
}
struct BinaryFixture(Vec<u8>);
impl BinaryFixture {
    fn nested(format: HeaderFormat, scope: ChildScope) -> Self {
        let child = Self::from_properties(FixtureProperties::OneInteger16, format);
        let width = match format {
            HeaderFormat::Narrow => 13,
            HeaderFormat::Wide => 25,
        };
        let mut record = child.0[27..child.0.len() - width].to_vec();
        let child_end = 27 + width + 1 + record.len();
        let parent_end = child_end + width;
        let declared_child_end = match scope {
            ChildScope::WithinParent => child_end,
            ChildScope::BeyondParent => parent_end + 1,
        };
        match format {
            HeaderFormat::Narrow => record[..4]
                .copy_from_slice(&u32::try_from(declared_child_end).unwrap().to_le_bytes()),
            HeaderFormat::Wide => record[..8]
                .copy_from_slice(&u64::try_from(declared_child_end).unwrap().to_le_bytes()),
        }
        let mut file = child.0[..27].to_vec();
        for word in [u64::try_from(parent_end).unwrap(), 0, 0] {
            match format {
                HeaderFormat::Narrow => {
                    file.extend_from_slice(&u32::try_from(word).unwrap().to_le_bytes())
                }
                HeaderFormat::Wide => file.extend_from_slice(&word.to_le_bytes()),
            }
        }
        file.extend_from_slice(&[1, b'P']);
        file.extend_from_slice(&record);
        file.extend_from_slice(&vec![0; width * 2]);
        Self(file)
    }
    fn from_properties(kind: FixtureProperties, format: HeaderFormat) -> Self {
        let properties = EncodedFixtureProperties::from_case(kind);
        let payload = properties.data;
        let count = properties.count;
        let mut file = b"Kaydara FBX Binary  \x00\x1a\x00".to_vec();
        file.extend_from_slice(
            &match format {
                HeaderFormat::Narrow => 7400u32,
                HeaderFormat::Wide => 7500,
            }
            .to_le_bytes(),
        );
        let width = match format {
            HeaderFormat::Narrow => 13,
            HeaderFormat::Wide => 25,
        };
        let end = 27 + width + 1 + payload.len();
        for word in [
            u64::try_from(end).unwrap(),
            count.0,
            u64::try_from(payload.len()).unwrap(),
        ] {
            match format {
                HeaderFormat::Narrow => {
                    file.extend_from_slice(&u32::try_from(word).unwrap().to_le_bytes())
                }
                HeaderFormat::Wide => file.extend_from_slice(&word.to_le_bytes()),
            }
        }
        file.push(1);
        file.push(b'N');
        file.extend_from_slice(&payload);
        file.extend_from_slice(&vec![0; width]);
        Self(file)
    }
    fn fault(mut self, fault: FixtureFault) -> Self {
        match fault {
            FixtureFault::EndOutside => self.0[27..31].copy_from_slice(&u32::MAX.to_le_bytes()),
            FixtureFault::EndBeforeHeader => self.0[27..31].copy_from_slice(&28u32.to_le_bytes()),
            FixtureFault::PropertyCount => self.0[31..35].copy_from_slice(&u32::MAX.to_le_bytes()),
            FixtureFault::PropertyLength => self.0[35..39].copy_from_slice(&1u32.to_le_bytes()),
            FixtureFault::UnconsumedPropertyByte => {
                self.0[35..39].copy_from_slice(&4u32.to_le_bytes());
                // Extend both the declared block and node by one trailing byte.
                self.0.insert(44, 0);
                self.0[27..31].copy_from_slice(&45u32.to_le_bytes());
            }
        }
        self
    }
    fn view(&self) -> StorageView<'_> {
        StorageView::from(self.0.as_slice())
    }
}
#[test]
fn narrow_and_wide_headers_preserve_scalars_names_order_and_raw_strings() {
    for format in [HeaderFormat::Narrow, HeaderFormat::Wide] {
        let fixture = BinaryFixture::from_properties(FixtureProperties::Scalars, format);
        let roots = parse(fixture.view()).unwrap();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].name, crate::FbxRecordName::from("N"));
        let props = &roots[0].props;
        assert!(matches!(props[0], Prop::I16(i16::MIN)));
        assert!(matches!(props[1], Prop::Bool(true)));
        assert!(matches!(props[2], Prop::I32(i32::MAX)));
        assert!(matches!(props[3], Prop::I64(i64::MIN)));
        match props[4] {
            Prop::F32(value) => assert_eq!(value.to_bits(), 0x8000_0000),
            _ => panic!("wrong float32 property"),
        }
        assert!(matches!(props[5], Prop::F64(16_777_217.0)));
        assert_eq!(props[6].as_str().unwrap(), [0xff, 0, 1]);
        assert_eq!(props[7].as_str().unwrap(), [9, 7]);
    }
}
#[test]
fn array_plain_zlib_and_trailing_decoded_bytes_preserve_declared_values() {
    for kind in [
        FixtureProperties::PlainArray,
        FixtureProperties::CompressedArray,
        FixtureProperties::ExtraArray,
        FixtureProperties::ExtraPlainArray,
    ] {
        let fixture = BinaryFixture::from_properties(kind, HeaderFormat::Narrow);
        let roots = parse(fixture.view()).unwrap();
        assert_eq!(roots[0].i64_array().unwrap(), [i64::MIN, i64::MAX]);
    }
}
#[test]
fn malformed_array_metadata_and_inflate_causes_are_structured() {
    let fixture =
        BinaryFixture::from_properties(FixtureProperties::UnknownEncoding, HeaderFormat::Narrow);
    assert!(matches!(
        parse(fixture.view()),
        Err(FbxDecodeError::ArrayEncoding(_))
    ));
    let fixture =
        BinaryFixture::from_properties(FixtureProperties::ShortArray, HeaderFormat::Narrow);
    assert!(matches!(
        parse(fixture.view()),
        Err(FbxDecodeError::ArrayShort { .. })
    ));
    let fixture =
        BinaryFixture::from_properties(FixtureProperties::BrokenInflate, HeaderFormat::Narrow);
    let error = parse(fixture.view()).unwrap_err();
    assert!(matches!(error, FbxDecodeError::Inflate { .. }));
    assert!(
        std::error::Error::source(&error)
            .unwrap()
            .is::<std::io::Error>()
    );
    let fixture =
        BinaryFixture::from_properties(FixtureProperties::BrokenChecksum, HeaderFormat::Narrow);
    assert!(matches!(
        parse(fixture.view()),
        Err(FbxDecodeError::Inflate { .. })
    ));
}
#[test]
fn node_and_property_metadata_reject_before_unbounded_allocation_or_rewind() {
    for fault in [FixtureFault::EndOutside, FixtureFault::EndBeforeHeader] {
        let fixture =
            BinaryFixture::from_properties(FixtureProperties::Scalars, HeaderFormat::Narrow)
                .fault(fault);
        assert!(matches!(
            parse(fixture.view()),
            Err(FbxDecodeError::NodeExtent { .. })
        ));
    }
    let fixture = BinaryFixture::from_properties(FixtureProperties::Scalars, HeaderFormat::Narrow)
        .fault(FixtureFault::PropertyCount);
    assert!(matches!(
        parse(fixture.view()),
        Err(FbxDecodeError::PropertyCount { .. })
    ));
    let fixture = BinaryFixture::from_properties(FixtureProperties::Scalars, HeaderFormat::Narrow)
        .fault(FixtureFault::PropertyLength);
    assert!(matches!(
        parse(fixture.view()),
        Err(FbxDecodeError::PropertyCount { .. })
    ));
}

#[test]
fn nested_nodes_stay_within_their_parent_extent_in_both_formats() {
    for format in [HeaderFormat::Narrow, HeaderFormat::Wide] {
        let fixture = BinaryFixture::nested(format, ChildScope::WithinParent);
        let roots = parse(fixture.view()).unwrap();
        assert_eq!(roots[0].name, crate::FbxRecordName::from("P"));
        assert_eq!(roots[0].children[0].name, crate::FbxRecordName::from("N"));
        assert!(matches!(roots[0].children[0].props[0], Prop::I16(i16::MIN)));
        let escaped = BinaryFixture::nested(format, ChildScope::BeyondParent);
        assert!(matches!(
            parse(escaped.view()),
            Err(FbxDecodeError::NodeExtent { .. })
        ));
    }
}

#[test]
fn declared_property_length_must_match_the_consumed_properties() {
    let fixture =
        BinaryFixture::from_properties(FixtureProperties::OneInteger16, HeaderFormat::Narrow)
            .fault(FixtureFault::UnconsumedPropertyByte);
    match parse(fixture.view()).unwrap_err() {
        FbxDecodeError::PropertyLength { declared, actual } => {
            assert_eq!(declared, Length::from(4u64));
            assert_eq!(actual, Length::from(3u64));
        }
        other => panic!("wrong error classification: {other:?}"),
    }
}

#[test]
fn all_array_kinds_preserve_precision_signed_zero_nan_bits_and_boolean_bytes() {
    let fixture = BinaryFixture::from_properties(FixtureProperties::ArrayKinds, HeaderFormat::Wide);
    let roots = parse(fixture.view()).unwrap();
    match &roots[0].props[..] {
        [
            Prop::ArrF32(single),
            Prop::ArrF64(double),
            Prop::ArrI32(narrow),
            Prop::ArrI64(wide),
            Prop::ArrBool(states),
        ] => {
            assert_eq!(single[0].to_bits(), 0x8000_0000);
            assert_eq!(single[1].to_bits(), 0x7fc0_1234);
            assert_eq!(double[0], 16_777_217.0);
            assert_eq!(double[1].to_bits(), 0x8000_0000_0000_0000);
            assert_eq!(narrow, &[i32::MIN, i32::MAX]);
            assert_eq!(wide, &[i64::MIN, i64::MAX]);
            assert_eq!(states, &[2, 255]);
        }
        _ => panic!("wrong array kind or ordering"),
    }
}
#[test]
fn format_property_tags_and_partial_records_retain_classification() {
    for bytes in [
        b"; FBX ascii".as_slice(),
        b"\xef\xbb\xbf; FBX ascii".as_slice(),
    ] {
        assert!(matches!(
            parse(StorageView::from(bytes)),
            Err(FbxDecodeError::Format(FbxFormatViolation::Ascii))
        ));
    }
    assert!(matches!(
        parse(StorageView::from(b"wrong".as_slice())),
        Err(FbxDecodeError::Format(FbxFormatViolation::NotBinary))
    ));
    let fixture =
        BinaryFixture::from_properties(FixtureProperties::UnknownTag, HeaderFormat::Narrow);
    assert!(matches!(
        parse(fixture.view()),
        Err(FbxDecodeError::PropertyTag { .. })
    ));
    let fixture = BinaryFixture::from_properties(FixtureProperties::Scalars, HeaderFormat::Narrow);
    for end in 41..fixture.0.len() - 13 {
        assert!(
            parse(StorageView::from(&fixture.0[..end])).is_err(),
            "prefix {end}"
        );
    }
}
