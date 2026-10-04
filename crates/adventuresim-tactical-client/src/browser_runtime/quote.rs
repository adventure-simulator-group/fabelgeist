//! Quote transport retains JSON and domain failures until the Wasm boundary.

#[derive(Debug)]
pub(crate) enum WeaponQuoteJsonError {
    Decode(serde_json::Error),
    Quote(adventuresim_core::smithing::ForgeQuoteError),
    Encode(serde_json::Error),
}

impl std::fmt::Display for WeaponQuoteJsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Decode(source) | Self::Encode(source) => source.fmt(f),
            Self::Quote(source) => source.fmt(f),
        }
    }
}

impl std::error::Error for WeaponQuoteJsonError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Decode(source) | Self::Encode(source) => Some(source),
            Self::Quote(source) => Some(source),
        }
    }
}

pub(crate) fn quote_design_json(json: &str) -> Result<String, WeaponQuoteJsonError> {
    let design: adventuresim_weapon_model::WeaponDesign =
        serde_json::from_str(json).map_err(WeaponQuoteJsonError::Decode)?;
    let quote =
        adventuresim_core::smithing::quote_weapon(&design).map_err(WeaponQuoteJsonError::Quote)?;
    serde_json::to_string(&quote).map_err(WeaponQuoteJsonError::Encode)
}
