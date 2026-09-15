//! Full exterior detail is resident only for the closest inspection subjects.
use super::*;

const INSPECTION_RADIUS_METRES: f32 = 75.0;
const MAX_DETAILED_CITY_BUILDINGS: usize = 16;

#[derive(Component)]
pub(super) struct StreamedCityBuilding {
    placement: DistantBuildingPlacement,
    detail: Option<Entity>,
}

impl StreamedCityBuilding {
    pub(super) fn new(placement: DistantBuildingPlacement) -> Self {
        Self {
            placement,
            detail: None,
        }
    }
}

#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct CityDetailAssets<'w> {
    materials: Res<'w, TacticalBuildingMaterials>,
    server: Res<'w, AssetServer>,
    prepared: Res<'w, Assets<prepared::PreparedCityAsset>>,
    residency: ResMut<'w, PreparedCityAssets>,
}

pub(super) fn update(
    mut commands: Commands,
    enabled: Option<Res<StreamCityTraffic>>,
    pending: Option<Res<PendingCityBuildings>>,
    cameras: Query<&GlobalTransform, With<TacticalGameplayCamera>>,
    mut buildings: Query<(Entity, &Transform, &mut StreamedCityBuilding, &Children)>,
    mut facades: Query<(&PresentedBuildingMesh, &mut VisibilityRange)>,
    mut assets: CityDetailAssets,
) -> Result {
    if enabled.is_none() || pending.is_some() {
        return Ok(());
    }
    let Some(camera) = cameras.iter().next() else {
        return Ok(());
    };
    let position = camera.translation();
    let mut wanted = buildings
        .iter()
        .filter_map(|(entity, transform, building, _)| {
            let distance = transform.translation.distance(position);
            (distance < INSPECTION_RADIUS_METRES)
                .then(|| (entity, distance, building.placement.program()))
        })
        .collect::<Vec<_>>();
    wanted.sort_by(|a, b| a.1.total_cmp(&b.1));
    wanted.truncate(MAX_DETAILED_CITY_BUILDINGS);
    assets.residency.retain_details(&wanted);
    for (entity, _, mut building, children) in &mut buildings {
        if !wanted.iter().any(|(id, _, _)| *id == entity)
            && let Some(detail) = building.detail.take()
        {
            commands.entity(detail).despawn();
            set_facade_detail(children, &mut facades, false);
        }
    }
    // One new detailed subject per frame. Other buildings keep their facade
    // until their full detail is ready; the city never develops empty holes.
    if let Some((entity, _, program)) = wanted.into_iter().find(|(id, _, _)| {
        buildings
            .get(*id)
            .is_ok_and(|(_, _, building, _)| building.detail.is_none())
    }) {
        let Some(compiled) = assets.residency.get(
            &assets.server,
            &assets.prepared,
            &program,
            BuildingDetail::Static,
        )?
        else {
            return Ok(());
        };
        let (_, _, mut building, children) = buildings.get_mut(entity)?;
        let detail = commands
            .spawn((Transform::default(), Visibility::default(), ChildOf(entity)))
            .with_children(|parent| {
                let mut compiled = compiled.clone();
                compiled.lod1.clear();
                compiled.lod2.clear();
                spawn_building_levels(
                    parent,
                    building.placement.id,
                    &compiled,
                    BuildingPresentationScope::DistantCity,
                    &assets.materials,
                );
            })
            .id();
        building.detail = Some(detail);
        set_facade_detail(children, &mut facades, true);
    }
    Ok(())
}

fn set_facade_detail(
    children: &Children,
    facades: &mut Query<(&PresentedBuildingMesh, &mut VisibilityRange)>,
    detailed: bool,
) {
    for child in children.iter() {
        if let Ok((mesh, mut range)) = facades.get_mut(child)
            && matches!(mesh.level, BuildingRenderLevel::Lod1)
        {
            range.start_margin = if detailed {
                DETAIL_LOD_END_START_METRES..DETAIL_LOD_END_END_METRES
            } else {
                0.0..0.0
            };
        }
    }
}
