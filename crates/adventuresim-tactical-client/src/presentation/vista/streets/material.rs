//! Physical material policy for both playable and distant urban ground.

use super::*;
use adventuresim_procedural_textures::{FOREST_SOIL_TILE_METRES, ROCK_TILE_METRES};

const CITY_GROUND_SHADER: &str = "shaders/tactical_city_ground.wgsl";
const FIELDSTONE_SPACING_METRES: f32 = 0.28;
const GRAVEL_SPACING_METRES: f32 = 0.055;

#[derive(Clone, Copy, Debug)]
pub(super) enum CityGroundKind {
    EarthStreet,
    GravelStreet,
    FieldstoneStreet,
    PackedYard,
    Garden,
}

impl CityGroundKind {
    pub(super) const ALL: [Self; 5] = [
        Self::EarthStreet,
        Self::GravelStreet,
        Self::FieldstoneStreet,
        Self::PackedYard,
        Self::Garden,
    ];

    pub(super) const fn index(self) -> usize {
        self as usize
    }

    pub(super) const fn is_yard(self) -> bool {
        matches!(self, Self::PackedYard | Self::Garden)
    }

    pub(super) fn material(
        self,
        weather: WeatherSnapshot,
        textures: &ProceduralTextureAssets,
    ) -> CityGroundMaterial {
        let (stone_cover, stone_spacing, garden) = match self {
            Self::EarthStreet | Self::PackedYard => (0.0, FIELDSTONE_SPACING_METRES, 0.0),
            Self::GravelStreet => (0.72, GRAVEL_SPACING_METRES, 0.0),
            Self::FieldstoneStreet => (1.0, FIELDSTONE_SPACING_METRES, 0.0),
            Self::Garden => (0.0, FIELDSTONE_SPACING_METRES, 1.0),
        };
        CityGroundMaterial {
            base: StandardMaterial {
                perceptual_roughness: 0.9,
                ..default()
            },
            extension: CityGroundExtension {
                surface: Vec4::new(stone_cover, stone_spacing, garden, 0.0),
                weather: Vec4::new(
                    bps(weather.ground_moisture_bps),
                    bps(weather.snow_cover_bps),
                    0.0,
                    0.0,
                ),
                texture_scale: Vec4::new(
                    1.0 / FOREST_SOIL_TILE_METRES,
                    1.0 / ROCK_TILE_METRES,
                    0.0,
                    0.0,
                ),
                soil_height_ao: textures.forest_soil.height_ao.clone(),
                stone_albedo: textures.rock.albedo.clone(),
                stone_arm: textures.rock.arm.clone(),
            },
        }
    }
}

impl From<CityStreetSurface> for CityGroundKind {
    fn from(surface: CityStreetSurface) -> Self {
        match surface {
            CityStreetSurface::CompactedEarth => Self::EarthStreet,
            CityStreetSurface::Gravel => Self::GravelStreet,
            CityStreetSurface::Fieldstone => Self::FieldstoneStreet,
        }
    }
}

impl From<CityYardSurface> for CityGroundKind {
    fn from(surface: CityYardSurface) -> Self {
        match surface {
            CityYardSurface::PackedEarth => Self::PackedYard,
            CityYardSurface::KitchenGarden => Self::Garden,
        }
    }
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub(in crate::presentation) struct CityGroundExtension {
    #[uniform(100)]
    surface: Vec4,
    #[uniform(100)]
    weather: Vec4,
    #[uniform(100)]
    texture_scale: Vec4,
    #[texture(101)]
    #[sampler(102)]
    soil_height_ao: Handle<Image>,
    #[texture(103)]
    #[sampler(104)]
    stone_albedo: Handle<Image>,
    #[texture(105)]
    #[sampler(106)]
    stone_arm: Handle<Image>,
}

impl MaterialExtension for CityGroundExtension {
    fn fragment_shader() -> ShaderRef {
        CITY_GROUND_SHADER.into()
    }
    fn deferred_fragment_shader() -> ShaderRef {
        CITY_GROUND_SHADER.into()
    }
}

pub(in crate::presentation) type CityGroundMaterial =
    ExtendedMaterial<StandardMaterial, CityGroundExtension>;
