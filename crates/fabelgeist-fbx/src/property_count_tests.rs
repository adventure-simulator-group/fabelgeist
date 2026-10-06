use super::{Node, Prop, parse};

// Counts, versions and offsets below author native serialized fixture fields.
fn header_width(version: u32) -> usize {
    if version >= 7500 { 25 } else { 13 }
}

fn word(bytes: &mut Vec<u8>, version: u32, value: u64) {
    if version >= 7500 {
        bytes.extend(value.to_le_bytes());
    } else {
        bytes.extend((value as u32).to_le_bytes());
    }
}

fn record(
    version: u32,
    offset: usize,
    name: &[u8],
    count: u64,
    properties: &[Vec<u8>],
    children: &[u8],
) -> Vec<u8> {
    let payload_len = properties.iter().map(Vec::len).sum::<usize>();
    let end = offset + header_width(version) + name.len() + payload_len + children.len();
    let mut bytes = Vec::new();
    for value in [end as u64, count, payload_len as u64] {
        word(&mut bytes, version, value);
    }
    bytes.push(name.len() as u8);
    bytes.extend(name);
    for property in properties {
        bytes.extend(property);
    }
    bytes.extend(children);
    bytes
}

fn file(version: u32, record: &[u8]) -> Vec<u8> {
    let mut bytes = b"Kaydara FBX Binary  \0\x1a\0".to_vec();
    bytes.extend(version.to_le_bytes());
    bytes.extend(record);
    bytes
}

fn int(value: i32) -> Vec<u8> {
    let mut bytes = vec![b'I'];
    bytes.extend(value.to_le_bytes());
    bytes
}

fn property_lists() -> Vec<Vec<Vec<u8>>> {
    let mut short = vec![b'Y'];
    short.extend((-9i16).to_le_bytes());
    let mut string = vec![b'S'];
    string.extend(3u32.to_le_bytes());
    string.extend([255, 0, b'x']);
    vec![
        vec![],
        vec![int(-17)],
        vec![int(99), vec![b'C', 255]],
        vec![vec![b'C', 0], vec![b'C', 255], short],
        vec![string, vec![b'C', 0]],
    ]
}

fn node(node: &Node) -> String {
    let properties = node
        .props
        .iter()
        .map(|prop| match prop {
            Prop::F32(v) => format!("f32bits:{}", v.to_bits()),
            Prop::F64(v) => format!("f64bits:{}", v.to_bits()),
            other => format!("{other:?}"),
        })
        .collect::<Vec<_>>();
    format!(
        "name={:?};props={properties:?};children={:?}",
        node.name,
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
    for version in [7400, 7500] {
        for (case, properties) in property_lists().iter().enumerate() {
            // Each payload stays below a node-header width, so extra properties
            // cannot accidentally become a speculative child allocation.
            assert!(properties.iter().map(Vec::len).sum::<usize>() < header_width(version));
            for count in [0, 1, 2, 3, 7] {
                let bytes = file(
                    version,
                    &record(version, 27, b"Root", count, properties, &[]),
                );
                out.push_str(&format!(
                    "{version}-{case}-declared-{count}:{}\n",
                    snapshot(&bytes)
                ));
            }
        }
        for (case, properties) in property_lists().into_iter().take(4).enumerate() {
            let parent_props = vec![int(11)];
            let child_offset = 27 + header_width(version) + 4 + parent_props[0].len();
            let child = record(
                version,
                child_offset,
                b"Kid",
                properties.len() as u64,
                &properties,
                &[],
            );
            let parent = record(version, 27, b"Root", 1, &parent_props, &child);
            out.push_str(&format!(
                "{version}-nested-{case}:{}\n",
                snapshot(&file(version, &parent))
            ));
        }
        for count in [0, 1, u64::from(u32::MAX), u64::MAX] {
            let mut null = Vec::new();
            for value in [0, count, 0] {
                word(&mut null, version, value);
            }
            null.push(0);
            out.push_str(&format!(
                "{version}-null-{count}:{}\n",
                snapshot(&file(version, &null))
            ));
            null.pop();
            null.extend([3, b'x']);
            out.push_str(&format!(
                "{version}-null-{count}-short-name:{}\n",
                snapshot(&file(version, &null))
            ));
        }
    }
    out
}

#[test]
fn property_cardinality_preserves_actual_main_records_and_errors() {
    assert_eq!(observe(), include_str!("fixtures/property_count.txt"));
}
