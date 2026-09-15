use super::*;

/// Touching level cores form one terrace, even when their reserved plots differ.
/// All medians sample the unchanged source terrain, so iteration order cannot
/// introduce a second elevation beneath a neighbour's wall or access route.
pub(super) fn shared_elevations(
    pads: &[BuildingPad],
    width: usize,
    depth: usize,
    spacing: f32,
    half_extent: Vec2,
    heights: &[f32],
) -> Vec<f32> {
    let mut groups = (0..pads.len()).collect::<Vec<_>>();
    for (i, a) in pads.iter().enumerate() {
        for (j, b) in pads.iter().enumerate().skip(i + 1) {
            if oriented_rectangles_overlap(
                a.centre,
                a.half_extents + Vec2::splat(LEVEL_MARGIN_METRES),
                a.orientation,
                b.centre,
                b.half_extents + Vec2::splat(LEVEL_MARGIN_METRES),
                b.orientation,
            ) {
                let from = groups[j];
                let to = groups[i];
                for group in &mut groups {
                    if *group == from {
                        *group = to;
                    }
                }
            }
        }
    }
    let mut levels = std::collections::BTreeMap::new();
    for group in groups.iter().copied() {
        levels.entry(group).or_insert_with(|| {
            let members = pads
                .iter()
                .zip(&groups)
                .filter(|(_, id)| **id == group)
                .map(|(pad, _)| pad)
                .collect::<Vec<_>>();
            let mut covered = sample_indices(width, depth, spacing, half_extent)
                .filter(|(_, p)| {
                    members
                        .iter()
                        .any(|pad| pad.local_offset(*p).abs().cmple(pad.half_extents).all())
                })
                .map(|(index, _)| heights[index])
                .collect::<Vec<_>>();
            covered.sort_by(f32::total_cmp);
            covered.get(covered.len() / 2).copied().unwrap_or_else(|| {
                nearest_height(width, depth, spacing, heights, pads[group].centre)
            })
        });
    }
    groups.iter().map(|group| levels[group]).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn neighbouring_property_cores_share_elevation_independently_of_order() {
        let a = BuildingPad {
            centre: Vec2::new(1.0, 6.0),
            half_extents: Vec2::new(8.25, 15.5),
            orientation: BuildingOrientation::IDENTITY,
            elevation_metres: 0.0,
        };
        let b = BuildingPad {
            centre: a.centre + Vec2::X * 16.5,
            ..a
        };
        let extent = Vec2::splat(40.0);
        let heights = sample_indices(161, 161, 0.5, extent)
            .map(|(_, p)| p.x * 0.1)
            .collect::<Vec<_>>();
        let forward = shared_elevations(&[a, b], 161, 161, 0.5, extent, &heights);
        let reverse = shared_elevations(&[b, a], 161, 161, 0.5, extent, &heights);
        assert_eq!(forward[0], forward[1]);
        assert_eq!(forward, reverse);
        assert!(a.contains_level_ground(Vec2::new(10.38, 12.0)));
        assert!(b.contains_level_ground(Vec2::new(10.38, 12.0)));
    }
}
