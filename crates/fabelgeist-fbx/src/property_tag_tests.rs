use super::{Node, Prop, parse};

// Tag bytes, lengths and version words are native serialized fixture fields.
fn file(properties: &[Vec<u8>]) -> Vec<u8> {
    let mut bytes = b"Kaydara FBX Binary  \0\x1a\0".to_vec();
    bytes.extend(7400u32.to_le_bytes());
    let payload_len = properties.iter().map(Vec::len).sum::<usize>();
    let end = (27 + 13 + 4 + payload_len) as u32;
    for value in [end, properties.len() as u32, payload_len as u32] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.push(4);
    bytes.extend(b"Root");
    for property in properties {
        bytes.extend(property);
    }
    bytes
}

fn payload(tag: u8) -> Vec<u8> {
    match tag {
        b'Y' => i16::MIN.to_le_bytes().to_vec(),
        b'C' => vec![255],
        b'I' => i32::MIN.to_le_bytes().to_vec(),
        b'L' => i64::MAX.to_le_bytes().to_vec(),
        b'F' => 0x7fc0_1234u32.to_le_bytes().to_vec(),
        b'D' => (-0.0f64).to_bits().to_le_bytes().to_vec(),
        b'f' | b'd' | b'i' | b'l' | b'b' => {
            let data = match tag {
                b'f' => (-0.0f32).to_bits().to_le_bytes().to_vec(),
                b'd' => 0x7ff8_0000_0000_4321u64.to_le_bytes().to_vec(),
                b'i' => (-17i32).to_le_bytes().to_vec(),
                b'l' => i64::MIN.to_le_bytes().to_vec(),
                _ => vec![127],
            };
            let mut out = Vec::new();
            for value in [1, 0, data.len() as u32] {
                out.extend(value.to_le_bytes());
            }
            out.extend(data);
            out
        }
        b'S' | b'R' => {
            let data = if tag == b'S' {
                vec![0xff, 0, b'x']
            } else {
                vec![0, 1, 255]
            };
            let mut out = (data.len() as u32).to_le_bytes().to_vec();
            out.extend(data);
            out
        }
        _ => Vec::new(),
    }
}

fn property(tag: u8, payload: &[u8]) -> Vec<u8> {
    let mut out = vec![tag];
    out.extend(payload);
    out
}

fn value(prop: &Prop) -> String {
    match prop {
        Prop::F32(v) => format!("f32bits:{}", v.to_bits()),
        Prop::F64(v) => format!("f64bits:{}", v.to_bits()),
        Prop::ArrF32(v) => format!(
            "f32bits:{:?}",
            v.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
        ),
        Prop::ArrF64(v) => format!(
            "f64bits:{:?}",
            v.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
        ),
        other => format!("{other:?}"),
    }
}

fn node(node: &Node) -> String {
    format!(
        "name={:?};props={:?};children={:?}",
        node.name,
        node.props.iter().map(value).collect::<Vec<_>>(),
        node.children.iter().map(self::node).collect::<Vec<_>>()
    )
}

fn snapshot(bytes: &[u8]) -> String {
    match parse(bytes) {
        Ok(nodes) => nodes.iter().map(node).collect::<Vec<_>>().join("|"),
        Err(error) => format!("error:{error:#}"),
    }
}

fn observe() -> String {
    let mut out = String::new();
    for tag in 0..=u8::MAX {
        out.push_str(&format!(
            "tag-{tag}:{}\n",
            snapshot(&file(&[property(tag, &payload(tag))]))
        ));
    }
    let tags = *b"YCILFDfdilbSR";
    for tag in tags {
        let data = payload(tag);
        for (label, truncated) in [
            ("missing", [].as_slice()),
            ("short", &data[..data.len() - 1]),
        ] {
            out.push_str(&format!(
                "tag-{tag}-{label}:{}\n",
                snapshot(&file(&[property(tag, truncated)]))
            ));
        }
    }
    let properties = tags
        .iter()
        .map(|&tag| property(tag, &payload(tag)))
        .collect::<Vec<_>>();
    out.push_str(&format!("mixed-forward:{}\n", snapshot(&file(&properties))));
    let reverse = properties.into_iter().rev().collect::<Vec<_>>();
    out.push_str(&format!("mixed-reverse:{}\n", snapshot(&file(&reverse))));
    out
}

#[test]
fn property_tags_preserve_actual_main_values_and_rejection_behavior() {
    assert_eq!(observe(), include_str!("fixtures/property_tags.txt"));
}
