//! Exact managed-ground ownership, independent of terrain grading and sampling.
use super::*;
use crate::{
    city_layout::{
        CityPlotBounds, CityStreetPatch, CityStreetSurface, CityYardPatch, SPATIAL_BUCKET_METRES,
    },
    scene_input::GeneratedBuilding,
};
use std::{collections::BTreeMap, sync::OnceLock};

#[derive(Default, Debug, Clone)]
struct SpatialBuckets(BTreeMap<(i32, i32), Vec<usize>>);

impl SpatialBuckets {
    fn new(bounds: impl IntoIterator<Item = (Vec2, Vec2)>) -> Self {
        let mut buckets = BTreeMap::<(i32, i32), Vec<usize>>::new();
        for (index, (minimum, maximum)) in bounds.into_iter().enumerate() {
            let minimum = (minimum / SPATIAL_BUCKET_METRES).floor().as_ivec2();
            let maximum = (maximum / SPATIAL_BUCKET_METRES).floor().as_ivec2();
            for x in minimum.x..=maximum.x {
                for y in minimum.y..=maximum.y {
                    buckets.entry((x, y)).or_default().push(index);
                }
            }
        }
        Self(buckets)
    }

    fn at(&self, point: Vec2) -> &[usize] {
        let cell = (point / SPATIAL_BUCKET_METRES).floor().as_ivec2();
        self.0.get(&(cell.x, cell.y)).map_or(&[], Vec::as_slice)
    }

    fn overlapping(&self, minimum: Vec2, maximum: Vec2) -> impl Iterator<Item = usize> + '_ {
        let minimum = (minimum / SPATIAL_BUCKET_METRES).floor().as_ivec2();
        let maximum = (maximum / SPATIAL_BUCKET_METRES).floor().as_ivec2();
        (minimum.x..=maximum.x)
            .flat_map(move |x| (minimum.y..=maximum.y).map(move |y| (x, y)))
            .filter_map(|cell| self.0.get(&cell))
            .flatten()
            .copied()
    }
}

/// Exact urban-surface queries accelerated by a coarse spatial lookup.
#[derive(Default, Debug, Clone)]
pub struct UrbanGroundLookup {
    streets: Vec<CityStreetPatch>,
    yards: Vec<CityYardPatch>,
    buildings: Vec<CityPlotBounds>,
    street_buckets: SpatialBuckets,
    yard_buckets: SpatialBuckets,
    building_buckets: SpatialBuckets,
}

impl UrbanGroundLookup {
    pub fn new(
        streets: &[CityStreetPatch],
        yards: &[CityYardPatch],
        buildings: &[CityPlotBounds],
    ) -> Self {
        Self {
            streets: streets.to_vec(),
            yards: yards.to_vec(),
            buildings: buildings.to_vec(),
            street_buckets: SpatialBuckets::new(streets.iter().map(street_bounds)),
            yard_buckets: SpatialBuckets::new(
                yards.iter().map(|yard| corners_bounds(yard.corners_metres)),
            ),
            building_buckets: SpatialBuckets::new(
                buildings
                    .iter()
                    .map(|building| corners_bounds(building.corners())),
            ),
        }
    }

    pub fn suppresses_grass(&self, point: Vec2) -> bool {
        self.street_buckets
            .at(point)
            .iter()
            .any(|&index| self.streets[index].contains(point))
            || self
                .yard_buckets
                .at(point)
                .iter()
                .any(|&index| self.yards[index].contains(point))
    }

    fn surface_at(&self, position: Vec2) -> Option<GroundSurface> {
        let substrate = if let Some(street) = self
            .street_buckets
            .at(position)
            .iter()
            .map(|&index| self.streets[index])
            .filter(|street| street.contains(position))
            .max_by_key(|street| street.surface().priority())
        {
            match street.surface() {
                CityStreetSurface::CompactedEarth => GroundSubstrate::Soil,
                CityStreetSurface::Gravel => GroundSubstrate::Gravel,
                CityStreetSurface::Fieldstone => GroundSubstrate::Road,
            }
        } else if self
            .building_buckets
            .at(position)
            .iter()
            .any(|&index| self.buildings[index].contains(position))
        {
            GroundSubstrate::Stone
        } else if self
            .yard_buckets
            .at(position)
            .iter()
            .any(|&index| self.yards[index].contains(position))
        {
            GroundSubstrate::Soil
        } else {
            return None;
        };
        Some(GroundSurface {
            substrate,
            cover: GroundCover::Bare,
            cover_density_bps: 0,
            cover_height_cm: 0,
        })
    }

    fn overlaps_disk(&self, centre: Vec2, radius_metres: f32) -> bool {
        let minimum = centre - Vec2::splat(radius_metres);
        let maximum = centre + Vec2::splat(radius_metres);
        self.street_buckets
            .overlapping(minimum, maximum)
            .any(|index| match self.streets[index] {
                CityStreetPatch::Corridor {
                    start_metres,
                    end_metres,
                    half_width_metres,
                    ..
                } => {
                    segment_distance(centre, start_metres, end_metres)
                        <= radius_metres + half_width_metres
                }
                CityStreetPatch::Market { corners_metres, .. } => {
                    self.streets[index].contains(centre)
                        || touches_edges(centre, radius_metres, corners_metres)
                }
            })
            || self
                .yard_buckets
                .overlapping(minimum, maximum)
                .any(|index| {
                    let yard = self.yards[index];
                    yard.contains(centre)
                        || touches_edges(centre, radius_metres, yard.corners_metres)
                })
            || self
                .building_buckets
                .overlapping(minimum, maximum)
                .any(|index| {
                    let bounds = self.buildings[index];
                    bounds.contains(centre)
                        || touches_edges(centre, radius_metres, bounds.corners())
                })
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Reflect, Clone)]
pub struct UrbanGroundSurfaces {
    streets: Vec<CityStreetPatch>,
    yards: Vec<CityYardPatch>,
    buildings: Vec<CityPlotBounds>,
    #[serde(skip)]
    #[reflect(ignore)]
    lookup: OnceLock<UrbanGroundLookup>,
}

impl PartialEq for UrbanGroundSurfaces {
    fn eq(&self, other: &Self) -> bool {
        self.streets == other.streets
            && self.yards == other.yards
            && self.buildings == other.buildings
    }
}

impl UrbanGroundSurfaces {
    pub(crate) fn new(
        streets: &[CityStreetPatch],
        yards: &[CityYardPatch],
        buildings: &[GeneratedBuilding],
    ) -> Self {
        let buildings = buildings
            .iter()
            .map(|building| CityPlotBounds {
                centre_metres: building.placement.centre_metres,
                dimensions_metres: building.collision.bounds.plan_half_extents() * 2.0,
                orientation: building.placement.orientation,
            })
            .collect::<Vec<_>>();
        let lookup = OnceLock::new();
        lookup
            .set(UrbanGroundLookup::new(streets, yards, &buildings))
            .expect("new urban ground lookup is empty");
        Self {
            streets: streets.to_vec(),
            yards: yards.to_vec(),
            buildings,
            lookup,
        }
    }

    pub fn surface_at(&self, position: Vec2) -> Option<GroundSurface> {
        self.lookup().surface_at(position)
    }

    /// Conservative exclusion for a rotating wild-scatter patch's full envelope.
    pub fn overlaps_disk(&self, centre: Vec2, radius_metres: f32) -> bool {
        self.lookup().overlaps_disk(centre, radius_metres)
    }

    fn lookup(&self) -> &UrbanGroundLookup {
        self.lookup
            .get_or_init(|| UrbanGroundLookup::new(&self.streets, &self.yards, &self.buildings))
    }

    #[cfg(test)]
    pub(super) fn garden_test() -> Self {
        Self {
            yards: vec![CityYardPatch {
                corners_metres: [
                    Vec2::new(0.3, 0.3),
                    Vec2::new(1.7, 0.3),
                    Vec2::new(1.7, 1.7),
                    Vec2::new(0.3, 1.7),
                ],
                surface: crate::city_layout::CityYardSurface::KitchenGarden,
            }],
            ..Default::default()
        }
    }
}

fn street_bounds(street: &CityStreetPatch) -> (Vec2, Vec2) {
    match *street {
        CityStreetPatch::Corridor {
            start_metres,
            end_metres,
            half_width_metres,
            ..
        } => (
            start_metres.min(end_metres) - Vec2::splat(half_width_metres),
            start_metres.max(end_metres) + Vec2::splat(half_width_metres),
        ),
        CityStreetPatch::Market { corners_metres, .. } => corners_bounds(corners_metres),
    }
}

fn corners_bounds(corners: [Vec2; 4]) -> (Vec2, Vec2) {
    corners.into_iter().fold(
        (Vec2::splat(f32::INFINITY), Vec2::splat(f32::NEG_INFINITY)),
        |(minimum, maximum), corner| (minimum.min(corner), maximum.max(corner)),
    )
}

fn segment_distance(point: Vec2, start: Vec2, end: Vec2) -> f32 {
    let edge = end - start;
    let fraction =
        ((point - start).dot(edge) / edge.length_squared().max(f32::EPSILON)).clamp(0.0, 1.0);
    point.distance(start + edge * fraction)
}
fn touches_edges(centre: Vec2, radius: f32, corners: [Vec2; 4]) -> bool {
    (0..4).any(|side| segment_distance(centre, corners[side], corners[(side + 1) % 4]) <= radius)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spatial_lookup_matches_exact_urban_ownership_across_bucket_boundaries() {
        let streets = [
            CityStreetPatch::Corridor {
                start_metres: Vec2::new(-45.0, 17.0),
                end_metres: Vec2::new(48.0, 17.0),
                half_width_metres: 3.0,
                surface: CityStreetSurface::CompactedEarth,
            },
            CityStreetPatch::Corridor {
                start_metres: Vec2::new(1.0, -12.0),
                end_metres: Vec2::new(1.0, 42.0),
                half_width_metres: 2.0,
                surface: CityStreetSurface::Fieldstone,
            },
        ];
        let yards = [CityYardPatch {
            corners_metres: [
                Vec2::new(31.0, -8.0),
                Vec2::new(39.0, -5.0),
                Vec2::new(36.0, 4.0),
                Vec2::new(28.0, 1.0),
            ],
            surface: crate::city_layout::CityYardSurface::PackedEarth,
        }];
        let lookup = UrbanGroundLookup::new(&streets, &yards, &[]);

        for z in -30..=50 {
            for x in -60..=60 {
                let point = Vec2::new(x as f32 + 0.37, z as f32 + 0.61);
                let expected_street = streets
                    .iter()
                    .filter(|street| street.contains(point))
                    .max_by_key(|street| street.surface().priority());
                let expected = expected_street
                    .map(|street| match street.surface() {
                        CityStreetSurface::CompactedEarth => GroundSubstrate::Soil,
                        CityStreetSurface::Gravel => GroundSubstrate::Gravel,
                        CityStreetSurface::Fieldstone => GroundSubstrate::Road,
                    })
                    .or_else(|| {
                        yards
                            .iter()
                            .any(|yard| yard.contains(point))
                            .then_some(GroundSubstrate::Soil)
                    });
                assert_eq!(
                    lookup.surface_at(point).map(|surface| surface.substrate),
                    expected,
                    "ownership differed at {point}"
                );
                assert_eq!(lookup.suppresses_grass(point), expected.is_some());
            }
        }
    }

    #[test]
    fn subcell_garden_ownership_survives_transport_without_clearing_neighbouring_grass() {
        let terrain = SceneTerrain::new(4, 4, 2.0, |_| 0.0);
        let mut ground = SceneGround::uniform_for_terrain(
            &terrain,
            GroundSurface {
                cover_density_bps: 9600,
                ..Default::default()
            },
        );
        ground.urban = UrbanGroundSurfaces::garden_test();
        assert!(ground.urban.overlaps_disk(Vec2::new(0.1, 1.0), 0.3));
        assert!(!ground.urban.overlaps_disk(Vec2::new(0.1, 1.0), 0.1));
        let transported: SceneGround =
            serde_json::from_slice(&serde_json::to_vec(&ground).unwrap()).unwrap();
        for point in [Vec2::new(0.31, 0.31), Vec2::new(1.69, 1.69)] {
            assert_eq!(transported.ground_at(point).unwrap().cover_density_bps, 0);
        }
        assert_eq!(
            transported
                .ground_at(Vec2::new(0.29, 0.31))
                .unwrap()
                .cover_density_bps,
            9600
        );
    }
}
