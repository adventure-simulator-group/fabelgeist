//! Record/class selectors retain rig, skin, blend-shape and UV admission.
use super::*;
use fabelgeist_fbx::{Node, Prop};

fn object(kind: &str, id: i64, class: Option<Prop>) -> Node {
    let mut props = vec![Prop::I64(id), Prop::Str(b"ns:item\0\x01ignored".to_vec())];
    if let Some(class) = class {
        props.push(class);
    }
    Node {
        name: kind.into(),
        props,
        children: vec![],
    }
}
#[test]
fn record_and_class_selection_keeps_original_rig_payload() -> Result<()> {
    let mut bone = object("Model", 10, Some(Prop::Str(b"Root".to_vec())));
    bone.props[1] = Prop::Str(b"root".to_vec());
    let mut geometry = object("Geometry", 20, Some(Prop::Str(b"Mesh".to_vec())));
    geometry.children.push(Node {
        name: "LayerElementUV".into(),
        props: vec![],
        children: vec![Node {
            name: "UV".into(),
            props: vec![Prop::ArrF64(vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0])],
            children: vec![],
        }],
    });
    let skin = object("Deformer", 30, Some(Prop::Str(b"Skin".to_vec())));
    let mut cluster = object("Deformer", 40, Some(Prop::Str(b"Cluster".to_vec())));
    cluster.children = vec![
        Node {
            name: "Indexes".into(),
            props: vec![Prop::ArrI32(vec![0, 1, 2])],
            children: vec![],
        },
        Node {
            name: "Weights".into(),
            props: vec![Prop::ArrF64(vec![0.25, 0.5, 1.0])],
            children: vec![],
        },
    ];
    let blend = object("Deformer", 50, Some(Prop::Str(b"BlendShape".to_vec())));
    let channel = object(
        "Deformer",
        60,
        Some(Prop::Str(b"BlendShapeChannel".to_vec())),
    );
    let mut shape = object("Geometry", 70, Some(Prop::Str(b"Shape".to_vec())));
    shape.children = vec![
        Node {
            name: "Indexes".into(),
            props: vec![Prop::ArrI32(vec![1])],
            children: vec![],
        },
        Node {
            name: "Vertices".into(),
            props: vec![Prop::ArrF64(vec![1.0, 2.0, 3.0])],
            children: vec![],
        },
    ];
    let objects = Node {
        name: "Objects".into(),
        props: vec![],
        children: vec![bone, geometry, skin, cluster, blend, channel, shape],
    };
    let links = Node {
        name: "Connections".into(),
        props: vec![],
        children: [
            (10, 0),
            (30, 20),
            (40, 30),
            (10, 40),
            (50, 20),
            (60, 50),
            (70, 60),
        ]
        .into_iter()
        .map(|(from, to)| Node {
            name: "C".into(),
            props: vec![Prop::Str(b"OO".to_vec()), Prop::I64(from), Prop::I64(to)],
            children: vec![],
        })
        .collect(),
    };
    let rig = Scene::from_roots(vec![objects, links])?;
    let (skeleton, ids) = parse_skeleton(&rig);
    let geometry = &rig
        .objects_of_kind(&FbxRecordName::GEOMETRY, &FbxClassName::MESH)
        .next()
        .unwrap();
    let mut inverse = vec![
        [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0]
        ];
        skeleton.len()
    ];
    let joints = ids.iter().enumerate().map(|(i, id)| (*id, i)).collect();
    let skin = parse_skin(&rig, geometry, &joints, 3, &mut inverse)?;
    let blend = parse_blend_shapes(&rig, geometry, 3);
    let uv = parse_uvs(geometry, 3)?;
    let observed = format!(
        "skeleton={skeleton:?};ids={ids:?};skin={skin:?};inverse={inverse:?};blend={blend:?};uv={uv:?}"
    );
    assert_eq!(observed, include_str!("fixtures/selectors.txt").trim_end());
    Ok(())
}
