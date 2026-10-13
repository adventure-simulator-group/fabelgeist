//! Authored settlement anchors precede subdivision and seeded household selection.
use super::*;

const BASE_DEVELOPMENT_POPULATION: u32 = 40_000;
const MAX_DEVELOPMENT_POPULATION: u32 = 100_000;
const BASE_STRIP_COUNT: usize = 32;

/// Keep the surveyed east/west route fixed while larger towns develop along it.
/// The allocation bound remains finite even for an unsupported population.
#[derive(Clone, Copy)]
pub(super) struct DevelopmentExtent {
    pub strip_count: usize,
    pub radius_y_metres: adventuresim_building_generator::spatial_geometry::PositiveLength,
}

/// A Central German market-town design site, rather than a named reconstruction.
#[derive(Clone, Debug, PartialEq)]
pub struct CitySite {
    pub(super) alignment: [ScenePlanPoint; 5],
    pub parish_policy: adventuresim_world_schema::settlement_buildings::AuthoredParishPolicy,
}

impl DevelopmentExtent {
    pub fn for_population(population: ResidentCount) -> GeometryResult<Self> {
        let population = population.get();
        let population = population.clamp(BASE_DEVELOPMENT_POPULATION, MAX_DEVELOPMENT_POPULATION);
        let scale = population as f32 / BASE_DEVELOPMENT_POPULATION as f32;
        Ok(Self {
            strip_count: ((BASE_STRIP_COUNT as f32 * scale * 0.5).ceil() as usize) * 2,
            radius_y_metres:
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                    CITY_RADIUS_Y_METRES * scale,
                )?,
        })
    }

    pub fn market_strip(self) -> usize {
        self.strip_count / 2
    }
}

impl CitySite {
    pub fn central_german_market_town() -> GeometryResult<Self> {
        Ok(Self {
            parish_policy: adventuresim_world_schema::settlement_buildings::AuthoredParishPolicy::CENTRAL_GERMAN_MARKET_TOWN,
            alignment: [
                ScenePlanPoint::try_from(Vec2::new(-1_800.0, -180.0))?,
                ScenePlanPoint::try_from(Vec2::new(-450.0, -90.0))?,
                ScenePlanPoint::ORIGIN,
                ScenePlanPoint::try_from(Vec2::new(450.0, 40.0))?,
                ScenePlanPoint::try_from(Vec2::new(1_800.0, 180.0))?,
            ],
        })
    }

    /// Ascending eastings make the surveyed trade route an unambiguous spine.
    pub fn from_trade_route(alignment: [ScenePlanPoint; 5]) -> Option<Self> {
        (alignment.iter().all(|p| p.metres().is_finite())
            && alignment.windows(2).all(|pair| pair[1].metres().x > pair[0].metres().x)
            && alignment[0].metres().x < -CITY_RADIUS_X_METRES
            && alignment[4].metres().x > CITY_RADIUS_X_METRES)
            .then_some(Self { alignment, parish_policy: adventuresim_world_schema::settlement_buildings::AuthoredParishPolicy::CENTRAL_GERMAN_MARKET_TOWN })
    }

    pub(super) fn route_height(&self, x: f32) -> f32 {
        let pair = self
            .alignment
            .windows(2)
            .find(|pair| x <= pair[1].metres().x)
            .unwrap_or(&self.alignment[3..]);
        pair[0].metres().y
            + (pair[1].metres().y - pair[0].metres().y)
                * ((x - pair[0].metres().x) / (pair[1].metres().x - pair[0].metres().x))
                    .clamp(0.0, 1.0)
    }

    pub(super) fn street_graph(
        &self,
        seed: fabelgeist_determinism::Seed,
        extent: DevelopmentExtent,
    ) -> CityCompileResult<StreetGraph> {
        subdivision::build(self, seed, extent)
    }
}
