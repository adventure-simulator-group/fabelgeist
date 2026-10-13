//! Urban ground retains canonical fine topology beneath static city surfaces.
//!
//! A road clipped to fine terrain cannot coexist with a coarse triangle that
//! crosses its valleys. City scenes therefore keep that same fine base mesh
//! throughout the bounded playable region and omit the redundant moving patch.

use super::*;

pub(in crate::presentation) struct UrbanGroundCoveragePlugin;

#[derive(Component)]
struct GroundCoverage {
    revision: u64,
    topology: GroundTopology,
}

/// Accepted owned surfaces retain vertical steps and multiple support levels;
/// streets also require canonical fine terrain beneath their clipped geometry.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum GroundTopology {
    Canonical,
    Sampled,
}

impl Plugin for UrbanGroundCoveragePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_coverage
                .after(present_pending_terrain)
                .after(update_terrain_detail_patch),
        );
    }
}

impl GroundTopology {
    pub(super) fn for_scene(
        terrain: &SceneTerrain,
        environment: &SceneEnvironment,
        vista: &ActiveVistaSurface,
    ) -> Self {
        if terrain.property_surface().is_some() || vista.is_urban_scene(&environment.scene_digest) {
            Self::Canonical
        } else {
            Self::Sampled
        }
    }
}

pub(in crate::presentation) fn urban_playable_mesh(
    terrain: &SceneTerrain,
    landform: Option<&TerrainLandformRecipe>,
) -> Mesh {
    if let Some(mesh) = super::super::vista::owned::playable_mesh(
        terrain,
        landform.map(|recipe| recipe.transition_collar()),
    ) {
        return mesh;
    }
    landform.map_or_else(
        || terrain.mesh(),
        |recipe| terrain.mesh_with_transition(recipe.transition_collar()),
    )
}

#[expect(
    clippy::type_complexity,
    reason = "the terrain coverage system borrows scene identity, presentation state, and independent Bevy asset stores"
)]
fn update_coverage(
    mut commands: Commands,
    vista: Res<ActiveVistaSurface>,
    scenes: Query<(
        &SceneTerrain,
        &SceneEnvironment,
        Option<&TerrainLandformRecipe>,
    )>,
    mut bases: Query<
        (
            Entity,
            &ScenePresentationOf,
            &mut Mesh3d,
            &MeshMaterial3d<TacticalTerrainMaterial>,
            &mut TerrainTriangleCount,
            Option<&GroundCoverage>,
        ),
        With<TerrainMaterialPresentation>,
    >,
    mut details: Query<(&ScenePresentationOf, &mut Visibility), With<TerrainDetailPatch>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TacticalTerrainMaterial>>,
) {
    for (entity, source, mut mesh, material, mut triangle_count, previous) in &mut bases {
        let Ok((terrain, environment, landform)) = scenes.get(source.0) else {
            continue;
        };
        let topology = GroundTopology::for_scene(terrain, environment, &vista);
        let cutout = if topology == GroundTopology::Canonical {
            0.0
        } else {
            1.0
        };
        if materials
            .get(&material.0)
            .is_some_and(|material| material.extension.detail_patch.x != cutout)
            && let Some(mut material) = materials.get_mut(&material.0)
        {
            material.extension.detail_patch.x = cutout;
        }
        let changed = previous.map_or(topology == GroundTopology::Canonical, |previous| {
            previous.topology != topology || previous.revision != vista.revision()
        });
        if changed {
            let replacement = if topology == GroundTopology::Canonical {
                urban_playable_mesh(terrain, landform)
            } else {
                landform.map_or_else(
                    || terrain.coarse_mesh(),
                    |recipe| terrain.coarse_mesh_with_transition(recipe.transition_collar()),
                )
            };
            triangle_count.0 = mesh_triangle_count(&replacement);
            mesh.0 = meshes.add(replacement);
            commands.entity(entity).insert(GroundCoverage {
                revision: vista.revision(),
                topology,
            });
            if topology == GroundTopology::Canonical {
                info!(triangles = triangle_count.0, scene = %environment.scene_digest, "Retained canonical urban terrain topology");
            }
        }
    }
    for (source, mut visibility) in &mut details {
        let Ok((terrain, environment, _)) = scenes.get(source.0) else {
            continue;
        };
        let desired = if GroundTopology::for_scene(terrain, environment, &vista)
            == GroundTopology::Canonical
        {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        if *visibility != desired {
            *visibility = desired;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn late_vista_switches_existing_base_to_fine_topology_and_can_restore_natural_lod() {
        let terrain = SceneTerrain::new(8, 8, 1.0, |_| 0.0)
            .unwrap()
            .refined(0.5, |point, base| base + point.x.sin() * 0.02)
            .unwrap();
        let fine_count = terrain.mesh().count_vertices();
        let coarse_mesh = terrain.coarse_mesh();
        let coarse_count = coarse_mesh.count_vertices();
        let environment = SceneEnvironmentFixture::TemperateHills.snapshot("late-city");
        let mut images = Assets::<Image>::default();
        let textures = generate_procedural_textures(
            &adventuresim_procedural_textures::TextureParameters {
                resolution: adventuresim_procedural_textures::BakeResolution::Draft,
                ..default()
            },
            &mut images,
        );
        let material = terrain_material(
            &terrain,
            &environment,
            None,
            &textures,
            &mut images,
            &TacticalGraphicsSettings::default().config.grass,
        );
        let mut app = App::new();
        app.init_resource::<ActiveVistaSurface>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<TacticalTerrainMaterial>>()
            .add_systems(Update, update_coverage);
        let mesh = app
            .world_mut()
            .resource_mut::<Assets<Mesh>>()
            .add(coarse_mesh);
        let material = app
            .world_mut()
            .resource_mut::<Assets<TacticalTerrainMaterial>>()
            .add(material);
        let scene = app.world_mut().spawn((terrain, environment)).id();
        let base = app
            .world_mut()
            .spawn((
                ScenePresentationOf(scene),
                TerrainMaterialPresentation,
                TerrainTriangleCount(0),
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
            ))
            .id();
        let detail = app
            .world_mut()
            .spawn((
                ScenePresentationOf(scene),
                TerrainDetailPatch {
                    centre: Vec2::ZERO,
                    vista_revision: 0,
                },
                Visibility::Inherited,
            ))
            .id();
        app.update();
        assert_eq!(
            app.world()
                .resource::<Assets<Mesh>>()
                .get(&app.world().get::<Mesh3d>(base).unwrap().0)
                .unwrap()
                .count_vertices(),
            coarse_count
        );
        // Render-only assets can leave CPU storage after their first GPU upload.
        app.world_mut()
            .resource_mut::<Assets<Mesh>>()
            .remove(mesh.id());
        let mut bundle = SceneVistaBundle {
            properties: None,
            scene_digest: "late-city".into(),
            playable_half_extent_metres: Vec2::splat(4.0),
            distant_buildings: vec![],
            establishments: vec![],
            furniture_groups: vec![],
            distant_furniture: vec![],
            lods: vec![],
            yards: vec![],
            parishes: Vec::new(),
            compounds: Vec::new(),
            gardens: Vec::new(),
            streets: vec![CityStreetPatch::Corridor {
                start_metres:
                    adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(
                        Vec2::new(-3.0, 0.0),
                    )
                    .unwrap(),
                end_metres:
                    adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(
                        Vec2::new(3.0, 0.0),
                    )
                    .unwrap(),
                half_width_metres:
                    adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                        1.0,
                    )
                    .unwrap(),
                surface: CityStreetSurface::Fieldstone,
            }],
        };
        app.world_mut()
            .resource_mut::<ActiveVistaSurface>()
            .update(&bundle);
        app.update();
        assert_eq!(
            app.world()
                .resource::<Assets<Mesh>>()
                .get(&app.world().get::<Mesh3d>(base).unwrap().0)
                .unwrap()
                .count_vertices(),
            fine_count
        );
        assert_eq!(
            *app.world().get::<Visibility>(detail).unwrap(),
            Visibility::Hidden
        );
        assert_eq!(
            app.world()
                .resource::<Assets<TacticalTerrainMaterial>>()
                .get(&material)
                .unwrap()
                .extension
                .detail_patch
                .x,
            0.0
        );
        bundle.streets.clear();
        app.world_mut()
            .resource_mut::<ActiveVistaSurface>()
            .update(&bundle);
        app.update();
        assert_eq!(
            app.world()
                .resource::<Assets<Mesh>>()
                .get(&app.world().get::<Mesh3d>(base).unwrap().0)
                .unwrap()
                .count_vertices(),
            coarse_count
        );
        assert_eq!(
            *app.world().get::<Visibility>(detail).unwrap(),
            Visibility::Inherited
        );
        assert_eq!(
            app.world()
                .resource::<Assets<TacticalTerrainMaterial>>()
                .get(&material)
                .unwrap()
                .extension
                .detail_patch
                .x,
            1.0
        );
    }
}

#[cfg(test)]
mod owned_tests;
