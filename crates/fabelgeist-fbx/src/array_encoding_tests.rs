use super::{Prop, parse};
use flate2::{Compression, write::ZlibEncoder};
use std::io::Write;

// These fields are native serialized FBX fixture metadata, before admission.
fn compressed(payload: &[u8]) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(payload).unwrap();
    encoder.finish().unwrap()
}

fn file(tag: u8, count: u32, encoding: u32, raw: &[u8]) -> Vec<u8> {
    let mut property = vec![tag];
    for value in [count, encoding, raw.len() as u32] {
        property.extend(value.to_le_bytes());
    }
    property.extend(raw);
    let mut bytes = b"Kaydara FBX Binary  \0\x1a\0".to_vec();
    bytes.extend(7400u32.to_le_bytes());
    let end = (27 + 13 + 4 + property.len() + 13) as u32;
    for value in [end, 1, property.len() as u32] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.push(4);
    bytes.extend(b"Root");
    bytes.extend(property);
    bytes.resize(bytes.len() + 13, 0);
    bytes
}

fn value(prop: &Prop) -> String {
    match prop {
        Prop::ArrF32(values) => format!(
            "f32bits:{:?}",
            values.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
        ),
        Prop::ArrF64(values) => format!(
            "f64bits:{:?}",
            values.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
        ),
        Prop::ArrI32(values) => format!("i32:{values:?}"),
        Prop::ArrI64(values) => format!("i64:{values:?}"),
        Prop::ArrBool(values) => format!("bool-bytes:{values:?}"),
        other => format!("other:{other:?}"),
    }
}

fn snapshot(bytes: &[u8]) -> String {
    match parse(bytes) {
        Ok(nodes) => nodes
            .iter()
            .map(|node| {
                format!(
                    "name={:?};children={};props={:?}",
                    node.name,
                    node.children.len(),
                    node.props.iter().map(value).collect::<Vec<_>>()
                )
            })
            .collect::<Vec<_>>()
            .join("|"),
        Err(error) => format!("error:{error:#}"),
    }
}

fn observe() -> String {
    let mut f32s = (-0.0f32).to_bits().to_le_bytes().to_vec();
    f32s.extend(0x7fc0_1234u32.to_le_bytes());
    let mut f64s = (-0.0f64).to_bits().to_le_bytes().to_vec();
    f64s.extend(0x7ff8_0000_0000_4321u64.to_le_bytes());
    let mut i32s = (-17i32).to_le_bytes().to_vec();
    i32s.extend(i32::MAX.to_le_bytes());
    let mut i64s = i64::MIN.to_le_bytes().to_vec();
    i64s.extend(99i64.to_le_bytes());
    let kinds = [
        (b'f', f32s),
        (b'd', f64s),
        (b'i', i32s),
        (b'l', i64s),
        (b'b', vec![0, 127]),
    ];
    let mut out = String::new();
    for (tag, payload) in kinds {
        for encoding in [0, 1, 7, u32::MAX] {
            let mut extra = payload.clone();
            extra.extend([0x33; 9]);
            for (label, count, decoded) in [
                ("normal", 2, payload.as_slice()),
                ("empty", 0, [].as_slice()),
                ("short", 2, [0x7f].as_slice()),
                ("extra", 2, extra.as_slice()),
            ] {
                let raw = if encoding == 0 {
                    decoded.to_vec()
                } else {
                    compressed(decoded)
                };
                out.push_str(&format!(
                    "{}-{encoding}-{label}:{}\n",
                    char::from(tag),
                    snapshot(&file(tag, count, encoding, &raw))
                ));
            }
            if encoding != 0 {
                out.push_str(&format!(
                    "{}-{encoding}-corrupt:{}\n",
                    char::from(tag),
                    snapshot(&file(tag, 2, encoding, &[0xff, 0xff]))
                ));
            }
        }
    }
    out
}

#[test]
fn array_encoding_preserves_actual_main_payload_and_error_behavior() {
    assert_eq!(observe(), include_str!("fixtures/array_encoding.txt"));
}
