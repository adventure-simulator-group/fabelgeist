//! Incremental city assembly keeps the standalone browser responsive during generation.
use super::*;
use std::collections::VecDeque;

// Standalone city preload publishes static geometry only after the queue drains.
// Yield often enough for loading UI while amortizing the intervening scene frames.
const BUILDING_BATCH_BUDGET: std::time::Duration = std::time::Duration::from_millis(64);
const REPRIORITIZE_DISTANCE_METRES: f32 = 10.0;

#[derive(Clone)]
struct PendingCityBuilding {
    placement: DistantBuildingPlacement,
    establishment: Option<SceneEstablishment>,
}

#[derive(Resource)]
pub(crate) struct PendingCityBuildings {
    placements: VecDeque<PendingCityBuilding>,
    pub(crate) total: usize,
    focus: Option<Vec2>,
    failure: Option<String>,
}

impl PendingCityBuildings {
    pub(in crate::presentation) fn new(
        placements: &[DistantBuildingPlacement],
        establishments: &[SceneEstablishment],
    ) -> Self {
        Self {
            placements: placements
                .iter()
                .map(|placement| PendingCityBuilding {
                    placement: *placement,
                    establishment: establishments
                        .iter()
                        .find(|establishment| establishment.building_id == placement.id)
                        .cloned(),
                })
                .collect(),
            total: placements.len(),
            focus: None,
            failure: None,
        }
    }
    pub(crate) fn completed(&self) -> usize {
        self.total - self.placements.len()
    }

    pub(crate) fn finished(&self) -> bool {
        self.placements.is_empty()
    }

    pub(crate) fn failure(&self) -> Option<&str> {
        self.failure.as_deref()
    }

    pub(super) fn report_failure(&mut self, error: impl std::fmt::Display) {
        self.failure
            .get_or_insert_with(|| format!("Some city buildings are unavailable: {error}"));
    }

    fn prioritize(&mut self, focus: Vec2) {
        if self
            .focus
            .is_some_and(|previous| previous.distance(focus) < REPRIORITIZE_DISTANCE_METRES)
        {
            return;
        }
        self.focus = Some(focus);
        self.placements.make_contiguous().sort_by(|a, b| {
            a.placement
                .centre_metres
                .metres()
                .distance_squared(focus)
                .total_cmp(&b.placement.centre_metres.metres().distance_squared(focus))
        });
    }

    fn advance(&mut self, mut spawn: impl FnMut(&PendingCityBuilding) -> Result<bool>) {
        let started = web_time::Instant::now();
        // Visit each outstanding placement at most once. A pending asset must
        // not block ready geometry later in the queue or spin within a frame.
        for _ in 0..self.placements.len() {
            let placement = self.placements.pop_front().expect("bounded queue pass");
            match spawn(&placement) {
                Ok(true) => {}
                Ok(false) => self.placements.push_back(placement),
                Err(error) => {
                    self.report_failure(error);
                }
            }
            if started.elapsed() >= BUILDING_BATCH_BUDGET {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn placements() -> Vec<DistantBuildingPlacement> {
        let input: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../../assets/art-demo/city-layout.json"
        ))
        .unwrap();
        input["buildings"]
            .as_array()
            .unwrap()
            .iter()
            .take(3)
            .map(|value| serde_json::from_value(value.clone()).unwrap())
            .collect()
    }

    #[test]
    fn delayed_and_failed_assets_do_not_block_ready_buildings() {
        let placements = placements();
        let mut pending = PendingCityBuildings::new(&placements, &[]);
        let mut visited = Vec::new();
        // Repeat frames to tolerate a test runner pause exceeding the time budget.
        for _ in 0..placements.len() {
            pending.advance(|pending| {
                visited.push(pending.placement.id);
                if pending.placement.id == placements[0].id {
                    Ok(false)
                } else if pending.placement.id == placements[1].id {
                    Err("missing fixture".into())
                } else {
                    Ok(true)
                }
            });
        }
        assert!(visited.contains(&placements[2].id));
        assert_eq!(pending.completed(), 2);
        assert!(!pending.finished());
        assert!(pending.failure().unwrap().contains("missing fixture"));
        pending.advance(|_| Ok(true));
        assert!(pending.finished());
        assert_eq!(pending.completed(), 3);
    }

    #[test]
    fn camera_movement_prioritizes_nearby_unfinished_buildings() {
        let mut placements = placements();
        for (index, placement) in placements.iter_mut().enumerate() {
            placement.centre_metres =
                adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(Vec2::new(
                    index as f32 * 100.0,
                    0.0,
                ))
                .unwrap();
        }
        let mut pending = PendingCityBuildings::new(&placements, &[]);
        pending.prioritize(Vec2::new(200.0, 0.0));
        assert_eq!(
            pending.placements.front().unwrap().placement.id,
            placements[2].id
        );
        pending.prioritize(Vec2::ZERO);
        assert_eq!(
            pending.placements.front().unwrap().placement.id,
            placements[0].id
        );
    }
}

#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct CityBuildingAssets<'w> {
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: Res<'w, TacticalBuildingMaterials>,
    cache: ResMut<'w, TacticalBuildingMeshCache>,
    signs: signs::SignAssets<'w>,
    pub(super) gpu: ResMut<'w, gpu::PendingGpuCities>,
}

impl CityBuildingAssets<'_> {
    pub(super) fn spawn(
        &mut self,
        commands: &mut Commands,
        placement: &DistantBuildingPlacement,
        establishment: Option<&SceneEstablishment>,
        detail: BuildingDetail,
    ) -> Result<bool> {
        let program = placement.occupied_program();
        let compiled =
            cached_building_levels(&mut self.cache, &program, detail, &mut self.meshes, None)?;
        let transform = Transform::from_xyz(
            placement.centre_metres.metres().x,
            placement.base_elevation_metres.metres() + compiled.local_origin.metres().y,
            placement.centre_metres.metres().y,
        )
        .with_rotation(Quat::from_rotation_y(placement.orientation.yaw_radians()));
        let mut entity = commands.spawn((
            Name::new(format!("Distant city building {}", placement.id)),
            DistantCityBuildingPresentation,
            crate::presentation::ownership::PresentationOwner::Scene,
            Visibility::default(),
            transform,
        ));
        self.gpu
            .owners
            .get_mut(crate::presentation::ownership::PresentationOwner::Scene)
            .push(entity.id(), &transform, *placement, &compiled);
        entity.with_children(|parent| {
            let sign = establishment.and_then(|establishment| {
                establishment.shop_name.clone().and_then(|name| {
                    adventuresim_building_generator::signs::ShopSign::for_establishment(
                        adventuresim_building_generator::signs::EstablishmentId(placement.id.0),
                        establishment.business_id.key.usage,
                        name,
                    )
                })
            });
            self.signs
                .spawn(parent, sign.as_ref(), &compiled, &mut self.meshes);
        });
        Ok(true)
    }
}

pub(super) fn present(
    mut commands: Commands,
    pending: Option<ResMut<PendingCityBuildings>>,
    mut assets: CityBuildingAssets,
    cameras: Query<&GlobalTransform, With<TacticalGameplayCamera>>,
) {
    let Some(mut pending) = pending else {
        return;
    };
    if pending.finished() {
        if !assets.cache.recipes.is_empty() {
            assets.cache.recipes.clear();
        }
        return;
    }
    if let Some(camera) = cameras.iter().next() {
        pending.prioritize(camera.translation().xz());
    }
    let detail = BuildingDetail::Facade;
    pending.advance(|pending| {
        assets.spawn(
            &mut commands,
            &pending.placement,
            pending.establishment.as_ref(),
            detail,
        )
    });
    if pending.finished() {
        // Promoted venues may leave recipes with no remaining facade instance.
        // Drop those temporary CPU plans once all city meshes are resident.
        assets.cache.recipes.clear();
        #[cfg(target_family = "wasm")]
        if let Err(error) = super::super::generation::release_unused_facades(
            crate::presentation::ownership::PresentationOwner::Scene,
        ) {
            warn!(%error, "Could not release temporary generated facades");
        }
    }
}
