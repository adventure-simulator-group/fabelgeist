//! Fortified spiral flights, occupied floors and landing portals share resolved solids.
use crate::{Architectural, SpatialBounds};
use crate::{
    BuildingArchetype, BuildingPlan, GeometryOwnerId, ResolvedGeometry, ResolvedItemId,
    ResolvedSolid, ResolvedSolidShape, SolidRole, Stair, StructuralNodeId, StructuralNodeKind,
    SupportInterface,
};
use bevy::math::{Vec2, Vec3};

mod bearings;
mod flight;
mod floors;
mod keep_core;
mod landing_policy;
pub(crate) use keep_core::{reserved_cells as keep_reserved_cells, stair as keep_stair};
#[cfg(test)]
mod tests;
pub use flight::{
    SpiralFlight, SpiralLanding, SpiralMember, arrival_angle, compile_flight, required_treads,
};

const FLOOR_OWNER: GeometryOwnerId = GeometryOwnerId(97_000);
const FIRST_FLIGHT_OWNER: u32 = 97_001;
const FLIGHT_OWNER_LIMIT: u32 = 98_000;
const ITEM_DOMAIN: u64 = 11_u64 << 60;
const SUPPORT_DOMAIN: u64 = 12_u64 << 60;
const NODE_DOMAIN: u64 = 13_u64 << 60;
const BEARING_OVERLAP_METRES: f32 = 0.03;

/// Portals exist only when their actual resolved landing was generated.
pub fn landings(plan: &BuildingPlan, stair_index: usize) -> Vec<SpiralLanding> {
    let Some(stair) = plan.stairs.get(stair_index) else {
        return Vec::new();
    };
    let owner = GeometryOwnerId(FIRST_FLIGHT_OWNER + stair_index as u32);
    if !plan
        .resolved_geometry
        .solids
        .iter()
        .any(|s| s.owner == owner && s.role == SolidRole::Landing)
    {
        return Vec::new();
    }
    compile_flight(*stair, plan.storey_height_metres).map_or_else(Vec::new, |flight| {
        flight
            .landings
            .into_iter()
            .filter(|landing| landing_policy::has_floor(plan, landing))
            .collect()
    })
}

pub fn owns_landing(solid: &ResolvedSolid) -> bool {
    solid.role == SolidRole::Landing
        && (FIRST_FLIGHT_OWNER..FLIGHT_OWNER_LIMIT).contains(&solid.owner.0)
}

pub fn well_bounds(stair: Stair) -> Option<(Vec2, Vec2)> {
    let Stair::Spiral {
        centre,
        outer_radius_metres,
        ..
    } = stair
    else {
        return None;
    };
    let half = Vec2::splat(outer_radius_metres + flight::WELL_MARGIN_METRES);
    Some((centre - half, centre + half))
}

pub(crate) fn resolve(mut plan: BuildingPlan) -> Result<BuildingPlan, crate::GenerationError> {
    if !matches!(
        plan.archetype,
        BuildingArchetype::CastleGatehouse
            | BuildingArchetype::CourtyardCastle
            | BuildingArchetype::WalledKeep
            | BuildingArchetype::ArtilleryRondelCastle
    ) {
        return Ok(plan);
    }
    let mut flights = plan
        .stairs
        .iter()
        .enumerate()
        .filter_map(|(index, stair)| {
            compile_flight(*stair, plan.storey_height_metres).map(|flight| (index, flight))
        })
        .collect::<Vec<_>>();
    for (_, flight) in &mut flights {
        landing_policy::resolve(&plan, flight);
    }
    if keep_core::owns_occupied_storeys(plan.archetype) {
        floors::resolve(&mut plan, &flights)?;
    }
    for (index, flight) in flights {
        let owner = GeometryOwnerId(FIRST_FLIGHT_OWNER + index as u32);
        let bearing = foundation(&mut plan.resolved_geometry, owner, flight.centre)?;
        for (member_index, member) in flight.members.into_iter().enumerate() {
            append(
                &mut plan.resolved_geometry,
                owner,
                member_index,
                bearing,
                member,
            )?;
        }
    }
    bearings::resolve(&mut plan.resolved_geometry)?;
    Ok(plan)
}

fn foundation(
    geometry: &mut ResolvedGeometry,
    owner: GeometryOwnerId,
    centre: Vec2,
) -> Result<StructuralNodeId, crate::GenerationError> {
    let id = StructuralNodeId(NODE_DOMAIN | (u64::from(owner.0) << 32));
    geometry
        .structural_nodes
        .push(crate::StructuralNode::from_metres(
            id,
            owner,
            StructuralNodeKind::WallBearing,
            Vec3::new(centre.x, 0.0, centre.y),
            Vec::new(),
            true,
        )?);
    Ok(id)
}

fn append(
    geometry: &mut ResolvedGeometry,
    owner: GeometryOwnerId,
    index: usize,
    bearing: StructuralNodeId,
    member: SpiralMember,
) -> Result<(), crate::GenerationError> {
    let suffix = (u64::from(owner.0) << 32) | index as u64;
    geometry.solids.push(crate::ResolvedSolid::new(
        crate::CollisionCuboid::<crate::Architectural>::from_metres(
            ResolvedItemId(ITEM_DOMAIN | suffix),
            member.centre,
            member.size,
            member.yaw_radians,
            0.0,
            0.0,
        )?,
        owner,
        member.role,
        ResolvedSolidShape::Cuboid,
        vec![bearing],
    ));
    // The interface is the member's lower contact slab. Flight treads embed
    // into their continuous newel; floor panels share the masonry deck bearing.
    let rotation = bevy::math::Quat::from_rotation_y(member.yaw_radians);
    let half = (rotation * Vec3::X).abs() * member.size.x * 0.5
        + Vec3::Y * member.size.y * 0.5
        + (rotation * Vec3::Z).abs() * member.size.z * 0.5;
    let min = member.centre - half;
    geometry.support_interfaces.push(SupportInterface {
        id: ResolvedItemId(SUPPORT_DOMAIN | suffix),
        owner,
        node: bearing,
        bounds: SpatialBounds::<Architectural>::from_metres(
            min,
            Vec3::new(
                member.centre.x + half.x,
                min.y + BEARING_OVERLAP_METRES,
                member.centre.z + half.z,
            ),
        )?,
    });

    Ok(())
}
