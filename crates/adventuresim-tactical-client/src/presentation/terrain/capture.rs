//! Read-only native capture boundary for actual terrain material handles.
use super::*;
use anyhow::{Result, bail};
use bevy::{asset::UntypedAssetId, ecs::system::SystemParam};

#[derive(bevy::ecs::query::QueryData)]
struct TerrainMaterials {
    playable: Option<&'static MeshMaterial3d<TacticalTerrainMaterial>>,
    vista: Option<&'static MeshMaterial3d<super::super::vista::TacticalVistaMaterial>>,
    city: Option<&'static MeshMaterial3d<super::super::vista::CityGroundMaterial>>,
}

#[derive(SystemParam)]
pub(crate) struct TerrainDrawMaterialInspection<'w, 's> {
    materials: Query<'w, 's, TerrainMaterials>,
}

impl TerrainDrawMaterialInspection<'_, '_> {
    pub(crate) fn material(&self, entity: Entity) -> Result<UntypedAssetId> {
        let actual = self.materials.get(entity)?;
        match (actual.playable, actual.vista, actual.city) {
            (Some(material), None, None) => Ok(material.0.id().untyped()),
            (None, Some(material), None) => Ok(material.0.id().untyped()),
            (None, None, Some(material)) => Ok(material.0.id().untyped()),
            _ => bail!("terrain capture material missing or ambiguous"),
        }
    }
}
