//! Species-specific leaf positions and terminal-shoot arrangement.
use super::*;
use libm::{cosf, sinf};

pub(in crate::presentation) fn procedural_oak_leaves(
    seed: Seed,
    branches: &[TreeBranchSegment],
    canopy_competition: f32,
) -> Vec<TreeLeaf> {
    let mut leaves = Vec::new();
    let _ = canopy_competition;
    // A compact terminal flush gives each shoot one readable foliage mass.
    // The pulsed shoot layout then cuts redundant alpha overlap while
    // retaining the oak's irregular shell.
    let leaves_per_shoot = 16_u64;
    for (shoot_id, shoot) in branches
        .iter()
        .filter_map(|branch| branch.foliage.map(|identity| (identity, branch)))
    {
        let direction = (shoot.end - shoot.start).normalize();
        let tangent = direction.cross(Vec3::Y).normalize_or_zero();
        let tangent = if tangent.length_squared() < 0.25 {
            Vec3::X
        } else {
            tangent
        };
        let binormal = direction.cross(tangent).normalize();
        for leaf_index in 0..leaves_per_shoot {
            let leaf_seed = streams::LEAF.seed(seed, &[shoot_id.seed().to_u64(), leaf_index]);
            let leaf_unit_draw = |purpose: fabelgeist_determinism::StreamId| {
                purpose.rng(leaf_seed, &[]).inclusive_unit_f32()
            };
            // Alternate leaves along each current-year shoot, then finish in
            // the tighter terminal flush characteristic of pedunculate oak.
            let along = if leaf_index < 2 {
                0.12 + leaf_index as f32 * 0.28
            } else {
                (0.70
                    + (leaf_index - 2) as f32 / (leaves_per_shoot - 3) as f32 * 0.285
                    + (leaf_unit_draw(streams::TERMINAL_ATTACHMENT) - 0.5) * 0.008)
                    .clamp(0.69, 0.99)
            };
            let alternate = if leaf_index & 1 == 0 { 1.0 } else { -1.0 };
            let spiral =
                leaf_index as f32 * 2.399_963_1 + (leaf_unit_draw(streams::SPIRAL) - 0.5) * 0.65;
            let radial = (tangent * cosf(spiral) + binormal * sinf(spiral)).normalize();
            let leaf_up = (radial * (0.46 + leaf_unit_draw(streams::RADIAL_LIFT) * 0.24)
                + direction * (0.42 + leaf_unit_draw(streams::DIRECTION_LIFT) * 0.18)
                + Vec3::Y * (0.08 + leaf_unit_draw(streams::VERTICAL_LIFT) * 0.18))
                .normalize();
            let leaf_normal_candidate = direction.cross(radial)
                + radial * (leaf_unit_draw(streams::NORMAL_RADIAL) - 0.5) * 0.7
                + Vec3::Y * (leaf_unit_draw(streams::NORMAL_VERTICAL) - 0.5) * 0.35;
            let leaf_normal = if leaf_normal_candidate.length_squared() > 0.001 {
                leaf_normal_candidate.normalize()
            } else {
                branch_frame(leaf_up).1
            };
            let right_candidate = leaf_up.cross(leaf_normal);
            let leaf_right = if right_candidate.length_squared() > 0.001 {
                right_candidate.normalize()
            } else {
                branch_frame(leaf_up).0 * alternate
            };
            let petiole_start = shoot.start.lerp(shoot.end, along.min(0.98));
            // Pedunculate-oak leaf stalks are only a few millimetres long.
            // Keeping this independent of blade size avoids the long-stalked,
            // bilateral-comb silhouette of other broadleaf genera.
            let petiole_length = 0.003 + leaf_unit_draw(streams::PETIOLE_LENGTH) * 0.004;
            let blade_base =
                petiole_start + (radial * 0.82 + leaf_up * 0.18).normalize() * petiole_length;
            let leaf_length = 0.1 + leaf_unit_draw(streams::LENGTH) * 0.06;
            let leaf_width = 0.065 + leaf_unit_draw(streams::WIDTH) * 0.04;
            let shell_exposure = ((shoot.end.xz().length() - 1.25) / 4.75).clamp(0.0, 1.0);
            let shade = if leaf_index < 2 {
                0.58 + shell_exposure * 0.22 + leaf_unit_draw(streams::SHADE) * 0.12
            } else {
                0.68 + shell_exposure * 0.25 + leaf_unit_draw(streams::SHADE) * 0.14
            };
            leaves.push(TreeLeaf {
                petiole_start,
                center: blade_base + leaf_up * leaf_length * 0.5,
                right: leaf_right.normalize(),
                up: leaf_up,
                length: leaf_length,
                width: leaf_width,
                primary_group: shoot.primary_group,
                secondary_group: shoot.secondary_group,
                shoot_id,
                leaf_ordinal: leaf_index as u8,
                shade,
                torsion: (leaf_unit_draw(streams::TORSION) - 0.5) * 0.42,
            });
        }
    }
    leaves
}

pub(in crate::presentation) fn procedural_woody_plant_leaves(
    seed: Seed,
    branches: &[TreeBranchSegment],
    canopy_competition: f32,
    parameters: WoodyPlantParameters,
) -> Vec<TreeLeaf> {
    match parameters.form {
        WoodyPlantForm::MatureOak => procedural_oak_leaves(seed, branches, canopy_competition),
        WoodyPlantForm::MatureBeech => procedural_beech_leaves(seed, branches, parameters),
        WoodyPlantForm::MultiStemShrub => {
            procedural_multistem_shrub_leaves(seed, branches, parameters)
        }
    }
}

/// Arranges beech leaves in overlapping, approximately two-ranked sprays.
/// The broad, mostly horizontal leaf planes build a closed shade-casting crown
/// from the existing twig and leaf budget instead of filling it by count.
fn procedural_beech_leaves(
    seed: Seed,
    branches: &[TreeBranchSegment],
    parameters: WoodyPlantParameters,
) -> Vec<TreeLeaf> {
    let mut leaves = Vec::new();
    let leaves_per_shoot = u64::from(parameters.leaves_per_shoot.max(4));
    for (shoot_id, shoot) in branches
        .iter()
        .filter_map(|branch| branch.foliage.map(|identity| (identity, branch)))
    {
        let direction = (shoot.end - shoot.start).normalize();
        let horizontal = Vec3::new(direction.x, 0.0, direction.z).normalize_or_zero();
        let spray_forward = if horizontal.length_squared() > 0.25 {
            horizontal
        } else {
            branch_frame(direction).0
        };
        let spray_side = Vec3::Y.cross(spray_forward).normalize();
        for leaf_index in 0..leaves_per_shoot {
            let leaf_seed = streams::LEAF.seed(seed, &[shoot_id.seed().to_u64(), leaf_index]);
            let leaf_unit_draw = |purpose: fabelgeist_determinism::StreamId| {
                purpose.rng(leaf_seed, &[]).inclusive_unit_f32()
            };
            let along = (0.1
                + leaf_index as f32 / (leaves_per_shoot - 1) as f32 * 0.84
                + (leaf_unit_draw(streams::ATTACHMENT) - 0.5) * 0.018)
                .clamp(0.06, 0.97);
            let side = if leaf_index & 1 == 0 { 1.0 } else { -1.0 };
            let blade_outward = (spray_forward * (0.56 + leaf_unit_draw(streams::SPIRAL) * 0.18)
                + spray_side * side * (0.7 + leaf_unit_draw(streams::RADIAL_LIFT) * 0.16)
                + Vec3::Y * (0.06 + leaf_unit_draw(streams::PETIOLE_LENGTH) * 0.1))
                .normalize();
            // Keep the lamina close to horizontal while allowing a small,
            // deterministic ripple through each spray.
            let horizontal_weight = if leaf_index & 3 < 2 { 1.0 } else { 0.62 };
            let posture_normal = (Vec3::Y * horizontal_weight
                + spray_forward * (1.0 - horizontal_weight) * 1.35
                + spray_side * (leaf_unit_draw(streams::LENGTH) - 0.5) * 0.28
                + spray_forward * (leaf_unit_draw(streams::WIDTH) - 0.5) * 0.18)
                .normalize();
            let right = blade_outward.cross(posture_normal).normalize_or_zero();
            let right = if right.length_squared() > 0.25 {
                right
            } else {
                branch_frame(blade_outward).0
            };
            let petiole_start = shoot.start.lerp(shoot.end, along);
            let petiole_length = parameters.petiole_length_metres[0]
                + leaf_unit_draw(streams::SHADE)
                    * (parameters.petiole_length_metres[1] - parameters.petiole_length_metres[0]);
            // At mature-tree scale this card represents a small overlapping
            // beech spray, not a single isolated lamina. Keeping the card
            // count bounded while expanding the spray footprint produces the
            // species' closed, shade-casting crown without multiplying draw
            // and streaming work across a dense stand.
            let length = (parameters.leaf_length_metres[0]
                + leaf_unit_draw(streams::DIRECTION_LIFT)
                    * (parameters.leaf_length_metres[1] - parameters.leaf_length_metres[0]))
                * 3.0;
            let width = length
                * (parameters.leaf_width_ratio[0]
                    + leaf_unit_draw(streams::VERTICAL_LIFT)
                        * (parameters.leaf_width_ratio[1] - parameters.leaf_width_ratio[0]));
            let blade_base = petiole_start + blade_outward * petiole_length;
            leaves.push(TreeLeaf {
                petiole_start,
                center: blade_base + blade_outward * length * 0.5,
                right,
                up: blade_outward,
                length,
                width,
                primary_group: shoot.primary_group,
                secondary_group: shoot.secondary_group,
                shoot_id,
                leaf_ordinal: leaf_index as u8,
                shade: 0.7 + leaf_unit_draw(streams::NORMAL_RADIAL) * 0.2,
                torsion: (leaf_unit_draw(streams::NORMAL_VERTICAL) - 0.5) * 0.18,
            });
        }
    }
    leaves
}

fn procedural_multistem_shrub_leaves(
    seed: Seed,
    branches: &[TreeBranchSegment],
    parameters: WoodyPlantParameters,
) -> Vec<TreeLeaf> {
    let mut leaves = Vec::new();
    let leaves_per_shoot = u64::from(parameters.leaves_per_shoot.max(4));
    for (shoot_id, shoot) in branches
        .iter()
        .filter_map(|branch| branch.foliage.map(|identity| (identity, branch)))
    {
        let direction = (shoot.end - shoot.start).normalize();
        let (frame_right, frame_up) = branch_frame(direction);
        for leaf_index in 0..leaves_per_shoot {
            let leaf_seed = streams::LEAF.seed(seed, &[shoot_id.seed().to_u64(), leaf_index]);
            let leaf_unit_draw = |purpose: fabelgeist_determinism::StreamId| {
                purpose.rng(leaf_seed, &[]).inclusive_unit_f32()
            };
            // Common hazel leaves are alternate and loosely distichous. The
            // golden-angle perturbation prevents a flat bilateral comb while
            // retaining opposite-side succession along each current shoot.
            let along = 0.08
                + leaf_index as f32 / (leaves_per_shoot - 1) as f32 * 0.84
                + (leaf_unit_draw(streams::ATTACHMENT) - 0.5) * 0.025;
            let side = if leaf_index & 1 == 0 { 1.0 } else { -1.0 };
            let phase =
                side * (0.82 + leaf_unit_draw(streams::SPIRAL) * 0.28) + leaf_index as f32 * 0.32;
            let radial = (frame_right * cosf(phase) + frame_up * sinf(phase)).normalize();
            let leaf_up = (radial * 0.72 + direction * 0.48 + Vec3::Y * 0.16).normalize();
            let azimuth_normal = direction.cross(radial).normalize_or_zero();
            let azimuth_normal = if azimuth_normal.length_squared() > 0.25 {
                azimuth_normal
            } else {
                frame_up
            };
            // Hazel blades are generally held obliquely upward rather than as
            // vertical fins around an upright shoot. Bias the generated plane
            // normal toward the sky while retaining azimuthal variation, then
            // project it perpendicular to the blade's midrib. This lets the
            // ordinary PBR response to the sun light the shrub naturally.
            let posture_normal = (Vec3::Y * 0.82 + azimuth_normal * 0.38).normalize();
            let right = leaf_up.cross(posture_normal).normalize_or_zero();
            let right = if right.length_squared() > 0.25 {
                right
            } else {
                leaf_up.cross(frame_right).normalize()
            };
            let petiole_start = shoot.start.lerp(shoot.end, along.clamp(0.04, 0.96));
            let petiole_length = parameters.petiole_length_metres[0]
                + leaf_unit_draw(streams::RADIAL_LIFT)
                    * (parameters.petiole_length_metres[1] - parameters.petiole_length_metres[0]);
            let length = parameters.leaf_length_metres[0]
                + leaf_unit_draw(streams::PETIOLE_LENGTH)
                    * (parameters.leaf_length_metres[1] - parameters.leaf_length_metres[0]);
            let width = length
                * (parameters.leaf_width_ratio[0]
                    + leaf_unit_draw(streams::LENGTH)
                        * (parameters.leaf_width_ratio[1] - parameters.leaf_width_ratio[0]));
            let blade_base = petiole_start + radial * petiole_length;
            leaves.push(TreeLeaf {
                petiole_start,
                center: blade_base + leaf_up * length * 0.5,
                right,
                up: leaf_up,
                length,
                width,
                primary_group: shoot.primary_group,
                secondary_group: shoot.secondary_group,
                shoot_id,
                leaf_ordinal: leaf_index as u8,
                shade: 0.68 + leaf_unit_draw(streams::WIDTH) * 0.24,
                torsion: (leaf_unit_draw(streams::SHADE) - 0.5) * 0.28,
            });
        }
    }
    leaves
}
