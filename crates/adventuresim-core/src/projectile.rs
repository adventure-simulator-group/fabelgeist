//! Physical projectile classification across combat and retained injuries.

/// Combat exchanges capture this classification; injury commitment preserves it
/// when the projectile remains in the body. Neither record owns another vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb::SpacetimeType))]
pub enum ProjectileKind {
    Arrowhead,
    Ball,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exchange_serialization_preserves_projectile_identity() {
        for (kind, expected) in [
            (ProjectileKind::Arrowhead, "Arrowhead"),
            (ProjectileKind::Ball, "Ball"),
        ] {
            let encoded = serde_json::to_value(kind).unwrap();
            assert_eq!(encoded, expected);
            assert_eq!(
                serde_json::from_value::<ProjectileKind>(encoded).unwrap(),
                kind
            );
        }
    }
}
