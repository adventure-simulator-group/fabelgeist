//! Original frontage constraints accompany the immutable selected property IDs.
use super::*;

#[derive(Clone, Debug, Default, PartialEq)]
pub(in crate::city_layout) struct CityPackingContext {
    pub(in crate::city_layout::packing) frontages: BTreeMap<CityPropertyId, ParcelFrontage>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ParcelFrontage {
    pub lot: CityBuildingLot,
    pub block: CityBlock,
    pub edge: usize,
}

impl CityPackingContext {
    pub fn from_selected(selected: &[CandidateLot], blocks: &[CityBlock]) -> Self {
        let blocks: BTreeMap<_, _> = blocks.iter().map(|b| (b.id, *b)).collect();
        Self {
            frontages: selected
                .iter()
                .map(|candidate| {
                    let lot = candidate.lot;
                    let block = blocks[&candidate.block_key];
                    let edge = (0..4)
                        .max_by(|&a, &b| {
                            let alignment = |i: usize| {
                                lot.orientation.local_to_world(Vec2::X).dot(
                                    (block.corners[(i + 1) % 4] - block.corners[i]).normalize(),
                                )
                            };
                            alignment(a).total_cmp(&alignment(b)).then(b.cmp(&a))
                        })
                        .expect("a block has four edges");
                    (CityPropertyId(lot.id), ParcelFrontage { lot, block, edge })
                })
                .collect(),
        }
    }
}

/// Allocation roles in deliberate precedence order, separate from edge ordinals.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum FrontagePriority {
    ParishChurch,
    Service,
    Residence,
}

impl ParcelFrontage {
    pub fn priority(self) -> FrontagePriority {
        use adventuresim_world_schema::settlement_buildings::ParishBuildingRole;
        match self.lot.service {
            Some(BuildingDemand::Parish {
                role: ParishBuildingRole::Church(_),
                ..
            }) => FrontagePriority::ParishChurch,
            Some(_) => FrontagePriority::Service,
            None => FrontagePriority::Residence,
        }
    }

    pub fn tangent(self) -> Vec2 {
        (self.block.corners[(self.edge + 1) % 4] - self.block.corners[self.edge]).normalize()
    }

    pub fn geometry(
        self,
        layout: &CompiledCityLayout,
        envelopes: &BTreeMap<u64, MeasuredBuildingEnvelope>,
    ) -> Result<ParcelGeometry, CityCompileError> {
        let id = CityPropertyId(self.lot.id);
        let reservation = CityPlotBounds::from(plots::reservation(self.lot));
        let front = envelopes
            .get(&self.lot.id)
            .ok_or(CityCompileError::Packing {
                property: id,
                issue: CityPackingIssue::MissingFrontage,
            })?;
        let mut buildings = vec![front.body];
        if let Some(compound) = layout.compounds.iter().find(|p| p.id == id) {
            buildings.push(envelopes[&compound.rear_building_id].body);
        }
        // Compound courts and enclosures own their complete support surface.
        // A single property's untouched rear garden is not a level foundation.
        let bearings = if layout.compounds.iter().any(|property| property.id == id) {
            vec![reservation.corners().to_vec()]
        } else {
            vec![front.bearing_outline.clone()]
        };
        Ok(ParcelGeometry {
            reservation,
            buildings,
            bearings,
            garden: layout
                .gardens
                .iter()
                .find(|garden| garden.owner == id)
                .cloned(),
        })
    }

    pub fn available_displacement(self, envelope: CityPlotBounds) -> Option<FrontageInterval> {
        let tangent = self.tangent().as_dvec2();
        let mut interval = FrontageInterval {
            minimum_metres: f64::NEG_INFINITY,
            maximum_metres: f64::INFINITY,
        };
        for edge in 0..4 {
            let start = self.block.corners[edge].as_dvec2();
            let normal = (self.block.corners[(edge + 1) % 4] - self.block.corners[edge])
                .normalize()
                .perp()
                .as_dvec2();
            for corner in envelope.corners() {
                let origin = normal.dot(corner.as_dvec2() - start)
                    - f64::from(self.block.streets[edge].half_width())
                    + f64::from(plots::STREET_EDGE_TOLERANCE_METRES);
                interval = interval.with_half_plane(origin, normal.dot(tangent))?;
            }
        }
        (interval.minimum_metres.is_finite() && interval.maximum_metres.is_finite())
            .then_some(interval)
    }
}

/// Ground ownership and elevated building envelopes are different constraints.
/// Only the private reservation bounds a block fit. Empty corners between a
/// roof projection and its garden are not occupied building volume.
#[derive(Clone, Debug)]
pub(super) struct ParcelGeometry {
    pub reservation: CityPlotBounds,
    pub buildings: Vec<CityPlotBounds>,
    pub bearings: Vec<Vec<Vec2>>,
    pub garden: Option<gardens::CityGarden>,
}
impl ParcelGeometry {
    pub fn translated(&self, delta: Vec2, tangent: Vec2) -> Self {
        let mut geometry = self.clone();
        geometry.reservation.centre_metres += delta;
        for body in &mut geometry.buildings {
            body.centre_metres += delta;
        }
        for bearing in &mut geometry.bearings {
            for point in bearing {
                *point += delta;
            }
        }
        if let Some(garden) = &mut geometry.garden {
            garden.translate(delta, tangent);
        }
        geometry
    }

    pub fn clears(&self, other: &Self) -> bool {
        self.forbidden_displacements(other, Vec2::X, DVec2::ZERO)
            .iter()
            .all(|range| range.minimum_metres >= 0.0 || range.maximum_metres <= 0.0)
            && self.garden_clears(other)
            && other.garden_clears(self)
    }

    pub fn garden_clears(&self, other: &Self) -> bool {
        self.garden.as_ref().is_none_or(|garden| {
            other
                .buildings
                .iter()
                .all(|body| garden.clears_building(*body))
        })
    }

    pub fn forbidden_displacements(
        &self,
        other: &Self,
        tangent: Vec2,
        other_translation_metres: DVec2,
    ) -> Vec<FrontageInterval> {
        self.conflicts(other, tangent, other_translation_metres)
            .into_iter()
            .flatten()
            .collect()
    }

    /// Every returned slot names the same geometric pair, including a pair
    /// that cannot intersect at the specified translation. A pair's overlap
    /// region is convex, so intersecting its two endpoint intervals identifies
    /// translations blocked for the entire intervening neighbour domain.
    pub fn conflicts(
        &self,
        other: &Self,
        tangent: Vec2,
        other_translation_metres: DVec2,
    ) -> Vec<Option<FrontageInterval>> {
        std::iter::once((
            self.reservation,
            other.reservation,
            intervals::PackingClearance::PropertyBoundary,
        ))
        .chain(self.buildings.iter().flat_map(|a| {
            other
                .buildings
                .iter()
                .map(move |b| (*a, *b, intervals::PackingClearance::BuildingBody))
        }))
        .map(|(a, b, clearance)| {
            FrontageInterval::overlap_displacements(
                a,
                b,
                tangent,
                clearance,
                other_translation_metres,
            )
        })
        .chain(self.bearings.iter().flat_map(|first| {
            other.bearings.iter().map(move |second| {
                FrontageInterval::overlap_polygons(
                    first,
                    second,
                    tangent,
                    intervals::PackingClearance::PropertyBoundary,
                    other_translation_metres,
                )
            })
        }))
        .collect()
    }
}
