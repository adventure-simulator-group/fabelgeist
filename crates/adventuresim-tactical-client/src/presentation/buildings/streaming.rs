//! Incremental city assembly keeps the standalone browser responsive during generation.
use super::*;
use std::collections::VecDeque;

const BUILDING_BATCH_BUDGET: std::time::Duration = std::time::Duration::from_millis(8);

#[derive(Resource)]
pub(crate) struct PendingCityBuildings {
    placements: VecDeque<DistantBuildingPlacement>,
    pub(crate) total: usize,
}

impl PendingCityBuildings {
    pub(in crate::presentation) fn new(placements: &[DistantBuildingPlacement]) -> Self {
        Self {
            placements: placements.iter().copied().collect(),
            total: placements.len(),
        }
    }
    pub(crate) fn completed(&self) -> usize {
        self.total - self.placements.len()
    }
}

#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct CityBuildingAssets<'w> {
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: Res<'w, TacticalBuildingMaterials>,
    cache: ResMut<'w, TacticalBuildingMeshCache>,
    signs: signs::SignAssets<'w>,
    server: Res<'w, AssetServer>,
    prepared: Res<'w, Assets<prepared::PreparedCityAsset>>,
    residency: ResMut<'w, PreparedCityAssets>,
}

impl CityBuildingAssets<'_> {
    pub(super) fn spawn(
        &mut self,
        commands: &mut Commands,
        placement: &DistantBuildingPlacement,
        detail: BuildingDetail,
    ) -> Result<bool> {
        let compiled = if detail == BuildingDetail::Facade {
            let Some(compiled) =
                self.residency
                    .get(&self.server, &self.prepared, &placement.program(), detail)?
            else {
                return Ok(false);
            };
            compiled
        } else {
            cached_building_levels(
                &mut self.cache,
                &placement.program(),
                detail,
                &mut self.meshes,
            )?
        };
        let mut entity = commands.spawn((
            Name::new(format!("Distant city building {}", placement.id)),
            DistantCityBuildingPresentation,
            Visibility::default(),
            Transform::from_xyz(
                placement.centre_metres.x,
                placement.base_elevation_metres + compiled.floor_offset_metres,
                placement.centre_metres.y,
            )
            .with_rotation(Quat::from_rotation_y(placement.orientation.yaw_radians())),
        ));
        if detail == BuildingDetail::Facade {
            entity.insert(super::city_detail::StreamedCityBuilding::new(*placement));
        }
        entity.with_children(|parent| {
            spawn_building_levels(
                parent,
                placement.id,
                &compiled,
                BuildingPresentationScope::DistantCity,
                &self.materials,
            );
            self.signs
                .spawn(parent, placement.id, None, &compiled, &mut self.meshes);
        });
        Ok(true)
    }
}

pub(super) fn present(
    mut commands: Commands,
    pending: Option<ResMut<PendingCityBuildings>>,
    mut assets: CityBuildingAssets,
) -> Result {
    let Some(mut pending) = pending else {
        return Ok(());
    };
    let started = web_time::Instant::now();
    while let Some(placement) = pending.placements.front() {
        if !assets.spawn(&mut commands, placement, BuildingDetail::Facade)? {
            break;
        }
        pending.placements.pop_front();
        if started.elapsed() >= BUILDING_BATCH_BUDGET {
            break;
        }
    }
    if pending.placements.is_empty() {
        commands.remove_resource::<PendingCityBuildings>();
    }
    Ok(())
}
