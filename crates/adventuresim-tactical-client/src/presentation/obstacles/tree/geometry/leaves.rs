use super::{TreeBranchSegment, WoodyPlantForm, WoodyPlantParameters, branch_frame};
pub(in crate::presentation) use arrangement::{
    procedural_oak_leaves, procedural_woody_plant_leaves,
};
use bevy::{
    asset::RenderAssetUsages,
    math::{Quat, Vec3, Vec3Swizzles},
    mesh::{Indices, PrimitiveTopology},
    prelude::Mesh,
};
use fabelgeist_determinism::Seed;
pub(in crate::presentation) use identity::TreeShootId;
pub(in crate::presentation) use selection::detailed_flat_card_group_leaves;
use selection::sparse_woody_far_card_leaves;
#[cfg(test)]
use selection::{detailed_flat_card_leaf_retained, sparse_woody_far_card_retained};

mod arrangement;
mod identity;
mod selection;
mod streams;

#[derive(Clone, Copy, Debug)]
pub(in crate::presentation) struct TreeLeaf {
    pub(in crate::presentation) petiole_start: Vec3,
    pub(in crate::presentation) center: Vec3,
    pub(in crate::presentation) right: Vec3,
    pub(in crate::presentation) up: Vec3,
    pub(in crate::presentation) length: f32,
    pub(in crate::presentation) width: f32,
    pub(in crate::presentation) primary_group: u8,
    pub(in crate::presentation) secondary_group: u16,
    pub(in crate::presentation) shoot_id: TreeShootId,
    /// Stable ordinal within the source shoot. This is deliberately stored on
    /// the leaf rather than inferred from a mesh-building iteration order so
    /// representation LODs can make deterministic per-leaf choices.
    pub(in crate::presentation) leaf_ordinal: u8,
    pub(in crate::presentation) shade: f32,
    /// Rotation accumulated from the base to the tip of the blade. Real oak
    /// leaves rarely present as perfectly planar cards, even in still air.
    pub(in crate::presentation) torsion: f32,
}

pub(in crate::presentation) fn oak_leaf_card_bounds(leaf: TreeLeaf) -> (Vec3, f32, f32) {
    let blade_tip = leaf.center + leaf.up * leaf.length * 0.5;
    let bottom = leaf.petiole_start.dot(leaf.up);
    let top = blade_tip.dot(leaf.up);
    let height = (top - bottom).max(leaf.length) * 1.04;
    let center = leaf.center + leaf.up * (((top + bottom) * 0.5) - leaf.center.dot(leaf.up));
    (center, leaf.width * 1.08, height)
}

fn leaf_shadow_selector(leaf: TreeLeaf) -> f32 {
    let shoot_key = leaf.shoot_id;
    streams::SHOOT_THRESHOLD
        .rng(shoot_key.seed(), &[])
        .inclusive_unit_f32()
}

/// Replaces every cambered production leaf with one alpha-masked quad while
/// retaining its biological attachment, orientation, scale, and wind UV.
pub(in crate::presentation) fn procedural_oak_leaf_card_mesh(leaves: &[TreeLeaf]) -> Mesh {
    procedural_woody_leaf_card_mesh(leaves)
}

pub(in crate::presentation) fn procedural_woody_leaf_card_mesh(leaves: &[TreeLeaf]) -> Mesh {
    procedural_woody_leaf_card_mesh_scaled(leaves, 1.0)
}

/// Builds the terminal shrub representation from one deterministic ordinal
/// lane per shoot. The cards grow into overlapping leaf clusters, so this is
/// substantially cheaper than retaining a card for every source leaf while
/// keeping the outline of each shrub species recognisable at distance.
pub(in crate::presentation) fn procedural_woody_sparse_leaf_card_mesh(leaves: &[TreeLeaf]) -> Mesh {
    let sparse_leaves = sparse_woody_far_card_leaves(leaves);
    procedural_woody_leaf_card_mesh_scaled(&sparse_leaves, 1.9)
}

fn procedural_woody_leaf_card_mesh_scaled(leaves: &[TreeLeaf], coverage_scale: f32) -> Mesh {
    let mut positions = Vec::with_capacity(leaves.len() * 4);
    let mut normals = Vec::with_capacity(leaves.len() * 4);
    let mut uvs = Vec::with_capacity(leaves.len() * 4);
    let mut colors = Vec::with_capacity(leaves.len() * 4);
    let mut indices = Vec::with_capacity(leaves.len() * 6);
    for leaf in leaves {
        let (mut center, width, height) = oak_leaf_card_bounds(*leaf);
        // A single plane loses the accepted leaf's projected camber when seen
        // obliquely. Enlarge about the fixed petiole (not the card centre) so
        // the intermediate LOD preserves crown coverage without swimming at
        // its biological attachment.
        const CARD_COVERAGE_SCALE: f32 = 1.24;
        let scaled_width = width * CARD_COVERAGE_SCALE * coverage_scale;
        let scaled_height = height * CARD_COVERAGE_SCALE * coverage_scale;
        center += leaf.up * (scaled_height - height) * 0.5;
        let right = leaf.right * scaled_width * 0.5;
        let up = leaf.up * scaled_height * 0.5;
        let normal = leaf.right.cross(leaf.up).normalize();
        let base = positions.len() as u32;
        positions.extend_from_slice(&[
            (center - right - up).to_array(),
            (center + right - up).to_array(),
            (center + right + up).to_array(),
            (center - right + up).to_array(),
        ]);
        normals.extend_from_slice(&[normal.to_array(); 4]);
        // Image-space V grows downward. Keep the generated petiole at the
        // biological attachment and the blade tip at the distal end.
        uvs.extend_from_slice(&[[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]]);
        let shade = (leaf.shade / 0.82).clamp(0.72, 1.18);
        let ambient_visibility = ((leaf.shade - 0.52) / 0.44).clamp(0.32, 1.0);
        let shadow_selector = leaf_shadow_selector(*leaf);
        colors.extend_from_slice(&[[shade, shade, shadow_selector, ambient_visibility]; 4]);
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

/// The near leaf is a small cambered grid: the geometry retains fold, cupping,
/// torsion, and grazing-angle area while the generated opacity texture owns its
/// fine lobed silhouette. It transitions directly to the flat card.
pub(in crate::presentation) fn procedural_oak_textured_leaf_mesh(leaves: &[TreeLeaf]) -> Mesh {
    procedural_woody_cambered_leaf_mesh(leaves)
}

pub(in crate::presentation) fn procedural_woody_cambered_leaf_mesh(leaves: &[TreeLeaf]) -> Mesh {
    const GRID: u32 = 3;
    const VERTICES_PER_LEAF: usize = (GRID * GRID) as usize;
    const INDICES_PER_LEAF: usize = ((GRID - 1) * (GRID - 1) * 6) as usize;
    let mut positions = Vec::with_capacity(leaves.len() * VERTICES_PER_LEAF);
    let mut normals = Vec::with_capacity(leaves.len() * VERTICES_PER_LEAF);
    let mut uvs = Vec::with_capacity(leaves.len() * VERTICES_PER_LEAF);
    let mut colors = Vec::with_capacity(leaves.len() * VERTICES_PER_LEAF);
    let mut indices = Vec::with_capacity(leaves.len() * INDICES_PER_LEAF);
    for leaf in leaves {
        let (mut center, width, height) = oak_leaf_card_bounds(*leaf);
        const COVERAGE_SCALE: f32 = 1.10;
        let scaled_width = width * COVERAGE_SCALE;
        let scaled_height = height * COVERAGE_SCALE;
        center += leaf.up * (scaled_height - height) * 0.5;
        let shade = (leaf.shade / 0.82).clamp(0.72, 1.18);
        let ambient_visibility = ((leaf.shade - 0.52) / 0.44).clamp(0.32, 1.0);
        let shadow_selector = leaf_shadow_selector(*leaf);
        let base = positions.len() as u32;
        let curl_sign = if leaf.torsion.is_sign_negative() {
            -1.0
        } else {
            1.0
        };
        for row in 0..GRID {
            let v = row as f32 / (GRID - 1) as f32;
            // Even an almost-untwisted source leaf needs enough geometric
            // change to justify this near representation over the flat card.
            // Accumulate a small asymmetric twist toward the tip while the
            // source torsion keeps every leaf from curling identically.
            let twist_angle = (leaf.torsion * 1.15 + curl_sign * 0.08) * (v - 0.20);
            let twist = Quat::from_axis_angle(leaf.up, twist_angle);
            let cross_right = (twist * leaf.right).normalize();
            let cross_normal = cross_right.cross(leaf.up).normalize();
            for column in 0..GRID {
                let u = column as f32 / (GRID - 1) as f32;
                let side = (u - 0.5) * 2.0;
                let lateral = side * scaled_width * 0.5;
                let length_profile = (core::f32::consts::PI * v).sin();
                let midrib_ridge = length_profile * (1.0 - side.abs()) * leaf.width * 0.08;
                let margin_cup = length_profile * side.abs() * leaf.width * 0.11;
                let tip_curl = v * v * v * scaled_height * 0.035 * curl_sign;
                positions.push(
                    (center
                        + leaf.up * (v - 0.5) * scaled_height
                        + cross_right * lateral
                        + cross_normal * (midrib_ridge - margin_cup + tip_curl))
                        .to_array(),
                );
                normals.push(
                    (cross_normal + cross_right * -side * 0.52 - leaf.up * v * curl_sign * 0.08)
                        .normalize()
                        .to_array(),
                );
                uvs.push([u, 1.0 - v]);
                colors.push([shade, shade, shadow_selector, ambient_visibility]);
            }
        }
        for row in 0..(GRID - 1) {
            for column in 0..(GRID - 1) {
                let lower_left = base + row * GRID + column;
                let lower_right = lower_left + 1;
                let upper_left = lower_left + GRID;
                let upper_right = upper_left + 1;
                indices.extend_from_slice(&[
                    lower_left,
                    lower_right,
                    upper_right,
                    lower_left,
                    upper_right,
                    upper_left,
                ]);
            }
        }
    }
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

pub(in crate::presentation) fn procedural_oak_leaf_card_group_mesh(
    leaves: &[TreeLeaf],
    primary_group: u8,
) -> Mesh {
    let group_leaves = detailed_flat_card_group_leaves(leaves, primary_group);
    procedural_woody_leaf_card_mesh(&group_leaves)
}

pub(in crate::presentation) fn procedural_oak_textured_leaf_group_mesh(
    leaves: &[TreeLeaf],
    primary_group: u8,
) -> Mesh {
    let group_leaves = leaves
        .iter()
        .filter(|leaf| leaf.primary_group == primary_group)
        .copied()
        .collect::<Vec<_>>();
    procedural_woody_cambered_leaf_mesh(&group_leaves)
}

/// Models the compact, scaled winter bud at every current-year shoot tip.
/// It is a separate production mesh so its warm color and overlapping scale
/// silhouette remain legible instead of disappearing into the bark tube cap.
pub(in crate::presentation) fn procedural_oak_bud_mesh(branches: &[TreeBranchSegment]) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    const SIDES: u32 = 6;
    const BUD_LENGTH: f32 = 0.008;
    for branch in branches.iter().filter(|branch| branch.foliage.is_some()) {
        let direction = (branch.end - branch.start).normalize();
        let (frame_right, frame_forward) = branch_frame(direction);
        let base = positions.len() as u32;
        let rings = [(0.0_f32, 0.0018_f32), (0.38, 0.0038), (0.78, 0.0025)];
        for (ring_index, (along, radius)) in rings.into_iter().enumerate() {
            let center = branch.end + direction * (along * BUD_LENGTH);
            let phase_offset = if ring_index & 1 == 0 { 0.0 } else { 0.22 };
            for side in 0..SIDES {
                let phase = side as f32 * core::f32::consts::TAU / SIDES as f32 + phase_offset;
                // Alternating ridges suggest overlapping protective scales at
                // the close-review distance without inflating the tiny bud.
                let scale_ridge = if (side + ring_index as u32) & 1 == 0 {
                    1.12
                } else {
                    0.94
                };
                let radial = frame_right * phase.cos() + frame_forward * phase.sin();
                positions.push((center + radial * radius * scale_ridge).to_array());
                normals.push((radial * 0.86 + direction * 0.18).normalize().to_array());
                uvs.push([side as f32 / SIDES as f32, along]);
            }
        }
        for ring in 0..rings.len() as u32 - 1 {
            let from = base + ring * SIDES;
            let to = from + SIDES;
            for side in 0..SIDES {
                let next = (side + 1) % SIDES;
                indices.extend_from_slice(&[
                    from + side,
                    to + side,
                    to + next,
                    from + side,
                    to + next,
                    from + next,
                ]);
            }
        }
        let tip = positions.len() as u32;
        positions.push((branch.end + direction * BUD_LENGTH).to_array());
        normals.push(direction.to_array());
        uvs.push([0.5, 1.0]);
        let last_ring = base + (rings.len() as u32 - 1) * SIDES;
        for side in 0..SIDES {
            let next = (side + 1) % SIDES;
            indices.extend_from_slice(&[tip, last_ring + side, last_ring + next]);
        }
    }
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

pub(in crate::presentation) fn procedural_oak_bud_group_mesh(
    branches: &[TreeBranchSegment],
    primary_group: u8,
) -> Mesh {
    let group = branches
        .iter()
        .filter(|branch| branch.primary_group == primary_group)
        .copied()
        .collect::<Vec<_>>();
    procedural_oak_bud_mesh(&group)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::obstacles::tree::geometry::{
        TREE_PRIMARY_GROUP_COUNT, procedural_tree_branch_mesh, procedural_tree_skeleton,
    };

    #[test]
    fn production_oak_has_finite_cambered_leaf_geometry() {
        let branches = procedural_tree_skeleton(fabelgeist_determinism::Seed::from_u64(42), 0.0);
        let leaves =
            procedural_oak_leaves(fabelgeist_determinism::Seed::from_u64(42), &branches, 0.0);
        assert!((45_000..60_000).contains(&leaves.len()));
        assert!(leaves.iter().all(|leaf| leaf.petiole_start.is_finite()
            && leaf.center.is_finite()
            && leaf.right.is_finite()
            && leaf.up.is_finite()
            && leaf.right.length_squared() > 0.9
            && leaf.up.length_squared() > 0.9
            && leaf.right.cross(leaf.up).length_squared() > 0.5
            && leaf.torsion.is_finite()));
        let mesh = procedural_oak_textured_leaf_mesh(&leaves);
        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .and_then(|attribute| attribute.as_float3())
            .expect("leaf mesh has float positions");
        assert_eq!(positions.len(), leaves.len() * 9);
        assert_eq!(
            mesh.indices().expect("leaf mesh has indices").len() / 3,
            leaves.len() * 8
        );
        assert!(
            positions
                .iter()
                .flatten()
                .all(|component| component.is_finite())
        );
        let buds = procedural_oak_bud_mesh(&branches);
        let bud_positions = buds
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .and_then(|attribute| attribute.as_float3())
            .expect("bud mesh has float positions");
        let terminal_shoots = branches
            .iter()
            .filter(|branch| branch.foliage.is_some())
            .count();
        assert_eq!(terminal_shoots, leaves.len() / 16);
        assert_eq!(bud_positions.len(), terminal_shoots * 19);
        assert_eq!(
            buds.indices().expect("bud mesh has indices").len() / 3,
            terminal_shoots * 30
        );
        assert!(
            bud_positions
                .iter()
                .flatten()
                .all(|component| component.is_finite())
        );
        let leaf_triangles = mesh.indices().expect("leaf mesh has indices").len() / 3;
        let bud_triangles = buds.indices().expect("bud mesh has indices").len() / 3;
        let branch_triangles = procedural_tree_branch_mesh(&branches, 3)
            .indices()
            .expect("branch mesh has indices")
            .len()
            / 3;
        assert!(leaf_triangles + bud_triangles + branch_triangles <= 3_600_000);
        let group_triangles = (0..TREE_PRIMARY_GROUP_COUNT)
            .map(|group| {
                procedural_oak_textured_leaf_group_mesh(&leaves, group)
                    .indices()
                    .expect("sector has indices")
                    .len()
                    / 3
            })
            .sum::<usize>();
        assert_eq!(group_triangles, leaf_triangles);
    }

    #[test]
    fn alpha_leaf_lod_uses_exactly_two_triangles_per_leaf() {
        let branches = procedural_tree_skeleton(fabelgeist_determinism::Seed::from_u64(42), 0.0);
        let leaves =
            procedural_oak_leaves(fabelgeist_determinism::Seed::from_u64(42), &branches, 0.0);
        let mesh = procedural_oak_leaf_card_mesh(&leaves);
        assert_eq!(mesh.count_vertices(), leaves.len() * 4);
        assert_eq!(
            mesh.indices().expect("leaf cards are indexed").len(),
            leaves.len() * 6
        );
    }

    #[test]
    fn detailed_flat_cards_keep_a_stable_balanced_three_quarters_per_cluster() {
        let branches = procedural_tree_skeleton(fabelgeist_determinism::Seed::from_u64(42), 0.0);
        let leaves =
            procedural_oak_leaves(fabelgeist_determinism::Seed::from_u64(42), &branches, 0.0);
        let source_count = leaves.len();
        let mut retained_count = 0;

        for primary_group in 0..TREE_PRIMARY_GROUP_COUNT {
            let source = leaves
                .iter()
                .filter(|leaf| leaf.primary_group == primary_group)
                .count();
            let retained = detailed_flat_card_group_leaves(&leaves, primary_group);
            let repeated = detailed_flat_card_group_leaves(&leaves, primary_group);
            let reordered = leaves.iter().rev().copied().collect::<Vec<_>>();
            let reordered_retained = detailed_flat_card_group_leaves(&reordered, primary_group);
            assert_eq!(retained.len(), source * 3 / 4);
            assert_eq!(retained.len(), repeated.len());
            assert!(
                retained
                    .iter()
                    .zip(repeated.iter())
                    .all(|(left, right)| left.shoot_id == right.shoot_id
                        && left.leaf_ordinal == right.leaf_ordinal)
            );
            assert!(
                retained
                    .iter()
                    .zip(reordered_retained.iter())
                    .all(|(left, right)| left.shoot_id == right.shoot_id
                        && left.leaf_ordinal == right.leaf_ordinal)
            );
            let mesh = procedural_oak_leaf_card_group_mesh(&leaves, primary_group);
            assert_eq!(mesh.count_vertices(), retained.len() * 4);
            assert_eq!(
                mesh.indices().expect("flat cards are indexed").len(),
                retained.len() * 6
            );
            assert!(
                mesh.attribute(Mesh::ATTRIBUTE_POSITION)
                    .and_then(|attribute| attribute.as_float3())
                    .expect("flat cards have positions")
                    .iter()
                    .flatten()
                    .all(|value| value.is_finite())
            );
            assert!(
                mesh.indices()
                    .expect("flat cards are indexed")
                    .iter()
                    .all(|index| index < mesh.count_vertices())
            );
            retained_count += retained.len();
        }

        assert_eq!(source_count, 50_784);
        assert_eq!(retained_count, 38_088);
    }

    #[test]
    fn textured_leaf_lod_uses_exactly_eight_triangles_per_leaf() {
        let branches = procedural_tree_skeleton(fabelgeist_determinism::Seed::from_u64(42), 0.0);
        let leaves =
            procedural_oak_leaves(fabelgeist_determinism::Seed::from_u64(42), &branches, 0.0);
        let mesh = procedural_oak_textured_leaf_mesh(&leaves);
        assert_eq!(mesh.count_vertices(), leaves.len() * 9);
        assert_eq!(
            mesh.indices().expect("textured leaves are indexed").len(),
            leaves.len() * 24
        );
    }

    #[test]
    fn leaf_shadow_transmission_is_stable_per_shoot_and_well_distributed() {
        let branches = procedural_tree_skeleton(fabelgeist_determinism::Seed::from_u64(42), 0.0);
        let leaves =
            procedural_oak_leaves(fabelgeist_determinism::Seed::from_u64(42), &branches, 0.0);
        let mut shoots = std::collections::BTreeMap::new();
        for leaf in leaves {
            let key = (leaf.primary_group, leaf.secondary_group, leaf.shoot_id);
            let selector = leaf_shadow_selector(leaf);
            let previous = shoots.entry(key).or_insert(selector);
            assert_eq!(*previous, selector);
        }
        let transmitting = shoots.values().filter(|selector| **selector < 0.42).count();
        let fraction = transmitting as f32 / shoots.len() as f32;
        assert!((0.37..=0.47).contains(&fraction));
    }
}
