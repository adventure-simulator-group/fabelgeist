//! Full exterior detail is resident only for the closest inspection subjects.
use super::*;

const INSPECTION_RADIUS_METRES: f32 = 75.0;
const MAX_DETAILED_CITY_BUILDINGS: usize = 16;

#[derive(Component)]
pub(super) struct StreamedCityBuilding {
    placement: DistantBuildingPlacement,
    facade: Option<Entity>,
    detail: Option<Entity>,
    facade_failed: bool,
    detail_failed: bool,
}

impl StreamedCityBuilding {
    pub(super) fn new(placement: DistantBuildingPlacement) -> Self {
        Self {
            placement,
            facade: None,
            detail: None,
            facade_failed: false,
            detail_failed: false,
        }
    }
}

#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct CityDetailAssets<'w> {
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: Res<'w, TacticalBuildingMaterials>,
    signs: signs::SignAssets<'w>,
    server: Res<'w, AssetServer>,
    prepared: Res<'w, Assets<prepared::PreparedCityAsset>>,
    residency: ResMut<'w, PreparedCityAssets>,
}

#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct CityDetailWorld<'w, 's> {
    cameras: Query<'w, 's, &'static GlobalTransform, With<TacticalGameplayCamera>>,
    buildings: Query<
        'w,
        's,
        (
            Entity,
            &'static Transform,
            &'static mut StreamedCityBuilding,
            &'static Children,
        ),
    >,
    hierarchy: Query<'w, 's, &'static Children>,
    levels: Query<'w, 's, (&'static PresentedBuildingMesh, &'static mut VisibilityRange)>,
}

type ResidencyCandidate = (Entity, f32, BuildingProgram);

fn residency_targets(
    buildings: &mut Query<(Entity, &Transform, &mut StreamedCityBuilding, &Children)>,
    position: Vec3,
) -> (Vec<ResidencyCandidate>, Vec<ResidencyCandidate>) {
    let mut facades = buildings
        .iter()
        .filter_map(|(entity, transform, building, _)| {
            let distance = transform.translation.distance(position);
            (distance < FACADE_LOD_END_END_METRES)
                .then(|| (entity, distance, building.placement.program()))
        })
        .collect::<Vec<_>>();
    facades.sort_by(|a, b| a.1.total_cmp(&b.1));
    let mut details = facades
        .iter()
        .filter(|(_, distance, _)| *distance < INSPECTION_RADIUS_METRES)
        .cloned()
        .collect::<Vec<_>>();
    details.truncate(MAX_DETAILED_CITY_BUILDINGS);
    (facades, details)
}

fn release_unwanted(
    commands: &mut Commands,
    buildings: &mut Query<(Entity, &Transform, &mut StreamedCityBuilding, &Children)>,
    hierarchy: &Query<&Children>,
    levels: &mut Query<(&PresentedBuildingMesh, &mut VisibilityRange)>,
    facades: &[ResidencyCandidate],
    details: &[ResidencyCandidate],
) {
    for (entity, _, mut building, root_children) in buildings {
        let wants_facade = facades.iter().any(|(id, _, _)| *id == entity);
        let wants_detail = details.iter().any(|(id, _, _)| *id == entity);
        if !wants_detail && let Some(detail) = building.detail.take() {
            commands.entity(detail).despawn();
            if let Some(facade) = building.facade
                && let Ok(facade_children) = hierarchy.get(facade)
            {
                set_facade_detail(facade_children, levels, false);
            }
        }
        if !wants_facade && let Some(facade) = building.facade.take() {
            commands.entity(facade).despawn();
            set_shell_facade(root_children, levels, false);
        }
    }
}

fn load_next_facade(
    commands: &mut Commands,
    pending: &mut Option<ResMut<PendingCityBuildings>>,
    buildings: &mut Query<(Entity, &Transform, &mut StreamedCityBuilding, &Children)>,
    levels: &mut Query<(&PresentedBuildingMesh, &mut VisibilityRange)>,
    candidates: Vec<ResidencyCandidate>,
    assets: &mut CityDetailAssets,
) -> Result<bool> {
    let Some((entity, _, program)) = candidates.into_iter().find(|(id, _, _)| {
        buildings
            .get(*id)
            .is_ok_and(|(_, _, building, _)| building.facade.is_none() && !building.facade_failed)
    }) else {
        return Ok(false);
    };
    let compiled = match assets.residency.get(
        &assets.server,
        &assets.prepared,
        &program,
        BuildingDetail::Facade,
    ) {
        Ok(Some(compiled)) => compiled,
        Ok(None) => return Ok(true),
        Err(error) => {
            buildings.get_mut(entity)?.2.facade_failed = true;
            if let Some(pending) = pending {
                pending.report_failure(error);
            }
            return Ok(true);
        }
    };
    let (_, _, mut building, root_children) = buildings.get_mut(entity)?;
    let facade = commands
        .spawn((Transform::default(), Visibility::default(), ChildOf(entity)))
        .with_children(|parent| {
            spawn_building_levels(
                parent,
                building.placement.id,
                &compiled,
                BuildingPresentationScope::DistantCity,
                &assets.materials,
            );
            assets.signs.spawn(
                parent,
                building.placement.id,
                None,
                &compiled,
                &mut assets.meshes,
            );
        })
        .id();
    building.facade = Some(facade);
    set_shell_facade(root_children, levels, true);
    Ok(true)
}

pub(super) fn update(
    mut commands: Commands,
    enabled: Option<Res<StreamCityTraffic>>,
    mut pending: Option<ResMut<PendingCityBuildings>>,
    mut world: CityDetailWorld,
    mut assets: CityDetailAssets,
) -> Result {
    if enabled.is_none() {
        return Ok(());
    }
    let Some(camera) = world.cameras.iter().next() else {
        return Ok(());
    };
    let (facades, details) = residency_targets(&mut world.buildings, camera.translation());
    assets.residency.retain_streamed(&facades, &details);
    release_unwanted(
        &mut commands,
        &mut world.buildings,
        &world.hierarchy,
        &mut world.levels,
        &facades,
        &details,
    );
    // One new facade per frame. Every building keeps its shell until the
    // replacement is resident, so streaming never creates empty city lots.
    if load_next_facade(
        &mut commands,
        &mut pending,
        &mut world.buildings,
        &mut world.levels,
        facades,
        &mut assets,
    )? {
        return Ok(());
    }
    // Full detail is likewise incremental, and the facade remains visible
    // until its replacement has loaded.
    if let Some((entity, _, program)) = details.into_iter().find(|(id, _, _)| {
        world.buildings.get(*id).is_ok_and(|(_, _, building, _)| {
            building.facade.is_some() && building.detail.is_none() && !building.detail_failed
        })
    }) {
        let compiled = match assets.residency.get(
            &assets.server,
            &assets.prepared,
            &program,
            BuildingDetail::Static,
        ) {
            Ok(Some(compiled)) => compiled,
            Ok(None) => return Ok(()),
            Err(error) => {
                world.buildings.get_mut(entity)?.2.detail_failed = true;
                if let Some(pending) = &mut pending {
                    pending.report_failure(error);
                }
                return Ok(());
            }
        };
        let (_, _, mut building, _) = world.buildings.get_mut(entity)?;
        let detail = commands
            .spawn((Transform::default(), Visibility::default(), ChildOf(entity)))
            .with_children(|parent| {
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
        if let Some(facade) = building.facade
            && let Ok(facade_children) = world.hierarchy.get(facade)
        {
            set_facade_detail(facade_children, &mut world.levels, true);
        }
    }
    Ok(())
}

fn set_shell_facade(
    children: &Children,
    levels: &mut Query<(&PresentedBuildingMesh, &mut VisibilityRange)>,
    facade_loaded: bool,
) {
    for child in children.iter() {
        if let Ok((mesh, mut range)) = levels.get_mut(child)
            && matches!(mesh.level, BuildingRenderLevel::Lod2)
        {
            range.start_margin = if facade_loaded {
                FACADE_LOD_END_START_METRES..FACADE_LOD_END_END_METRES
            } else {
                0.0..0.0
            };
        }
    }
}

fn set_facade_detail(
    children: &Children,
    levels: &mut Query<(&PresentedBuildingMesh, &mut VisibilityRange)>,
    detailed: bool,
) {
    for child in children.iter() {
        if let Ok((mesh, mut range)) = levels.get_mut(child)
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
