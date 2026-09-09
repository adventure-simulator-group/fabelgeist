//! One compositional leaf program; species select data only.
//!
//! Structural equations and coupled constraints are adapted from
//! adventure-simulator-group/leaves at revision
//! 17b60b0a6f817babd1ce27467645d9ecb8389277:
//! <https://github.com/adventure-simulator-group/leaves/tree/17b60b0a6f817babd1ce27467645d9ecb8389277>.
//! The crate's src/leaf/README.md records the mapping and intentional
//! deviations.
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
