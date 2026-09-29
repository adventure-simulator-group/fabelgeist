//! The garment library: parametric components assembled into whole garments.
//!
//! Ports `assets.garment_programs`.

pub mod bands;
pub mod bodice;
pub mod circle_skirt;
pub mod collars;
pub mod godet;
pub mod meta_garment;
pub mod pants;
pub mod prelude;
pub mod shapes;
pub mod skirt_levels;
pub mod skirt_paneled;
pub mod sleeves;
pub mod tee;

pub use meta_garment::MetaGarment;
