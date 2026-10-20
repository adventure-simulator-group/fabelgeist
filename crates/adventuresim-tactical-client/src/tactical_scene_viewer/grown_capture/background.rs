//! Terrain submission readiness for the shared static capture background.
use super::*;
use crate::presentation::{TerrainDetailPatch, TerrainMaterialPresentation, VistaTerrainMesh};
use bevy::{
    camera::visibility::VisibleEntities, ecs::system::SystemParam,
    render::batching::NoAutomaticBatching,
};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Member {
    pub entity: Entity,
    pub role: Role,
    pub mesh: AssetId<Mesh>,
    pub material: bevy::asset::UntypedAssetId,
    pub transform: Mat4,
}

#[derive(bevy::ecs::query::QueryData)]
struct TerrainPart {
    entity: Entity,
    mesh: &'static Mesh3d,
    transform: &'static GlobalTransform,
    playable: Has<TerrainMaterialPresentation>,
    detail: Has<TerrainDetailPatch>,
    vista: Has<VistaTerrainMesh>,
    unbatchable: Has<NoAutomaticBatching>,
}

#[derive(SystemParam)]
pub(super) struct Inspector<'w, 's> {
    terrain: Query<'w, 's, TerrainPart>,
    materials: crate::presentation::TerrainDrawMaterialInspection<'w, 's>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Role {
    Playable,
    Detail,
    Vista,
}

impl Inspector<'_, '_> {
    pub(super) fn collect(&self, visible: &VisibleEntities) -> Result<Vec<Member>> {
        let visible: HashSet<_> = visible
            .get(std::any::TypeId::of::<Mesh3d>())
            .iter()
            .copied()
            .collect();
        let mut members = Vec::new();
        for part in &self.terrain {
            let role = match (part.playable, part.detail, part.vista) {
                (true, false, false) => Role::Playable,
                (false, true, false) => Role::Detail,
                (false, false, true) => Role::Vista,
                (false, false, false) => continue,
                _ => anyhow::bail!("ambiguous static terrain role"),
            };
            if !visible.contains(&part.entity) {
                continue;
            }
            ensure!(part.unbatchable, "terrain batching policy drift");
            members.push(Member {
                entity: part.entity,
                role,
                mesh: part.mesh.0.id(),
                material: self.materials.material(part.entity)?,
                transform: part.transform.to_matrix(),
            });
        }
        ensure!(
            members
                .iter()
                .filter(|member| member.role == Role::Playable)
                .count()
                == 1,
            "held camera requires its actual playable terrain"
        );
        members.sort_by_key(|member| member.entity.to_bits());
        Ok(members)
    }
}

pub(super) fn instrument(world: &mut World) {
    let entities: Vec<_> = world
        .query_filtered::<Entity, (
            With<Mesh3d>,
            Or<(
                With<TerrainMaterialPresentation>,
                With<TerrainDetailPatch>,
                With<VistaTerrainMesh>,
            )>,
            Without<NoAutomaticBatching>,
        )>()
        .iter(world)
        .collect();
    for entity in entities {
        world.entity_mut(entity).insert(NoAutomaticBatching);
    }
}
