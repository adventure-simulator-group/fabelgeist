//! How a plate piece is built: one solid shell, or rows of small plates laid
//! over its surface and laced together.

use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::trim::Trim;

/// How a piece is built.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Construction {
    /// One plate shaped to the wearer.
    #[default]
    Solid,
    /// Narrow lames laced into rows, each row hung from the row above by
    /// lacing that runs down over the upper row's face.
    Lamellar(Tiling),
    /// Scales laced to their neighbours through their top holes, each row
    /// covering the lacing of the row below.
    Scale(Tiling),
}

impl Construction {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Solid => "Solid",
            Self::Lamellar(_) => "Lamellar",
            Self::Scale(_) => "Scale",
        }
    }

    /// The small plates, unless the piece is solid.
    pub fn tiling(&self) -> Option<&Tiling> {
        match self {
            Self::Solid => None,
            Self::Lamellar(tiling) | Self::Scale(tiling) => Some(tiling),
        }
    }

    pub fn tiling_mut(&mut self) -> Option<&mut Tiling> {
        match self {
            Self::Solid => None,
            Self::Lamellar(tiling) | Self::Scale(tiling) => Some(tiling),
        }
    }

    pub fn validate(&self) -> Result<(), ConstructionError> {
        self.tiling().map_or(Ok(()), Tiling::validate)
    }

    /// `trim` as it runs on this construction. On small plates the band
    /// follows each plate's rim, so it keeps to a share of the plate and the
    /// plate's face still shows.
    pub fn trim_on(&self, trim: &Trim) -> Trim {
        match self.tiling() {
            Some(tiling) => trim.narrowed(tiling.plate.widest_trim()),
            None => trim.clone(),
        }
    }
}

/// Small plates laid in overlapping rows over a piece's surface.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tiling {
    pub plate: Plate,
    /// Cord through the plates' holes; `None` leaves them unlaced.
    pub lacing: Option<Lacing>,
}

impl Tiling {
    /// Narrow, tall lames with three pairs of holes, laced in red silk.
    pub fn lamellar() -> Self {
        Self {
            plate: Plate {
                width: 0.028,
                height: 0.085,
                thickness: 0.0015,
                roundness: 0.25,
                bevel: 0.0004,
                gap: 0.001,
                overlap: 0.2,
                stagger: 0.0,
                hole_radius: 0.0022,
                hole_pairs: 3,
            },
            lacing: Some(Lacing {
                radius: 0.0014,
                color: [0.55, 0.09, 0.07],
                roughness: 0.6,
            }),
        }
    }

    /// Round-bottomed scales in staggered rows, laced in leather through one
    /// pair of holes at the top.
    pub fn scale() -> Self {
        Self {
            plate: Plate {
                width: 0.045,
                height: 0.065,
                thickness: 0.0015,
                roundness: 1.0,
                bevel: 0.0004,
                gap: 0.0015,
                overlap: 0.3,
                stagger: 0.5,
                hole_radius: 0.0025,
                hole_pairs: 1,
            },
            lacing: Some(Lacing::default()),
        }
    }

    pub fn validate(&self) -> Result<(), ConstructionError> {
        self.plate.validate()?;
        let Some(lacing) = &self.lacing else {
            return Ok(());
        };
        lacing.validate()?;
        if self.plate.hole_pairs == 0 || lacing.radius > Lacing::max_radius(&self.plate) {
            return Err(ConstructionError::LacingHoles);
        }
        Ok(())
    }
}

/// One small plate: its outline, gauge, spacing and lacing holes. Lengths
/// are metres.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plate {
    /// Across the row.
    pub width: f32,
    /// Along the piece, from the row above to the row below.
    pub height: f32,
    pub thickness: f32,
    /// Rounding of the lower corners, as a fraction of the largest radius
    /// the plate takes.
    pub roundness: f32,
    /// Rounding of the front edge.
    pub bevel: f32,
    /// Space between neighbouring plates of a row.
    pub gap: f32,
    /// Fraction of a plate's height the row above covers.
    pub overlap: f32,
    /// Shift of every other row, as a fraction of the plate pitch.
    pub stagger: f32,
    pub hole_radius: f32,
    /// Pairs of lacing holes, one pair per row of holes from the top.
    pub hole_pairs: u32,
}

impl Plate {
    pub const WIDTH: RangeInclusive<f32> = 0.012..=0.15;
    pub const HEIGHT: RangeInclusive<f32> = 0.02..=0.2;
    pub const THICKNESS: RangeInclusive<f32> = 0.0005..=0.012;
    pub const GAP: RangeInclusive<f32> = 0.0..=0.005;
    pub const OVERLAP: RangeInclusive<f32> = 0.0..=0.6;
    pub const HOLE_RADIUS: RangeInclusive<f32> = 0.0..=0.006;
    pub const MAX_HOLE_PAIRS: u32 = 3;
    /// The deepest a bevel cuts, as a fraction of the plate's thickness.
    pub const MAX_BEVEL: f32 = 0.45;
    /// The share of a plate's narrower side a trim band may cover.
    const TRIM_SHARE: f32 = 0.15;
    /// Holes stay this many radii clear of each other and the plate's edge.
    const HOLE_CLEARANCE_RADII: f32 = 5.0;
    /// Each hole's distance from the plate's centre line, as a fraction of
    /// its width.
    const HOLE_INSET: f32 = 0.27;
    /// Radii from the top edge to the first row of holes' centres.
    const FIRST_HOLE_RADII: f32 = 2.2;
    /// The height the rows of holes are spread over, as a fraction of the
    /// plate's; one row for each of the most pairs a plate has.
    const HOLE_ROW_SPAN: f32 = 0.55;

    pub fn validate(&self) -> Result<(), ConstructionError> {
        let within =
            |value: f32, range: RangeInclusive<f32>| value.is_finite() && range.contains(&value);
        if !within(self.width, Self::WIDTH)
            || !within(self.height, Self::HEIGHT)
            || !within(self.thickness, Self::THICKNESS)
            || !within(self.roundness, 0.0..=1.0)
            || !within(self.bevel, 0.0..=self.thickness * Self::MAX_BEVEL)
            || !within(self.gap, Self::GAP)
            || !within(self.overlap, Self::OVERLAP)
            || !within(self.stagger, 0.0..=1.0)
        {
            return Err(ConstructionError::PlateDimensions);
        }
        if !within(self.hole_radius, Self::HOLE_RADIUS)
            || self.hole_pairs > Self::MAX_HOLE_PAIRS
            || self.hole_radius > self.max_hole_radius()
        {
            return Err(ConstructionError::HoleSpacing);
        }
        Ok(())
    }

    /// The widest trim band along the plate's rim.
    pub fn widest_trim(&self) -> f32 {
        (self.width.min(self.height) * Self::TRIM_SHARE).max(Trim::MIN_WIDTH)
    }

    /// The widest hole the plate keeps clear of its edges and its other
    /// holes, the rows of holes included.
    pub fn max_hole_radius(&self) -> f32 {
        let across = self.width.min(self.height) / Self::HOLE_CLEARANCE_RADII;
        let between_rows = if self.hole_pairs > 1 {
            // Centres two and a half radii apart leave half a hole of metal.
            self.hole_row_pitch() * 2.0 / Self::HOLE_CLEARANCE_RADII
        } else {
            f32::INFINITY
        };
        across.min(between_rows).min(*Self::HOLE_RADIUS.end())
    }

    /// The distance between the centres of successive rows of holes.
    fn hole_row_pitch(&self) -> f32 {
        self.height * Self::HOLE_ROW_SPAN / Self::MAX_HOLE_PAIRS as f32
    }

    /// The centre of every lacing hole, a pair per row from the top, each
    /// pair left then right. None when the plate has no holes.
    pub fn holes(&self) -> Vec<[f32; 2]> {
        if self.hole_radius <= 0.0 {
            return Vec::new();
        }
        let x = self.width * Self::HOLE_INSET;
        let pitch = self.hole_row_pitch();
        (0..self.hole_pairs)
            .flat_map(|pair| {
                let y = self.height * 0.5
                    - self.hole_radius * Self::FIRST_HOLE_RADII
                    - pair as f32 * pitch;
                [[-x, y], [x, y]]
            })
            .collect()
    }
}

/// Cord through the plates' holes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lacing {
    /// Of the cord's section, metres.
    pub radius: f32,
    pub color: [f32; 3],
    pub roughness: f32,
}

impl Default for Lacing {
    /// Brown leather thong.
    fn default() -> Self {
        Self {
            radius: 0.0012,
            color: [0.33, 0.19, 0.1],
            roughness: 0.8,
        }
    }
}

impl Lacing {
    pub const RADIUS: RangeInclusive<f32> = 0.0004..=0.003;
    /// The share of its hole's radius a cord may fill.
    const HOLE_FILL: f32 = 0.9;

    /// The thickest cord that passes through `plate`'s holes.
    pub fn max_radius(plate: &Plate) -> f32 {
        (plate.hole_radius * Self::HOLE_FILL).min(*Self::RADIUS.end())
    }

    pub fn validate(&self) -> Result<(), ConstructionError> {
        let unit = |value: f32| (0.0..=1.0).contains(&value);
        if !(self.radius.is_finite() && Self::RADIUS.contains(&self.radius))
            || !self.color.into_iter().all(unit)
            || !unit(self.roughness)
        {
            return Err(ConstructionError::Lacing);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum ConstructionError {
    #[error("plate dimensions are outside their supported ranges")]
    PlateDimensions,
    #[error("lacing holes do not fit on the plate")]
    HoleSpacing,
    #[error("lacing cord is outside its supported range")]
    Lacing,
    #[error("lacing needs holes wider than its cord")]
    LacingHoles,
    #[error("the piece has no surface to lay plates over; build them before trimming")]
    NoSurfaceGrid,
    #[error("the piece's surface is too small or degenerate to lay plates over")]
    DegenerateSurface,
    #[error("the plates are too large for this piece; make them smaller")]
    NoPlateFits,
    #[error("the piece would take more than {} plates", crate::tiling::MAX_TILES)]
    TooManyPlates,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_tilings_start_valid() {
        for tiling in [Tiling::lamellar(), Tiling::scale()] {
            assert_eq!(tiling.validate(), Ok(()));
        }
    }

    #[test]
    fn lacing_needs_holes_it_fits_through() {
        let mut tiling = Tiling::scale();
        tiling.plate.hole_pairs = 0;
        assert_eq!(tiling.validate(), Err(ConstructionError::LacingHoles));
        let mut tiling = Tiling::scale();
        tiling.lacing.as_mut().unwrap().radius = tiling.plate.hole_radius;
        assert_eq!(tiling.validate(), Err(ConstructionError::LacingHoles));
        tiling.lacing.as_mut().unwrap().radius = Lacing::max_radius(&tiling.plate);
        assert_eq!(tiling.validate(), Ok(()));
        tiling.lacing = None;
        assert_eq!(tiling.validate(), Ok(()));
    }

    #[test]
    fn the_widest_hole_is_allowed_and_no_wider() {
        let mut plate = Tiling::scale().plate;
        plate.hole_radius = plate.max_hole_radius();
        assert_eq!(plate.validate(), Ok(()));
        plate.hole_radius *= 1.01;
        assert_eq!(plate.validate(), Err(ConstructionError::HoleSpacing));
    }

    #[test]
    fn trim_keeps_to_the_rim_of_small_plates() {
        let trim = Trim::default();
        assert_eq!(Construction::Solid.trim_on(&trim), trim);
        for construction in [
            Construction::Scale(Tiling::scale()),
            Construction::Lamellar(Tiling::lamellar()),
        ] {
            let plate = &construction.tiling().unwrap().plate;
            let band = construction.trim_on(&trim).width;
            assert!(band < plate.width.min(plate.height) * 0.25, "{band}");
        }
    }

    #[test]
    fn holes_run_in_pairs_from_the_top() {
        let plate = Tiling::lamellar().plate;
        let holes = plate.holes();
        assert_eq!(holes.len(), 2 * plate.hole_pairs as usize);
        for pair in holes.chunks(2) {
            assert_eq!(pair[0][0], -pair[1][0]);
            assert_eq!(pair[0][1], pair[1][1]);
        }
        assert!(holes.windows(3).step_by(2).all(|h| h[0][1] > h[2][1]));
        assert!(
            holes
                .iter()
                .all(|h| h[1] + plate.hole_radius < plate.height * 0.5)
        );
    }
}
