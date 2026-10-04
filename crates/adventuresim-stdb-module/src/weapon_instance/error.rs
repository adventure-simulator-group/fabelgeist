//! Errors retain recipe, projection, and stable-object admission stages.

use super::authentication_error::WeaponAuthenticationError;
use crate::inventory_container::InventoryObjectError;
use adventuresim_core::{
    identity::InventoryItemId, item_catalog::ItemDefinitionId,
    physical_object::CarriedInventoryScope,
};
use adventuresim_weapon_model::CodecError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WeaponRecipeKind {
    Weapon,
    Holder,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WeaponProjectionField {
    Mass,
    Length,
    GripToTip,
    HolderMass,
    HolderLength,
    HolderGripToTip,
}

impl std::fmt::Display for WeaponProjectionField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Mass => "mass",
            Self::Length => "length",
            Self::GripToTip => "grip-to-tip distance",
            Self::HolderMass => "holder mass",
            Self::HolderLength => "holder length",
            Self::HolderGripToTip => "holder anchor-to-tip distance",
        })
    }
}

#[derive(Debug)]
pub(crate) enum WeaponProjectionError {
    Evaluation(CodecError),
    Encoding(CodecError),
    RecipeTooLarge(WeaponRecipeKind),
    OutOfRange(WeaponProjectionField),
}

impl std::fmt::Display for WeaponProjectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Evaluation(source) | Self::Encoding(source) => source.fmt(f),
            Self::RecipeTooLarge(WeaponRecipeKind::Weapon) => {
                f.write_str("Weapon recipe exceeds the tactical transport limit")
            }
            Self::RecipeTooLarge(WeaponRecipeKind::Holder) => {
                f.write_str("Weapon holder recipe exceeds the tactical transport limit")
            }
            Self::OutOfRange(field) => write!(f, "Weapon {field} is outside the persisted range"),
        }
    }
}
impl std::error::Error for WeaponProjectionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Evaluation(source) | Self::Encoding(source) => Some(source),
            Self::RecipeTooLarge(_) | Self::OutOfRange(_) => None,
        }
    }
}

#[derive(Debug)]
pub(crate) enum WeaponInstanceError {
    Object(InventoryObjectError),
    Projection(WeaponProjectionError),
    Authentication(WeaponAuthenticationError),
    MissingDefinition(ItemDefinitionId),
    NonIndividual {
        scope: CarriedInventoryScope,
        row: InventoryItemId,
    },
    MissingStableObject {
        scope: CarriedInventoryScope,
        row: InventoryItemId,
    },
    MissingPhysicalObject(WeaponRecipeKind),
    MissingCatalogDefinition,
    MissingParametricRecipe(InventoryItemId),
    UnsupportedBodyHolder(ItemDefinitionId),
    MissingHolderTemplate(ItemDefinitionId),
    WrongHolder {
        weapon: ItemDefinitionId,
        expected: ItemDefinitionId,
        supplied: ItemDefinitionId,
    },
    NotMelee(ItemDefinitionId),
    HolderIdentityMismatch {
        expected: ItemDefinitionId,
        actual: ItemDefinitionId,
    },
    ChassisMismatch {
        role: WeaponRecipeKind,
        expected: ItemDefinitionId,
        supplied: ItemDefinitionId,
    },
}

impl From<InventoryObjectError> for WeaponInstanceError {
    fn from(source: InventoryObjectError) -> Self {
        Self::Object(source)
    }
}
impl From<WeaponProjectionError> for WeaponInstanceError {
    fn from(source: WeaponProjectionError) -> Self {
        Self::Projection(source)
    }
}
impl From<WeaponAuthenticationError> for WeaponInstanceError {
    fn from(source: WeaponAuthenticationError) -> Self {
        Self::Authentication(source)
    }
}
impl std::fmt::Display for WeaponInstanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Object(source) => source.fmt(f),
            Self::Projection(source) => source.fmt(f),
            Self::Authentication(source) => source.fmt(f),
            Self::HolderIdentityMismatch { expected, actual } => write!(
                f,
                "Holder object has the wrong catalog identity: expected {expected}, found {actual}"
            ),
            Self::MissingDefinition(id) => write!(f, "Unknown weapon definition {id}"),
            Self::NonIndividual { scope, row } => write!(
                f,
                "Parametric weapons must be individual {scope:?} inventory rows (row {row})"
            ),
            Self::MissingStableObject { scope, row } => write!(
                f,
                "Parametric {scope:?} weapon has no stable physical object identity (row {row})"
            ),
            Self::MissingPhysicalObject(role) => write!(f, "{role:?} physical object not found"),
            Self::MissingCatalogDefinition => f.write_str("Weapon catalog definition not found"),
            Self::MissingParametricRecipe(row) => {
                write!(f, "Fitted weapon has no parametric recipe (row {row})")
            }
            Self::UnsupportedBodyHolder(item) => write!(
                f,
                "Polearms cannot be fitted to a body-mounted holder ({item})"
            ),
            Self::MissingHolderTemplate(item) => {
                write!(f, "Weapon has no procedural holder template ({item})")
            }
            Self::WrongHolder {
                weapon,
                expected,
                supplied,
            } => write!(f, "{weapon} requires a {expected}, found {supplied}"),
            Self::NotMelee(id) => write!(
                f,
                "Only melee weapon objects accept parametric designs ({id})"
            ),
            Self::ChassisMismatch {
                role,
                expected,
                supplied,
            } => write!(
                f,
                "{role:?} design chassis does not match its inventory object: expected {expected}, found {supplied}"
            ),
        }
    }
}
impl std::error::Error for WeaponInstanceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Object(source) => Some(source),
            Self::Projection(source) => Some(source),
            Self::Authentication(source) => Some(source),
            _ => None,
        }
    }
}
