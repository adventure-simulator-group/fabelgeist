//! Bind wall contacts using cached solid membership and projected bounds.
use std::collections::{HashMap, HashSet};

use bevy::math::Vec3;

use crate::{
    GeometryOwnerId, JunctionBond, ResolvedBounds, ResolvedGeometry, ResolvedItemId, ResolvedSolid,
    SolidRole, WallAssembly, geometry_index::BoundsIndex,
};

const CONTACT_DEPTH_METRES: f32 = 0.025;
const CONTACT_AREA_FRACTION: f32 = 0.90;
const PENETRATION_MARGIN_METRES: f32 = 0.005;
const WALL_BOND_ID_PREFIX: u64 = 8_u64 << 60;

pub(super) fn resolve(walls: &[WallAssembly], geometry: &mut ResolvedGeometry) {
    let solids = CornerSolids::new(&geometry.solids);
    let mut serial = 0;
    solids.bind_walls(walls, &mut geometry.junction_bonds, &mut serial);
    solids.bind_other_contacts(walls, &mut geometry.junction_bonds, &mut serial);
}

struct CornerSolids<'a> {
    solids: &'a [ResolvedSolid],
    bounds: Vec<ResolvedBounds>,
    by_id: HashMap<ResolvedItemId, usize>,
    by_owner: HashMap<GeometryOwnerId, Vec<usize>>,
    spatial: BoundsIndex,
}

impl<'a> CornerSolids<'a> {
    fn new(solids: &'a [ResolvedSolid]) -> Self {
        let mut by_id = HashMap::new();
        let mut by_owner: HashMap<_, Vec<_>> = HashMap::new();
        for (i, solid) in solids.iter().enumerate() {
            by_id.entry(solid.id).or_insert(i);
            by_owner.entry(solid.owner).or_default().push(i);
        }
        let bounds: Vec<_> = solids.iter().map(ResolvedSolid::yaw_bounds).collect();
        let spatial = BoundsIndex::new(bounds.iter().copied());
        Self {
            solids,
            bounds,
            by_id,
            by_owner,
            spatial,
        }
    }

    fn wall_solids(&self, wall: &WallAssembly) -> Vec<usize> {
        wall.replaced_by_owner.map_or_else(
            || {
                wall.host_solids
                    .iter()
                    .filter_map(|id| self.by_id.get(id).copied())
                    .collect()
            },
            |owner| self.by_owner.get(&owner).cloned().unwrap_or_default(),
        )
    }

    fn bind_walls(&self, walls: &[WallAssembly], bonds: &mut Vec<JunctionBond>, serial: &mut u64) {
        let membership: Vec<_> = walls.iter().map(|wall| self.wall_solids(wall)).collect();
        for (left_index, left) in walls.iter().enumerate() {
            for (right_index, right) in walls.iter().enumerate().skip(left_index + 1) {
                if left.storey_level != right.storey_level
                    || left.owner == right.owner
                    || left.frame.tangent.dot(right.frame.tangent).abs() > 0.01
                {
                    continue;
                }
                for &a in &membership[left_index] {
                    if !wall_contact_role(self.solids[a].role) {
                        continue;
                    }
                    for &b in &membership[right_index] {
                        if !wall_contact_role(self.solids[b].role) {
                            continue;
                        }
                        if let Some(overlap) = self.overlap(a, b) {
                            self.append(bonds, serial, a, b, overlap);
                        }
                    }
                }
            }
        }
    }

    fn bind_other_contacts(
        &self,
        walls: &[WallAssembly],
        bonds: &mut Vec<JunctionBond>,
        serial: &mut u64,
    ) {
        let owners: HashSet<_> = walls
            .iter()
            .flat_map(|wall| [wall.owner, wall.replaced_by_owner.unwrap_or(wall.owner)])
            .collect();
        for (a, left) in self.solids.iter().enumerate() {
            if !assembly_contact_role(left.role) {
                continue;
            }
            for b in self
                .spatial
                .overlapping(self.bounds[a])
                .into_iter()
                .filter(|&b| b > a)
            {
                let right = &self.solids[b];
                if left.owner == right.owner
                    || (!owners.contains(&left.owner) && !owners.contains(&right.owner))
                    || !assembly_contact_role(right.role)
                {
                    continue;
                }
                let Some(overlap) = self.overlap(a, b) else {
                    continue;
                };
                if bonds.iter().any(|bond| {
                    bond.owners.contains(&left.owner)
                        && bond.owners.contains(&right.owner)
                        && overlap
                            .min
                            .cmpge(bond.bounds.min - Vec3::splat(CONTACT_DEPTH_METRES))
                            .all()
                        && overlap
                            .max
                            .cmple(bond.bounds.max + Vec3::splat(CONTACT_DEPTH_METRES))
                            .all()
                }) {
                    continue;
                }
                self.append(bonds, serial, a, b, overlap);
            }
        }
    }

    fn overlap(&self, a: usize, b: usize) -> Option<ResolvedBounds> {
        let min = self.bounds[a].min.max(self.bounds[b].min);
        let max = self.bounds[a].max.min(self.bounds[b].max);
        ((max - min).min_element() > CONTACT_DEPTH_METRES).then_some(ResolvedBounds { min, max })
    }

    fn append(
        &self,
        bonds: &mut Vec<JunctionBond>,
        serial: &mut u64,
        a: usize,
        b: usize,
        bounds: ResolvedBounds,
    ) {
        let overlap = bounds.max - bounds.min;
        let mut extents = overlap.to_array();
        extents.sort_by(f32::total_cmp);
        bonds.push(JunctionBond {
            id: ResolvedItemId(WALL_BOND_ID_PREFIX | *serial),
            owners: [self.solids[a].owner, self.solids[b].owner],
            bounds,
            minimum_interface_area_square_metres: extents[1] * extents[2] * CONTACT_AREA_FRACTION,
            maximum_penetration_metres: overlap.x.min(overlap.z) + PENETRATION_MARGIN_METRES,
        });
        *serial += 1;
    }
}

fn wall_contact_role(role: SolidRole) -> bool {
    matches!(
        role,
        SolidRole::WallHost
            | SolidRole::DefenseHostWall
            | SolidRole::CircuitWalk
            | SolidRole::OpeningJamb
            | SolidRole::OpeningSill
            | SolidRole::OpeningHead
    )
}

fn assembly_contact_role(role: SolidRole) -> bool {
    wall_contact_role(role)
        || matches!(
            role,
            SolidRole::LoadBearing
                | SolidRole::Breastwork
                | SolidRole::WalkSurface
                | SolidRole::DrainageChannel
                | SolidRole::Landing
                | SolidRole::DefenseHostButtress
                | SolidRole::ProjectionSupport
                | SolidRole::GalleryFloor
                | SolidRole::OpeningSpandrel
        )
}
