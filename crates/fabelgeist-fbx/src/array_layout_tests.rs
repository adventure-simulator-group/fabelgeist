use super::{Prop, parse};
use flate2::{Compression, write::ZlibEncoder};
use std::io::Write;

// These fields are native serialized FBX fixture metadata, before admission.
fn compressed(payload: &[u8]) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(payload).unwrap();
    encoder.finish().unwrap()
}

fn file(tag: u8, count: u32, encoding: u32, raw: &[u8], version: u32) -> Vec<u8> {
    let mut property = vec![tag];
    for value in [count, encoding, raw.len() as u32] {
        property.extend(value.to_le_bytes());
    }
    property.extend(raw);
    let mut bytes = b"Kaydara FBX Binary  \0\x1a\0".to_vec();
    bytes.extend(version.to_le_bytes());
    let header_len = if version >= 7500 { 25 } else { 13 };
    let end = (27 + header_len + 4 + property.len() + header_len) as u64;
    for value in [end, 1, property.len() as u64] {
        if version >= 7500 {
            bytes.extend(value.to_le_bytes());
        } else {
            bytes.extend((value as u32).to_le_bytes());
        }
    }
    bytes.push(4);
    bytes.extend(b"Root");
    bytes.extend(property);
    bytes.resize(bytes.len() + header_len, 0);
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
    for version in [7400, 7500] {
        for (tag, payload) in &kinds {
            for encoding in [0, 1, 7, u32::MAX] {
                let mut extra = payload.clone();
                extra.extend([0x33; 9]);
                for (label, count, decoded) in [
                    ("normal", 2, payload.as_slice()),
                    ("empty", 0, [].as_slice()),
                    ("short", 2, [0x7f].as_slice()),
                    ("extra", 2, extra.as_slice()),
                    ("one", 1, payload.as_slice()),
                    ("three-short", 3, payload.as_slice()),
                ] {
                    let raw = if encoding == 0 {
                        decoded.to_vec()
                    } else {
                        compressed(decoded)
                    };
                    out.push_str(&format!(
                        "{version}-{}-{encoding}-{label}:{}\n",
                        char::from(*tag),
                        snapshot(&file(*tag, count, encoding, &raw, version))
                    ));
                }
                if encoding != 0 {
                    out.push_str(&format!(
                        "{version}-{}-{encoding}-corrupt:{}\n",
                        char::from(*tag),
                        snapshot(&file(*tag, 2, encoding, &[0xff, 0xff], version))
                    ));
                }
            }
            // Raw short data returns before allocation based on the declared count.
            // It still copies raw bytes. This count fits native multiplication
            // even on 32-bit hosts; extreme compressed capacities are not run.
            let count = u32::MAX / 8;
            out.push_str(&format!(
                "{version}-{}-large-raw:{}\n",
                char::from(*tag),
                snapshot(&file(*tag, count, 0, &[0x7f], version))
            ));
        }
        out.push_str(&format!(
            "{version}-bool-max-raw:{}\n",
            snapshot(&file(b'b', u32::MAX, 0, &[0x7f], version))
        ));
    }
    out
}

#[test]
fn array_cardinality_and_layout_preserve_actual_main_values_and_errors() {
    assert_eq!(observe(), include_str!("fixtures/array_layout.txt"));
}
