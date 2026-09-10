use adventuresim_building_generator::{
    BuildingLodLevel, BuildingLodMaterial, BuildingProgram, LodMesh, compile_building_collision,
    compile_building_detail, compile_building_lod, compile_static_building_detail, generate,
};
use bevy::ecs::hierarchy::ChildSpawnerCommands;

use super::recipe_mesh::recipe_mesh;
use super::*;

mod materials;
mod signs;
pub(crate) use materials::TacticalBuildingMaterials;
pub(in crate::presentation) use materials::setup_tactical_building_materials;
pub(in crate::presentation) use signs::BuildingPresentationPlugin;
pub(crate) use signs::PresentedSign;

pub(super) const DETAIL_LOD_END_START_METRES: f32 = 55.0;
pub(super) const DETAIL_LOD_END_END_METRES: f32 = 70.0;
const FACADE_LOD_END_START_METRES: f32 = 150.0;
const FACADE_LOD_END_END_METRES: f32 = 175.0;

#[derive(Component)]
pub(crate) struct DistantCityBuildingPresentation;

#[derive(Component)]
pub(crate) struct PresentedBuildingMesh {
    pub(crate) scope: BuildingPresentationScope,
    pub(crate) level: BuildingRenderLevel,
    pub(crate) material: BuildingLodMaterial,
    pub(crate) triangles: usize,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum BuildingRenderLevel {
    Lod0,
    Lod1,
    Lod2,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum BuildingPresentationScope {
    Playable,
    DistantCity,
}

#[derive(Clone)]
struct CompiledBuildingBatch {
    material: BuildingLodMaterial,
    mesh: Handle<Mesh>,
    triangles: usize,
}

#[derive(Clone)]
struct CompiledBuildingLevels {
    program: BuildingProgram,
    dynamic_openings: bool,
    floor_offset_metres: f32,
    local_origin: Vec3,
    sign_sites: Vec<(
        adventuresim_building_generator::signs::SignMount,
        adventuresim_building_generator::signs::SignSite,
    )>,
    lod0: Vec<CompiledBuildingBatch>,
    lod1: Vec<CompiledBuildingBatch>,
    lod2: Vec<CompiledBuildingBatch>,
}

#[derive(Default, Resource)]
pub(in crate::presentation) struct TacticalBuildingMeshCache(Vec<CompiledBuildingLevels>);

fn on_scene_building_added(
    event: On<Add, SceneBuilding>,
    mut commands: Commands,
    buildings: Query<(
        &SceneBuilding,
        Option<&adventuresim_building_generator::signs::ShopSign>,
    )>,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<TacticalBuildingMaterials>,
    mut cache: ResMut<TacticalBuildingMeshCache>,
    mut signs: signs::SignAssets,
) -> Result {
    let (building, authored_sign) = buildings.get(event.entity)?;
    let compiled = cached_building_levels(&mut cache, &building.program, true, &mut meshes)?;
    commands
        .entity(event.entity)
        .insert(Visibility::default())
        .with_children(|parent| {
            spawn_building_levels(
                parent,
                building.id,
                &compiled,
                BuildingPresentationScope::Playable,
                &materials,
            );
            signs.spawn(parent, building.id, authored_sign, &compiled, &mut meshes);
        });
    Ok(())
}

fn on_scene_vista_buildings(
    bundle: On<SceneVistaBundle>,
    mut commands: Commands,
    existing: Query<Entity, With<DistantCityBuildingPresentation>>,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<TacticalBuildingMaterials>,
    mut cache: ResMut<TacticalBuildingMeshCache>,
    mut signs: signs::SignAssets,
) -> Result {
    for entity in &existing {
        commands.entity(entity).despawn();
    }

    for placement in &bundle.distant_buildings {
        let compiled =
            cached_building_levels(&mut cache, &placement.program(), false, &mut meshes)?;
        commands
            .spawn((
                Name::new(format!("Distant city building {}", placement.id)),
                DistantCityBuildingPresentation,
                Visibility::default(),
                Transform::from_xyz(
                    placement.centre_metres.x,
                    placement.base_elevation_metres + compiled.floor_offset_metres,
                    placement.centre_metres.y,
                )
                .with_rotation(Quat::from_rotation_y(placement.orientation.yaw_radians())),
            ))
            .with_children(|parent| {
                spawn_building_levels(
                    parent,
                    placement.id,
                    &compiled,
                    BuildingPresentationScope::DistantCity,
                    &materials,
                );
                signs.spawn(parent, placement.id, None, &compiled, &mut meshes);
            });
    }
    Ok(())
}

fn cached_building_levels(
    cache: &mut TacticalBuildingMeshCache,
    program: &BuildingProgram,
    dynamic_openings: bool,
    meshes: &mut Assets<Mesh>,
) -> Result<CompiledBuildingLevels> {
    if let Some(compiled) = cache.0.iter().find(|compiled| {
        compiled.program == *program && compiled.dynamic_openings == dynamic_openings
    }) {
        return Ok(compiled.clone());
    }

    let plan = generate(program)?;
    let collision = compile_building_collision(&plan);
    let local_origin = collision.bounds.centre();
    let floor_offset_metres = local_origin.y - collision.bounds.min.y;
    let detail = if dynamic_openings {
        compile_static_building_detail(&plan)
    } else {
        compile_building_detail(&plan)
    };
    let facade = compile_building_lod(&plan, BuildingLodLevel::Facade);
    let shell = compile_building_lod(&plan, BuildingLodLevel::Shell);
    let compile_batches = |source: &[LodMesh], meshes: &mut Assets<Mesh>| {
        source
            .iter()
            .map(|batch| CompiledBuildingBatch {
                material: batch.material,
                mesh: meshes.add(recipe_mesh(batch, local_origin)),
                triangles: batch.indices.len() / 3,
            })
            .collect()
    };
    let compiled = CompiledBuildingLevels {
        program: program.clone(),
        dynamic_openings,
        floor_offset_metres,
        local_origin,
        sign_sites: if program
            .usage
            .and_then(adventuresim_building_generator::signs::shop_trade)
            .is_some()
        {
            signs::sites(&plan)
        } else {
            Vec::new()
        },
        lod0: compile_batches(&detail.meshes, meshes),
        lod1: compile_batches(&facade.meshes, meshes),
        lod2: compile_batches(&shell.meshes, meshes),
    };
    cache.0.push(compiled.clone());
    Ok(compiled)
}

fn spawn_building_levels(
    parent: &mut ChildSpawnerCommands,
    building_id: u64,
    compiled: &CompiledBuildingLevels,
    scope: BuildingPresentationScope,
    materials: &TacticalBuildingMaterials,
) {
    for (level, batches) in [
        (BuildingRenderLevel::Lod0, &compiled.lod0),
        (BuildingRenderLevel::Lod1, &compiled.lod1),
        (BuildingRenderLevel::Lod2, &compiled.lod2),
    ] {
        for batch in batches {
            parent.spawn((
                Name::new(format!("Building {:?} {:?}", level, batch.material)),
                PresentedBuildingMesh {
                    scope,
                    level,
                    material: batch.material,
                    triangles: batch.triangles,
                },
                Mesh3d(batch.mesh.clone()),
                MeshMaterial3d(materials.get_for_building(building_id, batch.material)),
                building_lod_visibility(level),
            ));
        }
    }
}

pub(super) fn building_lod_visibility(level: BuildingRenderLevel) -> VisibilityRange {
    match level {
        BuildingRenderLevel::Lod0 => VisibilityRange {
            start_margin: 0.0..0.0,
            end_margin: DETAIL_LOD_END_START_METRES..DETAIL_LOD_END_END_METRES,
            use_aabb: false,
        },
        BuildingRenderLevel::Lod1 => VisibilityRange {
            start_margin: DETAIL_LOD_END_START_METRES..DETAIL_LOD_END_END_METRES,
            end_margin: FACADE_LOD_END_START_METRES..FACADE_LOD_END_END_METRES,
            use_aabb: false,
        },
        BuildingRenderLevel::Lod2 => VisibilityRange {
            start_margin: FACADE_LOD_END_START_METRES..FACADE_LOD_END_END_METRES,
            // The generated settlement is already finite. Keeping its cheapest
            // shell visible avoids cutting the far half of a city from elevated
            // or exterior viewpoints.
            end_margin: f32::MAX..f32::MAX,
            use_aabb: false,
        },
    }
}

#[cfg(test)]
mod tests {
    use adventuresim_building_generator::LodVertex;

    use super::recipe_mesh::recipe_mesh;
    use super::*;

    #[test]
    fn shell_lod_has_no_artificial_distance_cutoff() {
        let visibility = building_lod_visibility(BuildingRenderLevel::Lod2);
        assert_eq!(visibility.end_margin, f32::MAX..f32::MAX);
    }

    #[test]
    fn interior_plaster_mesh_carries_tangent_space_for_its_bound_normal_map() {
        let batch = LodMesh {
            material: BuildingLodMaterial::InteriorPlaster,
            vertices: vec![
                LodVertex {
                    position: Vec3::ZERO,
                    normal: Vec3::Z,
                    uv: Vec2::ZERO,
                },
                LodVertex {
                    position: Vec3::X,
                    normal: Vec3::Z,
                    uv: Vec2::X,
                },
                LodVertex {
                    position: Vec3::X + Vec3::Y,
                    normal: Vec3::Z,
                    uv: Vec2::ONE,
                },
                LodVertex {
                    position: Vec3::Y,
                    normal: Vec3::Z,
                    uv: Vec2::Y,
                },
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
        };

        let mesh = recipe_mesh(&batch, Vec3::ZERO);

        assert!(mesh.attribute(Mesh::ATTRIBUTE_TANGENT).is_some());
    }
}
