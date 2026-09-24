//! Woody centerline deformation and terminal shoot placement.
use super::*;

pub(super) fn oak_trunk_points(
    canopy_competition: f32,
    crown_phase: f32,
    lean_phase: f32,
    gnarling: OakGnarlingParameters,
) -> Vec<Vec3> {
    let bend_direction = Vec3::new(crown_phase.cos(), 0.0, crown_phase.sin());

    // Quercus robur is usually short-boled in the open.  The trunk loses
    // dominance inside a broad crown instead of continuing as a conifer-like
    // central spear.
    let trunk_length = 5.4_f32.lerp(9.2, canopy_competition);
    let lean_direction = Vec3::new(lean_phase.cos(), 0.0, lean_phase.sin());
    let trunk_deformation =
        gnarling.trunk_lean + gnarling.trunk_sweep + gnarling.trunk_twist + gnarling.trunk_crooks;
    let trunk_steps = if trunk_deformation > 0.001 { 24 } else { 6 };
    (0..=trunk_steps)
        .map(|index| {
            let t = index as f32 / trunk_steps as f32;
            let window = (core::f32::consts::PI * t).sin();
            let sweep = lean_direction * (gnarling.trunk_lean.clamp(0.0, 1.0) * 2.6 * t.powf(1.25));
            let crook = bend_direction
                * (gnarling.trunk_crooks.clamp(0.0, 1.0)
                    * 0.72
                    * (t * core::f32::consts::TAU * 2.4 + crown_phase).sin()
                    * window);
            let helical = Vec3::new(
                (crown_phase + t * core::f32::consts::TAU * 1.7).cos(),
                0.0,
                (crown_phase + t * core::f32::consts::TAU * 1.7).sin(),
            ) * (gnarling.trunk_twist.clamp(0.0, 1.0) * 0.48 * window);
            let lateral_sweep = Vec3::new(-lean_direction.z, 0.0, lean_direction.x)
                * (gnarling.trunk_sweep.clamp(0.0, 1.0)
                    * 0.8
                    * (t * core::f32::consts::PI).sin().powi(2));
            Vec3::new(0.0, -TREE_TRUNK_HEIGHT_METRES * 0.5, 0.0)
                + Vec3::Y * (trunk_length * t)
                + bend_direction * (0.28 * t.powf(1.45))
                + sweep
                + crook
                + helical
                + lateral_sweep
        })
        .collect::<Vec<_>>()
}

pub(super) fn append_shrub_shoots(
    branches: &mut Vec<TreeBranchSegment>,
    branch_seed: u64,
    branch_points: &[Vec3],
    branch_start_radius: f32,
    direction: Vec3,
    (primary_group, secondary_group): (u8, u16),
) {
    for shoot_index in 0..5_u64 {
        let shoot_seed = streams::SHRUB_SHOOT
            .seed(branch_seed, &[shoot_index])
            .to_u64();
        let shoot_unit_draw = |purpose: fabelgeist_determinism::StreamId| {
            purpose.rng(shoot_seed, &[]).inclusive_unit_f32()
        };
        let along = 0.18 + shoot_index as f32 * 0.19;
        let shoot_start = sample_polyline(branch_points, along);
        let (right, up) = branch_frame(direction);
        let shoot_phase = shoot_index as f32 * 2.399_963_1 + shoot_unit_draw(streams::SHOOT_PHASE);
        let shoot_direction = (direction * 0.34
            + right * shoot_phase.cos() * 0.62
            + up * shoot_phase.sin() * 0.42
            + Vec3::Y * 0.22)
            .normalize();
        let shoot_length = 0.18 + shoot_unit_draw(streams::SHOOT_RADIAL) * 0.14;
        append_branch_curve(
            branches,
            &[
                shoot_start,
                shoot_start + shoot_direction * shoot_length * 0.52,
                shoot_start + shoot_direction * shoot_length,
            ],
            child_base_radius(0.005, curve_radius_at(branch_start_radius, 0.0036, along)),
            0.0015,
            3,
            primary_group,
            secondary_group,
        );
    }
}
