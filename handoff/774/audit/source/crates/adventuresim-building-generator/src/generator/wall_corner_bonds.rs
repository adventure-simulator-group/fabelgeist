//! Bind wall contacts using cached solid membership and projected bounds.
use crate::{Architectural, SpatialBounds};
use std::collections::{HashMap, HashSet};

use bevy::math::Vec3;

use crate::{
    GeometryOwnerId, JunctionBond, ResolvedGeometry, ResolvedItemId, ResolvedSolid, SolidRole,
    WallAssembly, geometry_index::BoundsIndex,
};

const CONTACT_DEPTH_METRES: f32 = 0.025;
const CONTACT_AREA_FRACTION: f32 = 0.90;
const PENETRATION_MARGIN_METRES: f32 = 0.005;
const WALL_BOND_ID_PREFIX: u64 = 8_u64 << 60;

pub(super) fn resolve(
    walls: &[WallAssembly],
    geometry: &mut ResolvedGeometry,
) -> Result<(), crate::GenerationError> {
    let solids = CornerSolids::new(&geometry.solids)?;
    let mut serial = 0;
    solids.bind_walls(walls, &mut geometry.junction_bonds, &mut serial)?;
    solids.bind_other_contacts(walls, &mut geometry.junction_bonds, &mut serial)?;

    Ok(())
}

struct CornerSolids<'a> {
    solids: &'a [ResolvedSolid],
    bounds: Vec<SpatialBounds<Architectural>>,
    by_id: HashMap<ResolvedItemId, usize>,
    by_owner: HashMap<GeometryOwnerId, Vec<usize>>,
    spatial: BoundsIndex,
}

impl<'a> CornerSolids<'a> {
    fn new(solids: &'a [ResolvedSolid]) -> Result<Self, crate::GenerationError> {
        let mut by_id = HashMap::new();
        let mut by_owner: HashMap<_, Vec<_>> = HashMap::new();
        for (i, solid) in solids.iter().enumerate() {
            by_id.entry(solid.id).or_insert(i);
            by_owner.entry(solid.owner).or_default().push(i);
        }
        let bounds = solids
            .iter()
            .map(ResolvedSolid::yaw_bounds)
            .collect::<Result<Vec<_>, _>>()?;
        let spatial = BoundsIndex::new(bounds.iter().copied())?;
        Ok(Self {
            solids,
            bounds,
            by_id,
            by_owner,
            spatial,
        })
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

    fn bind_walls(
        &self,
        walls: &[WallAssembly],
        bonds: &mut Vec<JunctionBond>,
        serial: &mut u64,
    ) -> Result<(), crate::GenerationError> {
        let membership: Vec<_> = walls.iter().map(|wall| self.wall_solids(wall)).collect();
        let _: () = for (left_index, left) in walls.iter().enumerate() {
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
                        if let Some(overlap) = self.overlap(a, b)? {
                            self.append(bonds, serial, a, b, overlap);
                        }
                    }
                }
            }
        };
        Ok(())
    }

    fn bind_other_contacts(
        &self,
        walls: &[WallAssembly],
        bonds: &mut Vec<JunctionBond>,
        serial: &mut u64,
    ) -> Result<(), crate::GenerationError> {
        let owners: HashSet<_> = walls
            .iter()
            .flat_map(|wall| [wall.owner, wall.replaced_by_owner.unwrap_or(wall.owner)])
            .collect();
        let _: () = for (a, left) in self.solids.iter().enumerate() {
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
                let Some(overlap) = self.overlap(a, b)? else {
                    continue;
                };
                if bonds.iter().any(|bond| {
                    bond.owners.contains(&left.owner)
                        && bond.owners.contains(&right.owner)
                        && overlap
                            .min()
                            .metres()
                            .cmpge(bond.bounds.min().metres() - Vec3::splat(CONTACT_DEPTH_METRES))
                            .all()
                        && overlap
                            .max()
                            .metres()
                            .cmple(bond.bounds.max().metres() + Vec3::splat(CONTACT_DEPTH_METRES))
                            .all()
                }) {
                    continue;
                }
                self.append(bonds, serial, a, b, overlap);
            }
        };
        Ok(())
    }

    fn overlap(
        &self,
        a: usize,
        b: usize,
    ) -> Result<Option<SpatialBounds<Architectural>>, crate::GenerationError> {
        let min = self.bounds[a]
            .min()
            .metres()
            .max(self.bounds[b].min().metres());
        let max = self.bounds[a]
            .max()
            .metres()
            .min(self.bounds[b].max().metres());
        Ok(if (max - min).min_element() > CONTACT_DEPTH_METRES {
            Some(SpatialBounds::<Architectural>::from_metres(min, max)?)
        } else {
            None
        })
    }

    fn append(
        &self,
        bonds: &mut Vec<JunctionBond>,
        serial: &mut u64,
        a: usize,
        b: usize,
        bounds: SpatialBounds<Architectural>,
    ) {
        let overlap = bounds.max().metres() - bounds.min().metres();
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
