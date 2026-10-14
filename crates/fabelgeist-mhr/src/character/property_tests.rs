//! Property-table keys preserve joint selection and rest transforms.
use super::*;
use fabelgeist_fbx::{Node, Prop};

fn node(name: &str, props: Vec<Prop>, children: Vec<Node>) -> Node {
    Node {
        name: name.into(),
        props,
        children,
    }
}
fn property(name: Prop, values: Vec<Prop>) -> Node {
    let mut props = vec![
        name,
        Prop::Str(b"kind".to_vec()),
        Prop::Str(vec![]),
        Prop::Str(vec![]),
    ];
    props.extend(values);
    node("P", props, vec![])
}
fn table(entries: Vec<Node>) -> Node {
    node("Properties70", vec![], entries)
}
fn object(kind: &str, id: i64, class: &str, children: Vec<Node>) -> Node {
    node(
        kind,
        vec![
            Prop::I64(id),
            Prop::Str(format!("ns:item{id}").into_bytes()),
            Prop::Str(class.as_bytes().to_vec()),
        ],
        children,
    )
}
fn link(from: i64, to: i64, property: Option<&str>) -> Node {
    let mut props = vec![
        Prop::Str(if property.is_some() {
            b"OP".to_vec()
        } else {
            b"OO".to_vec()
        }),
        Prop::I64(from),
        Prop::I64(to),
    ];
    if let Some(p) = property {
        props.push(Prop::Str(p.as_bytes().to_vec()));
    }
    node("C", props, vec![])
}
fn character_observation() -> anyhow::Result<String> {
    let objects = node(
        "Objects",
        vec![],
        vec![
            object("Model", 1, "Null", vec![]),
            object(
                "Model",
                2,
                "Root",
                vec![table(vec![
                    property(Prop::Raw(b"RotationOrder".to_vec()), vec![Prop::I16(5)]),
                    property(
                        Prop::Str(b"Lcl Rotation".to_vec()),
                        vec![Prop::F64(10.0), Prop::F32(20.0), Prop::I64(30)],
                    ),
                    property(
                        Prop::Raw(b"PreRotation".to_vec()),
                        vec![Prop::I32(1), Prop::I32(2), Prop::I32(3)],
                    ),
                    property(
                        Prop::Str(b"Lcl Translation".to_vec()),
                        vec![Prop::F32(1.25), Prop::I64(2), Prop::F64(-3.5)],
                    ),
                ])],
            ),
            object(
                "Model",
                3,
                "LimbNode",
                vec![table(vec![
                    property(
                        Prop::Str(b"Lcl Rotation".to_vec()),
                        vec![Prop::F64(10.0), Prop::Str(b"bad".to_vec()), Prop::I64(30)],
                    ),
                    property(Prop::Str(b"Lcl Translation".to_vec()), vec![Prop::I32(1)]),
                ])],
            ),
            object(
                "Model",
                4,
                "Null",
                vec![table(vec![property(
                    Prop::Raw(b"col_type".to_vec()),
                    vec![],
                )])],
            ),
            object("Model", 5, "Root", vec![]),
        ],
    );
    let links = node(
        "Connections",
        vec![],
        vec![
            link(1, 0, None),
            link(2, 1, None),
            link(3, 2, None),
            link(4, 0, None),
            link(5, 4, None),
        ],
    );
    let scene = Scene::from_roots(vec![objects, links])?;
    let (skeleton, ids) = parse_skeleton(&scene);
    Ok(format!(
        "skeleton={skeleton:?};ids={ids:?};bind={:?}",
        skeleton.bind_pose()
    ))
}

#[test]
fn property_queries_keep_original_skeleton_payload() -> Result<()> {
    assert_eq!(
        character_observation()?,
        include_str!("fixtures/property_keys.txt").trim_end()
    );
    Ok(())
}
