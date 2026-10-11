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
    let full =
        PreparedCityGround::from_scene(&input, terrain, &[], input.vista.lods.len()).unwrap();
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
        let prepared = PreparedCityGround::from_scene(&input, terrain, &[], maximum_lods).unwrap();
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
