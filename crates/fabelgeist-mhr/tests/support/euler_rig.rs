//! Authored narrow-header binary FBX fixture with a root, mesh and skin.
//! It encodes wire bytes only; the public Character loader parses the fixture.

struct Node {
    name: &'static str,
    properties: Vec<u8>,
    count: u32,
    children: Vec<Node>,
}
fn node(name: &'static str, props: Vec<Vec<u8>>, children: Vec<Node>) -> Node {
    Node {
        name,
        count: props.len() as u32,
        properties: props.concat(),
        children,
    }
}
fn string(value: &str) -> Vec<u8> {
    let mut bytes = vec![b'S'];
    bytes.extend((value.len() as u32).to_le_bytes());
    bytes.extend(value.as_bytes());
    bytes
}
fn integer(value: i64) -> Vec<u8> {
    let mut bytes = vec![b'L'];
    bytes.extend(value.to_le_bytes());
    bytes
}
fn real(value: f64) -> Vec<u8> {
    let mut bytes = vec![b'D'];
    bytes.extend(value.to_le_bytes());
    bytes
}
fn reals(values: &[f64]) -> Vec<u8> {
    let mut bytes = vec![b'd'];
    bytes.extend((values.len() as u32).to_le_bytes());
    bytes.extend(0u32.to_le_bytes());
    bytes.extend((values.len() as u32 * 8).to_le_bytes());
    for value in values {
        bytes.extend(value.to_le_bytes());
    }
    bytes
}
fn integers(values: &[i64]) -> Vec<u8> {
    let mut bytes = vec![b'l'];
    bytes.extend((values.len() as u32).to_le_bytes());
    bytes.extend(0u32.to_le_bytes());
    bytes.extend((values.len() as u32 * 8).to_le_bytes());
    for value in values {
        bytes.extend(value.to_le_bytes());
    }
    bytes
}
fn property(name: &str, values: Vec<Vec<u8>>) -> Node {
    let mut props = vec![string(name), string("kind"), string(""), string("")];
    props.extend(values);
    node("P", props, vec![])
}
fn object(kind: &'static str, id: i64, class: &str, children: Vec<Node>) -> Node {
    node(
        kind,
        vec![integer(id), string(&format!("ns:item{id}")), string(class)],
        children,
    )
}
fn connection(child: i64, parent: i64) -> Node {
    node(
        "C",
        vec![string("OO"), integer(child), integer(parent)],
        vec![],
    )
}
fn encode(record: &Node, bytes: &mut Vec<u8>) {
    let start = bytes.len();
    bytes.resize(start + 13, 0);
    bytes[start + 12] = record.name.len() as u8;
    bytes.extend(record.name.as_bytes());
    bytes.extend(&record.properties);
    for child in &record.children {
        encode(child, bytes);
    }
    bytes.extend([0; 13]);
    let end = bytes.len() as u32;
    bytes[start..start + 4].copy_from_slice(&end.to_le_bytes());
    bytes[start + 4..start + 8].copy_from_slice(&record.count.to_le_bytes());
    bytes[start + 8..start + 12].copy_from_slice(&(record.properties.len() as u32).to_le_bytes());
}
pub fn rig(order: Option<i64>) -> Vec<u8> {
    let mut props = vec![
        property("Lcl Rotation", [10.0, -20.0, 30.0].map(real).to_vec()),
        property("PreRotation", [-5.0, 15.0, 7.0].map(real).to_vec()),
        property("Lcl Translation", [1.5, -2.0, 0.25].map(real).to_vec()),
    ];
    if let Some(code) = order {
        props.push(property("RotationOrder", vec![integer(code)]));
    }
    let objects = node(
        "Objects",
        vec![],
        vec![
            object(
                "Model",
                1,
                "Root",
                vec![node("Properties70", vec![], props)],
            ),
            object("Model", 10, "Mesh", vec![]),
            object(
                "Geometry",
                11,
                "Mesh",
                vec![
                    node(
                        "Vertices",
                        vec![reals(&[0., 0., 0., 1., 0., 0., 0., 1., 0.])],
                        vec![],
                    ),
                    node("PolygonVertexIndex", vec![integers(&[0, 1, -3])], vec![]),
                ],
            ),
            object("Deformer", 12, "Skin", vec![]),
            object(
                "Deformer",
                13,
                "Cluster",
                vec![
                    node("Indexes", vec![integers(&[0, 1, 2])], vec![]),
                    node("Weights", vec![reals(&[1., 1., 1.])], vec![]),
                ],
            ),
        ],
    );
    let links = node(
        "Connections",
        vec![],
        vec![
            connection(1, 0),
            connection(10, 0),
            connection(11, 10),
            connection(12, 11),
            connection(13, 12),
            connection(1, 13),
        ],
    );
    let mut bytes = b"Kaydara FBX Binary  \0\x1a\0".to_vec();
    bytes.extend(7400u32.to_le_bytes());
    encode(&objects, &mut bytes);
    encode(&links, &mut bytes);
    bytes.extend([0; 13]);
    bytes
}
