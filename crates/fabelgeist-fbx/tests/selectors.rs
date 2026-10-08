use fabelgeist_fbx::{FbxClassName, FbxRecordName, Node, Prop, Scene};

fn leaf(name: &str, marker: i64) -> Node {
    Node {
        name: name.into(),
        props: vec![Prop::I64(marker)],
        children: vec![],
    }
}
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
fn native_root(name: &[u8], version: u32) -> Vec<u8> {
    let mut bytes = b"Kaydara FBX Binary  \0\x1a\0".to_vec();
    bytes.extend(version.to_le_bytes());
    let wide = version >= 7500;
    let header = if wide { 25 } else { 13 };
    let end = (27 + header + name.len() + header) as u64;
    for value in [end, 0, 0] {
        if wide {
            bytes.extend(value.to_le_bytes());
        } else {
            bytes.extend((value as u32).to_le_bytes());
        }
    }
    bytes.push(name.len() as u8);
    bytes.extend(name);
    bytes.resize(bytes.len() + header, 0);
    bytes
}
#[test]
fn native_decoding_and_ordered_selectors_match_original() -> anyhow::Result<()> {
    let mut output = String::new();
    for version in [7400, 7500] {
        for bytes in [
            b"unknown: record\0\x01suffix".as_slice(),
            b" bad\xffLabel ".as_slice(),
            b"".as_slice(),
            b"Model".as_slice(),
        ] {
            let roots = fabelgeist_fbx::parse(&native_root(bytes, version))?;
            output.push_str(&format!(
                "decode{version}:{roots:?};encoded={:?}",
                roots[0].name.encoded()
            ));
            output.push('\n');
        }
    }
    let parent = Node {
        name: "Parent".into(),
        props: vec![],
        children: vec![
            leaf("Repeat", 1),
            leaf("other", 2),
            leaf("Repeat", 3),
            leaf("repeat", 4),
        ],
    };
    output.push_str(&format!(
        "first:{:?}",
        parent.child(&FbxRecordName::from("Repeat"))
    ));
    output.push('\n');
    output.push_str(&format!(
        "all:{:?}",
        parent
            .children_named(&FbxRecordName::from("Repeat"))
            .collect::<Vec<_>>()
    ));
    output.push('\n');
    output.push_str(&format!(
        "unknown:{:?};case:{:?}",
        parent.child(&FbxRecordName::from("absent")),
        parent.child(&FbxRecordName::from("repeat"))
    ));
    output.push('\n');
    let objects = Node {
        name: "Objects".into(),
        props: vec![],
        children: vec![
            object("Model", 1, Some(Prop::Str(b"Root".to_vec()))),
            object("Model", 2, Some(Prop::Str(b"LimbNode".to_vec()))),
            object("Model", 3, Some(Prop::Str(b"Null".to_vec()))),
            object("Geometry", 4, Some(Prop::Str(b"Mesh".to_vec()))),
            object("Geometry", 5, Some(Prop::Raw(b"Mesh".to_vec()))),
            object("Model", 6, Some(Prop::Str(b"Root\0\x01suffix".to_vec()))),
            object("unknown: kind", 7, Some(Prop::Str(vec![0xff, b'X']))),
            object("Model", 8, None),
            object("Model", 9, Some(Prop::I32(5))),
        ],
    };
    let links = Node {
        name: "Connections".into(),
        props: vec![],
        children: [5, 4, 1, 3, 2]
            .into_iter()
            .map(|from| Node {
                name: "C".into(),
                props: vec![Prop::Str(b"OO".to_vec()), Prop::I64(from), Prop::I64(0)],
                children: vec![],
            })
            .collect(),
    };
    let scene = Scene::from_roots(vec![
        leaf("GlobalSettings", 10),
        objects,
        links,
        leaf("GlobalSettings", 11),
    ])?;
    for object in &scene.objects {
        output.push_str(&format!(
            "object:{object:?};node={};limb={};null={}",
            object.is_node(),
            object.is_limb(),
            object.is_null_node()
        ));
        output.push('\n');
    }
    output.push_str(&format!(
        "root:{:?}",
        scene.root(&FbxRecordName::from("GlobalSettings"))
    ));
    output.push('\n');
    output.push_str(&format!(
        "mesh:{:?}",
        scene
            .objects_of_kind(&FbxRecordName::GEOMETRY, &FbxClassName::MESH)
            .map(|o| o.id)
            .collect::<Vec<_>>()
    ));
    output.push('\n');
    output.push_str(&format!(
        "children:{:?}",
        scene.children(0).map(|o| o.id).collect::<Vec<_>>()
    ));
    output.push('\n');
    output.push_str(&format!(
        "first_mesh:{:?}",
        scene
            .child_of_kind(0, &FbxRecordName::GEOMETRY, &FbxClassName::MESH)
            .map(|o| o.id)
    ));
    output.push('\n');
    output.push_str(&format!(
        "transposed:{:?}",
        scene
            .objects_of_kind(
                &FbxRecordName::from("Mesh"),
                &FbxClassName::from("Geometry")
            )
            .map(|o| o.id)
            .collect::<Vec<_>>()
    ));
    output.push('\n');
    output.push_str(&format!(
        "unknown_class:{:?}",
        scene
            .objects_of_kind(
                &FbxRecordName::from("unknown: kind"),
                &FbxClassName::from("�X")
            )
            .map(|o| o.id)
            .collect::<Vec<_>>()
    ));
    output.push('\n');
    output.push_str(&format!(
        "empty_class:{:?}",
        scene
            .objects_of_kind(&FbxRecordName::MODEL, &FbxClassName::from(""))
            .map(|o| o.id)
            .collect::<Vec<_>>()
    ));
    output.push('\n');
    assert_eq!(output, include_str!("fixtures/selectors.txt"));
    Ok(())
}

#[test]
fn selected_references_borrow_tree_and_scene_after_selectors_are_dropped() -> anyhow::Result<()> {
    let parent = Node {
        name: "Parent".into(),
        props: vec![],
        children: vec![leaf("Repeat", 1), leaf("Repeat", 2)],
    };
    let selected = {
        let name = FbxRecordName::from("Repeat");
        parent.children_named(&name).collect::<Vec<_>>()
    };
    assert_eq!(
        selected
            .iter()
            .map(|node| node.prop(0).unwrap().as_i64().unwrap())
            .collect::<Vec<_>>(),
        [1, 2]
    );
    let scene = Scene::from_roots(vec![Node {
        name: FbxRecordName::OBJECTS,
        props: vec![],
        children: vec![object("Geometry", 3, Some(Prop::Str(b"Mesh".to_vec())))],
    }])?;
    let selected = {
        let record = FbxRecordName::from("Geometry");
        let class = FbxClassName::from("Mesh");
        scene.objects_of_kind(&record, &class).collect::<Vec<_>>()
    };
    assert_eq!(selected[0].id, 3);
    Ok(())
}
