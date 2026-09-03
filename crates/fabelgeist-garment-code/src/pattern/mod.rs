//! The sewing-pattern specification, its normalisation and its rendering.
//!
//! Ports `pygarment.pattern`.

pub mod core;
pub mod spec;
pub mod svg;

pub use spec::PatternSpec;
pub use svg::{Annotations, EdgeDrawing, Layout, PanelDrawing, PatternDrawing};
