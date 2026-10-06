use fabelgeist_fbx::{FbxPropertyName, Node, Prop, Scene};
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
fn property_observation() -> anyhow::Result<String> {
    let parent = node(
        "Model",
        vec![],
        vec![
            table(vec![
                property(Prop::I64(7), vec![Prop::I32(900)]),
                property(
                    Prop::Str(b"Unknown:key\0suffix".to_vec()),
                    vec![Prop::I16(42)],
                ),
                property(Prop::Raw(b"Scalar".to_vec()), vec![Prop::Bool(true)]),
                property(Prop::Str(b"Scalar".to_vec()), vec![Prop::I64(99)]),
                property(
                    Prop::Str(b"Vector".to_vec()),
                    vec![Prop::F32(1.25), Prop::I32(2), Prop::F64(-3.5)],
                ),
                property(
                    Prop::Raw(b"Partial".to_vec()),
                    vec![Prop::I64(8), Prop::Str(b"bad".to_vec()), Prop::I64(9)],
                ),
                property(Prop::Str(b"Short".to_vec()), vec![Prop::I64(8)]),
                property(Prop::Str(b"Empty".to_vec()), vec![]),
                property(Prop::Str(b"".to_vec()), vec![Prop::I32(-5)]),
                node(
                    "NotP",
                    vec![
                        Prop::Str(b"OtherRecord".to_vec()),
                        Prop::I32(0),
                        Prop::I32(0),
                        Prop::I32(0),
                        Prop::I64(17),
                    ],
                    vec![],
                ),
            ]),
            table(vec![property(
                Prop::Str(b"OnlySecondBlock".to_vec()),
                vec![Prop::I32(6)],
            )]),
        ],
    );
    let mut out = String::new();
    for key in [
        "Unknown:key\0suffix",
        "Scalar",
        "scalar",
        "",
        "Empty",
        "OnlySecondBlock",
        "OtherRecord",
        "missing",
    ] {
        out.push_str(&format!(
            "lookup:{key:?}={:?};integer={}\n",
            parent.property70(FbxPropertyName::from(key)),
            parent.property70_i64(FbxPropertyName::from(key), -7)
        ));
    }
    for key in ["Vector", "Partial", "Short", "missing"] {
        out.push_str(&format!(
            "vector:{key}={:?}\n",
            parent.property70_vec3(FbxPropertyName::from(key), [10.0, 20.0, 30.0])
        ));
    }
    let selected = {
        let name = String::from("Scalar");
        parent.property70(FbxPropertyName::from(name.as_str()))
    };
    out.push_str(&format!("temporary:{selected:?}\n"));
    for duration in [0, 1, 2] {
        let mut times = vec![property(
            Prop::Raw(b"LocalStart".to_vec()),
            vec![Prop::I64(46_186_158_000)],
        )];
        if duration != 1 {
            times.push(property(
                Prop::Str(b"LocalStop".to_vec()),
                vec![Prop::I64(if duration == 0 { 92_372_316_000 } else { 0 })],
            ));
        }
        let curve = node(
            "AnimationCurve",
            vec![
                Prop::I64(5),
                Prop::Str(b"Curve".to_vec()),
                Prop::Str(vec![]),
            ],
            vec![
                node(
                    "KeyTime",
                    vec![Prop::ArrI64(vec![0, 92_372_316_000])],
                    vec![],
                ),
                node("KeyValueFloat", vec![Prop::ArrF32(vec![4.0, 6.0])], vec![]),
            ],
        );
        let objects = node(
            "Objects",
            vec![],
            vec![
                object("Model", 1, "Root", vec![]),
                object("AnimationStack", 2, "", vec![table(times)]),
                object("AnimationLayer", 3, "", vec![]),
                object(
                    "AnimationCurveNode",
                    4,
                    "",
                    vec![table(vec![
                        property(Prop::Raw(b"d|X".to_vec()), vec![Prop::F64(1.25)]),
                        property(Prop::Str(b"d|Y".to_vec()), vec![Prop::I32(2)]),
                        property(Prop::Str(b"d|Z".to_vec()), vec![Prop::Str(b"bad".to_vec())]),
                    ])],
                ),
                curve,
            ],
        );
        let links = node(
            "Connections",
            vec![],
            vec![
                link(3, 2, None),
                link(4, 3, None),
                link(4, 1, Some("Lcl Translation")),
                link(5, 4, Some("d|X")),
            ],
        );
        let scene = Scene::from_roots(vec![objects, links])?;
        out.push_str(&format!("take{duration}:{:?}\n", scene.takes()));
    }
    Ok(out)
}

#[test]
fn property_lookup_and_animation_match_original() -> anyhow::Result<()> {
    assert_eq!(
        property_observation()?,
        include_str!("fixtures/property_keys.txt")
    );
    Ok(())
}

#[test]
fn native_key_bytes_remain_distinct_without_lossy_normalization() {
    let key = [b'X', 0xff];
    let parent = node(
        "Model",
        vec![],
        vec![table(vec![
            property(Prop::Str(key.to_vec()), vec![Prop::I64(7)]),
            property(Prop::Raw(b"X\xef\xbf\xbd".to_vec()), vec![Prop::I64(9)]),
        ])],
    );
    assert_eq!(
        parent.property70_i64(FbxPropertyName::from(key.as_slice()), 0),
        7
    );
    assert_eq!(parent.property70_i64(FbxPropertyName::from("X�"), 0), 9);
    assert!(parent.property70(FbxPropertyName::from("x�")).is_none());
}
