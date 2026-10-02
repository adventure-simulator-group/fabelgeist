use super::*;
use crate::BuildingLodMaterial;

#[test]
fn finish_keys_roundtrip_without_aliases_or_unsupported_states() {
    let keys = FurnitureKey::ALL
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(keys.len(), FurnitureKey::ALL.len());
    for key in keys {
        let encoded = serde_json::to_string(&key).unwrap();
        assert_eq!(serde_json::from_str::<FurnitureKey>(&encoded).unwrap(), key);
    }
    for invalid in [
        r#"{"kind":"baptismal_font","variant":"compact","wood_state":"painted"}"#,
        r#"{"kind":"chair","variant":"compact"}"#,
        r#"{"kind":"chair","variant":"compact","wood_state":"natural","extra":1}"#,
    ] {
        assert!(serde_json::from_str::<FurnitureKey>(invalid).is_err());
    }
}

#[test]
fn finishes_preserve_physical_envelopes_and_replace_only_authored_surfaces() {
    for kind in FinishableFurnitureKind::ALL {
        for variant in FurnitureVariant::ALL {
            let natural = FurnitureKey::natural(kind.kind(), variant).recipe();
            for state in FurnitureWoodState::ALL {
                let key = FurnitureKey::wood(kind, variant, state);
                let recipe = key.recipe();
                assert_eq!(recipe.bounds.min, natural.bounds.min, "{key:?}");
                assert_eq!(recipe.bounds.max, natural.bounds.max, "{key:?}");
                assert_eq!(
                    recipe.support_points_metres, natural.support_points_metres,
                    "{key:?}"
                );
                assert_eq!(
                    serde_json::to_value(&recipe.colliders).unwrap(),
                    serde_json::to_value(&natural.colliders).unwrap(),
                    "{key:?}"
                );
                assert_eq!(
                    format!("{:?}", recipe.clearances),
                    format!("{:?}", natural.clearances),
                    "{key:?}"
                );
                let expected = match state {
                    FurnitureWoodState::Natural => None,
                    FurnitureWoodState::Handled => Some(FurnitureWoodSurface::Handled),
                    FurnitureWoodState::Repaired => Some(FurnitureWoodSurface::Replacement),
                    FurnitureWoodState::Painted => Some(FurnitureWoodSurface::Painted),
                };
                if let Some(surface) = expected {
                    assert!(
                        recipe.meshes.iter().any(
                            |mesh| mesh.material == BuildingLodMaterial::FurnitureWood(surface)
                        ),
                        "{key:?}"
                    );
                    assert!(!std::ptr::eq(recipe, natural));
                }
                for mesh in &recipe.meshes {
                    for triangle in mesh.indices.as_chunks::<3>().0 {
                        let vertices = triangle.map(|index| mesh.vertices[index as usize]);
                        let normal = (vertices[1].position - vertices[0].position)
                            .cross(vertices[2].position - vertices[0].position);
                        assert!(
                            normal.dot(vertices[0].normal) > 0.0,
                            "{key:?}: inverted finish face"
                        );
                    }
                }
            }
        }
    }
}
