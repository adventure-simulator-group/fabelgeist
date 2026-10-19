//! Retained CPU values, including fields ignored by StandardMaterial reflection.
use super::*;
use bevy::{
    reflect::{PartialReflect, ReflectRef},
    render::storage::ShaderBuffer,
};
use std::collections::HashSet;

#[derive(SystemParam)]
pub(super) struct Stores<'w> {
    meshes: Res<'w, Assets<Mesh>>,
    standard: Res<'w, Assets<StandardMaterial>>,
    buffers: Res<'w, Assets<ShaderBuffer>>,
    bark: Res<'w, Assets<TacticalTreeBarkMaterial>>,
    leaves: Res<'w, Assets<TacticalTreeLeafCardMaterial>>,
    interior: Res<'w, Assets<InteriorMaterial>>,
    images: Res<'w, Assets<Image>>,
}

pub(super) struct Snapshot {
    meshes: Vec<(AssetId<Mesh>, Mesh)>,
    authored_buds: Vec<(AssetId<StandardMaterial>, StandardMaterial)>,
    buffers: Vec<(AssetId<ShaderBuffer>, ShaderBuffer)>,
    materials: Vec<MaterialValue>,
    images: Vec<(AssetId<Image>, Image)>,
}

enum MaterialValue {
    Bark {
        id: AssetId<TacticalTreeBarkMaterial>,
        value: Box<TacticalTreeBarkMaterial>,
    },
    Leaves {
        id: AssetId<TacticalTreeLeafCardMaterial>,
        value: Box<TacticalTreeLeafCardMaterial>,
    },
    Interior {
        id: AssetId<InteriorMaterial>,
        value: Box<InteriorMaterial>,
    },
}

impl Snapshot {
    pub(super) fn capture(stores: &Stores, members: &[Member]) -> Result<Self> {
        let meshes = members
            .iter()
            .map(|member| {
                Ok((
                    member.mesh,
                    stores
                        .meshes
                        .get(member.mesh)
                        .context("owned mesh asset")?
                        .clone(),
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        let ids: HashSet<_> = members.iter().map(|member| member.material).collect();
        let materials = ids
            .into_iter()
            .map(|id| MaterialValue::capture(stores, id))
            .collect::<Result<Vec<_>>>()?;
        let mut image_ids = HashSet::new();
        for material in &materials {
            image_handles(material.reflected(), &mut image_ids);
        }
        let images = image_ids
            .into_iter()
            .map(|id| {
                Ok((
                    id,
                    stores
                        .images
                        .get(id)
                        .context("owned material image")?
                        .clone(),
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        let authored_buds = members
            .iter()
            .filter_map(|member| {
                member
                    .authored_material
                    .try_typed::<StandardMaterial>()
                    .ok()
            })
            .map(|id| {
                Ok((
                    id,
                    stores
                        .standard
                        .get(id)
                        .context("exact authored bud source")?
                        .clone(),
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        let buffers = materials
            .iter()
            .filter_map(|material| match material {
                MaterialValue::Interior { value, .. } => Some(value.extension.field.id()),
                _ => None,
            })
            .map(|id| {
                let value = stores
                    .buffers
                    .get(id)
                    .context("selected interior field buffer")?;
                ensure!(
                    value.data.is_some(),
                    "selected interior field CPU data unavailable"
                );
                Ok((id, value.clone()))
            })
            .collect::<Result<Vec<_>>>()?;
        let snapshot = Self {
            meshes,
            materials,
            images,
            authored_buds,
            buffers,
        };
        snapshot.check(stores)?;
        Ok(snapshot)
    }

    pub(super) fn check(&self, stores: &Stores) -> Result<()> {
        ensure!(
            self.meshes.iter().all(|(id, value)| stores
                .meshes
                .get(*id)
                .is_some_and(|current| same_geometry(value, current))),
            "held mesh content drift"
        );
        ensure!(
            self.images
                .iter()
                .all(|(id, value)| stores.images.get(*id) == Some(value)),
            "held image content drift"
        );
        ensure!(
            self.authored_buds.iter().all(|(id, value)| stores
                .standard
                .get(*id)
                .is_some_and(|current| value.reflect_partial_eq(current) == Some(true)
                    && value.cull_mode == current.cull_mode)),
            "held authored bud material drift"
        );
        ensure!(
            self.buffers.iter().all(
                |(id, value)| stores.buffers.get(*id).is_some_and(|current| current.data
                    == value.data
                    && current.buffer_description == value.buffer_description
                    && current.asset_usage == value.asset_usage
                    && current.copy_on_resize == value.copy_on_resize)
            ),
            "held CPU interior field content drift"
        );
        for material in &self.materials {
            material.check(stores)?;
        }
        Ok(())
    }
}

impl MaterialValue {
    fn capture(stores: &Stores, id: UntypedAssetId) -> Result<Self> {
        if let Ok(id) = id.try_typed::<TacticalTreeBarkMaterial>() {
            return Ok(Self::Bark {
                id,
                value: Box::new(stores.bark.get(id).context("owned bark")?.clone()),
            });
        }
        if let Ok(id) = id.try_typed::<TacticalTreeLeafCardMaterial>() {
            return Ok(Self::Leaves {
                id,
                value: Box::new(stores.leaves.get(id).context("owned leaves")?.clone()),
            });
        }
        let id = id.try_typed::<InteriorMaterial>()?;
        Ok(Self::Interior {
            id,
            value: Box::new(
                stores
                    .interior
                    .get(id)
                    .context("owned effective buds")?
                    .clone(),
            ),
        })
    }

    fn reflected(&self) -> &dyn PartialReflect {
        match self {
            Self::Bark { value, .. } => value.as_ref(),
            Self::Leaves { value, .. } => value.as_ref(),
            Self::Interior { value, .. } => value.as_ref(),
        }
    }

    fn check(&self, stores: &Stores) -> Result<()> {
        let equal = match self {
            Self::Bark { id, value } => stores.bark.get(*id).is_some_and(|current| {
                value.reflect_partial_eq(current) == Some(true)
                    && value.base.cull_mode == current.base.cull_mode
            }),
            Self::Leaves { id, value } => stores
                .leaves
                .get(*id)
                .is_some_and(|current| value.reflect_partial_eq(current) == Some(true)),
            Self::Interior { id, value } => stores.interior.get(*id).is_some_and(|current| {
                value.reflect_partial_eq(current) == Some(true)
                    && value.base.cull_mode == current.base.cull_mode
            }),
        };
        ensure!(equal, "held material content drift or uncomparable field");
        Ok(())
    }
}

fn image_handles(value: &dyn PartialReflect, images: &mut HashSet<AssetId<Image>>) {
    if let Some(handle) = value.try_downcast_ref::<Handle<Image>>() {
        images.insert(handle.id());
        return;
    }
    match value.reflect_ref() {
        ReflectRef::Struct(value) => {
            for (_, field) in value.iter_fields() {
                image_handles(field, images);
            }
        }
        ReflectRef::TupleStruct(value) => {
            for field in value.iter_fields() {
                image_handles(field, images);
            }
        }
        ReflectRef::Tuple(value) => {
            for field in value.iter_fields() {
                image_handles(field, images);
            }
        }
        ReflectRef::Enum(value) => {
            for field in value.iter_fields() {
                image_handles(field.value(), images);
            }
        }
        ReflectRef::List(value) => {
            for field in value.iter() {
                image_handles(field, images);
            }
        }
        ReflectRef::Array(value) => {
            for field in value.iter() {
                image_handles(field, images);
            }
        }
        _ => {}
    }
}

fn same_geometry(left: &Mesh, right: &Mesh) -> bool {
    left.primitive_topology() == right.primitive_topology()
        && left.indices() == right.indices()
        && left.attributes().eq(right.attributes())
        && left.asset_usage == right.asset_usage
}

pub(super) fn retain_cpu_data(
    world: &mut World,
    bindings: impl Iterator<Item = (AssetId<Mesh>, UntypedAssetId)>,
) -> Result<()> {
    let mut images = HashSet::new();
    for (mesh, material) in bindings {
        world
            .resource_mut::<Assets<Mesh>>()
            .get_mut(mesh)
            .context("selected mesh retention")?
            .asset_usage |= bevy::asset::RenderAssetUsages::MAIN_WORLD;
        if let Ok(id) = material.try_typed::<TacticalTreeBarkMaterial>() {
            image_handles(
                world
                    .resource::<Assets<TacticalTreeBarkMaterial>>()
                    .get(id)
                    .context("selected bark")?,
                &mut images,
            );
        } else if let Ok(id) = material.try_typed::<TacticalTreeLeafCardMaterial>() {
            image_handles(
                world
                    .resource::<Assets<TacticalTreeLeafCardMaterial>>()
                    .get(id)
                    .context("selected leaf")?,
                &mut images,
            );
        } else {
            let id = material.try_typed::<StandardMaterial>()?;
            image_handles(
                world
                    .resource::<Assets<StandardMaterial>>()
                    .get(id)
                    .context("selected buds")?,
                &mut images,
            );
        }
    }
    for image in images {
        world
            .resource_mut::<Assets<Image>>()
            .get_mut(image)
            .context("selected image retention")?
            .asset_usage |= bevy::asset::RenderAssetUsages::MAIN_WORLD;
    }
    Ok(())
}
