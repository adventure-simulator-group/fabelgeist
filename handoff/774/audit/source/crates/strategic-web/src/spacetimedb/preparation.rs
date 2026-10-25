//! Generated ingredient vocabulary conversion at the database boundary.

use adventuresim_core::ingredient_preparation::IngredientPreparationAction as DomainAction;
use adventuresim_stdb_client::IngredientPreparationAction;

pub(crate) fn ingredient_preparation_action(action: IngredientPreparationAction) -> DomainAction {
    match action {
        IngredientPreparationAction::Cut => DomainAction::Cut,
        IngredientPreparationAction::Grind => DomainAction::Grind,
    }
}

pub(crate) fn parse_ingredient_preparation_action(
    value: &str,
) -> Option<IngredientPreparationAction> {
    Some(match DomainAction::parse(value)? {
        DomainAction::Cut => IngredientPreparationAction::Cut,
        DomainAction::Grind => IngredientPreparationAction::Grind,
    })
}
