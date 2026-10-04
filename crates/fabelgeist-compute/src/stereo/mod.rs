//! Depth from a stereo pair: the geometry around a matcher.
//!
//! ```text
//! rig ─ rectification grid ─ (a matcher's disparity) ─ triangulate ─ temporal ─ unrectify
//! ```
//!
//! Matching itself is not here; a learned matcher does it far better than a
//! kernel could. What is here is everything a matcher needs on either side:
//!
//! * **Rectification** -- [`rig`] describes where each eye's picture is, how it
//!   was projected and how the eyes are set apart, and the grid both eyes are
//!   resampled onto so that a match is a search along a row.
//!   [`Rectified::source`] says where each grid pixel is read from.
//! * **Triangulate** turns disparity into a point with the triangle the grid
//!   implies: `Z = f B / d` for a pinhole grid, and the angular equivalents
//!   for hemispheres and omnidirectional panoramas.
//! * **Temporal** reprojects the history through the rig's motion, averages
//!   it with the current frame where the two agree, and fills what the matcher
//!   rejected while the history still vouches for it.
//! * **Unrectify** puts the depth back on each eye's own picture -- in its
//!   own projection, distortion and layout, measured from its own centre --
//!   which is what a player or a headset draws.
//!
//! [`StereoDepth`] runs the stages after the match on the card; [`synthetic`]
//! is a scene with a known answer to measure them against.

mod kernels;
mod pipeline;
pub mod rig;
pub mod synthetic;
#[cfg(test)]
mod tests;

pub use pipeline::*;
pub use rig::{
    Eye, FisheyeModel, Geometry, Grid, Matrix3, Projection, Rectification, Rectified, StereoRig,
};
