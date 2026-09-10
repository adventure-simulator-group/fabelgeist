//! Declare bounded deck/wall and tread/landing contacts, never stair obstructions.
use super::*;

const BOND_DOMAIN: u64 = 14_u64 << 60;
const BOND_TOLERANCE_METRES: f32 = 0.005;

pub(super) fn resolve(geometry: &mut ResolvedGeometry) {
    let solids = &geometry.solids;
    for (index, a) in solids.iter().enumerate() {
        for b in &solids[index + 1..] {
            if a.owner == b.owner || !bearing_pair(a, b) {
                continue;
            }
            let (amin, amax) = bounds(a);
            let (bmin, bmax) = bounds(b);
            let min = amin.max(bmin);
            let max = amax.min(bmax);
            let overlap = max - min;
            if overlap.min_element() <= 0.0
                || overlap.y > flight::DECK_THICKNESS_METRES + BOND_TOLERANCE_METRES
            {
                continue;
            }
            let mut axes = [overlap.x, overlap.y, overlap.z];
            axes.sort_by(f32::total_cmp);
            geometry.junction_bonds.push(crate::JunctionBond {
                id: ResolvedItemId(BOND_DOMAIN | geometry.junction_bonds.len() as u64),
                owners: [a.owner, b.owner],
                bounds: ResolvedBounds { min, max },
                minimum_interface_area_square_metres: axes[1] * axes[2] * 0.9,
                maximum_penetration_metres: overlap.x.min(overlap.z) + BOND_TOLERANCE_METRES,
            });
        }
    }
}

fn bearing_pair(a: &ResolvedSolid, b: &ResolvedSolid) -> bool {
    [(a, b), (b, a)].into_iter().any(|(deck, host)| {
        ((deck.role == SolidRole::InteriorFloor || owns_landing(deck))
            && matches!(
                host.role,
                SolidRole::WallHost
                    | SolidRole::DefenseHostWall
                    | SolidRole::OpeningJamb
                    | SolidRole::OpeningHead
                    | SolidRole::OpeningSpandrel
                    | SolidRole::OpeningSill
                    | SolidRole::Landing
            ))
            || (matches!(deck.role, SolidRole::StairTread | SolidRole::Landing)
                && deck.owner.0 >= FIRST_FLIGHT_OWNER
                && matches!(
                    host.role,
                    SolidRole::WalkSurface
                        | SolidRole::Landing
                        | SolidRole::CircuitWalk
                        | SolidRole::GalleryFloor
                ))
    })
}

fn bounds(solid: &ResolvedSolid) -> (Vec3, Vec3) {
    let q = bevy::math::Quat::from_rotation_y(solid.yaw_radians);
    let half = (q * Vec3::X).abs() * solid.size.x * 0.5
        + Vec3::Y * solid.size.y * 0.5
        + (q * Vec3::Z).abs() * solid.size.z * 0.5;
    (solid.centre - half, solid.centre + half)
}
