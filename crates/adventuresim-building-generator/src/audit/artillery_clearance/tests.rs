use super::*;
use crate::{
    ArtilleryRouteEdge, ArtilleryRouteNode, ArtilleryRouteNodeId, GeometryOwnerId, ResolvedSurface,
    ResolvedSurfaceShape, SurfaceRole,
};
use std::collections::HashMap;

#[test]
fn route_assessments_distinguish_binding_geometry_and_obstruction_with_identity() {
    let from = ArtilleryRouteNode {
        id: ArtilleryRouteNodeId(23),
        surface: ResolvedItemId(203),
        position: Vec3::new(100.0, 0.0, 100.0),
    };
    let to = ArtilleryRouteNode {
        id: ArtilleryRouteNodeId(24),
        surface: ResolvedItemId(203),
        position: Vec3::new(102.0, 0.0, 100.0),
    };
    let surface = ResolvedSurface {
        id: from.surface,
        owner: GeometryOwnerId(7),
        bounds: SpatialBounds::from_metres(from.position, to.position).unwrap(),
        role: SurfaceRole::ArtilleryRoute,
        shape: ResolvedSurfaceShape::RouteCorridor {
            start: from.position,
            end: to.position,
            width_metres: 0.9,
        },
    };
    let mut edge = ArtilleryRouteEdge {
        from: from.id,
        to: to.id,
        width_metres: 0.9,
        headroom_metres: 1.9,
        portal_void: None,
        traversal_surface: Some(surface.id),
        connector_solids: vec![],
        sweep_path: vec![from.position, to.position],
    };
    let nodes = HashMap::from([(from.id, &from), (to.id, &to)]);
    let surfaces = HashMap::from([(surface.id, &surface)]);
    let empty = ArtilleryClearance::new(&[]).unwrap();
    let assess = |e: &ArtilleryRouteEdge, n: &HashMap<_, _>| {
        empty
            .route_contract(e, &HashMap::new(), &surfaces, &HashMap::new(), n)
            .unwrap()
    };
    assert_eq!(assess(&edge, &nodes).outcome, RouteOutcome::Clear);
    let missing = assess(&edge, &HashMap::from([(from.id, &from)]));
    assert_eq!(missing.route.from, from.id);
    assert_eq!(missing.route.to, to.id);
    assert_eq!(missing.outcome, RouteOutcome::MissingNode(to.id));
    edge.connector_solids.push(ResolvedItemId(501));
    assert_eq!(
        assess(&edge, &nodes).outcome,
        RouteOutcome::MissingConnector(ResolvedItemId(501))
    );
    edge.connector_solids.clear();
    edge.width_metres = f32::NAN;
    assert!(matches!(
        assess(&edge, &nodes).outcome,
        RouteOutcome::InvalidGeometry(_)
    ));
    edge.width_metres = 0.9;
    let plan = crate::generate(&crate::BuildingProgram::fixture(
        crate::BuildingArchetype::TownHouse,
        fabelgeist_determinism::Seed::from_u64(42),
    ))
    .unwrap();
    let mut blocker = plan.resolved_geometry.solids[0].clone();
    blocker.id = ResolvedItemId(502);
    blocker.role = SolidRole::FramePost;
    blocker.centre = Position::from_metres(Vec3::new(101.0, 1.0, 100.0)).unwrap();
    blocker.size =
        crate::spatial_geometry::CuboidDimensions::from_metres(Vec3::splat(2.0)).unwrap();
    let solids = [blocker];
    let visibility = ArtilleryClearance::new(&solids).unwrap();
    let blocked = visibility
        .route_contract(
            &edge,
            &HashMap::from([(solids[0].id, &solids[0])]),
            &surfaces,
            &HashMap::new(),
            &nodes,
        )
        .unwrap();
    assert_eq!(blocked.outcome, RouteOutcome::Blocked(ResolvedItemId(502)));
}
