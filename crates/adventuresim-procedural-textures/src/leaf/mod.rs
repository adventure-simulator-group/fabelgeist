//! One compositional leaf program; species select data only.
mod baking;
mod bipinnate;
mod constraints;
mod kernel;
mod math;
mod organs;
mod presets;
mod profile;
mod radial;
mod ranges;
mod relief;
mod secondary;
mod shape;
mod species;
mod uniform;
mod veins;
pub(crate) use baking::generate;
pub use ranges::parameter_range;
pub use shape::LeafShape;
pub use species::{LeafParameters, LeafPresets};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod parity;
