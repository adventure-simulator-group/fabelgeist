//! Surface-root placement and species-specific root morphology.
use super::*;

pub(super) fn root_plan_angles(plan_seed: u64, crown_phase: f32, root_count: usize) -> Vec<f32> {
    // Allocate the full circle as unequal positive gaps. Normalizing the
    // weights keeps complete coverage without returning to equal radial rays.
    let gap_weights = (0..root_count)
        .map(|index| {
            0.62 + streams::ROOT_GAP
                .rng(plan_seed, &[index as u64])
                .inclusive_unit_f32()
                * 0.82
        })
        .collect::<Vec<_>>();
    let gap_total = gap_weights.iter().sum::<f32>();
    let rotation = crown_phase
        + streams::ROOT_ROTATION
            .rng(plan_seed, &[])
            .inclusive_unit_f32()
            * 0.74;
    let mut cursor = rotation;
    let mut angles = Vec::with_capacity(root_count);
    for gap in gap_weights {
        angles.push(cursor);
        cursor += gap / gap_total * core::f32::consts::TAU;
    }

    angles
}

pub(super) fn procedural_oak_root_specs_with_gnarling(
    seed: u64,
    crown_phase: f32,
    gnarling: OakGnarlingParameters,
) -> Vec<OakRootSpec> {
    let plan_seed = streams::ROOT_PLAN.seed(seed, &[]).to_u64();
    let root_count = OAK_ROOT_MIN_COUNT
        + streams::ROOT_COUNT
            .rng(plan_seed, &[])
            .index(OAK_ROOT_MAX_COUNT - OAK_ROOT_MIN_COUNT + 1);
    let dominant_count = 2 + streams::DOMINANT_ROOT_COUNT.rng(plan_seed, &[]).index(2);

    let mut angles = root_plan_angles(plan_seed, crown_phase, root_count);

    // Pull distinct nearby roots toward the heaviest scaffold axes. This is a
    // restrained azimuthal bias, not a one-root-per-branch radial layout.
    let mut dominant = vec![false; root_count];
    for primary_index in 0..dominant_count as u64 {
        let primary_seed = streams::OAK_PRIMARY.seed(seed, &[primary_index]).to_u64();
        let load_phase = oak_primary_scaffold_phase(crown_phase, primary_index, primary_seed);
        let nearest = angles
            .iter()
            .enumerate()
            .filter(|(index, _)| !dominant[*index])
            .min_by(|(_, left), (_, right)| {
                signed_angular_delta(**left, load_phase)
                    .abs()
                    .total_cmp(&signed_angular_delta(**right, load_phase).abs())
            })
            .map(|(index, _)| index)
            .expect("an oak root plan always has more roots than dominant scaffolds");
        let previous = if nearest == 0 {
            angles[root_count - 1] - core::f32::consts::TAU
        } else {
            angles[nearest - 1]
        };
        let next = if nearest + 1 == root_count {
            angles[0] + core::f32::consts::TAU
        } else {
            angles[nearest + 1]
        };
        let desired = angles[nearest] + signed_angular_delta(angles[nearest], load_phase) * 0.68;
        angles[nearest] = desired.clamp(
            previous + OAK_ROOT_MIN_ANGULAR_GAP,
            next - OAK_ROOT_MIN_ANGULAR_GAP,
        );
        dominant[nearest] = true;
    }

    let mut fork_count = 0;
    let mut roots = angles
        .into_iter()
        .enumerate()
        .map(|(index, angle)| {
            let root_seed = streams::OAK_ROOT.seed(plan_seed, &[index as u64]).to_u64();
            let root_unit_draw = |purpose: fabelgeist_determinism::StreamId| {
                purpose.rng(root_seed, &[]).inclusive_unit_f32()
            };
            let is_dominant = dominant[index];
            let reach = (if is_dominant {
                0.88 + root_unit_draw(streams::OAK_ROOT_REACH) * 0.16
            } else {
                0.55 + root_unit_draw(streams::OAK_ROOT_REACH) * 0.3
            }) * (1.0 + gnarling.root_spread.clamp(0.0, 1.0) * 1.8);
            let base_radius = if is_dominant {
                0.22 + root_unit_draw(streams::OAK_ROOT_RADIUS) * 0.05
            } else {
                0.14 + root_unit_draw(streams::OAK_ROOT_RADIUS) * 0.05
            };
            let fork_threshold = 0.82 - gnarling.root_forking.clamp(0.0, 1.0) * 0.72;
            let fork = (fork_count < OAK_ROOT_MAX_FORKS
                && root_unit_draw(streams::OAK_ROOT_FORK) > fork_threshold)
                .then(|| {
                    fork_count += 1;
                    OakRootFork {
                        attach: 0.54 + root_unit_draw(streams::OAK_ROOT_FORK_ATTACH) * 0.17,
                        angle_offset: if streams::ROOT_FORK_SIDE.rng(root_seed, &[]).boolean() {
                            0.5 + root_unit_draw(streams::OAK_ROOT_FORK_ANGLE) * 0.35
                        } else {
                            -0.5 - root_unit_draw(streams::OAK_ROOT_FORK_ANGLE) * 0.35
                        },
                        reach: 0.16 + root_unit_draw(streams::OAK_ROOT_FORK_REACH) * 0.16,
                    }
                });
            OakRootSpec {
                angle: angle.rem_euclid(core::f32::consts::TAU),
                reach,
                base_radius,
                tip_radius: 0.008 + root_unit_draw(streams::OAK_ROOT_TIP_RADIUS) * 0.012,
                shoulder_lift: root_unit_draw(streams::OAK_ROOT_LIFT) * 0.012,
                burial: 0.24 + root_unit_draw(streams::OAK_ROOT_BURIAL) * 0.12,
                dominant: is_dominant,
                fork,
            }
        })
        .collect::<Vec<_>>();
    roots.sort_by(|left, right| left.angle.total_cmp(&right.angle));
    roots
}

pub(super) fn append_beech_roots(
    branches: &mut Vec<TreeBranchSegment>,
    seed: u64,
    crown_phase: f32,
    trunk_base: Vec3,
) {
    // Beech commonly shows a broad, shallow root plate rather than a set of
    // exposed radial cables. A few low shoulders taper below the soil within
    // a metre, softening the trunk-ground junction without creating oak-like
    // buttresses or changing the authoritative cylindrical collider.
    for root_index in 0..3_u64 {
        let root_seed = streams::BEECH_ROOT.seed(seed, &[root_index]).to_u64();
        let root_unit_draw = |purpose: fabelgeist_determinism::StreamId| {
            purpose.rng(root_seed, &[]).inclusive_unit_f32()
        };
        let phase = crown_phase
            + root_index as f32 * 2.399_963_1
            + (root_unit_draw(streams::BEECH_ROOT_PHASE) - 0.5) * 0.32;
        let outward = Vec3::new(phase.cos(), 0.0, phase.sin());
        let tangent = Vec3::new(-phase.sin(), 0.0, phase.cos());
        let reach = 0.3 + root_unit_draw(streams::BEECH_ROOT_REACH) * 0.16;
        let shoulder = trunk_base
            + outward * (0.3 + root_unit_draw(streams::BEECH_ROOT_SHOULDER) * 0.025)
            + Vec3::Y * (0.004 + root_unit_draw(streams::BEECH_ROOT_LIFT) * 0.008);
        let root_points = [
            shoulder,
            shoulder
                + outward * reach * 0.42
                + tangent * (root_unit_draw(streams::BEECH_ROOT_MID_BEND) - 0.5) * 0.08
                - Vec3::Y * 0.11,
            trunk_base
                + outward * reach
                + tangent * (root_unit_draw(streams::BEECH_ROOT_END_BEND) - 0.5) * 0.12
                - Vec3::Y * (0.25 + root_unit_draw(streams::BEECH_ROOT_DEPTH) * 0.08),
        ];
        append_branch_curve(
            branches,
            &root_points,
            0.11 + root_unit_draw(streams::BEECH_ROOT_RADIUS) * 0.025,
            0.006 + root_unit_draw(streams::BEECH_ROOT_TIP_RADIUS) * 0.004,
            0,
            u8::MAX,
            u16::MAX,
        );
    }
}
