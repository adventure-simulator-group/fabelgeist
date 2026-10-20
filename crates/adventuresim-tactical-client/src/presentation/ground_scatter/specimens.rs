//! Resolved woody specimens shared by wild and property-owned planting.
use super::*;
use adventuresim_tactical_core::city_layout::GardenSpecimen;
use bevy::camera::visibility::VisibilityRange;
use std::ops::Range;

/// Close shrub wood carries twigs and full-detail stems; beyond this band a
/// twig is under a pixel wide and the aggregate stems take over.
const SHRUB_NEAR_WOOD_HANDOFF_METRES: Range<f32> = 12.0..16.0;
/// Cambered leaves only read as curved surfaces at arm's length; past this
/// band every leaf continues as a flat alpha card.
const SHRUB_CAMBERED_LEAF_HANDOFF_METRES: Range<f32> = 4.0..6.0;
/// Individual leaf cards hand over to enlarged per-shoot cluster cards once a
/// leaf spans only a few pixels; the cluster outline keeps the species read.
const SHRUB_SPARSE_LEAF_HANDOFF_METRES: Range<f32> = 26.0..34.0;
const SHRUB_WOOD_RETIREMENT_METRES: Range<f32> = 44.0..52.0;
const SHRUB_LEAF_RETIREMENT_METRES: Range<f32> = 84.0..96.0;

/// Distance-banded shrub representations shared by wild understory and
/// property gardens. Adjacent bands share endpoints so complementary
/// dithering hands one representation to the next exactly.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ShrubRepresentation {
    NearWood,
    FarWood,
    CamberedLeaves,
    LeafCards,
    SparseLeafCards,
}

impl ShrubRepresentation {
    pub(super) const ALL: [Self; 5] = [
        Self::NearWood,
        Self::FarWood,
        Self::CamberedLeaves,
        Self::LeafCards,
        Self::SparseLeafCards,
    ];

    /// Fade-in and fade-out margins in metres from the camera.
    fn margins(self) -> (Range<f32>, Range<f32>) {
        match self {
            Self::NearWood => (0.0..0.0, SHRUB_NEAR_WOOD_HANDOFF_METRES),
            Self::FarWood => (SHRUB_NEAR_WOOD_HANDOFF_METRES, SHRUB_WOOD_RETIREMENT_METRES),
            Self::CamberedLeaves => (0.0..0.0, SHRUB_CAMBERED_LEAF_HANDOFF_METRES),
            Self::LeafCards => (
                SHRUB_CAMBERED_LEAF_HANDOFF_METRES,
                SHRUB_SPARSE_LEAF_HANDOFF_METRES,
            ),
            Self::SparseLeafCards => (
                SHRUB_SPARSE_LEAF_HANDOFF_METRES,
                SHRUB_LEAF_RETIREMENT_METRES,
            ),
        }
    }

    pub(super) fn visibility_range(self) -> VisibilityRange {
        let (start_margin, end_margin) = self.margins();
        VisibilityRange {
            start_margin,
            end_margin,
            use_aabb: false,
        }
    }

    /// Eidolon's per-instance band packs both margins into one vector. Its
    /// fade-in divides by the margin width, so an always-visible start keeps
    /// a millimetre-wide margin.
    pub(super) fn instance_range(self) -> Vec4 {
        let (start, end) = self.margins();
        Vec4::new(
            start.start,
            start.end.max(start.start + 0.001),
            end.start,
            end.end,
        )
    }

    pub(super) fn mesh(self, presentation: &WoodyUnderstoryPresentation) -> Option<Handle<Mesh>> {
        match self {
            Self::NearWood => presentation.branches.clone(),
            Self::FarWood => presentation.far_branches.clone(),
            Self::CamberedLeaves => presentation.cambered_leaves.clone(),
            Self::LeafCards => presentation.leaf_cards.clone(),
            Self::SparseLeafCards => presentation.sparse_leaf_cards.clone(),
        }
    }

    pub(super) fn is_wood(self) -> bool {
        matches!(self, Self::NearWood | Self::FarWood)
    }

    pub(super) fn label(self) -> &'static str {
        match self {
            Self::NearWood => "near wood",
            Self::FarWood => "far wood",
            Self::CamberedLeaves => "cambered leaves",
            Self::LeafCards => "leaf cards",
            Self::SparseLeafCards => "sparse leaf cards",
        }
    }
}

pub(super) fn ensure_understory_presentations(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    leaf_materials: &mut Assets<TacticalTreeLeafCardMaterial>,
    cache: &mut WoodyUnderstoryPresentationCache,
    procedural_assets: &ProceduralTextureAssets,
) {
    if cache.hazel.branches.is_some() {
        return;
    }
    // One deterministic specimen is shared by every scattered shrub. Instance
    // transforms still vary placement, rotation, and scale without generating
    // unique botanical geometry per occurrence.
    let species = [
        (
            &mut cache.hazel,
            GardenSpecimen::CommonHazel.envelope().seed,
            COMMON_HAZEL_PARAMETERS,
            Color::srgb_u8(118, 104, 78),
            hazel_leaf_material(procedural_assets),
        ),
        (
            &mut cache.blackthorn,
            fabelgeist_determinism::Seed::from_u64(0x00b1_ac7a_0e31_u64),
            BLACKTHORN_PARAMETERS,
            Color::srgb_u8(61, 52, 44),
            blackthorn_leaf_material(procedural_assets),
        ),
        (
            &mut cache.hawthorn,
            fabelgeist_determinism::Seed::from_u64(0x00a7_a74a_0e51_u64),
            COMMON_HAWTHORN_PARAMETERS,
            Color::srgb_u8(91, 76, 60),
            hawthorn_leaf_material(procedural_assets),
        ),
    ];
    for (cache, seed, parameters, bark_color, mut leaf_material) in species {
        leaf_material.parameters.z =
            adventuresim_tactical_core::city_layout::gardens::GARDEN_LEAF_WIND_STRENGTH_METRES;
        let branches = procedural_woody_plant_skeleton(seed, 0.0, parameters);
        let leaves = procedural_woody_plant_leaves(seed, &branches, 0.0, parameters);
        let [mut branch_mesh, mut far_branch_mesh] = [
            procedural_woody_branch_mesh(&branches, 3),
            procedural_woody_aggregate_shrub_mesh(&branches),
        ];
        // Tree bark uses COLOR for root-height geometry. Plain shrub bark must
        // not multiply that payload into its pigment.
        branch_mesh.remove_attribute(Mesh::ATTRIBUTE_COLOR);
        far_branch_mesh.remove_attribute(Mesh::ATTRIBUTE_COLOR);
        cache.branches = Some(meshes.add(branch_mesh));
        cache.far_branches = Some(meshes.add(far_branch_mesh));
        cache.cambered_leaves = Some(meshes.add(procedural_woody_cambered_leaf_mesh(&leaves)));
        cache.leaf_cards = Some(meshes.add(procedural_woody_leaf_card_mesh(&leaves)));
        cache.sparse_leaf_cards = Some(meshes.add(procedural_woody_sparse_leaf_card_mesh(&leaves)));
        cache.bark = Some(materials.add(StandardMaterial {
            base_color: bark_color,
            perceptual_roughness: 0.96,
            ..default()
        }));
        cache.leaves = Some(leaf_materials.add(leaf_material));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shrub_bands_hand_off_without_gaps() {
        let margins = |representation: ShrubRepresentation| representation.margins();
        let wood = [ShrubRepresentation::NearWood, ShrubRepresentation::FarWood];
        let leaves = [
            ShrubRepresentation::CamberedLeaves,
            ShrubRepresentation::LeafCards,
            ShrubRepresentation::SparseLeafCards,
        ];
        for chain in [&wood[..], &leaves[..]] {
            assert_eq!(margins(chain[0]).0, 0.0..0.0);
            for pair in chain.windows(2) {
                assert_eq!(margins(pair[0]).1, margins(pair[1]).0);
            }
        }
        // Wood retires before the leaf cards that hide it.
        assert!(
            margins(ShrubRepresentation::FarWood).1.end
                < margins(ShrubRepresentation::SparseLeafCards).1.end
        );
    }

    #[test]
    fn managed_hazel_matches_shared_envelope() {
        let envelope = GardenSpecimen::CommonHazel.envelope();
        let seed = envelope.seed;
        let branches = procedural_woody_plant_skeleton(seed, 0.0, COMMON_HAZEL_PARAMETERS);
        let leaves = procedural_woody_plant_leaves(seed, &branches, 0.0, COMMON_HAZEL_PARAMETERS);
        let meshes = [
            procedural_woody_branch_mesh(&branches, 3),
            procedural_woody_aggregate_shrub_mesh(&branches),
            procedural_woody_cambered_leaf_mesh(&leaves),
            procedural_woody_leaf_card_mesh(&leaves),
            procedural_woody_sparse_leaf_card_mesh(&leaves),
        ];
        let mut points = Vec::new();
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        for mesh in meshes {
            let Some(bevy::mesh::VertexAttributeValues::Float32x3(vertices)) =
                mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            else {
                panic!("specimen positions missing")
            };
            for p in vertices {
                assert!(Vec3::from_array(*p).is_finite());
                if std::env::var_os("FABELGEIST_UPDATE_GARDEN_SPECIMEN").is_none() {
                    assert!(
                        envelope.contains_local_vertex(Vec3::from_array(*p)),
                        "rendered vertex outside shared specimen envelope: {p:?}"
                    );
                }
                points.push(Vec2::new(p[0], p[2]));
                min_y = min_y.min(p[1]);
                max_y = max_y.max(p[1]);
            }
        }
        let vertex_count = points.len();
        points.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
        points.dedup();
        let chain = |points: Vec<Vec2>| {
            let mut hull: Vec<Vec2> = vec![];
            for point in points {
                while hull.len() >= 2
                    && (hull[hull.len() - 1] - hull[hull.len() - 2])
                        .perp_dot(point - hull[hull.len() - 1])
                        <= 0.0
                {
                    hull.pop();
                }
                hull.push(point);
            }
            hull.pop();
            hull
        };
        let mut hull = chain(points.clone());
        points.reverse();
        hull.extend(chain(points));
        assert!(hull.len() >= 3 && min_y.is_finite() && max_y > min_y);
        if std::env::var_os("FABELGEIST_UPDATE_GARDEN_SPECIMEN").is_some() {
            let document = serde_json::json!({"geometry_revision":envelope.geometry_revision,"seed":seed,"hull_metres":hull,"min_height_metres":min_y,"max_height_metres":max_y,"vertex_count":vertex_count});
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../content/tactical/garden-hazel-envelope.json");
            std::fs::write(
                path,
                serde_json::to_string_pretty(&document).unwrap() + "\n",
            )
            .unwrap();
        } else {
            assert_eq!(
                vertex_count, envelope.vertex_count,
                "a changed specimen requires a reviewed envelope export"
            );
            assert_eq!(hull.len(), envelope.hull_metres.len());
            for (actual, shared) in hull.iter().zip(&envelope.hull_metres) {
                assert!(actual.distance(*shared) < 0.00001);
            }
        }
    }
}
