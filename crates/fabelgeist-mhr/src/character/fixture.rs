//! Small authored rig trees for portable admission and loader precedence tests.
use super::{Character, CharacterDecodeError};
use fabelgeist_fbx::{Node, Prop, Scene};
use fabelgeist_fs::FileContents;
use fabelgeist_rig::RigJointName;

#[derive(Clone, Copy)]
pub(crate) enum RigCase {
    Valid,
    NoJoints,
    MissingMesh,
    MissingGeometry,
    MissingVertices,
    IncompletePosition,
    MissingPolygons,
    ShortPolygon,
    TrailingPolygon,
    IncompleteUv,
    MissingSkin,
    UnknownBone,
    SkinArrayMismatch,
    SkinVertexOutside,
    NoSkinWeights,
    NaNWeight,
}
#[derive(Clone)]
pub(crate) struct RigFixture {
    roots: Vec<Node>,
}
impl RigFixture {
    pub(crate) fn from_case(case: RigCase) -> Self {
        let mut objects = vec![
            Self::joint(),
            Self::mesh_model(),
            Self::geometry(case),
            Self::skin(),
            Self::cluster(case),
        ];
        let links = match case {
            RigCase::UnknownBone => vec![(1, 0), (2, 0), (3, 2), (4, 3), (5, 4)],
            _ => vec![(1, 0), (2, 0), (3, 2), (4, 3), (5, 4), (1, 5)],
        };
        match case {
            RigCase::NoJoints => {
                objects.remove(0);
            }
            RigCase::MissingMesh => {
                objects.remove(1);
            }
            RigCase::MissingGeometry => {
                objects.remove(2);
            }
            RigCase::MissingSkin => {
                objects.remove(3);
            }
            _ => {}
        }
        let mut connections = Vec::new();
        for (from, to) in links {
            connections.push(Node {
                name: "C".into(),
                props: vec![Prop::Str(b"OO".to_vec()), Prop::I64(from), Prop::I64(to)],
                children: Vec::new(),
            });
        }
        Self {
            roots: vec![
                Node {
                    name: "Objects".into(),
                    props: Vec::new(),
                    children: objects,
                },
                Node {
                    name: "Connections".into(),
                    props: Vec::new(),
                    children: connections,
                },
            ],
        }
    }
    fn joint() -> Node {
        Node {
            name: "Model".into(),
            props: vec![
                Prop::I64(1),
                Prop::Str(b"namespace:root\0\x01Model".to_vec()),
                Prop::Str(b"Root".to_vec()),
            ],
            children: Vec::new(),
        }
    }
    fn mesh_model() -> Node {
        Node {
            name: "Model".into(),
            props: vec![
                Prop::I64(2),
                Prop::Str(b"body".to_vec()),
                Prop::Str(b"Mesh".to_vec()),
            ],
            children: Vec::new(),
        }
    }
    fn skin() -> Node {
        Node {
            name: "Deformer".into(),
            props: vec![
                Prop::I64(4),
                Prop::Str(b"skin".to_vec()),
                Prop::Str(b"Skin".to_vec()),
            ],
            children: Vec::new(),
        }
    }
    fn geometry(case: RigCase) -> Node {
        let positions = match case {
            RigCase::IncompletePosition => vec![0.0, 0.0, 0.0, 1.0],
            _ => vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
        };
        let corners = match case {
            RigCase::ShortPolygon => vec![0, -2],
            RigCase::TrailingPolygon => vec![0, 1, 2],
            _ => vec![0, 1, -3],
        };
        let mut children = vec![
            Node {
                name: "Vertices".into(),
                props: vec![Prop::ArrF64(positions)],
                children: Vec::new(),
            },
            Node {
                name: "PolygonVertexIndex".into(),
                props: vec![Prop::ArrI64(corners)],
                children: Vec::new(),
            },
            Self::uv(case),
        ];
        match case {
            RigCase::MissingVertices => {
                children.remove(0);
            }
            RigCase::MissingPolygons => {
                children.remove(1);
            }
            _ => {}
        }
        Node {
            name: "Geometry".into(),
            props: vec![
                Prop::I64(3),
                Prop::Str(b"geometry".to_vec()),
                Prop::Str(b"Mesh".to_vec()),
            ],
            children,
        }
    }
    fn uv(case: RigCase) -> Node {
        let uv = match case {
            RigCase::IncompleteUv => vec![0.0, 0.25, 1.0],
            _ => vec![0.0, 0.25, 1.0, 0.25, 0.0, 0.75],
        };
        Node {
            name: "LayerElementUV".into(),
            props: Vec::new(),
            children: vec![
                Node {
                    name: "UV".into(),
                    props: vec![Prop::ArrF64(uv)],
                    children: Vec::new(),
                },
                Node {
                    name: "ReferenceInformationType".into(),
                    props: vec![Prop::Str(b"IndexToDirect".to_vec())],
                    children: Vec::new(),
                },
                Node {
                    name: "UVIndex".into(),
                    props: vec![Prop::ArrI64(vec![0, 1, 2])],
                    children: Vec::new(),
                },
            ],
        }
    }
    fn cluster(case: RigCase) -> Node {
        let indices = match case {
            RigCase::SkinVertexOutside => vec![3, 1, 2],
            _ => vec![0, 1, 2],
        };
        let weights = match case {
            RigCase::SkinArrayMismatch => vec![1.0, 1.0],
            RigCase::NoSkinWeights => vec![0.0, 1.0, 1.0],
            RigCase::NaNWeight => vec![f64::NAN, 1.0, 1.0],
            _ => vec![1.0, 1.0, 1.0],
        };
        Node {
            name: "Deformer".into(),
            props: vec![
                Prop::I64(5),
                Prop::Str(b"cluster".to_vec()),
                Prop::Str(b"Cluster".to_vec()),
            ],
            children: vec![
                Node {
                    name: "Indexes".into(),
                    props: vec![Prop::ArrI64(indices)],
                    children: Vec::new(),
                },
                Node {
                    name: "Weights".into(),
                    props: vec![Prop::ArrF64(weights)],
                    children: Vec::new(),
                },
            ],
        }
    }
    pub(crate) fn scene(&self) -> Scene {
        Scene::from_roots(self.roots.clone())
    }
    pub(crate) fn character(&self) -> Result<Character, CharacterDecodeError> {
        Character::from_scene(self.scene())
    }
    pub(crate) fn bytes(&self) -> FileContents {
        use fabelgeist_storage::{StorageByteLength, StorageByteOffset};
        let mut file = b"Kaydara FBX Binary  \x00\x1a\x00".to_vec();
        file.extend_from_slice(&7400u32.to_le_bytes());
        for node in &self.roots {
            let record = EncodedNode::from_tree(
                node,
                StorageByteOffset::from(u64::from(StorageByteLength::from(file.len()))),
            );
            file.extend_from_slice(&record.0);
        }
        file.extend_from_slice(&[0; 13]);
        FileContents::from(file)
    }
}

struct EncodedNode(Vec<u8>);
impl EncodedNode {
    fn from_tree(node: &Node, at: fabelgeist_storage::StorageByteOffset) -> Self {
        use fabelgeist_storage::{StorageByteLength as Length, StorageByteOffset as Offset};
        let mut properties = Vec::new();
        for property in &node.props {
            properties.extend_from_slice(&EncodedProperty::from(property).0);
        }
        let name = node.name.encoded();
        let prefix_length = Length::from(13u64)
            .checked_add(name.length())
            .unwrap()
            .checked_add(Length::from(properties.len()))
            .unwrap();
        let mut children = Vec::new();
        for child in &node.children {
            let position = at
                .advance(
                    prefix_length
                        .checked_add(Length::from(children.len()))
                        .unwrap(),
                )
                .unwrap();
            children.extend_from_slice(&Self::from_tree(child, position).0);
        }
        if !node.children.is_empty() {
            children.extend_from_slice(&[0; 13]);
        }
        let end = at
            .advance(
                prefix_length
                    .checked_add(Length::from(children.len()))
                    .unwrap(),
            )
            .unwrap();
        let mut record = Vec::new();
        record.extend_from_slice(
            &u32::try_from(u64::from(end.distance_from(Offset::default()).unwrap()))
                .unwrap()
                .to_le_bytes(),
        );
        record.extend_from_slice(&u32::try_from(node.props.len()).unwrap().to_le_bytes());
        record.extend_from_slice(&u32::try_from(properties.len()).unwrap().to_le_bytes());
        record.push(u8::try_from(u64::from(name.length())).unwrap());
        record.extend_from_slice(name.as_ref());
        record.extend_from_slice(&properties);
        record.extend_from_slice(&children);
        Self(record)
    }
}

struct EncodedProperty(Vec<u8>);
impl From<&Prop> for EncodedProperty {
    fn from(property: &Prop) -> Self {
        let mut data = Vec::new();
        match property {
            Prop::I64(value) => {
                data.push(b'L');
                data.extend_from_slice(&value.to_le_bytes());
            }
            Prop::Str(value) => {
                data.push(b'S');
                data.extend_from_slice(&u32::try_from(value.len()).unwrap().to_le_bytes());
                data.extend_from_slice(value);
            }
            Prop::ArrF64(values) => {
                data.push(b'd');
                data.extend_from_slice(&u32::try_from(values.len()).unwrap().to_le_bytes());
                data.extend_from_slice(&0u32.to_le_bytes());
                data.extend_from_slice(&u32::try_from(values.len() * 8).unwrap().to_le_bytes());
                for value in values {
                    data.extend_from_slice(&value.to_le_bytes());
                }
            }
            Prop::ArrI64(values) => {
                data.push(b'l');
                data.extend_from_slice(&u32::try_from(values.len()).unwrap().to_le_bytes());
                data.extend_from_slice(&0u32.to_le_bytes());
                data.extend_from_slice(&u32::try_from(values.len() * 8).unwrap().to_le_bytes());
                for value in values {
                    data.extend_from_slice(&value.to_le_bytes());
                }
            }
            _ => panic!("unsupported rig-fixture property"),
        }
        Self(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy)]
    enum GroupingCase {
        Root,
        CollisionRoot,
        Nested,
    }

    #[test]
    fn root_grouping_collision_and_nested_null_keep_joint_membership() {
        for case in [
            GroupingCase::Root,
            GroupingCase::CollisionRoot,
            GroupingCase::Nested,
        ] {
            let mut fixture = RigFixture::from_case(RigCase::Valid);
            let mut grouping = Node {
                name: "Model".into(),
                props: vec![
                    Prop::I64(6),
                    Prop::Str(b"group".to_vec()),
                    Prop::Str(b"Null".to_vec()),
                ],
                children: Vec::new(),
            };
            if matches!(case, GroupingCase::CollisionRoot) {
                grouping.children.push(Node {
                    name: "Properties70".into(),
                    props: Vec::new(),
                    children: vec![Node {
                        name: "P".into(),
                        props: vec![Prop::Str(b"col_type".to_vec())],
                        children: Vec::new(),
                    }],
                });
            }
            fixture.roots[0].children.push(grouping);
            let destination = match case {
                GroupingCase::Root | GroupingCase::CollisionRoot => {
                    // The first link originally connects joint 1 to the root.
                    fixture.roots[1].children[0].props[2] = Prop::I64(6);
                    Prop::I64(0)
                }
                GroupingCase::Nested => {
                    fixture.roots[0].children.push(Node {
                        name: "Model".into(),
                        props: vec![
                            Prop::I64(7),
                            Prop::Str(b"nested".to_vec()),
                            Prop::Str(b"LimbNode".to_vec()),
                        ],
                        children: Vec::new(),
                    });
                    fixture.roots[1].children.push(Node {
                        name: "C".into(),
                        props: vec![Prop::Str(b"OO".to_vec()), Prop::I64(7), Prop::I64(6)],
                        children: Vec::new(),
                    });
                    Prop::I64(1)
                }
            };
            fixture.roots[1].children.push(Node {
                name: "C".into(),
                props: vec![Prop::Str(b"OO".to_vec()), Prop::I64(6), destination],
                children: Vec::new(),
            });
            let character = fixture.character();
            match case {
                GroupingCase::CollisionRoot => {
                    assert!(matches!(character, Err(CharacterDecodeError::NoJoints)))
                }
                GroupingCase::Root | GroupingCase::Nested => {
                    assert_eq!(character.unwrap().skeleton.names, [RigJointName::ROOT])
                }
            }
        }
    }
}
