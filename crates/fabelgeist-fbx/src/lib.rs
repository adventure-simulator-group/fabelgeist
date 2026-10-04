//! Reader for binary FBX files (Kaydara FBX Binary, versions 7100-7700).
//!
//! What is modelled: the node/property tree, the object table under `Objects`,
//! the connection list including the property names that object-property links
//! carry, and the animation curves reachable through them. Object resolution
//! follows OpenFBX (the loader momentum itself uses), because joint ordering in
//! a momentum character is defined by the order connections appear in the file.
//!
//! This is a container reader. What the objects *mean* — which nodes are joints,
//! how FBX's transform chain composes, what units the file is in — is left to
//! callers, because rigs disagree about all three.
//!
//! Uses dependency-free checked storage spans and a pure-Rust inflate,
//! so an asset pipeline can read FBX without pulling in a tensor runtime.

use std::collections::HashMap;

mod binary;
mod class;
mod property_name;
mod record;
pub use class::{FbxClassName, ModelRole};
pub use property_name::{CurveAxis, FbxPropertyName, TransformProperty};
pub use record::FbxRecordName;
mod connection;
mod identity;
mod object_name;
pub use binary::{
    FbxArrayCount, FbxArrayEncodingCode, FbxArrayKind, FbxDecodeError, FbxFormatViolation,
    FbxPropertyCount, FbxPropertyTag, FbxSection, FbxVersion, parse,
};
pub use connection::FbxConnectionProperty;
use fabelgeist_storage::StorageView;
pub use identity::FbxObjectId;
pub use object_name::{FbxObjectName, FbxQualifiedName};

pub mod animation;

pub use animation::{Curve, NodeAnimation, Take, TransformChannel};

/// A typed FBX property value.
#[derive(Debug, Clone)]
pub enum Prop {
    I16(i16),
    Bool(bool),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    ArrF32(Vec<f32>),
    ArrF64(Vec<f64>),
    ArrI32(Vec<i32>),
    ArrI64(Vec<i64>),
    ArrBool(Vec<u8>),
    /// FBX strings are not UTF-8 in general; object names embed a `\0\x01` separator.
    Str(Vec<u8>),
    Raw(Vec<u8>),
}

impl Prop {
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Prop::I16(v) => Some(*v as i64),
            Prop::I32(v) => Some(*v as i64),
            Prop::I64(v) => Some(*v),
            Prop::Bool(v) => Some(*v as i64),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Prop::F32(v) => Some(*v as f64),
            Prop::F64(v) => Some(*v),
            Prop::I16(v) => Some(*v as f64),
            Prop::I32(v) => Some(*v as f64),
            Prop::I64(v) => Some(*v as f64),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&[u8]> {
        match self {
            Prop::Str(v) | Prop::Raw(v) => Some(v),
            _ => None,
        }
    }

    /// Numeric array widened to `f64`, whatever the on-disk element type.
    pub fn as_f64_array(&self) -> Option<Vec<f64>> {
        match self {
            Prop::ArrF32(v) => Some(v.iter().map(|x| *x as f64).collect()),
            Prop::ArrF64(v) => Some(v.clone()),
            Prop::ArrI32(v) => Some(v.iter().map(|x| *x as f64).collect()),
            Prop::ArrI64(v) => Some(v.iter().map(|x| *x as f64).collect()),
            _ => None,
        }
    }

    /// Integer array widened to `i64`, whatever the on-disk element type.
    pub fn as_i64_array(&self) -> Option<Vec<i64>> {
        match self {
            Prop::ArrI32(v) => Some(v.iter().map(|x| *x as i64).collect()),
            Prop::ArrI64(v) => Some(v.clone()),
            Prop::ArrBool(v) => Some(v.iter().map(|x| *x as i64).collect()),
            _ => None,
        }
    }
}

/// One node record of the FBX tree.
#[derive(Debug, Clone)]
pub struct Node {
    pub name: FbxRecordName,
    pub props: Vec<Prop>,
    pub children: Vec<Node>,
}

impl Node {
    pub fn child(&self, name: &FbxRecordName) -> Option<&Node> {
        self.children
            .iter()
            .find(|c: &&Node| -> bool { &c.name == name })
    }

    pub fn children_named<'a>(&'a self, name: &'a FbxRecordName) -> impl Iterator<Item = &'a Node> {
        self.children
            .iter()
            .filter(move |c: &&Node| -> bool { &c.name == name })
    }

    pub fn prop(&self, index: usize) -> Option<&Prop> {
        self.props.get(index)
    }

    /// First property as a numeric array, widened to `f64`.
    pub fn f64_array(&self) -> Option<Vec<f64>> {
        self.props.first()?.as_f64_array()
    }

    /// First property as an integer array, widened to `i64`.
    pub fn i64_array(&self) -> Option<Vec<i64>> {
        self.props.first()?.as_i64_array()
    }

    pub fn str_prop(&self, index: usize) -> Option<&[u8]> {
        self.props.get(index)?.as_str()
    }

    /// Look up an entry of this node's `Properties70` block by name.
    ///
    /// Mirrors OpenFBX `resolveProperty`: a `P` record whose first property is
    /// the requested name; values start at index 4.
    pub fn property70(&self, name: &FbxPropertyName<'_>) -> Option<&Node> {
        let props = self.child(&FbxRecordName::PROPERTIES70)?;
        props.children.iter().find(|p: &&Node| -> bool {
            p.props
                .first()
                .and_then(FbxPropertyName::from_property)
                .is_some_and(|key: FbxPropertyName<'_>| -> bool { &key == name })
        })
    }

    /// A three-component `Properties70` value such as `Lcl Translation`.
    pub fn property70_vec3(&self, name: &FbxPropertyName<'_>, default: [f64; 3]) -> [f64; 3] {
        let Some(p) = self.property70(name) else {
            return default;
        };
        let mut out = default;
        for (i, slot) in out.iter_mut().enumerate() {
            match p.props.get(4 + i).and_then(Prop::as_f64) {
                Some(v) => *slot = v,
                None => return default,
            }
        }
        out
    }

    /// A scalar integer `Properties70` value such as `RotationOrder`.
    pub fn property70_i64(&self, name: &FbxPropertyName<'_>, default: i64) -> i64 {
        self.property70(name)
            .and_then(|p| p.props.get(4))
            .and_then(Prop::as_i64)
            .unwrap_or(default)
    }
}

/// An entry of the `Objects` block.
#[derive(Debug)]
pub struct Object {
    pub id: FbxObjectId,
    /// Object name with the `\0\x01Class` suffix and any `namespace:` prefix removed.
    pub name: FbxObjectName,
    /// Object name with its namespace intact, e.g. `mixamorig:Hips`.
    ///
    /// Rig profiles are usually written against the namespaced name, so an
    /// importer wants this one even though momentum matches on the stripped one.
    pub qualified: FbxQualifiedName,
    /// The sub-class token, e.g. `LimbNode`, `Mesh`, `Cluster`, `BlendShapeChannel`.
    pub class: FbxClassName,
    /// The record name, e.g. `Model`, `Geometry`, `Deformer`.
    pub kind: FbxRecordName,
    pub node: Node,
}

impl Object {
    /// Classifies Models without asserting object existence or rig membership.
    /// OpenFBX interprets both Root and LimbNode as joint candidates.
    pub fn model_role(&self) -> Option<ModelRole> {
        if self.kind != FbxRecordName::MODEL {
            return None;
        }
        Some(
            if self.class == FbxClassName::ROOT || self.class == FbxClassName::LIMB_NODE {
                ModelRole::Joint
            } else if self.class == FbxClassName::NULL {
                ModelRole::Null
            } else {
                ModelRole::Uninterpreted
            },
        )
    }
}

/// One entry of the connection list.
#[derive(Debug, Clone)]
pub struct Link {
    /// The object being connected in.
    pub from: FbxObjectId,
    /// The property it connects to, for object-property (`OP`) links.
    ///
    /// Animation is addressed entirely through these: a curve node connects to
    /// a model's `Lcl Rotation`, and a curve connects to that node's `d|X`.
    pub property: Option<FbxConnectionProperty>,
}

/// The object table plus the connection graph of an FBX file.
/// Admitted records are immutable, so lookup indices stay consistent.
///
/// ```compile_fail
/// use fabelgeist_fbx::Scene;
/// let mut scene = Scene::from_roots(Vec::new());
/// scene.objects.clear();
/// ```
pub struct Scene {
    objects: Vec<Object>,
    /// The file's top-level records, kept for `GlobalSettings` and friends.
    roots: Vec<Node>,
    by_id: HashMap<FbxObjectId, usize>,
    /// Incoming links per object id, in file order (id 0 is the scene root).
    links: HashMap<FbxObjectId, Vec<Link>>,
}

impl Scene {
    /// Admitted object records in their original encounter order.
    pub fn objects(&self) -> impl Iterator<Item = &Object> {
        self.objects.iter()
    }

    /// Admitted top-level records, including uninterpreted metadata.
    pub fn roots(&self) -> impl Iterator<Item = &Node> {
        self.roots.iter()
    }

    pub fn from_roots(roots: Vec<Node>) -> Self {
        let mut objects = Vec::new();
        let mut by_id = HashMap::new();
        let mut links: HashMap<FbxObjectId, Vec<Link>> = HashMap::new();

        for root in &roots {
            if root.name != FbxRecordName::OBJECTS {
                continue;
            }
            for node in &root.children {
                let Some(id) = node.props.first().and_then(FbxObjectId::from_property) else {
                    continue;
                };
                let qualified = node
                    .props
                    .get(1)
                    .and_then(FbxQualifiedName::from_property)
                    .unwrap_or_default();
                let name = FbxObjectName::from(&qualified);
                let class = node
                    .props
                    .get(2)
                    .and_then(FbxClassName::from_property)
                    .unwrap_or_default();
                by_id.insert(id, objects.len());
                objects.push(Object {
                    id,
                    name,
                    qualified,
                    class,
                    kind: node.name.clone(),
                    node: node.clone(),
                });
            }
        }

        for root in &roots {
            if root.name != FbxRecordName::CONNECTIONS {
                continue;
            }
            for c in &root.children {
                // C: [type, from, to, (property name)]
                let (Some(from), Some(to)) = (
                    c.props.get(1).and_then(FbxObjectId::from_property),
                    c.props.get(2).and_then(FbxObjectId::from_property),
                ) else {
                    continue;
                };
                if from == FbxObjectId::SCENE_ROOT {
                    continue;
                }
                let property = c
                    .props
                    .get(3)
                    .and_then(FbxConnectionProperty::from_property);
                links.entry(to).or_default().push(Link { from, property });
            }
        }

        Self {
            objects,
            roots,
            by_id,
            links,
        }
    }

    pub fn parse(data: StorageView<'_>) -> Result<Self, FbxDecodeError> {
        Ok(Self::from_roots(parse(data)?))
    }

    pub fn get(&self, id: FbxObjectId) -> Option<&Object> {
        self.by_id.get(&id).map(|i| &self.objects[*i])
    }

    /// A top-level record such as `GlobalSettings` or `Definitions`.
    pub fn root(&self, name: &FbxRecordName) -> Option<&Node> {
        self.roots
            .iter()
            .find(|root: &&Node| -> bool { &root.name == name })
    }

    /// Objects connected as children of `id`, in file order. This is OpenFBX's
    /// `resolveObjectLink` ordering, which fixes the joint order of the rig.
    pub fn children(&self, id: FbxObjectId) -> impl Iterator<Item = &Object> {
        self.incoming(id).filter_map(|link| self.get(link.from))
    }

    /// As [`Scene::children`], keeping the property each link targets.
    pub fn children_with_property(
        &self,
        id: FbxObjectId,
    ) -> impl Iterator<Item = (&Object, Option<&FbxConnectionProperty>)> {
        self.incoming(id).filter_map(
            |link: &Link| -> Option<(&Object, Option<&FbxConnectionProperty>)> {
                self.get(link.from).map(
                    |object: &Object| -> (&Object, Option<&FbxConnectionProperty>) {
                        (object, link.property.as_ref())
                    },
                )
            },
        )
    }

    fn incoming(&self, id: FbxObjectId) -> impl Iterator<Item = &Link> {
        self.links.get(&id).map(Vec::as_slice).unwrap_or(&[]).iter()
    }

    /// The first child of `id` whose record name and class match.
    pub fn child_of_kind(
        &self,
        id: FbxObjectId,
        kind: &FbxRecordName,
        class: &FbxClassName,
    ) -> Option<&Object> {
        self.children(id)
            .find(|o: &&Object| -> bool { &o.kind == kind && &o.class == class })
    }

    pub fn objects_of_kind<'a>(
        &'a self,
        kind: &'a FbxRecordName,
        class: &'a FbxClassName,
    ) -> impl Iterator<Item = &'a Object> {
        self.objects
            .iter()
            .filter(move |o: &&Object| -> bool { &o.kind == kind && &o.class == class })
    }
}

#[cfg(test)]
mod scene_tests;
#[cfg(test)]
mod selector_tests;

#[cfg(test)]
mod name_tests;
