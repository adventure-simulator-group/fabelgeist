use std::mem::size_of;

use super::{Node, Prop, parse};

// Expected wire layout stays independent of the parser's layout calculation.
const FIRST_WIDE_HEADER_VERSION: u32 = 7500;
const NARROW_HEADER_FIXTURE_VERSION: u32 = 7400;
const FIXTURE_VERSIONS: [u32; 2] = [NARROW_HEADER_FIXTURE_VERSION, FIRST_WIDE_HEADER_VERSION];
// Three offset/count/length words followed by a one-byte name length.
const NARROW_NODE_HEADER_BYTES: usize = 13;
const WIDE_NODE_HEADER_BYTES: usize = 25;
const FBX_BINARY_SIGNATURE: &[u8] = b"Kaydara FBX Binary  \0\x1a\0";
const FILE_HEADER_BYTES: usize = FBX_BINARY_SIGNATURE.len() + size_of::<u32>();
const ROOT_NODE_NAME: &[u8] = b"Root";
const CHILD_NODE_NAME: &[u8] = b"Kid";

// Payloads have zero to three properties. This declaration exceeds every payload.
const EXCESS_DECLARED_PROPERTY_COUNT: u64 = 7;
const DECLARED_PROPERTY_COUNTS: [u64; 5] = [0, 1, 2, 3, EXCESS_DECLARED_PROPERTY_COUNT];
// Nest the empty, integer, integer/boolean and boolean/short payload cases.
const NESTED_SCALAR_CASE_COUNT: usize = 4;
// Null records must ignore their property count, even at either wire-word maximum.
const UNUSED_NULL_PROPERTY_COUNTS: [u64; 4] = [0, 1, u32::MAX as u64, u64::MAX];
const NULL_NODE_END_OFFSET: u64 = 0;
const TRUNCATED_NULL_NAME_DECLARED_BYTES: u8 = 3;
const TRUNCATED_NULL_NAME_BYTES: &[u8] = b"x";

// Counts, versions and offsets below author native serialized fixture fields.
fn header_width(version: u32) -> usize {
    if version >= FIRST_WIDE_HEADER_VERSION {
        WIDE_NODE_HEADER_BYTES
    } else {
        NARROW_NODE_HEADER_BYTES
    }
}

fn word(bytes: &mut Vec<u8>, version: u32, value: u64) {
    if version >= FIRST_WIDE_HEADER_VERSION {
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
    let mut bytes = FBX_BINARY_SIGNATURE.to_vec();
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
    let raw_text = b"\xff\0x";
    string.extend((raw_text.len() as u32).to_le_bytes());
    string.extend(raw_text);
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
    for version in FIXTURE_VERSIONS {
        for (case, properties) in property_lists().iter().enumerate() {
            // Each payload stays below a node-header width, so extra properties
            // cannot accidentally become a speculative child allocation.
            assert!(properties.iter().map(Vec::len).sum::<usize>() < header_width(version));
            for count in DECLARED_PROPERTY_COUNTS {
                let bytes = file(
                    version,
                    &record(
                        version,
                        FILE_HEADER_BYTES,
                        ROOT_NODE_NAME,
                        count,
                        properties,
                        &[],
                    ),
                );
                out.push_str(&format!(
                    "{version}-{case}-declared-{count}:{}\n",
                    snapshot(&bytes)
                ));
            }
        }
        for (case, properties) in property_lists()
            .into_iter()
            .take(NESTED_SCALAR_CASE_COUNT)
            .enumerate()
        {
            let parent_props = vec![int(11)];
            let parent_payload_bytes = parent_props.iter().map(Vec::len).sum::<usize>();
            let child_offset = FILE_HEADER_BYTES
                + header_width(version)
                + ROOT_NODE_NAME.len()
                + parent_payload_bytes;
            let child = record(
                version,
                child_offset,
                CHILD_NODE_NAME,
                properties.len() as u64,
                &properties,
                &[],
            );
            let parent = record(
                version,
                FILE_HEADER_BYTES,
                ROOT_NODE_NAME,
                parent_props.len() as u64,
                &parent_props,
                &child,
            );
            out.push_str(&format!(
                "{version}-nested-{case}:{}\n",
                snapshot(&file(version, &parent))
            ));
        }
        for count in UNUSED_NULL_PROPERTY_COUNTS {
            let mut null = Vec::new();
            word(&mut null, version, NULL_NODE_END_OFFSET);
            word(&mut null, version, count);
            word(&mut null, version, 0); // No property payload.
            null.push(0); // Empty node name.
            out.push_str(&format!(
                "{version}-null-{count}:{}\n",
                snapshot(&file(version, &null))
            ));
            null.pop();
            // Declare three name bytes but provide only one.
            null.push(TRUNCATED_NULL_NAME_DECLARED_BYTES);
            null.extend(TRUNCATED_NULL_NAME_BYTES);
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
