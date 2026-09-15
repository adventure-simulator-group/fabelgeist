use super::*;

#[test]
fn spatial_candidates_keep_every_exact_overlap_for_tilted_and_annular_solids() {
    let solids: Vec<_> = (0..64)
        .map(|i| crate::ResolvedSolid {
            id: ResolvedItemId(i),
            owner: crate::GeometryOwnerId(i as u32),
            centre: Vec3::new((i % 7) as f32, (i % 3) as f32, (i % 11) as f32),
            size: Vec3::new(1.5, 4.0, 0.5),
            yaw_radians: i as f32 * 0.13,
            crossfall_radians: i as f32 * 0.07,
            longfall_radians: i as f32 * 0.19,
            role: SolidRole::FrameMember,
            shape: if i % 5 == 0 {
                crate::ResolvedSolidShape::AnnularPrism {
                    inner_radius_metres: 0.3,
                    outer_radius_metres: 0.75,
                    inner_top_offset_metres: 0.0,
                    outer_top_offset_metres: 0.0,
                    drainage_outlet_count: 0,
                    circumferential_fall_metres: 0.0,
                }
            } else {
                crate::ResolvedSolidShape::Cuboid
            },
            supported_by: Vec::new(),
        })
        .collect();
    let candidates: BTreeSet<_> = crate::geometry_index::overlapping_solids(&solids)
        .map(|(a, b)| (a.id, b.id))
        .collect();
    for (i, a) in solids.iter().enumerate() {
        for b in &solids[i + 1..] {
            if resolved_shape_overlap(a, b, 0.025) || oriented_cuboids_overlap(a, b, 0.008) {
                assert!(
                    candidates.contains(&(a.id, b.id)),
                    "missed {:?} {:?}",
                    a.id,
                    b.id
                );
            }
        }
    }
}
