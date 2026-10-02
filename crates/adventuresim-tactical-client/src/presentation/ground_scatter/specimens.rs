//! Resolved woody specimens shared by wild and property-owned planting.
use super::*;
use adventuresim_tactical_core::city_layout::GardenSpecimen;

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
            0x00b1_ac7a_0e31_u64,
            BLACKTHORN_PARAMETERS,
            Color::srgb_u8(61, 52, 44),
            blackthorn_leaf_material(procedural_assets),
        ),
        (
            &mut cache.hawthorn,
            0x00a7_a74a_0e51_u64,
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
        let mut branch_mesh = procedural_woody_branch_mesh(&branches, 3);
        // Tree bark uses COLOR for root-height geometry. Plain shrub bark must
        // not multiply that payload into its pigment.
        branch_mesh.remove_attribute(Mesh::ATTRIBUTE_COLOR);
        cache.branches = Some(meshes.add(branch_mesh));
        cache.cambered_leaves = Some(meshes.add(procedural_woody_cambered_leaf_mesh(&leaves)));
        // A single minimal card tier replaces the former full-card and far
        // sparse-card tiers. It carries the close shrub silhouette only until
        // the terrain and distant canopy can take over.
        cache.minimal_leaf_cards =
            Some(meshes.add(procedural_woody_sparse_leaf_card_mesh(&leaves)));
        // Full-coverage cards for the instanced shrub renderer's leaf-card tier.
        cache.leaf_cards = Some(meshes.add(procedural_woody_leaf_card_mesh(&leaves)));
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
    fn managed_hazel_matches_shared_envelope() {
        let envelope = GardenSpecimen::CommonHazel.envelope();
        let seed = envelope.seed;
        let branches = procedural_woody_plant_skeleton(seed, 0.0, COMMON_HAZEL_PARAMETERS);
        let leaves = procedural_woody_plant_leaves(seed, &branches, 0.0, COMMON_HAZEL_PARAMETERS);
        let meshes = [
            procedural_woody_branch_mesh(&branches, 3),
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
