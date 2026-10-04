//! Forge quotation retains derivation stage, stock identity, and exact causes.

use crate::item_catalog::ItemDefinitionId;
use adventuresim_weapon_model::{Material, ValidationError};

#[derive(Debug)]
pub struct ForgeDerivationErrors {
    errors: Vec<ValidationError>,
}

impl From<Vec<ValidationError>> for ForgeDerivationErrors {
    fn from(errors: Vec<ValidationError>) -> Self {
        Self { errors }
    }
}

impl ForgeDerivationErrors {
    pub fn errors(&self) -> &[ValidationError] {
        &self.errors
    }
}

impl std::fmt::Display for ForgeDerivationErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.errors)
    }
}

impl std::error::Error for ForgeDerivationErrors {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.errors
            .first()
            .map(|error| error as &(dyn std::error::Error + 'static))
    }
}

#[derive(Debug)]
pub enum ForgeQuoteError {
    Properties(ForgeDerivationErrors),
    MaterialMasses(ForgeDerivationErrors),
    UnsupportedStock(Material),
    RequirementOutOfRange(ItemDefinitionId),
}

impl std::fmt::Display for ForgeQuoteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Properties(source) | Self::MaterialMasses(source) => source.fmt(f),
            Self::UnsupportedStock(material) => {
                write!(f, "No forge stock is traded for {material:?}")
            }
            Self::RequirementOutOfRange(_) => {
                f.write_str("Weapon material requirement is outside the supported range")
            }
        }
    }
}

impl std::error::Error for ForgeQuoteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Properties(source) | Self::MaterialMasses(source) => Some(source),
            _ => None,
        }
    }
}
