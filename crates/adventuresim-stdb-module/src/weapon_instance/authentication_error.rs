//! Authentication distinguishes invalid recipes from forged stored projections.

use super::error::{WeaponProjectionError, WeaponRecipeKind};
use adventuresim_core::item_catalog::ItemDefinitionId;
use adventuresim_weapon_model::CodecError;

#[derive(Debug)]
pub(crate) enum WeaponAuthenticationError {
    GeneratorVersion(WeaponRecipeKind),
    DigestLength(WeaponRecipeKind),
    RecipeTooLarge(WeaponRecipeKind),
    Decode(CodecError),
    Chassis {
        expected: ItemDefinitionId,
        actual: ItemDefinitionId,
    },
    Projection(WeaponProjectionError),
    ProjectionMismatch(WeaponRecipeKind),
}

impl From<WeaponProjectionError> for WeaponAuthenticationError {
    fn from(source: WeaponProjectionError) -> Self {
        Self::Projection(source)
    }
}

impl std::fmt::Display for WeaponAuthenticationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GeneratorVersion(role) => {
                write!(f, "{role:?} instance uses another generator version")
            }
            Self::DigestLength(role) => {
                write!(f, "{role:?} instance has an invalid design digest length")
            }
            Self::RecipeTooLarge(role) => {
                write!(f, "{role:?} instance recipe exceeds the transport limit")
            }
            Self::Decode(source) => write!(f, "Could not decode instance recipe: {source}"),
            Self::Chassis { expected, actual } => write!(
                f,
                "Instance recipe chassis {actual} differs from {expected}"
            ),
            Self::Projection(source) => source.fmt(f),
            Self::ProjectionMismatch(role) => write!(
                f,
                "{role:?} instance does not match its authenticated construction"
            ),
        }
    }
}

impl std::error::Error for WeaponAuthenticationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Decode(source) => Some(source),
            Self::Projection(source) => Some(source),
            _ => None,
        }
    }
}
