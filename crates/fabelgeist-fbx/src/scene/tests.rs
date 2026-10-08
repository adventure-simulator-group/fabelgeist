use crate::{
    CurveAxis, FbxConnectionProperty, FbxObjectId, Node, Object, Prop, Scene, TransformProperty,
};

/// Authored object records used by the graph and animation scenarios.
enum ObjectRecord {
    FirstDuplicate,
    LastDuplicate,
    Negative,
    InvalidIdentity,
    Zero,
    Model,
    Stack,
    EmptyStack,
    BaseLayer,
    SecondLayer,
    TranslationNode,
    RotationNode,
    ScalingNode,
    TranslationCurve,
    RotationCurve,
    ScalingCurve,
    SecondLayerCurve,
}

impl From<ObjectRecord> for Node {
    fn from(record: ObjectRecord) -> Self {
        let (identity, name, kind, class): (Prop, &[u8], &str, &str) = match record {
            ObjectRecord::FirstDuplicate => {
                (Prop::I16(1), b"old\0\x01Model".as_slice(), "Model", "Root")
            }
            ObjectRecord::LastDuplicate => {
                (Prop::I32(1), b"scope:last\0\x01Model", "Model", "Mesh")
            }
            ObjectRecord::Negative => (
                Prop::I64(-2),
                b"scope:negative\xff\0\x01Model",
                "Model",
                "LimbNode",
            ),
            ObjectRecord::InvalidIdentity => (Prop::F64(4.0), b"invalid", "Model", "Root"),
            ObjectRecord::Zero => (Prop::Bool(false), b"zero", "Model", "Null"),
            ObjectRecord::Model => (Prop::Bool(true), b"scope:joint\0\x01Model", "Model", "Root"),
            ObjectRecord::Stack => (Prop::I64(10), b"take", "AnimationStack", ""),
            ObjectRecord::EmptyStack => (Prop::I64(11), b"empty", "AnimationStack", ""),
            ObjectRecord::BaseLayer => (Prop::I64(20), b"base", "AnimationLayer", ""),
            ObjectRecord::SecondLayer => (Prop::I64(21), b"second", "AnimationLayer", ""),
            ObjectRecord::TranslationNode => {
                (Prop::I64(30), b"translation", "AnimationCurveNode", "")
            }
            ObjectRecord::RotationNode => (Prop::I64(31), b"rotation", "AnimationCurveNode", ""),
            ObjectRecord::ScalingNode => (Prop::I64(32), b"scaling", "AnimationCurveNode", ""),
            ObjectRecord::TranslationCurve => {
                (Prop::I64(40), b"translation-X", "AnimationCurve", "")
            }
            ObjectRecord::RotationCurve => (Prop::I64(41), b"rotation-Y", "AnimationCurve", ""),
            ObjectRecord::ScalingCurve => (Prop::I64(42), b"scaling-Z", "AnimationCurve", ""),
            ObjectRecord::SecondLayerCurve => (Prop::I64(43), b"second-X", "AnimationCurve", ""),
        };
        Self {
            name: kind.to_owned(),
            props: vec![
                identity,
                Prop::Str(name.to_vec()),
                Prop::Str(class.as_bytes().to_vec()),
            ],
            children: Vec::new(),
        }
    }
}

/// A connection record constructor; raw properties are its wire inputs.
struct ConnectionRecord {
    source: Prop,
    destination: Prop,
    property: Option<Prop>,
}

impl From<ConnectionRecord> for Node {
    fn from(record: ConnectionRecord) -> Self {
        let mut props = vec![Prop::Str(b"OP".to_vec()), record.source, record.destination];
        props.extend(record.property);
        Self {
            name: "C".to_owned(),
            props,
            children: Vec::new(),
        }
    }
}

#[test]
fn duplicate_lookup_and_connection_order_preserve_record_semantics() {
    let mut unusual_link = Node::from(ConnectionRecord {
        source: Prop::I16(-2),
        destination: Prop::Bool(false),
        property: None,
    });
    unusual_link.name = "UninterpretedRecordName".to_owned();
    unusual_link.props[0] = Prop::I64(123);
    let scene = Scene::from_roots(vec![
        Node {
            name: "Objects".to_owned(),
            props: Vec::new(),
            children: vec![
                ObjectRecord::FirstDuplicate.into(),
                ObjectRecord::InvalidIdentity.into(),
                ObjectRecord::Negative.into(),
                ObjectRecord::Zero.into(),
            ],
        },
        Node {
            name: "Objects".to_owned(),
            props: Vec::new(),
            children: vec![ObjectRecord::LastDuplicate.into()],
        },
        Node {
            name: "Connections".to_owned(),
            props: Vec::new(),
            children: vec![
                unusual_link,
                ConnectionRecord {
                    source: Prop::I64(999),
                    destination: Prop::I32(0),
                    property: None,
                }
                .into(),
                ConnectionRecord {
                    source: Prop::I32(1),
                    destination: Prop::I64(0),
                    property: None,
                }
                .into(),
                ConnectionRecord {
                    source: Prop::I64(0),
                    destination: Prop::I64(1),
                    property: None,
                }
                .into(),
                ConnectionRecord {
                    source: Prop::F32(1.0),
                    destination: Prop::I64(0),
                    property: None,
                }
                .into(),
            ],
        },
        Node {
            name: "Connections".to_owned(),
            props: Vec::new(),
            children: vec![
                ConnectionRecord {
                    source: Prop::Bool(true),
                    destination: Prop::Bool(false),
                    property: None,
                }
                .into(),
            ],
        },
    ]);
    assert_eq!(scene.objects.len(), 4);
    assert_eq!(scene.objects[0].name, "old");
    assert_eq!(scene.objects[3].name, "last");
    let selected = scene.get(FbxObjectId::from(1)).unwrap();
    assert_eq!(selected.name, "last");
    assert_eq!(selected.qualified, "scope:last");
    assert_eq!(selected.class, "Mesh");
    assert_eq!(scene.get(FbxObjectId::from(-2)).unwrap().name, "negative�");
    assert!(scene.get(FbxObjectId::from(999)).is_none());
    assert_eq!(
        scene
            .children(FbxObjectId::SCENE_ROOT)
            .map(|object: &Object| object.id)
            .collect::<Vec<_>>(),
        [
            FbxObjectId::from(-2),
            FbxObjectId::from(1),
            FbxObjectId::from(1)
        ]
    );
    assert!(scene.children(FbxObjectId::from(1)).next().is_none());
    assert_eq!(scene.incoming(FbxObjectId::SCENE_ROOT).count(), 4);
    assert!(scene.get(FbxObjectId::SCENE_ROOT).unwrap().is_null_node());
}

#[test]
fn identity_admission_preserves_integer_boolean_and_signed_domains() {
    for (property, expected) in [
        (
            Prop::I16(i16::MIN),
            Some(FbxObjectId::from(i64::from(i16::MIN))),
        ),
        (
            Prop::I32(i32::MAX),
            Some(FbxObjectId::from(i64::from(i32::MAX))),
        ),
        (Prop::I64(i64::MIN), Some(FbxObjectId::from(i64::MIN))),
        (Prop::I64(i64::MAX), Some(FbxObjectId::from(i64::MAX))),
        (Prop::Bool(false), Some(FbxObjectId::SCENE_ROOT)),
        (Prop::Bool(true), Some(FbxObjectId::from(1))),
        (Prop::F32(1.0), None),
        (Prop::F64(1.0), None),
        (Prop::Str(b"1".to_vec()), None),
        (Prop::ArrI64(vec![1]), None),
    ] {
        assert_eq!(FbxObjectId::from_property(&property), expected);
    }
    assert!(matches!(
        Prop::from(FbxObjectId::from(i64::MIN)),
        Prop::I64(i64::MIN)
    ));
}

#[test]
fn connection_properties_retain_unknown_spelling_and_classify_known_roles() {
    let scene = Scene::from_roots(vec![
        Node {
            name: "Objects".to_owned(),
            props: Vec::new(),
            children: vec![ObjectRecord::Model.into()],
        },
        Node {
            name: "Connections".to_owned(),
            props: Vec::new(),
            children: vec![
                ConnectionRecord {
                    source: Prop::I64(1),
                    destination: Prop::I64(0),
                    property: Some(Prop::Raw(b"d|Y".to_vec())),
                }
                .into(),
                ConnectionRecord {
                    source: Prop::I64(1),
                    destination: Prop::I64(0),
                    property: Some(Prop::Str(b"Lcl Scaling".to_vec())),
                }
                .into(),
                ConnectionRecord {
                    source: Prop::I64(1),
                    destination: Prop::I64(0),
                    property: Some(Prop::Str(b"unknown\xff\0\x01suffix".to_vec())),
                }
                .into(),
                ConnectionRecord {
                    source: Prop::I64(1),
                    destination: Prop::I64(0),
                    property: Some(Prop::I64(7)),
                }
                .into(),
            ],
        },
    ]);
    let links = scene
        .children_with_property(FbxObjectId::SCENE_ROOT)
        .collect::<Vec<_>>();
    let axis = links[0].1.unwrap();
    assert_eq!(axis.axis(), Some(CurveAxis::Y));
    assert_eq!(axis.transform(), None);
    let transform = links[1].1.unwrap();
    assert_eq!(transform.axis(), None);
    assert_eq!(transform.transform(), Some(TransformProperty::Scaling));
    let unknown = links[2].1.unwrap();
    assert_eq!(unknown.to_string(), "unknown�\0\x01suffix");
    assert_eq!(unknown.axis(), None);
    assert_eq!(unknown.transform(), None);
    assert!(links[3].1.is_none());
    let near_match = FbxConnectionProperty::from(b"Lcl Rotation ".as_slice());
    assert_eq!(near_match.transform(), None);
}

struct AnimationFixture {
    roots: Vec<Node>,
}

impl AnimationFixture {
    fn objects() -> Vec<Node> {
        let mut objects = vec![
            ObjectRecord::Model.into(),
            ObjectRecord::Stack.into(),
            ObjectRecord::EmptyStack.into(),
            ObjectRecord::BaseLayer.into(),
            ObjectRecord::SecondLayer.into(),
            ObjectRecord::TranslationNode.into(),
            ObjectRecord::RotationNode.into(),
            ObjectRecord::ScalingNode.into(),
        ];
        for (record, values) in [
            (
                ObjectRecord::TranslationCurve,
                Prop::ArrF64(vec![3.0, 5.0, 99.0]),
            ),
            (ObjectRecord::RotationCurve, Prop::ArrF32(vec![10.0, 20.0])),
            (ObjectRecord::ScalingCurve, Prop::ArrI32(vec![1, 2])),
            (
                ObjectRecord::SecondLayerCurve,
                Prop::ArrF64(vec![80.0, 90.0]),
            ),
        ] {
            let mut curve = Node::from(record);
            curve.children = vec![
                Node {
                    name: "KeyTime".to_owned(),
                    props: vec![Prop::ArrI64(vec![0, 46_186_158_000])],
                    children: Vec::new(),
                },
                Node {
                    name: "KeyValueFloat".to_owned(),
                    props: vec![values],
                    children: Vec::new(),
                },
            ];
            objects.push(curve);
        }
        objects
    }

    fn connections() -> Vec<Node> {
        // An unrecognized axis property must leave the base X curve intact.
        vec![
            ConnectionRecord {
                source: Prop::I64(20),
                destination: Prop::I64(10),
                property: None,
            }
            .into(),
            ConnectionRecord {
                source: Prop::I64(21),
                destination: Prop::I64(10),
                property: None,
            }
            .into(),
            ConnectionRecord {
                source: Prop::I64(30),
                destination: Prop::I64(20),
                property: None,
            }
            .into(),
            ConnectionRecord {
                source: Prop::I64(31),
                destination: Prop::I64(20),
                property: None,
            }
            .into(),
            ConnectionRecord {
                source: Prop::I64(32),
                destination: Prop::I64(20),
                property: None,
            }
            .into(),
            ConnectionRecord {
                source: Prop::I64(30),
                destination: Prop::I64(1),
                property: Some(Prop::Str(b"Lcl Translation".to_vec())),
            }
            .into(),
            ConnectionRecord {
                source: Prop::I64(31),
                destination: Prop::I64(1),
                property: Some(Prop::Str(b"Lcl Rotation".to_vec())),
            }
            .into(),
            ConnectionRecord {
                source: Prop::I64(32),
                destination: Prop::I64(1),
                property: Some(Prop::Str(b"Lcl Scaling".to_vec())),
            }
            .into(),
            ConnectionRecord {
                source: Prop::I64(40),
                destination: Prop::I64(30),
                property: Some(Prop::Str(b"d|X".to_vec())),
            }
            .into(),
            ConnectionRecord {
                source: Prop::I64(41),
                destination: Prop::I64(31),
                property: Some(Prop::Str(b"d|Y".to_vec())),
            }
            .into(),
            ConnectionRecord {
                source: Prop::I64(42),
                destination: Prop::I64(32),
                property: Some(Prop::Str(b"d|Z".to_vec())),
            }
            .into(),
            ConnectionRecord {
                source: Prop::I64(43),
                destination: Prop::I64(30),
                property: Some(Prop::Str(b"d|unsupported".to_vec())),
            }
            .into(),
        ]
    }
}

impl Default for AnimationFixture {
    fn default() -> Self {
        Self {
            roots: vec![
                Node {
                    name: "Objects".to_owned(),
                    props: Vec::new(),
                    children: Self::objects(),
                },
                Node {
                    name: "Connections".to_owned(),
                    props: Vec::new(),
                    children: Self::connections(),
                },
            ],
        }
    }
}

#[test]
fn animation_roles_preserve_axis_mapping_truncation_names_and_empty_stacks() {
    let scene = Scene::from_roots(AnimationFixture::default().roots);
    let takes = scene.takes();
    assert_eq!(takes.len(), 1);
    assert_eq!(takes[0].name, "take");
    assert_eq!(takes[0].duration, 1.0);
    assert_eq!(takes[0].key_times(), [0.0, 1.0]);
    let node = &takes[0].nodes[0];
    assert_eq!(node.node, FbxObjectId::from(1));
    assert_eq!(node.name, "scope:joint");
    let translation = node.translation.as_ref().unwrap();
    assert_eq!(translation.x.values, [3.0, 5.0]);
    assert!(translation.y.times.is_empty());
    assert!(translation.z.times.is_empty());
    assert_eq!(node.rotation.as_ref().unwrap().y.values, [10.0, 20.0]);
    assert_eq!(node.scale.as_ref().unwrap().z.values, [1.0, 2.0]);
}

#[test]
fn animation_uses_first_layer_and_last_axis_connection() {
    let mut fixture = AnimationFixture::default();
    let connections = &mut fixture.roots[1].children;
    connections.push(
        ConnectionRecord {
            source: Prop::I64(43),
            destination: Prop::I64(30),
            property: Some(Prop::Str(b"d|X".to_vec())),
        }
        .into(),
    );
    let scene = Scene::from_roots(fixture.roots.clone());
    assert_eq!(
        scene.takes()[0].nodes[0]
            .translation
            .as_ref()
            .unwrap()
            .x
            .values,
        [80.0, 90.0]
    );
    // Attach all existing curve nodes to the second layer instead. The base
    // layer is still first and has no curves, so the stack is dropped.
    for link in &mut fixture.roots[1].children {
        if matches!(link.props.get(2), Some(Prop::I64(20))) {
            link.props[2] = Prop::I64(21);
        }
    }
    assert!(Scene::from_roots(fixture.roots).takes().is_empty());
}
