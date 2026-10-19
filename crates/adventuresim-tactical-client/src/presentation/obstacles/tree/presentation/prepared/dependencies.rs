//! Reflected material image handles, independent of CPU pixel retention.
use super::*;
use bevy::reflect::{PartialReflect, ReflectRef};
use std::collections::HashSet;

pub(super) struct PartDependencies<'a> {
    pub(super) meshes: &'a Assets<Mesh>,
    pub(super) materials: &'a Assets<StandardMaterial>,
    pub(super) bark: &'a Assets<TacticalTreeBarkMaterial>,
    pub(super) aggregate_bark: &'a Assets<TacticalTreeAggregateBarkMaterial>,
    pub(super) leaves: &'a Assets<TacticalTreeLeafCardMaterial>,
    pub(super) cards: &'a Assets<TacticalTreeImpostorMaterial>,
    pub(super) images: &'a Assets<Image>,
}

impl PartDependencies<'_> {
    pub(super) fn verify(
        &self,
        parts: &[InstalledTreePart],
    ) -> Result<std::collections::HashSet<AssetId<Image>>> {
        let mut images = std::collections::HashSet::new();
        for part in parts {
            ensure!(
                self.meshes.contains(part.mesh),
                "retained prepared mesh missing"
            );
            let material: &dyn bevy::reflect::PartialReflect = match part.material {
                InstalledTreeMaterial::Bark(id) => self
                    .bark
                    .get(id)
                    .context("retained prepared bark material missing")?,
                InstalledTreeMaterial::Aggregate(id) => self
                    .aggregate_bark
                    .get(id)
                    .context("retained prepared aggregate material missing")?,
                InstalledTreeMaterial::Leaves(id) => self
                    .leaves
                    .get(id)
                    .context("retained prepared leaf material missing")?,
                InstalledTreeMaterial::Buds(id) => self
                    .materials
                    .get(id)
                    .context("retained prepared bud material missing")?,
                InstalledTreeMaterial::Card { material, image } => {
                    let card = self
                        .cards
                        .get(material)
                        .context("retained prepared canopy material missing")?;
                    ensure!(
                        card.baked_color.id() == image,
                        "prepared canopy image binding changed"
                    );
                    card
                }
            };
            material_image_ids(material, &mut images);
        }
        ensure!(
            images.iter().all(|image| self.images.contains(*image)),
            "retained prepared material image missing"
        );
        Ok(images)
    }
}

pub(crate) fn material_image_ids(value: &dyn PartialReflect, images: &mut HashSet<AssetId<Image>>) {
    if let Some(handle) = value.try_downcast_ref::<Handle<Image>>() {
        images.insert(handle.id());
        return;
    }
    match value.reflect_ref() {
        ReflectRef::Struct(value) => {
            for (_, field) in value.iter_fields() {
                material_image_ids(field, images);
            }
        }
        ReflectRef::TupleStruct(value) => {
            for field in value.iter_fields() {
                material_image_ids(field, images);
            }
        }
        ReflectRef::Tuple(value) => {
            for field in value.iter_fields() {
                material_image_ids(field, images);
            }
        }
        ReflectRef::Enum(value) => {
            for field in value.iter_fields() {
                material_image_ids(field.value(), images);
            }
        }
        ReflectRef::List(value) => {
            for field in value.iter() {
                material_image_ids(field, images);
            }
        }
        ReflectRef::Array(value) => {
            for field in value.iter() {
                material_image_ids(field, images);
            }
        }
        _ => {}
    }
}
