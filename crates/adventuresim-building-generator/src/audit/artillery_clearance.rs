//! Preserve the sampled firing contract while querying only nearby blockers.
use super::{ResolvedSolid, SolidRole, Vec3, resolved_solid_contains_point};
use crate::{GeometryOwnerId, ResolvedBounds, ResolvedItemId, geometry_index::BoundsIndex};

const RAY_SAMPLE_COUNT: usize = 24;
const EXIT_DISTANCE_METRES: f32 = 1.30;
const MIN_EXIT_FRACTION: f32 = 0.04;
const MAX_EXIT_FRACTION: f32 = 0.45;
const TARGET_APPROACH_FRACTION: f32 = 0.88;
const BLOCKER_INSET_METRES: f32 = 0.02;
const ROUTE_BLOCKER_INSET_METRES: f32 = 0.015;

pub(super) struct ArtilleryClearance<'a> {
    solids: &'a [ResolvedSolid],
    spatial: BoundsIndex,
}

impl<'a> ArtilleryClearance<'a> {
    pub(super) fn route_blocked(&self, point: Vec3, connectors: &[ResolvedItemId]) -> bool {
        self.spatial
            .overlapping(ResolvedBounds {
                min: point,
                max: point,
            })
            .into_iter()
            .any(|i| {
                let solid = &self.solids[i];
                let supporting = connectors.contains(&solid.id)
                    || matches!(
                        solid.role,
                        SolidRole::ArtilleryTerreplein
                            | SolidRole::ArtilleryCasemateFloor
                            | SolidRole::ArtilleryRamp
                            | SolidRole::ArtilleryStairTread
                            | SolidRole::ArtilleryBridgeDeck
                            | SolidRole::ArtilleryBridgeAbutment
                            | SolidRole::OpeningClosure
                            | SolidRole::DrainageFloor
                    );
                !supporting
                    && super::artillery_route_solid_contains(
                        solid,
                        point,
                        -ROUTE_BLOCKER_INSET_METRES,
                    )
            })
    }

    pub(super) fn new(solids: &'a [ResolvedSolid]) -> Self {
        Self {
            solids,
            spatial: BoundsIndex::new(solids.iter().map(ResolvedSolid::query_bounds)),
        }
    }

    pub(super) fn blocked(
        &self,
        origin: Vec3,
        target: Vec3,
        opening_owner: GeometryOwnerId,
    ) -> bool {
        let exit = (EXIT_DISTANCE_METRES / (target - origin).length())
            .clamp(MIN_EXIT_FRACTION, MAX_EXIT_FRACTION);
        (0..RAY_SAMPLE_COUNT).any(|sample| {
            let t = exit
                + (TARGET_APPROACH_FRACTION - exit) * sample as f32 / (RAY_SAMPLE_COUNT - 1) as f32;
            let point = origin.lerp(target, t);
            self.spatial
                .overlapping(ResolvedBounds {
                    min: point,
                    max: point,
                })
                .into_iter()
                .any(|i| {
                    let solid = &self.solids[i];
                    !matches!(
                        solid.role,
                        SolidRole::DitchFloor
                            | SolidRole::DitchScarp
                            | SolidRole::DitchCounterscarp
                            | SolidRole::DrainageFloor
                    ) && solid.owner != opening_owner
                        && resolved_solid_contains_point(solid, point, -BLOCKER_INSET_METRES)
                })
        })
    }
}
