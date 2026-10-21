//! The accepted graded surface replaces raw vista geometry without overlap.
use super::*;
use adventuresim_tactical_core::city_layout::grounding::GeographicSurface;
use adventuresim_tactical_core::city_layout::{
    CitySceneLayout, CitySingleProperty, CompoundGradingPolicy,
};
use adventuresim_tactical_core::scene_input::GeneratedBuildingRecipes;
use bevy::mesh::VertexAttributeValues;

fn surface(meshes: &[Mesh]) -> GeographicSurface {
    GeographicSurface::from_triangles(meshes.iter().flat_map(|mesh| {
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
            .map(|t| t.map(|i| Vec3::from_array(positions[i])))
            .filter(|triangle| {
                let [a, b, c] = triangle.map(Vec3::as_dvec3);
                let ab = b - a;
                let ac = c - a;
                // Bound interpolation, coordinate differences and the cross
                // product at the scene-coordinate scale, in area units. Using
                // only edge products misses a short edge created by rounding
                // two copies of the same clipped endpoint differently.
                let projection_roundoff = 8.0
                    * f64::EPSILON
                    * (a.length() + b.length() + c.length())
                    * (ab.length() + ac.length());
                // Retaining walls and unmeasurable graphics slivers cannot
                // supply an affine ground height. Use geographic admission;
                // the assertions below still require complete plot coverage.
                ab.cross(ac).y > projection_roundoff
                    && GeographicSurface::from_triangles([*triangle]).is_some()
            })
            .collect::<Vec<_>>()
    }))
    .unwrap()
}

#[test]
fn ground_projection_excludes_vertical_roundoff_at_each_mesh_scale() {
    for scale in [1.0e-8, 1.0, 1.0e8] {
        let ground = [Vec3::ZERO, Vec3::Z, Vec3::X].map(|point| point * scale);
        let wall = [
            Vec3::new(-2.7755576e-17, -0.19810131, 63.133064),
            Vec3::new(0.31322417, -0.19810131, 63.018383),
            Vec3::new(3.081488e-33, 0.0018986965, 63.133064),
        ]
        .map(|point| point * scale);
        let sliver = [
            Vec3::new(-2.7755576e-17, 0.0018986965, 63.133064),
            Vec3::new(0.31322417, -0.19810131, 63.018383),
            Vec3::new(3.081488e-33, 0.0018986965, 63.133064),
        ]
        .map(|point| point * scale);
        let mesh = triangle_mesh([ground, wall, sliver]);
        assert_eq!(surface(&[mesh]).triangles().count(), 1);
    }
}

#[test]
fn graded_distant_garden_frontage_has_one_surface_and_complete_ring_seams() {
    let mut input = TacticalSceneInput::load(std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/tactical-scenes/garden-review.json"
    )))
    .unwrap();
    input.playable = adventuresim_tactical_core::scene_input::TerrainSampleGrid {
        width: 3,
        depth: 3,
        spacing_metres: 1.0,
        heights_metres: vec![0.0; 9],
        environment: vec![Default::default(); 9],
    };
    for lod in &mut input.vista.lods {
        for (i, height) in lod.heights_metres.iter_mut().enumerate() {
            *height = ((i % usize::from(lod.width)) as f32 - f32::from(lod.width - 1) * 0.5)
                * lod.spacing_metres
                * 0.03;
        }
    }
    let layout = CitySceneLayout {
        playable: input.buildings.clone(),
        distant: input.distant_buildings.clone(),
        streets: input.streets.clone(),
        yards: input.yards.clone(),
        gardens: input.gardens.clone(),
        compounds: input.compounds.clone(),
        parishes: input.parishes.clone(),
        single_properties: input
            .gardens
            .iter()
            .map(|g| CitySingleProperty {
                id: g.owner,
                building_id: g.front_building_id,
                plot: g.plot,
            })
            .collect(),
        ..Default::default()
    };
    input.grounding = None;
    let input = input
        .ground_generated_city(&layout, CompoundGradingPolicy::bounded_settlement())
        .unwrap();
    let generated = input
        .generate_unfurnished(GeneratedBuildingRecipes::default())
        .unwrap();
    let terrain = &generated.terrain;
    let mut meshes = vec![playable_mesh(terrain, None).unwrap()];
    let mut inner = Vec2::new(terrain.width(), terrain.depth()) * 0.5;
    let environment = input.environment_snapshot(generated.digest);
    for (i, lod) in input.vista.lods.iter().enumerate() {
        meshes.extend(vista_lod_meshes_with_morph(
            lod,
            inner,
            input.vista.lods.get(i + 1),
            Some(terrain),
            Some(&environment),
            environment.weather,
            None,
        ).unwrap());
        inner = Vec2::new(f32::from(lod.width - 1), f32::from(lod.depth - 1))
            * lod.spacing_metres
            * 0.5;
    }
    let presented = surface(&meshes);
    let canonical = terrain.physical_geographic_surface().unwrap();
    for garden in &input.gardens {
        let comparison = presented
            .compare_in_outline(&canonical, &adventuresim_tactical_core::scene_coordinates::ScenePlanPolygon::from_ordered_vertices((garden.plot.corners()).iter().copied().map(adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from).collect::<Result<Vec<_>, _>>().unwrap()).unwrap())
            .unwrap();
        assert!(
            (comparison.covered_area_square_metres.square_metres()
                - comparison.required_area_square_metres.square_metres())
            .abs()
                < 0.01,
            "property {:?}: {comparison:?}",
            garden.owner
        );
        // Retaining edges have both floor and outside-soil datums. Check each
        // presented face against all canonical candidates below; a pairwise
        // single-valued comparison across that edge invents a height error.
        for route in &garden.access {
            for p in [route.start_metres(), route.end_metres()] {
                let p = adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(p)
                    .unwrap();
                assert!(
                    (presented.elevation_at(p).unwrap().metres()
                        - terrain.height_at(p).unwrap().metres())
                    .abs()
                        < 0.001
                );
            }
        }
    }
    for triangle in presented.triangles() {
        for point in triangle.into_iter().chain([
            (triangle[0] + triangle[1]) * 0.5,
            (triangle[1] + triangle[2]) * 0.5,
            (triangle[2] + triangle[0]) * 0.5,
            (triangle[0] + triangle[1] + triangle[2]) / 3.0,
        ]) {
            assert!(
                terrain
                    .support_elevations_at(
                        adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::from_metres(
                            point.xz()
                        )
                        .unwrap()
                    )
                    .iter()
                    .any(|h| (h.metres() - point.y).abs() < 0.001),
                "presented {point:?}, triangle {triangle:?}, canonical {:?}",
                terrain.support_elevations_at(
                    adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::from_metres(
                        point.xz()
                    )
                    .unwrap()
                )
            );
        }
    }
    for point in [
        Vec2::new(1.0, 0.37),
        Vec2::new(-1.0, -0.31),
        Vec2::new(1.01, 0.37),
    ] {
        let point =
            adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(point).unwrap();
        assert!(
            (presented.elevation_at(point).unwrap().metres()
                - terrain.height_at(point).unwrap().metres())
            .abs()
                < 0.001
        );
    }
}

/// Frozen pre-partition traversal: rectangle order, source face order, then
/// the existing clipper, normals and pigments. This is a test-only reference.
fn repeated_traversal_meshes(
    terrain: &SceneTerrain,
    lod: &VistaLod,
    inner: Vec2,
    coarser: Option<&VistaLod>,
    environment: Option<&SceneEnvironment>,
    weather: WeatherSnapshot,
) -> Vec<Mesh> {
    let surface = terrain.property_surface().unwrap();
    let half =
        Vec2::new(f32::from(lod.width - 1), f32::from(lod.depth - 1)) * lod.spacing_metres * 0.5;
    let cells = VISTA_CHUNK_CELLS.min(
        (usize::from(lod.width.max(lod.depth)) - 1)
            .div_ceil(2)
            .max(1),
    );
    let presentation = GroundPresentation::new(surface);
    let colors = VistaVertexColors::new(lod, coarser, environment, weather);
    let mut meshes = Vec::new();
    for z in (0..usize::from(lod.depth - 1)).step_by(cells) {
        for x in (0..usize::from(lod.width - 1)).step_by(cells) {
            let minimum = Vec2::new(x as f32, z as f32) * lod.spacing_metres - half;
            let maximum = (minimum + Vec2::splat(cells as f32 * lod.spacing_metres)).min(half);
            let triangles = cell_rectangles_outside_inner_rectangle(minimum, maximum, inner)
                .into_iter()
                .flat_map(|[x0, x1, z0, z1]| {
                    presentation.triangles(None).flat_map(move |triangle| {
                        clip::PreparedTriangle::new(triangle)
                            .in_rectangle(Vec2::new(x0, z0), Vec2::new(x1, z1))
                    })
                });
            let mut mesh = triangle_mesh(triangles);
            let positions = mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .as_float3()
                .unwrap();
            if positions.is_empty() {
                continue;
            }
            let values = positions
                .iter()
                .map(|p| colors.at(Vec2::new(p[0], p[2]), inner).unwrap())
                .collect::<Vec<_>>();
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, values);
            meshes.push(mesh);
        }
    }
    meshes
}

#[test]
#[ignore = "requires the frozen terrain acceptance input manifest"]
fn required_owned_partitions_preserve_ordered_mesh_bytes() {
    let manifest = std::env::var_os("FABELGEIST_PARTITION_ACCEPTANCE_MANIFEST")
        .expect("set the absolute fixed-camera manifest path");
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(manifest).unwrap()).unwrap();
    let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let fixtures = manifest["fixtures"].as_array().unwrap();
    assert_eq!(fixtures.len(), 12);
    for fixture in fixtures {
        let input =
            TacticalSceneInput::load(&workspace.join(fixture["input"].as_str().unwrap())).unwrap();
        let generated = input.generate_unfurnished(Default::default()).unwrap();
        let environment = input.environment_snapshot(generated.digest);
        let mut inner = Vec2::new(generated.terrain.width(), generated.terrain.depth()) * 0.5;
        let mut examined = 0;
        for (i, lod) in input.vista.lods.iter().enumerate() {
            let coarser = input.vista.lods.get(i + 1);
            let playable = (i == 0).then_some(&environment);
            let expected = repeated_traversal_meshes(
                &generated.terrain,
                lod,
                inner,
                coarser,
                playable,
                environment.weather,
            );
            let actual = vista_meshes(
                &generated.terrain,
                lod,
                inner,
                coarser,
                playable,
                environment.weather,
                None,
            );
            assert_eq!(
                actual.len(),
                expected.len(),
                "fixture {}, LOD {}",
                fixture["fixture"],
                lod.level
            );
            for (actual, expected) in actual.iter().zip(&expected) {
                for attribute in [
                    Mesh::ATTRIBUTE_POSITION,
                    Mesh::ATTRIBUTE_NORMAL,
                    Mesh::ATTRIBUTE_COLOR,
                ] {
                    assert!(
                        actual.attribute(attribute).unwrap().get_bytes()
                            == expected.attribute(attribute).unwrap().get_bytes(),
                        "fixture {}, LOD {}, attribute {}",
                        fixture["fixture"],
                        lod.level,
                        attribute.name
                    );
                }
                assert!(
                    actual
                        .indices()
                        .unwrap()
                        .iter()
                        .eq(expected.indices().unwrap().iter()),
                    "fixture {}, LOD {}, index order",
                    fixture["fixture"],
                    lod.level
                );
                examined += actual.count_vertices();
            }
            inner = Vec2::new(f32::from(lod.width - 1), f32::from(lod.depth - 1))
                * lod.spacing_metres
                * 0.5;
        }
        assert!(examined > 0);
        println!(
            "{}: {examined} ordinary terrain vertices preserve position, normal, colour and index bytes",
            fixture["fixture"]
        );
    }
}
