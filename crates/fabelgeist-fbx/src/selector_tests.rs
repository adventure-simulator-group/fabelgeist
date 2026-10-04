use crate::{FbxClassName, FbxPropertyName, FbxRecordName, ModelRole, Node, Prop, Scene};
use fabelgeist_storage::StorageView;

#[test]
fn decoded_record_and_class_spelling_retains_unknown_bytes_without_normalization() {
    let name = FbxRecordName::from(StorageView::from(b" scope:Thing\0\x01\xff ".as_slice()));
    assert_eq!(name.to_string(), " scope:Thing\0\x01� ");
    assert_eq!(name.encoded().as_ref(), b" scope:Thing\0\x01\xef\xbf\xbd ");
    let class = FbxClassName::from(StorageView::from(b"Root\0\x01\xff".as_slice()));
    assert_eq!(class.to_string(), "Root\0\x01�");
    assert_ne!(class, FbxClassName::ROOT);
    assert_ne!(FbxRecordName::from(" Model "), FbxRecordName::MODEL);
}

/// Authored property-table record; values follow the four wire metadata slots.
struct PropertyRecord {
    key: Prop,
    values: Vec<Prop>,
}
impl From<PropertyRecord> for Node {
    fn from(record: PropertyRecord) -> Self {
        let mut props = vec![
            record.key,
            Prop::Str(Vec::new()),
            Prop::Str(Vec::new()),
            Prop::Str(Vec::new()),
        ];
        props.extend(record.values);
        Self {
            name: "P".into(),
            props,
            children: Vec::new(),
        }
    }
}

#[test]
fn property_lookup_uses_exact_bytes_first_table_and_first_matching_entry() {
    let node = Node {
        name: FbxRecordName::MODEL,
        props: Vec::new(),
        children: vec![
            Node {
                name: FbxRecordName::PROPERTIES70,
                props: Vec::new(),
                children: vec![
                    PropertyRecord {
                        key: Prop::Str(vec![0xff]),
                        values: vec![Prop::I64(11)],
                    }
                    .into(),
                    PropertyRecord {
                        key: Prop::Raw(vec![0xfe]),
                        values: vec![Prop::I64(22)],
                    }
                    .into(),
                    PropertyRecord {
                        key: Prop::Str(vec![0xff]),
                        values: vec![Prop::I64(33)],
                    }
                    .into(),
                ],
            },
            Node {
                name: FbxRecordName::PROPERTIES70,
                props: Vec::new(),
                children: vec![
                    PropertyRecord {
                        key: Prop::Str(vec![0xff]),
                        values: vec![Prop::I64(44)],
                    }
                    .into(),
                ],
            },
        ],
    };
    let first = node
        .property70(&FbxPropertyName::from(b"\xff".as_slice()))
        .unwrap();
    assert!(matches!(first.props.get(4), Some(Prop::I64(11))));
    let second = node
        .property70(&FbxPropertyName::from(b"\xfe".as_slice()))
        .unwrap();
    assert!(matches!(second.props.get(4), Some(Prop::I64(22))));
    assert!(
        node.property70(&FbxPropertyName::from("�".as_bytes()))
            .is_none()
    );
}

#[test]
fn property_numeric_fallbacks_remain_whole_vector_or_integer_default() {
    let node = Node {
        name: FbxRecordName::MODEL,
        props: Vec::new(),
        children: vec![Node {
            name: FbxRecordName::PROPERTIES70,
            props: Vec::new(),
            children: vec![
                PropertyRecord {
                    key: Prop::Str(b"Lcl Translation".to_vec()),
                    values: vec![
                        Prop::I32(10),
                        Prop::Str(b"invalid".to_vec()),
                        Prop::F64(30.0),
                    ],
                }
                .into(),
                PropertyRecord {
                    key: Prop::Str(b"Lcl Rotation".to_vec()),
                    values: vec![Prop::I16(1), Prop::F32(2.5), Prop::I64(-3)],
                }
                .into(),
                PropertyRecord {
                    key: Prop::Str(b"RotationOrder".to_vec()),
                    values: vec![Prop::F32(1.0)],
                }
                .into(),
                PropertyRecord {
                    key: Prop::Str(b"LocalStart".to_vec()),
                    values: vec![Prop::Bool(true)],
                }
                .into(),
            ],
        }],
    };
    assert_eq!(
        node.property70_vec3(&FbxPropertyName::LOCAL_TRANSLATION, [9.0, 8.0, 7.0]),
        [9.0, 8.0, 7.0]
    );
    assert_eq!(
        node.property70_vec3(&FbxPropertyName::LOCAL_ROTATION, [9.0, 8.0, 7.0]),
        [1.0, 2.5, -3.0]
    );
    assert_eq!(node.property70_i64(&FbxPropertyName::ROTATION_ORDER, 5), 5);
    assert_eq!(node.property70_i64(&FbxPropertyName::LOCAL_START, 5), 1);
    assert_eq!(node.property70_i64(&FbxPropertyName::LOCAL_STOP, 5), 5);
}

#[test]
fn model_role_preserves_root_limb_null_and_unknown_class_policy() {
    let scene = Scene::from_roots(vec![Node {
        name: FbxRecordName::OBJECTS,
        props: Vec::new(),
        children: vec![
            Node {
                name: FbxRecordName::MODEL,
                props: vec![
                    Prop::I64(1),
                    Prop::Str(Vec::new()),
                    Prop::Str(b"Root".to_vec()),
                ],
                children: Vec::new(),
            },
            Node {
                name: FbxRecordName::MODEL,
                props: vec![
                    Prop::I64(2),
                    Prop::Str(Vec::new()),
                    Prop::Raw(b"LimbNode".to_vec()),
                ],
                children: Vec::new(),
            },
            Node {
                name: FbxRecordName::MODEL,
                props: vec![
                    Prop::I64(3),
                    Prop::Str(Vec::new()),
                    Prop::Str(b"Null".to_vec()),
                ],
                children: Vec::new(),
            },
            Node {
                name: FbxRecordName::MODEL,
                props: vec![
                    Prop::I64(4),
                    Prop::Str(Vec::new()),
                    Prop::Str(b"Null\0\x01Class".to_vec()),
                ],
                children: Vec::new(),
            },
            Node {
                name: FbxRecordName::MODEL,
                props: vec![Prop::I64(5), Prop::Str(Vec::new()), Prop::I64(7)],
                children: Vec::new(),
            },
            Node {
                name: FbxRecordName::GEOMETRY,
                props: vec![
                    Prop::I64(6),
                    Prop::Str(Vec::new()),
                    Prop::Str(b"Root".to_vec()),
                ],
                children: Vec::new(),
            },
        ],
    }]);
    let objects = scene.objects().collect::<Vec<_>>();
    assert_eq!(objects[0].model_role(), Some(ModelRole::Joint));
    assert_eq!(objects[1].model_role(), Some(ModelRole::Joint));
    assert_eq!(objects[2].model_role(), Some(ModelRole::Null));
    assert_eq!(objects[3].model_role(), Some(ModelRole::Uninterpreted));
    assert_eq!(objects[4].model_role(), Some(ModelRole::Uninterpreted));
    assert_eq!(objects[4].class, FbxClassName::default());
    assert_eq!(objects[5].model_role(), None);
}

#[test]
fn typed_record_queries_keep_first_root_and_child_encounter_order() {
    let first = Node {
        name: FbxRecordName::from("CustomRoot"),
        props: vec![Prop::I64(10)],
        children: vec![
            Node {
                name: FbxRecordName::VERTICES,
                props: vec![Prop::I64(1)],
                children: Vec::new(),
            },
            Node {
                name: FbxRecordName::INDEXES,
                props: vec![Prop::I64(2)],
                children: Vec::new(),
            },
            Node {
                name: FbxRecordName::VERTICES,
                props: vec![Prop::I64(3)],
                children: Vec::new(),
            },
        ],
    };
    let second = Node {
        name: FbxRecordName::from("CustomRoot"),
        props: vec![Prop::I64(20)],
        children: Vec::new(),
    };
    let scene = Scene::from_roots(vec![first, second]);
    assert_eq!(scene.roots().count(), 2);
    let root = scene.root(&FbxRecordName::from("CustomRoot")).unwrap();
    assert!(matches!(root.props.first(), Some(Prop::I64(10))));
    assert!(matches!(
        root.child(&FbxRecordName::VERTICES).unwrap().props.first(),
        Some(Prop::I64(1))
    ));
    let mut children = root.children_named(&FbxRecordName::VERTICES);
    assert!(matches!(
        children.next().unwrap().props.first(),
        Some(Prop::I64(1))
    ));
    assert!(matches!(
        children.next().unwrap().props.first(),
        Some(Prop::I64(3))
    ));
    assert!(children.next().is_none());
}
