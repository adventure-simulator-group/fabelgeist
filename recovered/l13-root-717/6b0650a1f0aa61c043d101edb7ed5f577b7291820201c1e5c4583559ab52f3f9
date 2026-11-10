//! Mechanical course identities authored while the generator divides a carrier.
use crate::Permille;
use serde::{Deserialize, Serialize};

/// Topological order within one assembly: a parent precedes its children.
/// Paired front and back plates share a course so their side closure is owned
/// by one rigid joint.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct PlateCourse(pub u16);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PlateGridEnd {
    First,
    Last,
}

impl PlateGridEnd {
    pub const fn row(self, rows: u32) -> u32 {
        match self {
            Self::First => 0,
            Self::Last => rows - 1,
        }
    }

    pub const fn opposite(self) -> Self {
        match self {
            Self::First => Self::Last,
            Self::Last => Self::First,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PlateJointMotion {
    Hinge,
    Flexible,
}

/// Coordinates refer to the exported outer surface grid, so wearer fitting
/// moves the attachment with the actual plate. No anatomical owner is inferred
/// from a component's display role or skin weights.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PlateParent {
    pub course: PlateCourse,
    pub edge: PlateGridEnd,
}

/// An authored connection and body-following fraction for one rigid course.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PlateMount {
    pub course: PlateCourse,
    pub parent: Option<PlateParent>,
    pub incoming: PlateGridEnd,
    pub motion: PlateJointMotion,
    /// Exactly one leaf per assembly follows the distal body target fully;
    /// this selects the driven endpoint independently of course ordering.
    pub follow: Permille,
}
