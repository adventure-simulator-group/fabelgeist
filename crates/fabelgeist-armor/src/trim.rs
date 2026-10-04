//! A trim band along every edge of a plate.
//!
//! Real plate is often finished differently along its edges: gilded, blued,
//! or etched with a running border. The band is found on the finished mesh
//! from its [`PlateFace`](crate::PlateFace)s alone, so it follows whatever edges a generator
//! produced: the outer face's boundary is the rim, the band is the outer face
//! within [`TrimBand::width`] of it, and the edge walls always belong to it.
//! Triangles crossing the band's inner border are cut along it, so the band
//! keeps its width however coarse the plate's sampling.
//!
//! Each band vertex gets a coordinate along its nearest rim and its distance
//! in from it. Along a closed rim the coordinate is stretched slightly so the
//! rim holds a whole number of [`TrimBand::period`]s, and an ornament
//! repeating at that period closes on itself.

use std::ops::Range;

use thiserror::Error;

use crate::GeneratedArmor;

mod edges;
mod finish;
mod split;

pub use finish::{Trim, TrimFinishError};

/// The band's dimensions on the surface.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrimBand {
    /// How far the band reaches in from the rim, metres.
    pub width: f32,
    /// The length along the rim an ornament repeats over, metres.
    pub period: f32,
}

/// The trim band of a generated piece.
#[derive(Clone, Debug, PartialEq)]
pub struct ArmorTrim {
    /// Per vertex, metres along the nearest rim and in from it. Only band
    /// triangles read them.
    pub coordinates: Vec<[f32; 2]>,
    /// The band's triangles of each surface, as index ranges; see
    /// [`GeneratedArmor::surfaces`].
    pub bands: Vec<Range<usize>>,
}

/// One separately named surface of a piece: a component, or the whole piece
/// when it has none. Its plate triangles come first, then its band.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArmorSurface {
    /// The component, as an index into [`GeneratedArmor::components`].
    pub component: Option<usize>,
    pub plate: Range<usize>,
    pub trim: Range<usize>,
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum TrimError {
    #[error("trim width and ornament period must be positive lengths")]
    InvalidBand,
    #[error("the piece does not record which face of the plate each triangle lies on")]
    NoPlateFaces,
    #[error("the piece is already trimmed")]
    AlreadyTrimmed,
}

impl TrimBand {
    fn validate(self) -> Result<Self, TrimError> {
        if self.width.is_finite()
            && self.width > 0.0
            && self.period.is_finite()
            && self.period > 0.0
        {
            Ok(self)
        } else {
            Err(TrimError::InvalidBand)
        }
    }
}

impl GeneratedArmor {
    /// Cut a trim band along every edge of every plate.
    pub fn trimmed(self, band: TrimBand) -> Result<Self, TrimError> {
        let band = band.validate()?;
        if self.trim.is_some() {
            return Err(TrimError::AlreadyTrimmed);
        }
        if self.faces.is_empty() || self.faces.len() * 3 != self.indices.len() {
            return Err(TrimError::NoPlateFaces);
        }
        let rims = edges::Rims::new(&self.positions, &self.indices, &self.faces, band);
        Ok(split::Split::new(self, &rims, band).finish())
    }

    /// The piece's surfaces, each with its plate and band triangles.
    pub fn surfaces(&self) -> Vec<ArmorSurface> {
        let ranges = if self.components.is_empty() {
            vec![(None, 0..self.indices.len())]
        } else {
            self.components
                .iter()
                .enumerate()
                .map(|(index, component)| (Some(index), component.indices.clone()))
                .collect()
        };
        ranges
            .into_iter()
            .enumerate()
            .map(|(surface, (component, indices))| {
                let trim = self
                    .trim
                    .as_ref()
                    .map_or(indices.end..indices.end, |trim| trim.bands[surface].clone());
                ArmorSurface {
                    component,
                    plate: indices.start..trim.start,
                    trim,
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests;
