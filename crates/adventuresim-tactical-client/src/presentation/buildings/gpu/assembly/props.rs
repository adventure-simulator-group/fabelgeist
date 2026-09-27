//! Static vista furniture shares canonical geometry and GPU visibility records.
use super::*;
use crate::presentation::{
    furniture::{VistaFurniturePresentation, outdoor_range},
    interior_lighting::InteriorMaterialSource,
};
use adventuresim_tactical_core::prelude::{FurnitureLocation, SceneFurniture};

pub(super) fn parts(world: &mut World) -> Vec<Part> {
    let owners: HashMap<_, _> = world
        .query_filtered::<(Entity, &SceneFurniture), With<VistaFurniturePresentation>>()
        .iter(world)
        .filter(|(_, furniture)| matches!(furniture.location, FurnitureLocation::Outdoor { .. }))
        .map(|(entity, furniture)| (entity, outdoor_range(furniture.key.kind())))
        .collect();
    let parts: Vec<_> = world
        .query::<(
            Entity,
            &ChildOf,
            &GlobalTransform,
            &Mesh3d,
            Option<&MeshMaterial3d<StandardMaterial>>,
            Option<&InteriorMaterialSource>,
        )>()
        .iter(world)
        .filter_map(|(entity, parent, transform, mesh, standard, interior)| {
            let fade = owners.get(&parent.parent())?.clone();
            let material = standard
                .map(|material| material.0.clone())
                .or_else(|| interior.map(|material| material.0.clone()))?;
            Some(Part {
                entity: Some(entity),
                root: parent.parent(),
                transform: transform.to_matrix(),
                mesh: mesh.0.clone(),
                material,
                level: 2,
                fade: Some(fade),
            })
        })
        .collect();
    info!(
        instances = owners.len(),
        parts = parts.len(),
        "GPU city outdoor furniture"
    );
    parts
}
