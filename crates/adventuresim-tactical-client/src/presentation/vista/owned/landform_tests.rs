//! Landform ownership must also survive clipping into distant terrain rings.
use super::*;
use adventuresim_world_schema::{SedimentaryRock, SurfaceLithology};
use std::collections::BTreeSet;

type TriangleBits = [[u32; 3]; 3];

fn triangle_bits(triangle: [Vec3; 3]) -> TriangleBits {
    triangle.map(|point| point.to_array().map(f32::to_bits))
}

fn mesh_triangles(mesh: &Mesh) -> BTreeSet<TriangleBits> {
    let positions = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .unwrap()
        .as_float3()
        .unwrap();
    let indices = mesh.indices().unwrap().iter().collect::<Vec<_>>();
    indices
        .as_chunks::<3>()
        .0
        .iter()
        .map(|indices| triangle_bits(indices.map(|index| Vec3::from_array(positions[index]))))
        .collect()
}

#[test]
fn landform_cutout_crossing_playable_boundary_retains_distant_foundation_bearings() {
    let input = TacticalSceneInput::load(std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/tactical-scenes/facade-review.json"
    )))
    .unwrap();
    let terrain = input
        .generate_unfurnished(Default::default())
        .unwrap()
        .terrain;
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
    let source = terrain.property_surface().unwrap();
    let removed: BTreeSet<_> = source
        .natural_triangles()
        .iter()
        .filter(|triangle| {
            adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(
                (triangle.iter().copied().sum::<Vec3>() / 3.0).xz(),
            )
            .is_ok_and(|point| collar.cuts_out(point))
        })
        .map(|[a, b, c]| triangle_bits([*a, *c, *b]))
        .collect();
    let bearings: BTreeSet<_> = source
        .foundations()
        .iter()
        .flat_map(|foundation| {
            foundation.support_triangles().iter().map(|indices| {
                triangle_bits(indices.map(|index| foundation.positions()[index as usize]))
            })
        })
        .collect();
    let lod = VistaLod {
        level: adventuresim_tactical_core::scene_input::VistaLevelIndex::new(0),
        width: 9,
        depth: 9,
        spacing_metres: 50.0,
        origin_east_metres: 0.0,
        origin_north_metres: 0.0,
        heights_metres: vec![0.0; 81],
        environment: vec![Default::default(); 81],
    };
    let ring_triangles = |collar| {
        vista_meshes(
            &terrain,
            &lod,
            Vec2::splat(1.0),
            None,
            None,
            input.environment_snapshot("owned-ring".into()).weather,
            collar,
        )
        .iter()
        .flat_map(mesh_triangles)
        .collect::<BTreeSet<_>>()
    };
    let uncut = ring_triangles(None);
    assert!(
        !uncut.is_disjoint(&removed),
        "fixture must exercise natural cutout faces in the ring"
    );
    let cut = ring_triangles(Some(collar));
    assert!(cut.is_disjoint(&removed));
    assert!(
        !bearings.is_disjoint(&uncut),
        "fixture must exercise distant foundation bearings"
    );
    assert!(bearings.intersection(&uncut).all(|face| cut.contains(face)));
}
