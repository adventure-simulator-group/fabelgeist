//! Material overlays retain the complete accepted physical support surface.
use super::*;
use crate::presentation::vista::streets::prepared::PreparedCityGround;
use adventuresim_tactical_core::city_layout::{
    CitySceneLayout, CitySingleProperty, CompoundGradingPolicy,
};
use adventuresim_tactical_core::scene_input::{GeneratedBuildingRecipes, TerrainSampleGrid};

fn graded_garden() -> TacticalSceneInput {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/tactical-scenes/garden-review.json");
    let mut input = TacticalSceneInput::load(&path).unwrap();
    input.playable = TerrainSampleGrid {
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
            .map(|garden| CitySingleProperty {
                id: garden.owner,
                building_id: garden.front_building_id,
                plot: garden.plot,
            })
            .collect(),
        ..Default::default()
    };
    input.grounding = None;
    input
        .ground_generated_city(&layout, CompoundGradingPolicy::bounded_settlement())
        .unwrap()
}

#[test]
fn prepared_owned_ground_keeps_full_physical_support_across_graded_boundaries() {
    let input = match std::env::var("FABELGEIST_GROUND_ACCEPTANCE_INPUT") {
        Ok(path) => TacticalSceneInput::load(std::path::Path::new(&path)).unwrap(),
        Err(_) => graded_garden(),
    };
    let generated = input
        .generate_unfurnished(GeneratedBuildingRecipes::default())
        .unwrap();
    let terrain = &generated.terrain;
    let prepared = PreparedCityGround::from_scene(
        &input,
        terrain,
        &crate::presentation::terrain::PreparedTerrainLandform::from(
            generated.terrain_patch.clone(),
        ),
        &generated.furniture.groups,
        input.vista.lods.len(),
    )
    .unwrap();
    let mut assets = Assets::<Mesh>::default();
    let mut checked = 0;
    for (_, kind, handle, _) in prepared
        .meshes(
            crate::presentation::ownership::PresentationOwner::Scene,
            &mut assets,
        )
        .unwrap()
    {
        let mesh = assets.get(handle).unwrap();
        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .unwrap()
            .as_float3()
            .unwrap();
        let bevy::mesh::VertexAttributeValues::Float32x4(footprints) =
            mesh.attribute(Mesh::ATTRIBUTE_COLOR).unwrap()
        else {
            panic!("ground footprints use four floating-point components");
        };
        let priority = match kind {
            CityGroundKind::EarthStreet => CityStreetSurface::CompactedEarth.priority(),
            CityGroundKind::GravelStreet => CityStreetSurface::Gravel.priority(),
            CityGroundKind::FieldstoneStreet => CityStreetSurface::Fieldstone.priority(),
            CityGroundKind::PackedYard | CityGroundKind::Garden => 0,
        };
        for (face, footprint) in positions
            .as_chunks::<3>()
            .0
            .iter()
            .zip(footprints.as_chunks::<3>().0)
        {
            let lift = if kind.is_yard() {
                YARD_SURFACE_LIFT_METRES
            } else {
                SURFACE_LIFT_METRES
                    + f32::from(priority) * SURFACE_PRIORITY_LIFT_METRES
                    + if footprint[0][2] == PatchKind::Market as u8 as f32 {
                        MARKET_PRIORITY_LIFT_METRES
                    } else {
                        0.0
                    }
            };
            let points = face.map(Vec3::from_array);
            let centre = (points[0].as_dvec3() + points[1].as_dvec3() + points[2].as_dvec3()) / 3.0;
            for mut point in points.into_iter().chain([centre.as_vec3()]) {
                point.y -= lift;
                let heights = terrain.support_elevations_at(
                    adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::from_metres(
                        point.xz(),
                    )
                    .unwrap(),
                );
                assert!(
                    heights
                        .iter()
                        .any(|h| (h.metres() - point.y).abs() <= 0.001),
                    "scene {}, material {kind:?}, point {point:?}, physical {heights:?}",
                    input.scene_key
                );
                checked += 1;
            }
        }
    }
    assert!(checked > 0);
}
