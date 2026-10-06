use super::*;
use adventuresim_world_schema::{MixedLithology, SurfaceLithology};

pub(super) fn fixture() -> Result<Fixture, Box<dyn std::error::Error>> {
    Ok(Fixture {
        name: "fault-scarp-cliff",
        scene_key: "fault-scarp",
        seed: 47_114,
        terrain: rolling,
        environment: rocky_open,
        weather: clear(),
        vista: VistaKind::Ordinary,
        buildings: BuildingFixture::Empty,
        playable_spacing_metres: 12.5,
        landform: Some(TerrainLandformRecipe::from_quantized(
            adventuresim_tactical_core::volumetric_terrain::QuantizedLandformRecipe {
                kind: TerrainLandformKind::FaultScarp,
                surface: TerrainSurfaceRecipe::new(
                    SurfaceLithology::Mixed(MixedLithology::Breccia),
                    TerrainSurfaceSource::AuthoredFixture,
                    47_114.into(),
                    [10_000, 0],
                )?,
                seed: 47_114.into(),
                origin_cm: [0, 0],
                tangent_permyriad: [10_000, 0],
                relief_cm: 800,
                half_length_cm: 4_500,
                half_width_cm: 1_800,
                collar_cm: 400,
                lod: TerrainLandformLod::Detail,
            },
        )?),
    })
}
