//! A heraldic identity becomes vector artwork, then physically scaled paint.
//! Generation has no renderer, filesystem, network or game-state dependency.
mod animals;
pub mod artwork;
pub mod bake;
mod composition;
pub mod document;
pub mod export;
mod fields;
pub mod geometry;
pub mod paint;
pub mod presets;
pub mod provenance;
mod validate;

/// Failures at document, geometry and export boundaries.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid heraldry document: {0}")]
    Invalid(String),
    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("image encoding: {0}")]
    Image(#[from] image::ImageError),
}
