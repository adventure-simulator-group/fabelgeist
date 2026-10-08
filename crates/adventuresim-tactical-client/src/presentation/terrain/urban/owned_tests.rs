//! Actual grounded catalogue scenes must retain owned render topology.
use super::*;
use bevy::ecs::world::CommandQueue;
use bevy::mesh::VertexAttributeValues;
use std::collections::BTreeSet;

type TriangleBits = [[u32; 3]; 3];

fn triangle_bits(triangle: [Vec3; 3]) -> TriangleBits {
    triangle.map(|point| point.to_array().map(f32::to_bits))
}

fn mesh_triangles(mesh: &Mesh) -> BTreeSet<TriangleBits> {
    let VertexAttributeValues::Float32x3(positions) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap()
    else {
        panic!("terrain position format");
    };
    let indices = mesh.indices().unwrap().iter().collect::<Vec<_>>();
    indices
        .as_chunks::<3>()
        .0
        .iter()
        .map(|indices| triangle_bits(indices.map(|index| Vec3::from_array(positions[index]))))
        .collect()
}

fn catalogue_terrain(name: &str) -> SceneTerrain {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../assets/tactical-scenes/{name}.json"));
    let input = TacticalSceneInput::load(&path).unwrap();
    assert!(input.streets.is_empty() && input.yards.is_empty());
    input
        .generate_unfurnished(Default::default())
        .unwrap()
        .terrain
}

struct GroundFixture {
    app: App,
    base: Entity,
    detail: Entity,
    material: Handle<TacticalTerrainMaterial>,
}

impl GroundFixture {
    fn new(terrain: SceneTerrain, landform: Option<TerrainLandformRecipe>) -> Self {
        let environment = SceneEnvironmentFixture::TemperateHills.snapshot("owned-no-streets");
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
            .add_systems(Update, update_coverage);
        let scene = app.world_mut().spawn((terrain.clone(), environment)).id();
        if let Some(recipe) = landform {
            app.world_mut().entity_mut(scene).insert(recipe);
        }
        let mut queue = CommandQueue::default();
        let mut meshes = Assets::<Mesh>::default();
        let mut materials = Assets::<TacticalTerrainMaterial>::default();
        {
            let mut commands = Commands::new(&mut queue, app.world());
            super::super::volumetric::spawn_base_and_fault(
                &mut commands,
                &mut meshes,
                &mut materials,
                scene,
                &SceneId("owned-no-streets".into()),
                &terrain,
                material,
                landform.as_ref(),
                landform.map(|recipe| recipe.transition_collar()),
            );
        }
        queue.apply(app.world_mut());
        app.insert_resource(meshes).insert_resource(materials);
        let mut bases = app.world_mut().query_filtered::<
            (Entity, &MeshMaterial3d<TacticalTerrainMaterial>), With<TerrainMaterialPresentation>>();
        let (base, material) = bases.single(app.world()).unwrap();
        let material = material.0.clone();
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
        Self {
            app,
            base,
            detail,
            material,
        }
    }

    fn base_mesh(&self) -> &Mesh {
        self.app
            .world()
            .resource::<Assets<Mesh>>()
            .get(&self.app.world().get::<Mesh3d>(self.base).unwrap().0)
            .unwrap()
    }
}

#[test]
fn grounded_catalogues_without_streets_never_cover_foundations_with_sampled_detail() {
    for name in ["facade-review", "heating-review"] {
        let terrain = catalogue_terrain(name);
        assert!(terrain.property_surface().is_some());
        let mut fixture = GroundFixture::new(terrain, None);
        for _ in 0..2 {
            fixture.app.update();
            assert_eq!(
                *fixture
                    .app
                    .world()
                    .get::<Visibility>(fixture.detail)
                    .unwrap(),
                Visibility::Hidden,
                "{name}: sampled detail obscures canonical owned topology"
            );
            assert_eq!(
                fixture
                    .app
                    .world()
                    .resource::<Assets<TacticalTerrainMaterial>>()
                    .get(&fixture.material)
                    .unwrap()
                    .extension
                    .detail_patch
                    .x,
                0.0,
                "{name}: canonical base must not discard the camera-local footprint"
            );
        }
    }
}

#[test]
fn owned_landform_presentation_omits_natural_cutout_and_retains_exact_bearings() {
    use adventuresim_world_schema::{SedimentaryRock, SurfaceLithology};
    let terrain = catalogue_terrain("facade-review");
    let original = serde_json::to_vec(&terrain).unwrap();
    let recipe = TerrainLandformRecipe::from_quantized(
        adventuresim_tactical_core::volumetric_terrain::QuantizedLandformRecipe {
            kind: TerrainLandformKind::FaultScarp,
            surface: TerrainSurfaceRecipe::new(
                SurfaceLithology::Sedimentary(SedimentaryRock::Sandstone),
                TerrainSurfaceSource::AuthoredFixture,
                17.into(),
                [10_000, 0],
            )
            .unwrap(),
            seed: 17.into(),
            origin_cm: [0, 0],
            tangent_permyriad: [10_000, 0],
            relief_cm: 600,
            half_length_cm: 1_200,
            half_width_cm: 1_000,
            collar_cm: 250,
            lod: TerrainLandformLod::Detail,
        },
    )
    .unwrap();
    let collar = recipe.transition_collar();
    let half = Vec2::new(terrain.width(), terrain.depth()) * 0.5;
    let in_playable = |triangle: &&[Vec3; 3]| {
        triangle
            .iter()
            .all(|point| point.xz().abs().cmple(half).all())
    };
    let source = terrain.property_surface().unwrap();
    let removed: BTreeSet<_> = source
        .natural_triangles()
        .iter()
        .filter(in_playable)
        .filter(|triangle| {
            adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(
                (triangle.iter().copied().sum::<Vec3>() / 3.0).xz(),
            )
            .is_ok_and(|point| collar.cuts_out(point))
        })
        .map(|[a, b, c]| triangle_bits([*a, *c, *b]))
        .collect();
    assert!(
        !removed.is_empty(),
        "fixture must exercise natural triangles inside the cutout"
    );
    let bearings: BTreeSet<_> = source
        .foundations()
        .iter()
        .flat_map(|foundation| {
            foundation
                .support_triangles()
                .iter()
                .map(|indices| indices.map(|index| foundation.positions()[index as usize]))
        })
        .filter(|triangle| in_playable(&triangle))
        .map(triangle_bits)
        .collect();
    assert!(!bearings.is_empty());
    let mut fixture = GroundFixture::new(terrain.clone(), Some(recipe));
    for _ in 0..2 {
        let presented = mesh_triangles(fixture.base_mesh());
        assert!(
            presented.is_disjoint(&removed),
            "natural ground covers the collision landform cutout"
        );
        assert!(
            bearings.is_subset(&presented),
            "landform cutout must retain accepted foundation bearings"
        );
        fixture.app.update();
    }
    assert_eq!(serde_json::to_vec(&terrain).unwrap(), original);
}
