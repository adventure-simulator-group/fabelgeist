//! Object tables and incoming connection order for one decoded FBX scene.
use std::collections::HashMap;

use anyhow::Result;

use crate::{FbxConnectionProperty, FbxObjectId, Node};

/// An entry of the `Objects` block.
#[derive(Debug)]
pub struct Object {
    pub id: FbxObjectId,
    /// Object name with the `\0\x01Class` suffix and any `namespace:` prefix removed.
    pub name: String,
    /// Object name with its namespace intact, e.g. `mixamorig:Hips`.
    ///
    /// Rig profiles are usually written against the namespaced name, so an
    /// importer wants this one even though momentum matches on the stripped one.
    pub qualified: String,
    /// The sub-class token, e.g. `LimbNode`, `Mesh`, `Cluster`, `BlendShapeChannel`.
    pub class: String,
    /// The record name, e.g. `Model`, `Geometry`, `Deformer`.
    pub kind: String,
    pub node: Node,
}

impl Object {
    /// OpenFBX maps `Model::Root` onto a limb node, which is why `body_world`
    /// becomes joint 0 of the MHR skeleton rather than a plain null node.
    pub fn is_limb(&self) -> bool {
        self.kind == "Model" && (self.class == "LimbNode" || self.class == "Root")
    }

    pub fn is_null_node(&self) -> bool {
        self.kind == "Model" && self.class == "Null"
    }

    pub fn is_node(&self) -> bool {
        self.kind == "Model"
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
pub struct Scene {
    pub objects: Vec<Object>,
    /// The file's top-level records, kept for `GlobalSettings` and friends.
    pub roots: Vec<Node>,
    by_id: HashMap<FbxObjectId, usize>,
    /// Incoming links per object id, in file order (id 0 is the scene root).
    links: HashMap<FbxObjectId, Vec<Link>>,
}

fn object_name(raw: &[u8]) -> String {
    let name = match raw.windows(2).position(|w| w == [0, 1]) {
        Some(pos) => &raw[..pos],
        None => raw,
    };
    let name = String::from_utf8_lossy(name).into_owned();
    // momentum strips namespaces before matching joints against the .model file.
    match name.rfind(':') {
        Some(pos) => name[pos + 1..].to_string(),
        None => name,
    }
}

/// The object name with any namespace left on, e.g. `mixamorig:Hips`.
fn qualified_name(raw: &[u8]) -> String {
    let name = match raw.windows(2).position(|w| w == [0, 1]) {
        Some(pos) => &raw[..pos],
        None => raw,
    };
    String::from_utf8_lossy(name).into_owned()
}

impl Scene {
    /// Assemble decoded records, preserving encounter order and last-ID lookup.
    /// Malformed identity records are skipped; missing linked objects remain
    /// unresolved. This interpretation adds no binary-decoding failure path.
    pub fn from_roots(roots: Vec<Node>) -> Self {
        let mut objects = Vec::new();
        let mut by_id = HashMap::new();
        let mut links: HashMap<FbxObjectId, Vec<Link>> = HashMap::new();

        for root in &roots {
            if root.name != "Objects" {
                continue;
            }
            for node in &root.children {
                let Some(id) = node.props.first().and_then(FbxObjectId::from_property) else {
                    continue;
                };
                let name = node.str_prop(1).map(object_name).unwrap_or_default();
                let qualified = node.str_prop(1).map(qualified_name).unwrap_or_default();
                let class = node
                    .str_prop(2)
                    .map(|c| String::from_utf8_lossy(c).into_owned())
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
            if root.name != "Connections" {
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

    pub fn parse(data: &[u8]) -> Result<Self> {
        Ok(Self::from_roots(crate::parse(data)?))
    }

    pub fn get(&self, id: FbxObjectId) -> Option<&Object> {
        self.by_id.get(&id).map(|i| &self.objects[*i])
    }

    /// A top-level record such as `GlobalSettings` or `Definitions`.
    pub fn root(&self, name: &str) -> Option<&Node> {
        self.roots.iter().find(|root| root.name == name)
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
        self.incoming(id).filter_map(|link| {
            self.get(link.from)
                .map(|object| (object, link.property.as_ref()))
        })
    }

    fn incoming(&self, id: FbxObjectId) -> impl Iterator<Item = &Link> {
        self.links.get(&id).map(Vec::as_slice).unwrap_or(&[]).iter()
    }

    /// The first child of `id` whose record name and class match.
    pub fn child_of_kind(&self, id: FbxObjectId, kind: &str, class: &str) -> Option<&Object> {
        self.children(id)
            .find(|o| o.kind == kind && o.class == class)
    }

    pub fn objects_of_kind<'a>(
        &'a self,
        kind: &'a str,
        class: &'a str,
    ) -> impl Iterator<Item = &'a Object> {
        self.objects
            .iter()
            .filter(move |o| o.kind == kind && o.class == class)
    }
}

#[cfg(test)]
mod tests;
