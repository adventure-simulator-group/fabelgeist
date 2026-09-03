//! The GarmentCode DSL: edges, panels, interfaces, stitches and operators.
//!
//! Ports `pygarment.garmentcode`.

pub mod clone;
pub mod component;
pub mod connector;
pub mod edge;
pub mod edge_factory;
pub mod interface;
pub mod operators;
pub mod panel;

pub use component::{Alignment, CompRef, Component, Element, LengthMode};
pub use connector::{Stitches, StitchingRule};
pub use edge::{Edge, EdgeKind, EdgeRef, EdgeSequence, Vert};
pub use interface::{Interface, InterfaceExt, InterfaceRef};
pub use panel::{InterfaceMap, Panel, PanelRef};
