//! Bakes accepted activity footprints into ground vertices, without GPU scans.

use super::*;

const ACTIVITY_FEATHER_METRES: f32 = 0.9;

pub(super) struct ActivityWear<'a> {
    groups: Vec<&'a FurnitureGroup>,
}

impl<'a> ActivityWear<'a> {
    pub(super) fn for_patch(corners: [Vec2; 4], groups: &'a [FurnitureGroup]) -> Self {
        let (minimum, maximum) = bounds(corners);
        let groups = groups
            .iter()
            .filter(|group| {
                let (group_minimum, group_maximum) = bounds(group.footprint.corners());
                group_maximum.cmpge(minimum).all() && group_minimum.cmple(maximum).all()
            })
            .collect();
        Self { groups }
    }

    #[cfg(test)]
    pub(super) fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }

    pub(super) fn at(&self, point: Vec2) -> Vec2 {
        self.groups.iter().fold(Vec2::ZERO, |combined, group| {
            let footprint = group.footprint;
            let local = footprint
                .orientation
                .world_to_local(point - footprint.centre_metres)
                .abs();
            let inside = (footprint.half_extents_metres - local).min_element();
            let feather = (inside / ACTIVITY_FEATHER_METRES).clamp(0.0, 1.0);
            let coverage = feather * feather * (3.0 - 2.0 * feather);
            let (wear, dampness) = match group.kind {
                FurnitureGroupKind::Vendor => (0.55, 0.12),
                FurnitureGroupKind::Receiving => (0.7, 0.25),
                FurnitureGroupKind::HorseStop => (0.9, 0.85),
                FurnitureGroupKind::Domestic => (0.25, 0.1),
                FurnitureGroupKind::Workshop => (0.65, 0.2),
            };
            combined.max(Vec2::new(wear, dampness) * coverage)
        })
    }
}

fn bounds(corners: [Vec2; 4]) -> (Vec2, Vec2) {
    corners.into_iter().fold(
        (Vec2::splat(f32::INFINITY), Vec2::splat(f32::NEG_INFINITY)),
        |(minimum, maximum), point| (minimum.min(point), maximum.max(point)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepted_rotated_activity_stays_inside_its_footprint() {
        let footprint = FurnitureFootprint {
            centre_metres: Vec2::new(7.0, -3.0),
            half_extents_metres: Vec2::new(3.0, 2.0),
            orientation: BuildingOrientation::from_radians(0.7).unwrap(),
        };
        let groups = [FurnitureGroup {
            id: FurnitureGroupId(1),
            kind: FurnitureGroupKind::HorseStop,
            anchor: FurnitureAnchor::Market { patch_index: 0 },
            footprint,
        }];
        let wear = ActivityWear::for_patch(footprint.corners(), &groups);
        assert!(wear.at(footprint.centre_metres).x > 0.8);
        for corner in footprint.corners() {
            assert!(wear.at(corner).length() < 0.001);
        }
        assert_eq!(wear.at(Vec2::new(-20.0, 13.0)), Vec2::ZERO);
        let distant = footprint.corners().map(|point| point + Vec2::splat(100.0));
        assert!(ActivityWear::for_patch(distant, &groups).is_empty());
    }
}
