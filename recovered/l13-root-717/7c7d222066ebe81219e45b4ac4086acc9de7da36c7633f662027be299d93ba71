//! Which face of a thickened plate a triangle lies on.
use serde::{Deserialize, Serialize};

/// The face of a plate a triangle belongs to. Every generator that thickens
/// a carrier knows this from its topology alone; decoration that follows the
/// plate's edges, such as a trim band, is found from it.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlateFace {
    /// The visible, outward face: the carrier itself.
    Outer,
    /// The face against the wearer.
    Inner,
    /// The narrow wall closing the plate's thickness along a cut edge.
    Edge,
}
