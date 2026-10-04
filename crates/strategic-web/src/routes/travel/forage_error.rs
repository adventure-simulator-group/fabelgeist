//! Immutable terrain sampling failures retain the package decoder cause.

#[derive(Debug, thiserror::Error)]
pub(in crate::routes) enum TerrainForageError {
    #[error("{0}")]
    Sample(#[from] adventuresim_terrain::Error),
    #[error("The current location is outside the terrain package")]
    OutsidePackage,
}
