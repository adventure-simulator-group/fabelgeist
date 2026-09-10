//! Market-local paired rows scale with usable area while preserving connected aisles.
use super::*;
use crate::city_layout::CityStreetPatch;
use adventuresim_building_generator::furniture::FurnitureKind;

const CUSTOMER_AISLE_METRES: f32 = 2.0;
const ROW_SEPARATION_METRES: f32 = 0.25;
const STALL_SEPARATION_METRES: f32 = 0.25;
const BOUNDARY_MARGIN_METRES: f32 = 0.15;
const MAX_MARKET_ROWS_PER_HALF: usize = 128;
const MAX_ROW_CANDIDATES: usize = 512;
const ROW_ID_SHIFT: u32 = 32;

pub(in crate::scene_input::furniture) struct MarketCandidates {
    pub groups: Vec<Candidate>,
    pub aisles: Vec<FurnitureFootprint>,
}

struct MarketFrame {
    centre: Vec2,
    orientation: BuildingOrientation,
    minimum: Vec2,
    maximum: Vec2,
}

impl MarketFrame {
    fn new(input: &TacticalSceneInput, corners: [Vec2; 4]) -> Option<Self> {
        let centre = corners.into_iter().sum::<Vec2>() * 0.25;
        let orientation = BuildingOrientation::from_frontage_tangent(corners[1] - corners[0])?;
        let mut minimum = Vec2::splat(f32::INFINITY);
        let mut maximum = Vec2::splat(f32::NEG_INFINITY);
        for corner in corners {
            let point = orientation.world_to_local(corner - centre);
            minimum = minimum.min(point);
            maximum = maximum.max(point);
        }
        let mut lower_inset = Vec2::ZERO;
        let mut upper_inset = Vec2::ZERO;
        for edge in 0..4 {
            let start = corners[edge];
            let end = corners[(edge + 1) % 4];
            let tangent = (end - start).normalize_or_zero();
            let mut inward = Vec2::new(-tangent.y, tangent.x);
            if inward.dot(centre - (start + end) * 0.5) < 0.0 {
                inward = -inward;
            }
            let clearance = reservations::market_edge_clearance(input, start, end, inward)
                + BOUNDARY_MARGIN_METRES;
            let local_inward = orientation.world_to_local(inward);
            let axis = if local_inward.x.abs() > local_inward.y.abs() {
                0
            } else {
                1
            };
            if local_inward[axis] > 0.0 {
                lower_inset[axis] = lower_inset[axis].max(clearance);
            } else {
                upper_inset[axis] = upper_inset[axis].max(clearance);
            }
        }
        Some(Self {
            centre,
            orientation,
            minimum: minimum + lower_inset,
            maximum: maximum - upper_inset,
        })
    }

    fn point(&self, local: Vec2) -> Vec2 {
        self.centre + self.orientation.local_to_world(local)
    }
}

pub(in crate::scene_input::furniture) fn market(input: &TacticalSceneInput) -> MarketCandidates {
    let mut result = MarketCandidates {
        groups: Vec::new(),
        aisles: Vec::new(),
    };
    let row_depth = FurnitureVariant::ALL
        .into_iter()
        .map(|variant| {
            let (min, max) = reservation_bounds(FurnitureKey {
                kind: FurnitureKind::CanvasStall,
                variant,
            });
            max.y - min.y
        })
        .fold(0.0_f32, f32::max);
    for (index, patch) in input.streets.iter().enumerate() {
        let CityStreetPatch::Market { corners_metres, .. } = *patch else {
            continue;
        };
        let Some(frame) = MarketFrame::new(input, corners_metres) else {
            continue;
        };
        for (half, direction) in [-1.0, 1.0].into_iter().enumerate() {
            let limit = if direction < 0.0 {
                -frame.minimum.y
            } else {
                frame.maximum.y
            };
            let mut cursor = reservations::MARKET_AISLE_HALF_WIDTH_METRES + BOUNDARY_MARGIN_METRES;
            for row in 0..MAX_MARKET_ROWS_PER_HALF {
                if cursor + row_depth > limit {
                    break;
                }
                let y = direction * (cursor + row_depth * 0.5);
                MarketRow {
                    patch_index: index,
                    patch: *patch,
                    y,
                    row: half * MAX_MARKET_ROWS_PER_HALF + row,
                    direction,
                }
                .append(input, &frame, &mut result.groups);
                cursor += row_depth;
                if row.is_multiple_of(2) {
                    cursor += ROW_SEPARATION_METRES;
                } else {
                    cursor += BOUNDARY_MARGIN_METRES;
                    let aisle_y = direction * (cursor + CUSTOMER_AISLE_METRES * 0.5);
                    result.aisles.push(reservations::route(
                        frame.point(Vec2::new(frame.minimum.x, aisle_y)),
                        frame.point(Vec2::new(frame.maximum.x, aisle_y)),
                        CUSTOMER_AISLE_METRES * 0.5,
                    ));
                    cursor += CUSTOMER_AISLE_METRES + BOUNDARY_MARGIN_METRES;
                }
            }
        }
    }
    result
}

struct MarketRow {
    patch_index: usize,
    patch: CityStreetPatch,
    y: f32,
    row: usize,
    direction: f32,
}
impl MarketRow {
    fn append(
        &self,
        input: &TacticalSceneInput,
        frame: &MarketFrame,
        candidates: &mut Vec<Candidate>,
    ) {
        let Self {
            patch_index,
            patch,
            y,
            row,
            direction,
        } = *self;
        let facing_centre = row.is_multiple_of(2);
        let reversed = (direction < 0.0) == facing_centre;
        let orientation = if reversed {
            BuildingOrientation::from_radians(
                frame.orientation.yaw_radians() + std::f32::consts::PI,
            )
            .unwrap()
        } else {
            frame.orientation
        };
        let mut x = frame.minimum.x;
        for column in 0..MAX_ROW_CANDIDATES {
            let slot = ((row as u64) << ROW_ID_SHIFT) | column as u64;
            let mut candidate = Candidate::new(
                input.seed,
                slot,
                FurnitureGroupKind::Vendor,
                FurnitureAnchor::Market {
                    patch_index: patch_index as u32,
                },
            );
            let half_width = candidate.footprint.half_extents_metres.x;
            // Restart beyond the central through-route instead of wasting a crossing slot.
            let crossing = reservations::MARKET_AISLE_HALF_WIDTH_METRES + BOUNDARY_MARGIN_METRES;
            if x < crossing && x + half_width * 2.0 > -crossing {
                x = crossing;
            }
            if x + half_width * 2.0 > frame.maximum.x {
                break;
            }
            candidate.footprint.centre_metres = frame.point(Vec2::new(x + half_width, y));
            candidate.footprint.orientation = orientation;
            candidate.market = Some(patch);
            x += half_width * 2.0 + STALL_SEPARATION_METRES;
            candidates.push(candidate);
        }
    }
}
