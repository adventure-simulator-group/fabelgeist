//! A native Rust port of [GarmentCode](https://github.com/maria-korosteleva/GarmentCode)
//! -- programming parametric sewing patterns.
//!
//! The crate covers the parametric-pattern half of the reference project:
//!
//! * [`assets`] -- the bundled body and design presets.
//! * [`curve`] -- the `svgpathtools` subset the pattern maths needs.
//! * [`optimize`] -- the `scipy.optimize.minimize` subset.
//! * [`garment`] -- the GarmentCode DSL: edges, panels, interfaces, stitches
//!   and the operators that cut and join them.
//! * [`measure`] -- taking a [`Body`] off a body mesh, so a pattern can be
//!   made to measure for a body model rather than for an average.
//! * [`pattern`] -- the sewing-pattern specification (the JSON format shared
//!   with the GarmentCodeData dataset) and its SVG rendering.
//! * [`programs`] -- the garment library (bodices, sleeves, collars, skirts,
//!   pants, bands) and the `MetaGarment` that assembles them.
//!
//! Cloth simulation, the Maya/Qualoth tooling and the web GUI of the reference
//! project are out of scope.

pub mod assets;
pub mod curve;
pub mod design;
pub mod garment;
pub mod math;
pub mod measure;
pub mod optimize;
pub mod pattern;
pub mod programs;

pub use design::{Body, Design, ParamKind, ParamSpec};
