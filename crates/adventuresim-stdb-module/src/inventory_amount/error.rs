//! Measured consumption and transfer retain the missing source binding.

use adventuresim_core::identity::InventoryItemId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InventoryAmountError {
    MissingPersonal(InventoryItemId),
    MissingParty(InventoryItemId),
}

impl std::fmt::Display for InventoryAmountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::MissingPersonal(_) => "Measured personal item state is missing",
            Self::MissingParty(_) => "Measured party item state is missing",
        })
    }
}

impl std::error::Error for InventoryAmountError {}
