//! Preserve the sampled firing contract while querying only nearby blockers.
use super::{
    ResolvedSolid, Result, SolidRole, Vec2, Vec3, VoidRole, resolved_solid_contains_point,
};
use crate::{Architectural, SpatialBounds};
use crate::{GeometryOwnerId, ResolvedItemId, geometry_index::BoundsIndex};

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
    pub(super) fn route_blocked(&self, point: Vec3, connectors: &[ResolvedItemId]) -> Result<bool> {
        Ok(self
            .spatial
            .overlapping(SpatialBounds::<Architectural>::from_metres(point, point)?)
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
            }))
    }

    pub(super) fn new(solids: &'a [ResolvedSolid]) -> Result<Self> {
        Ok(Self {
            solids,
            spatial: BoundsIndex::new(
                solids
                    .iter()
                    .map(ResolvedSolid::query_bounds)
                    .collect::<Result<Vec<_>>>()?,
            )?,
        })
    }

    pub(super) fn blocked(
        &self,
        origin: Vec3,
        target: Vec3,
        opening_owner: GeometryOwnerId,
    ) -> Result<bool> {
        let exit = (EXIT_DISTANCE_METRES / (target - origin).length())
            .clamp(MIN_EXIT_FRACTION, MAX_EXIT_FRACTION);
        crate::geometry_index::try_any(0..RAY_SAMPLE_COUNT, |sample| {
            let t = exit
                + (TARGET_APPROACH_FRACTION - exit) * sample as f32 / (RAY_SAMPLE_COUNT - 1) as f32;
            let point = origin.lerp(target, t);
            Result::<bool>::Ok(
                self.spatial
                    .overlapping(SpatialBounds::<Architectural>::from_metres(point, point)?)
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
                    }),
            )
        })
    }
}

impl ArtilleryClearance<'_> {
    pub(super) fn route_contract(
        &self,
        edge: &crate::ArtilleryRouteEdge,
        solids: &std::collections::HashMap<ResolvedItemId, &ResolvedSolid>,
        surfaces: &std::collections::HashMap<ResolvedItemId, &crate::ResolvedSurface>,
        voids: &std::collections::HashMap<ResolvedItemId, &crate::ResolvedVoid>,
        route_nodes: &std::collections::HashMap<
            crate::ArtilleryRouteNodeId,
            &crate::ArtilleryRouteNode,
        >,
    ) -> Result<bool> {
        let Some((from, to)) = route_nodes.get(&edge.from).zip(route_nodes.get(&edge.to)) else {
            return Ok(false);
        };
        let Some(surface) = edge
            .traversal_surface
            .and_then(|id| surfaces.get(&id).copied())
        else {
            return Ok(false);
        };
        let shape_valid = matches!(surface.shape, crate::ResolvedSurfaceShape::RouteCorridor { start, end, width_metres }
            if start.distance(from.position) <= 0.02 && end.distance(to.position) <= 0.02 && (width_metres-edge.width_metres).abs() <= 0.01);
        let connectors_valid = edge.connector_solids.iter().all(|id| {
            solids.get(id).is_some_and(|solid| {
                matches!(
                    solid.role,
                    SolidRole::ArtilleryRamp
                        | SolidRole::ArtilleryStairTread
                        | SolidRole::ArtilleryBridgeDeck
                )
            })
        });
        let portal_valid = edge.portal_void.is_none_or(|id| {
            voids.get(&id).is_some_and(|void| {
                matches!(
                    void.role,
                    VoidRole::Passage | VoidRole::AccessPortal | VoidRole::ArtilleryCasemate
                )
            })
        });
        let path_valid = edge.sweep_path.len() >= 2
            && edge
                .sweep_path
                .first()
                .is_some_and(|point| point.distance(from.position) <= 0.02)
            && edge
                .sweep_path
                .last()
                .is_some_and(|point| point.distance(to.position) <= 0.02)
            && edge
                .sweep_path
                .windows(2)
                .all(|pair| pair[0].distance(pair[1]) <= 3.0);
        let portal_crossed = edge.portal_void.is_none_or(|id| {
            voids.get(&id).is_some_and(|void| {
                edge.sweep_path.windows(2).any(|pair| {
                    (0..=8).any(|sample| {
                        let point = pair[0].lerp(pair[1], sample as f32 / 8.0) + Vec3::Y * 0.25;
                        point
                            .cmpge(void.bounds.min().metres() - Vec3::splat(0.02))
                            .all()
                            && point
                                .cmple(void.bounds.max().metres() + Vec3::splat(0.02))
                                .all()
                    })
                })
            })
        });
        let swept_clear = self.route_sweep_clear(edge)?;
        Result::<bool>::Ok(
            shape_valid
                && connectors_valid
                && portal_valid
                && path_valid
                && portal_crossed
                && swept_clear,
        )
    }
    fn route_sweep_clear(&self, edge: &crate::ArtilleryRouteEdge) -> Result<bool> {
        crate::geometry_index::try_all(edge.sweep_path.windows(2), |pair| {
            let delta = Vec2::new(pair[1].x - pair[0].x, pair[1].z - pair[0].z);
            let along = if delta.length() > 0.01 {
                delta.normalize()
            } else {
                Vec2::X
            };
            let across = Vec2::new(-along.y, along.x);
            let steps = ((pair[0].distance(pair[1]) / 0.35).ceil() as usize).max(1);
            crate::geometry_index::try_all(0..=steps, |step| {
                let foot = pair[0].lerp(pair[1], step as f32 / steps as f32);
                let samples = [-0.45_f32, 0.0, 0.45].into_iter().flat_map(|side| {
                    [0.25_f32, 1.0, 1.85].into_iter().map(move |height| {
                        foot + Vec3::new(across.x * side, height, across.y * side)
                    })
                });
                crate::geometry_index::try_all(samples, |point| {
                    Result::<bool>::Ok(!self.route_blocked(point, &edge.connector_solids)?)
                })
            })
        })
    }
}
