//! Occupied floor bridges and auxiliary tower crown arrivals are distinct destinations.
use super::*;

pub(super) fn has_floor(plan: &BuildingPlan, landing: &SpiralLanding) -> bool {
    if !keep_core::owns_occupied_storeys(plan.archetype) {
        return false;
    }
    plan.storeys.iter().any(|storey| {
        storey.level == landing.storey
            && storey
                .rooms
                .iter()
                .flat_map(|room| &room.cells)
                .any(|cell| {
                    (landing.position_metres - cell.centre())
                        .abs()
                        .max_element()
                        <= crate::CELL_SIZE_METRES * 0.5
                })
    })
}

pub(super) fn resolve(plan: &BuildingPlan, flight: &mut SpiralFlight) {
    let keep = flight
        .landings
        .iter()
        .filter(|landing| has_floor(plan, landing))
        .map(|landing| landing.elevation_metres)
        .collect::<Vec<_>>();
    flight.members.retain(|member| {
        member.role != SolidRole::Landing
            || keep
                .iter()
                .any(|height| (member.centre.y + member.size.y * 0.5 - height).abs() < 0.001)
    });
    flight.landings.retain(|landing| has_floor(plan, landing));
    if flight.landings.is_empty() {
        // Auxiliary tower flights terminate on the crown instead of inventing
        // intermediate floors through the hollow tower shell.
        let Some(last) = flight
            .members
            .iter()
            .rev()
            .find(|member| member.role == SolidRole::StairTread)
        else {
            return;
        };
        let radial = bevy::math::Quat::from_rotation_y(last.yaw_radians) * Vec3::X;
        let length = 0.45;
        let centre = Vec3::new(
            flight.centre.x,
            flight.top_height_metres - flight::DECK_THICKNESS_METRES * 0.5,
            flight.centre.y,
        ) + radial * (flight.outer_radius_metres + length * 0.5 - 0.12);
        flight.members.push(SpiralMember {
            centre,
            size: Vec3::new(length, flight::DECK_THICKNESS_METRES, 0.9),
            yaw_radians: last.yaw_radians,
            role: SolidRole::Landing,
        });
    }
}
