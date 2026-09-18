//! Incremental city assembly keeps the standalone browser responsive during generation.
use super::*;
use std::collections::VecDeque;

const BUILDING_BATCH_BUDGET: std::time::Duration = std::time::Duration::from_millis(8);
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
                .distance_squared(focus)
                .total_cmp(&b.placement.centre_metres.distance_squared(focus))
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
            placement.centre_metres = Vec2::new(index as f32 * 100.0, 0.0);
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
    server: Res<'w, AssetServer>,
    prepared: Res<'w, Assets<prepared::PreparedCityAsset>>,
    residency: ResMut<'w, PreparedCityAssets>,
}

impl CityBuildingAssets<'_> {
    pub(super) fn spawn(
        &mut self,
        commands: &mut Commands,
        placement: &DistantBuildingPlacement,
        establishment: Option<&SceneEstablishment>,
        detail: BuildingDetail,
    ) -> Result<bool> {
        let compiled = if matches!(detail, BuildingDetail::Facade | BuildingDetail::Shell) {
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
        if detail == BuildingDetail::Shell {
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
            let sign = establishment.and_then(|establishment| {
                establishment.shop_name.clone().and_then(|name| {
                    adventuresim_building_generator::signs::ShopSign::for_establishment(
                        adventuresim_building_generator::signs::EstablishmentId(placement.id),
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
    if let Some(camera) = cameras.iter().next() {
        pending.prioritize(camera.translation().xz());
    }
    pending.advance(|pending| {
        assets.spawn(
            &mut commands,
            &pending.placement,
            pending.establishment.as_ref(),
            BuildingDetail::Shell,
        )
    });
}
