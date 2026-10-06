//! Preserve the sampled firing contract while querying only nearby blockers.
use super::{
    ResolvedSolid, Result, SolidRole, Vec2, Vec3, VoidRole, resolved_solid_contains_point,
};
use crate::spatial_geometry::{GeometryError, GeometryResult, Position, SignedLength};
use crate::{Architectural, SpatialBounds};

#[derive(Clone, Copy, Debug)]
pub(super) struct RouteIdentity {
    pub from: crate::ArtilleryRouteNodeId,
    pub to: crate::ArtilleryRouteNodeId,
}
#[derive(Clone, Debug)]
pub(super) struct RouteAssessment {
    pub route: RouteIdentity,
    pub outcome: RouteOutcome,
}
#[derive(Clone, Debug, PartialEq)]
pub(super) enum RouteOutcome {
    Clear,
    MissingNode(crate::ArtilleryRouteNodeId),
    MissingSurface(Option<ResolvedItemId>),
    MissingConnector(ResolvedItemId),
    MissingPortal(ResolvedItemId),
    InvalidGeometry(GeometryError),
    InvalidShape(ResolvedItemId),
    InvalidConnector(ResolvedItemId),
    InvalidPortal(ResolvedItemId),
    InvalidPath,
    PortalNotCrossed(ResolvedItemId),
    Blocked(ResolvedItemId),
}
impl RouteAssessment {
    pub fn is_clear(&self) -> bool {
        self.outcome == RouteOutcome::Clear
    }
}
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
    fn route_blocker(
        &self,
        point: Position<Architectural>,
        connectors: &[ResolvedItemId],
    ) -> Result<Option<ResolvedItemId>> {
        let inset = SignedLength::from_metres(-ROUTE_BLOCKER_INSET_METRES)?;
        Ok(self
            .spatial
            .overlapping(SpatialBounds::new(point, point)?)
            .into_iter()
            .find_map(|i| {
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
                (!supporting
                    && super::artillery_route_solid_contains(solid, point.metres(), inset.metres()))
                .then_some(solid.id)
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
        origin: Position<Architectural>,
        target: Position<Architectural>,
        opening_owner: GeometryOwnerId,
    ) -> Result<bool> {
        // Ray interpolation and shape containment are one bounded native kernel.
        let origin = origin.metres();
        let target = target.metres();
        let inset = SignedLength::from_metres(-BLOCKER_INSET_METRES)?;
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
                            && resolved_solid_contains_point(solid, point, inset.metres())
                    }),
            )
        })
    }
}

impl ArtilleryClearance<'_> {
    pub(super) fn first_route_failure(
        &self,
        edges: &[crate::ArtilleryRouteEdge],
        solids: &std::collections::HashMap<ResolvedItemId, &ResolvedSolid>,
        surfaces: &std::collections::HashMap<ResolvedItemId, &crate::ResolvedSurface>,
        voids: &std::collections::HashMap<ResolvedItemId, &crate::ResolvedVoid>,
        nodes: &std::collections::HashMap<crate::ArtilleryRouteNodeId, &crate::ArtilleryRouteNode>,
    ) -> Result<Option<RouteAssessment>> {
        for edge in edges {
            let assessment = self.route_contract(edge, solids, surfaces, voids, nodes)?;
            if !assessment.is_clear() {
                return Ok(Some(assessment));
            }
        }
        Ok(None)
    }
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
    ) -> Result<RouteAssessment> {
        let route = RouteIdentity {
            from: edge.from,
            to: edge.to,
        };
        let assess = |outcome| RouteAssessment { route, outcome };
        for node in [edge.from, edge.to] {
            if !route_nodes.contains_key(&node) {
                return Ok(assess(RouteOutcome::MissingNode(node)));
            }
        }
        let Some((from, to)) = route_nodes.get(&edge.from).zip(route_nodes.get(&edge.to)) else {
            return Ok(assess(RouteOutcome::MissingNode(edge.from)));
        };
        let Some(surface) = edge
            .traversal_surface
            .and_then(|id| surfaces.get(&id).copied())
        else {
            return Ok(assess(RouteOutcome::MissingSurface(edge.traversal_surface)));
        };
        let admission = || {
            Position::<Architectural>::from_metres(from.position)?;
            Position::<Architectural>::from_metres(to.position)?;
            crate::spatial_geometry::PositiveLength::from_metres(edge.width_metres)?;
            crate::spatial_geometry::PositiveLength::from_metres(edge.headroom_metres)?;
            edge.sweep_path
                .iter()
                .copied()
                .map(Position::<Architectural>::from_metres)
                .collect::<GeometryResult<Vec<_>>>()
        };
        let sweep = match admission() {
            Ok(sweep) => sweep,
            Err(cause) => return Ok(assess(RouteOutcome::InvalidGeometry(cause))),
        };
        if let Some(outcome) = route_bindings(edge, solids, voids) {
            return Ok(assess(outcome));
        }
        let shape_valid = matches!(surface.shape, crate::ResolvedSurfaceShape::RouteCorridor { start, end, width_metres }
            if start.distance(from.position) <= 0.02 && end.distance(to.position) <= 0.02 && (width_metres-edge.width_metres).abs() <= 0.01);
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
        if !shape_valid {
            return Ok(assess(RouteOutcome::InvalidShape(surface.id)));
        }
        if !path_valid {
            return Ok(assess(RouteOutcome::InvalidPath));
        }
        if !portal_crossed && let Some(id) = edge.portal_void {
            return Ok(assess(RouteOutcome::PortalNotCrossed(id)));
        }
        Ok(assess(match self.route_sweep_blocker(edge, &sweep)? {
            Some(id) => RouteOutcome::Blocked(id),
            None => RouteOutcome::Clear,
        }))
    }
    fn route_sweep_blocker(
        &self,
        edge: &crate::ArtilleryRouteEdge,
        sweep: &[Position<Architectural>],
    ) -> Result<Option<ResolvedItemId>> {
        for pair in sweep.windows(2) {
            let pair = [pair[0].metres(), pair[1].metres()];
            let delta = Vec2::new(pair[1].x - pair[0].x, pair[1].z - pair[0].z);
            let along = if delta.length() > 0.01 {
                delta.normalize()
            } else {
                Vec2::X
            };
            let across = Vec2::new(-along.y, along.x);
            let steps = ((pair[0].distance(pair[1]) / 0.35).ceil() as usize).max(1);
            for step in 0..=steps {
                let foot = pair[0].lerp(pair[1], step as f32 / steps as f32);
                let samples = [-0.45_f32, 0.0, 0.45].into_iter().flat_map(|side| {
                    [0.25_f32, 1.0, 1.85].into_iter().map(move |height| {
                        foot + Vec3::new(across.x * side, height, across.y * side)
                    })
                });
                for point in samples {
                    if let Some(id) =
                        self.route_blocker(Position::from_metres(point)?, &edge.connector_solids)?
                    {
                        return Ok(Some(id));
                    }
                }
            }
        }
        Ok(None)
    }
}

fn route_bindings(
    edge: &crate::ArtilleryRouteEdge,
    solids: &std::collections::HashMap<ResolvedItemId, &ResolvedSolid>,
    voids: &std::collections::HashMap<ResolvedItemId, &crate::ResolvedVoid>,
) -> Option<RouteOutcome> {
    for &id in &edge.connector_solids {
        let Some(solid) = solids.get(&id) else {
            return Some(RouteOutcome::MissingConnector(id));
        };
        if !matches!(
            solid.role,
            SolidRole::ArtilleryRamp
                | SolidRole::ArtilleryStairTread
                | SolidRole::ArtilleryBridgeDeck
        ) {
            return Some(RouteOutcome::InvalidConnector(id));
        }
    }
    if let Some(id) = edge.portal_void {
        let Some(portal) = voids.get(&id) else {
            return Some(RouteOutcome::MissingPortal(id));
        };
        if !matches!(
            portal.role,
            VoidRole::Passage | VoidRole::AccessPortal | VoidRole::ArtilleryCasemate
        ) {
            return Some(RouteOutcome::InvalidPortal(id));
        }
    }
    None
}

#[cfg(test)]
mod tests;
