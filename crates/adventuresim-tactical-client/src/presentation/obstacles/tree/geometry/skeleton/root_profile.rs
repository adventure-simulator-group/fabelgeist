//! Root shoulder exposure and the tapered return into soil.

use bevy::math::{FloatExt, Vec3};

use super::{OakGnarlingParameters, OakRootFork, OakRootSpec, polyline_tangent, sample_polyline};

pub(super) fn oak_root_points(
    trunk_base: Vec3,
    root: OakRootSpec,
    gnarling: OakGnarlingParameters,
) -> [Vec3; 3] {
    let outward = Vec3::new(root.angle.cos(), 0.0, root.angle.sin());
    let tangent = Vec3::new(-root.angle.sin(), 0.0, root.angle.cos());
    let contact_radius = 0.36 + (root.base_radius - 0.14) * 0.32;
    let meander = (gnarling.root_meander.clamp(0.0, 1.0) * root.reach * 0.32)
        * (root.angle * 2.7 + root.reach * 3.1).sin();
    let exposure = gnarling.root_exposure.clamp(0.0, 1.0);
    let contact =
        trunk_base + outward * contact_radius + tangent * (root.shoulder_lift - 0.0375) * 1.4;
    [
        contact,
        contact
            + outward
                * (0.19 + root.reach * 0.18)
                    .lerp((root.reach - contact_radius) * 0.72, exposure)
            + tangent * ((root.tip_radius - 0.016) * 3.0 + meander)
            // Exposure brings the broad shoulder above grade; the tapered
            // continuation still descends into the soil.
            + Vec3::Y * (root.shoulder_lift - 0.16 + exposure * 0.45),
        trunk_base + outward * root.reach + tangent * meander * 0.42
            - Vec3::Y * root.burial * (1.0 - exposure * 0.3),
    ]
}

pub(super) fn oak_root_fork_points(
    parent: &[Vec3; 3],
    root: OakRootSpec,
    fork: OakRootFork,
) -> [Vec3; 2] {
    let fork_start = sample_polyline(parent, fork.attach);
    let parent_tangent = polyline_tangent(parent, fork.attach);
    let horizontal = Vec3::new(parent_tangent.x, 0.0, parent_tangent.z).normalize();
    let lateral = Vec3::new(-horizontal.z, 0.0, horizontal.x);
    let fork_direction =
        (horizontal * 0.76 + lateral * fork.angle_offset.signum() * 0.42).normalize();
    [
        fork_start,
        fork_start + fork_direction * fork.reach - Vec3::Y * root.burial * 0.55,
    ]
}

#[cfg(test)]
mod tests {
    use super::super::{
        NATURAL_OAK_GNARLING, procedural_oak_root_specs, procedural_oak_root_specs_with_gnarling,
        unit_hash,
    };
    use super::*;
    use adventuresim_tactical_core::prelude::TREE_TRUNK_RADIUS_METRES;
    use bevy::math::Vec3Swizzles;

    #[test]
    fn natural_oak_roots_have_no_above_grade_continuations() {
        for seed in 0..256 {
            let crown_phase = unit_hash(seed ^ 0x9182_64ac) * core::f32::consts::TAU;
            for root in procedural_oak_root_specs(seed, crown_phase) {
                let points = oak_root_points(-Vec3::Y * 0.07, root, NATURAL_OAK_GNARLING);
                // The contact capsule may break grade as a short trunk flare;
                // both continuation controls and their conservative radii
                // remain below grade, so no radial toe can terminate visibly.
                assert!(points[1].y + root.base_radius * 0.65 < 0.0);
                assert!(points[2].y + root.tip_radius < 0.0);
                if let Some(fork) = root.fork {
                    let fork_points = oak_root_fork_points(&points, root, fork);
                    assert!(
                        fork_points
                            .iter()
                            .all(|point| point.y + root.base_radius * 0.34 < 0.0)
                    );
                }
            }
        }
    }

    #[test]
    fn exposed_oak_roots_reveal_shoulders_but_keep_their_tips_buried() {
        let exposed = OakGnarlingParameters {
            root_exposure: 1.0,
            root_spread: 1.0,
            ..NATURAL_OAK_GNARLING
        };
        for seed in 0..32 {
            let phase = unit_hash(seed ^ 0x9182_64ac) * core::f32::consts::TAU;
            for root in procedural_oak_root_specs_with_gnarling(seed, phase, exposed) {
                let points = oak_root_points(-Vec3::Y * 0.07, root, exposed);
                assert!(points[1].y > 0.2);
                assert!(points[1].xz().length() > TREE_TRUNK_RADIUS_METRES);
                assert!(points[2].y + root.tip_radius < 0.0);
                assert!(points[1].xz().length() < points[2].xz().length());
            }
        }
    }
}
