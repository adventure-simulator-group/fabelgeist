//! Count resident tree representations independently from generation work.
use bevy::prelude::*;

/// Active-scene accounting for procedurally generated playable-tree assets.
///
/// Unlike `Assets` totals, these counters separate the representations that
/// the tree streamer requested. The scene benchmark records this resource so
/// a faster frame cannot hide an accidental increase in active geometry.
/// Previous scenes retain additional assets, reported in separate counters.
#[derive(Resource, Debug, Clone, Default, serde::Serialize)]
pub(crate) struct TreeAssetResidencyDiagnostics {
    pub(crate) retained_scenes: usize,
    pub(crate) retained_variants: usize,
    pub(crate) retained_impostor_texture_bytes: usize,
    pub(crate) variants: usize,
    pub(crate) source_branches: usize,
    pub(crate) source_leaves: usize,
    /// Full-detail trunk and root vertices retained for near views.
    pub(crate) detailed_trunk_vertices: usize,
    /// Low-poly upright-bole vertices retained for the mid-distance handoff.
    pub(crate) mid_trunk_vertices: usize,
    pub(crate) detailed_branch_vertices: usize,
    pub(crate) cambered_leaf_vertices: usize,
    /// Source detailed leaves considered when building streamed flat-card
    /// clusters. This makes deterministic card thinning auditable.
    pub(crate) leaf_card_source_leaves: usize,
    /// Flat cards retained from `leaf_card_source_leaves` after deterministic
    /// per-shoot thinning.
    pub(crate) leaf_card_retained_leaves: usize,
    pub(crate) leaf_card_vertices: usize,
    pub(crate) bud_vertices: usize,
    pub(crate) aggregate_branch_vertices: usize,
    /// Aggregate live-wood geometry in LOD1/LOD2 order. This keeps the two
    /// deliberately different tube budgets attributable in benchmark output.
    pub(crate) aggregate_branch_vertices_by_lod: [usize; 2],
    pub(crate) aggregate_branch_triangles_by_lod: [usize; 2],
    /// LOD1 is the near aggregate canopy. Tracking it separately makes the
    /// macro-cluster card reduction visible in the scene benchmark rather than
    /// burying it in the combined impostor total.
    pub(crate) lod1_impostor_cards: usize,
    pub(crate) lod1_impostor_vertices: usize,
    pub(crate) impostor_vertices: usize,
    /// Uncompressed RGBA8 atlas bytes by aggregate LOD, in LOD1..LOD4 order.
    /// The total remains for existing report consumers, while this split makes
    /// tile-resolution regressions attributable to an individual LOD.
    pub(crate) impostor_texture_bytes_by_lod: [usize; 4],
    pub(crate) impostor_texture_bytes: usize,
    pub(crate) generated_lod_mask: u8,
    /// Cumulative generation work for this scene, including before restoration.
    pub(crate) generation_milliseconds: u128,
}

impl TreeAssetResidencyDiagnostics {
    pub(super) fn add_resident_assets(&mut self, added: &Self) {
        self.variants += added.variants;
        self.source_branches += added.source_branches;
        self.source_leaves += added.source_leaves;
        self.detailed_trunk_vertices += added.detailed_trunk_vertices;
        self.mid_trunk_vertices += added.mid_trunk_vertices;
        self.detailed_branch_vertices += added.detailed_branch_vertices;
        self.cambered_leaf_vertices += added.cambered_leaf_vertices;
        self.leaf_card_source_leaves += added.leaf_card_source_leaves;
        self.leaf_card_retained_leaves += added.leaf_card_retained_leaves;
        self.leaf_card_vertices += added.leaf_card_vertices;
        self.bud_vertices += added.bud_vertices;
        self.aggregate_branch_vertices += added.aggregate_branch_vertices;
        for index in 0..self.aggregate_branch_vertices_by_lod.len() {
            self.aggregate_branch_vertices_by_lod[index] +=
                added.aggregate_branch_vertices_by_lod[index];
        }
        for index in 0..self.aggregate_branch_triangles_by_lod.len() {
            self.aggregate_branch_triangles_by_lod[index] +=
                added.aggregate_branch_triangles_by_lod[index];
        }
        self.lod1_impostor_cards += added.lod1_impostor_cards;
        self.lod1_impostor_vertices += added.lod1_impostor_vertices;
        self.impostor_vertices += added.impostor_vertices;
        for index in 0..self.impostor_texture_bytes_by_lod.len() {
            self.impostor_texture_bytes_by_lod[index] += added.impostor_texture_bytes_by_lod[index];
        }
        self.impostor_texture_bytes += added.impostor_texture_bytes;
        self.generated_lod_mask |= added.generated_lod_mask;
    }
}
