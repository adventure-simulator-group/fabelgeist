use super::*;

impl PreparedCityGround {
    fn inspect_geometry(&self) -> (Vec<Vec3>, [f64; 5]) {
        let mut assets = Assets::<Mesh>::default();
        let mut points = Vec::new();
        let mut areas = [0.0; 5];
        for (_, kind, handle, _) in self.meshes(PresentationOwner::Scene, &mut assets).unwrap() {
            let positions = assets
                .get(handle)
                .unwrap()
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .as_float3()
                .unwrap();
            for face in positions.as_chunks::<3>().0 {
                let [a, b, c] = face.map(Vec3::from_array).map(Vec3::as_dvec3);
                areas[kind.index()] += (b.xz() - a.xz()).perp_dot(c.xz() - a.xz()).abs() * 0.5;
            }
            points.extend(positions.iter().copied().map(Vec3::from_array));
        }
        (points, areas)
    }
}

#[test]
fn prepared_owned_paving_respects_selected_rings_and_matches_native_coverage() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/tactical-scenes/massive-city.json");
    let input = TacticalSceneInput::load(&path).unwrap();
    let generated = input.generate_unfurnished(Default::default()).unwrap();
    let terrain = &generated.terrain;
    let landform = PreparedTerrainLandform::from(generated.terrain_patch.clone());
    let full =
        PreparedCityGround::from_scene(&input, terrain, &landform, &[], input.vista.lods.len())
            .unwrap();
    let (full_points, _) = full.inspect_geometry();
    assert!(
        full_points.iter().any(|p| p.x < -1182.0),
        "fixture must expose paving outside the first ring"
    );
    let environment = input.environment_snapshot(generated.digest);
    for maximum_lods in [1, 2, 3] {
        let lods = input
            .vista
            .lods
            .iter()
            .take(maximum_lods)
            .collect::<Vec<_>>();
        let mut inner = Vec2::new(terrain.width(), terrain.depth()) * 0.5;
        let mut native = GroundSupport::default();
        native.add_mesh(
            &crate::presentation::terrain::urban_playable_mesh(terrain, input.landform.as_ref()),
            Vec3::ZERO,
        );
        if let PreparedTerrainLandform::Patch(patch) = &landform {
            native.add_patch(patch);
        }
        for (index, lod) in lods.iter().copied().enumerate() {
            for mesh in crate::presentation::vista::vista_lod_meshes_with_morph(
                lod,
                inner,
                lods.get(index + 1).copied(),
                Some(terrain),
                (index == 0).then_some(&environment),
                environment.weather,
                input.landform.map(|r| r.transition_collar()),
            ) {
                native.add_mesh(
                    &mesh,
                    Vec3::new(
                        lod.origin_east_metres as f32,
                        0.0,
                        lod.origin_north_metres as f32,
                    ),
                );
            }
            inner = Vec2::new(f32::from(lod.width - 1), f32::from(lod.depth - 1))
                * lod.spacing_metres
                * 0.5;
        }
        let prepared =
            PreparedCityGround::from_scene(&input, terrain, &landform, &[], maximum_lods).unwrap();
        let (points, areas) = prepared.inspect_geometry();
        assert!(!points.is_empty());
        assert!(
            points
                .iter()
                .all(|p| p.x.abs() <= inner.x + 0.001 && p.z.abs() <= inner.y + 0.001),
            "selected {maximum_lods} rings leave paving without displayed terrain"
        );
        let (_, native_areas) =
            PreparedCityGround::new(&input.streets, &input.yards, &[], &native).inspect_geometry();
        for (actual, expected) in areas.into_iter().zip(native_areas) {
            assert!(
                (actual - expected).abs() <= expected * 0.00001 + 0.001,
                "selected {maximum_lods}: prepared area {actual} vs native area {expected}"
            );
        }
    }
}

#[test]
fn streets_and_yards_cross_the_displayed_landform_without_lower_heightfield_paving() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/tactical-scenes/fault-scarp-cliff.json");
    let mut input = TacticalSceneInput::load(&path).unwrap();
    input.streets = vec![CityStreetPatch::Corridor {
        start_metres: Vec2::new(-5.0, 0.0),
        end_metres: Vec2::new(5.0, 0.0),
        half_width_metres: 2.0,
        surface: CityStreetSurface::Fieldstone,
    }];
    input.yards = vec![CityYardPatch {
        corners_metres: [
            Vec2::new(-3.0, 6.0),
            Vec2::new(-3.0, 10.0),
            Vec2::new(3.0, 10.0),
            Vec2::new(3.0, 6.0),
        ],
        surface: CityYardSurface::PackedEarth,
    }];
    let generated = input.generate_unfurnished(Default::default()).unwrap();
    let patch = generated.terrain_patch.as_ref().unwrap();
    let mut support = GroundSupport::default();
    support.add_patch(patch);
    let prepared = PreparedCityGround::from_scene(
        &input,
        &generated.terrain,
        &PreparedTerrainLandform::from(Some(patch.clone())),
        &[],
        input.vista.lods.len(),
    )
    .unwrap();
    let (points, areas) = prepared.inspect_geometry();
    assert!(
        (areas[CityGroundKind::FieldstoneStreet.index()] - 40.0).abs() < 0.01,
        "the complete ten-by-four-metre street must cross the fault patch"
    );
    assert!(
        (areas[CityGroundKind::PackedYard.index()] - 24.0).abs() < 0.01,
        "the complete six-by-four-metre yard must retain patch support"
    );
    for point in points {
        let plan =
            adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(point.xz())
                .unwrap();
        let ground = support.position(plan).unwrap();
        assert!(
            (0.0..0.02).contains(&(point.y - ground.y)),
            "paving must sit just above the displayed patch, not a lower heightfield"
        );
    }
}
