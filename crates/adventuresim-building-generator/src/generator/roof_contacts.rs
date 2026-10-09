//! Wall pieces stay fixed while roof weathering is emitted. Index them once
//! rather than searching the growing geometry for every contact station.
use crate::{Architectural, SpatialBounds};
use std::collections::{BTreeMap, BTreeSet};

use crate::{GeometryOwnerId, ResolvedSolid, WallAssembly};

const POSITIVE_CONTACT_DEPTH_METRES: f32 = 0.025;

pub(super) struct RoofContacts(BTreeMap<GeometryOwnerId, Vec<SpatialBounds<Architectural>>>);

impl RoofContacts {
    pub(super) fn new(
        walls: &[WallAssembly],
        solids: &[ResolvedSolid],
    ) -> Result<Self, crate::GenerationError> {
        let mut owners: BTreeMap<_, Vec<_>> =
            walls.iter().map(|wall| (wall.owner, Vec::new())).collect();
        for solid in solids {
            if let Some(bounds) = owners.get_mut(&solid.owner) {
                bounds.push(solid.yaw_bounds()?);
            }
        }
        Ok(Self(owners))
    }

    pub(super) fn touching(
        &self,
        weathering: &[ResolvedSolid],
    ) -> Result<BTreeSet<GeometryOwnerId>, crate::GenerationError> {
        let weather: Vec<_> = weathering
            .iter()
            .map(ResolvedSolid::yaw_bounds)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(self
            .0
            .iter()
            .filter_map(|(owner, hosts)| {
                hosts
                    .iter()
                    .any(|host| {
                        weather.iter().any(|weather| {
                            let min = host.min().metres().max(weather.min().metres());
                            let max = host.max().metres().min(weather.max().metres());
                            (max - min).min_element() > POSITIVE_CONTACT_DEPTH_METRES
                        })
                    })
                    .then_some(*owner)
            })
            .collect())
    }
}

/// Preserve the native wall-contact ranking at one authored weather station.
pub(super) fn abutment_host(
    walls: &[WallAssembly],
    abutment_kind: crate::RoofAbutmentKind,
    point: bevy::math::Vec3,
    plan_point: bevy::math::Vec2,
) -> Option<&WallAssembly> {
    use crate::RoofAbutmentKind;
    walls
        .iter()
        .filter(|wall| {
            if abutment_kind == RoofAbutmentKind::Tower {
                matches!(wall.source, crate::WallSourceId::SquareTowerFace { .. })
            } else {
                !matches!(wall.source, crate::WallSourceId::RoofChildFront { .. })
            }
        })
        .filter_map(|wall| {
            let offset = plan_point - wall.frame.origin;
            let signed_normal = offset.dot(wall.frame.outward);
            // A weatherable roof abutment lies on the exterior
            // masonry face, never the wall centreline. Clipped
            // fragments on the interior side are opening-cut
            // boundaries, not valid contact contours.
            let normal_distance = (signed_normal - wall.thickness_metres * 0.5).abs();
            let along = offset.dot(wall.frame.tangent).abs();
            let corner_return = if abutment_kind == RoofAbutmentKind::Tower {
                wall.thickness_metres * 0.5
            } else {
                0.0
            };
            (normal_distance <= wall.thickness_metres * 0.5 + 0.18
                && along <= wall.length_metres * 0.5 + corner_return + 0.18
                && point.y >= wall.base_elevation_metres - 0.08
                && point.y <= wall.base_elevation_metres + wall.height_metres + 0.18)
                .then_some((wall, normal_distance))
        })
        .min_by(|(left_wall, left), (right_wall, right)| {
            let priority = |wall: &crate::WallAssembly| {
                if abutment_kind == RoofAbutmentKind::Wall
                    && matches!(wall.source, crate::WallSourceId::ChurchArcade { .. })
                {
                    0_u8
                } else {
                    1_u8
                }
            };
            priority(left_wall)
                .cmp(&priority(right_wall))
                .then_with(|| left.total_cmp(right))
        })
        .map(|(wall, _)| wall)
}
