//! Ingredient operations shared by food and medicinal material planning.

use serde::{Deserialize, Serialize};

/// The requested operation, distinct from the resulting material preparation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb::SpacetimeType))]
pub enum IngredientPreparationAction {
    Cut,
    Grind,
}

impl IngredientPreparationAction {
    const CUT_ID: &'static str = "cut";
    const GRIND_ID: &'static str = "grind";
    pub const fn stable_id(self) -> &'static str {
        match self {
            Self::Cut => Self::CUT_ID,
            Self::Grind => Self::GRIND_ID,
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            Self::CUT_ID => Some(Self::Cut),
            Self::GRIND_ID => Some(Self::Grind),
            _ => None,
        }
    }

    /// Stable byte in versioned request, attempt, and authority hash frames.
    pub const fn stable_code(self) -> u8 {
        match self {
            Self::Cut => 1,
            Self::Grind => 2,
        }
    }

    pub const fn definition_id(self) -> &'static str {
        match self {
            Self::Cut => "ingredient-preparation:cut",
            Self::Grind => "ingredient-preparation:grind",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boundary_vocabulary_round_trips_and_rejects_unknown_operations() {
        for action in [
            IngredientPreparationAction::Cut,
            IngredientPreparationAction::Grind,
        ] {
            assert_eq!(
                IngredientPreparationAction::parse(action.stable_id()),
                Some(action)
            );
            let encoded = serde_json::to_string(&action).unwrap();
            assert_eq!(
                serde_json::from_str::<IngredientPreparationAction>(&encoded).unwrap(),
                action
            );
        }
        for unknown in ["", "Cut", "ground", "bake"] {
            assert_eq!(IngredientPreparationAction::parse(unknown), None);
        }
        assert!(serde_json::from_str::<IngredientPreparationAction>("\"Bake\"").is_err());
    }
}
