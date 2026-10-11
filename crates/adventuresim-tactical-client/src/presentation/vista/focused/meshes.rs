//! Admit complete source ground, including the implicit landform cutout owner.
use super::*;
use crate::presentation::terrain::{
    PreparedTerrainLandform, terrain_patch_mesh, urban_playable_mesh,
};

pub(super) struct GroundMeshes {
    pub fine: GroundChunk,
    pub landform: Option<LandformChunk>,
    pub vista: Vec<GroundChunk>,
    pub support: streets::GroundSupport,
    /// Native producer boundary: canonical east/north half extents in metres.
    pub half_extent: Vec2,
}

pub(super) struct LandformChunk {
    pub chunk: GroundChunk,
    pub surface: TerrainSurfaceRecipe,
}

struct VistaChunks {
    chunks: Vec<GroundChunk>,
    /// Native producer boundary: canonical east/north half extents in metres.
    half_extent: Vec2,
}

#[derive(Debug, thiserror::Error)]
pub(super) enum GroundPreparationError {
    #[error("city landform geometry has no surface recipe")]
    LandformRecipe,
    #[error(transparent)]
    Reflection(#[from] geographic_ground::GeographicGroundError),
}

impl GroundMeshes {
    pub(super) fn new(
        product: &PreparedCityProduct,
        environment: &SceneEnvironment,
        maximum_lods: usize,
    ) -> std::result::Result<Self, GroundPreparationError> {
        let input = product.document.input();
        let fine = GroundChunk {
            mesh: urban_playable_mesh(&product.terrain, input.landform.as_ref()),
            origin: Vec3::ZERO,
        };
        let landform = match &product.landform {
            PreparedTerrainLandform::Natural => None,
            PreparedTerrainLandform::Patch(patch) => Some(LandformChunk {
                chunk: GroundChunk {
                    mesh: terrain_patch_mesh(patch.clone(), &product.terrain),
                    origin: Vec3::ZERO,
                },
                surface: input
                    .landform
                    .ok_or(GroundPreparationError::LandformRecipe)?
                    .surface(),
            }),
        };
        let vista = VistaChunks::new(product, environment, maximum_lods);
        let mut support = streets::GroundSupport::default();
        for chunk in std::iter::once(&fine)
            .chain(landform.iter().map(|landform| &landform.chunk))
            .chain(&vista.chunks)
        {
            support.add_mesh(&chunk.mesh, chunk.origin);
        }
        // Finish all fallible reflection before publishing renderer entities.
        Ok(Self {
            fine: fine.reflected()?,
            landform: landform
                .map(|landform| {
                    Ok::<_, GroundPreparationError>(LandformChunk {
                        chunk: landform.chunk.reflected()?,
                        surface: landform.surface,
                    })
                })
                .transpose()?,
            vista: vista
                .chunks
                .into_iter()
                .map(GroundChunk::reflected)
                .collect::<std::result::Result<_, _>>()?,
            support,
            half_extent: vista.half_extent,
        })
    }
}

impl VistaChunks {
    fn new(
        product: &PreparedCityProduct,
        environment: &SceneEnvironment,
        maximum_lods: usize,
    ) -> Self {
        let input = product.document.input();
        let lods = input
            .vista
            .lods
            .iter()
            .take(maximum_lods)
            .collect::<Vec<_>>();
        let mut inner = Vec2::new(product.terrain.width(), product.terrain.depth()) * 0.5;
        let mut chunks = Vec::new();
        for (index, lod) in lods.iter().copied().enumerate() {
            let origin = Vec3::new(
                lod.origin_east_metres as f32,
                0.0,
                lod.origin_north_metres as f32,
            );
            chunks.extend(
                vista_lod_meshes_with_morph(
                    lod,
                    inner,
                    lods.get(index + 1).copied(),
                    Some(&product.terrain),
                    (index == 0).then_some(environment),
                    environment.weather,
                    input.landform.map(|recipe| recipe.transition_collar()),
                )
                .into_iter()
                .map(|mesh| GroundChunk { mesh, origin }),
            );
            inner = Vec2::new(f32::from(lod.width - 1), f32::from(lod.depth - 1))
                * lod.spacing_metres
                * 0.5;
        }
        Self {
            chunks,
            half_extent: inner,
        }
    }
}
