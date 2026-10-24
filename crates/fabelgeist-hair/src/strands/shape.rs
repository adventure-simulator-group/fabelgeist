//! Checked host collection shape shared by host and native strand producers.

use super::Strands;
use crate::parameters::{PointsPerStrand, StrandCount};

pub type StrandShapeResult<T> = Result<T, StrandShapeError>;

/// Fixed point stride and population admitted in host collection addresses.
/// This does not certify geometric values, native widths, or source identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StrandShape {
    strands: StrandCount,
    points: PointsPerStrand,
}

/// Total point records, distinct from the point stride of one strand.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StrandSampleCount(usize);

/// A per-strand column paired with the root UV population.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StrandAttribute {
    RootNormal,
    Id,
    Group,
    Radius,
}

#[derive(Debug, thiserror::Error)]
pub enum StrandShapeError {
    #[error("nonempty strands need a root and tip, received {points:?} points")]
    MissingRootOrTip { points: PointsPerStrand },
    #[error("{strands:?} strands of {points:?} points exceed host collection addresses")]
    SampleCount { strands: StrandCount, points: PointsPerStrand },
    #[error("{shape:?} has {actual} position records")]
    PositionCount { shape: StrandShape, actual: usize },
    #[error("{attribute:?} has {actual} records for {strands:?} strands")]
    AttributeCount { attribute: StrandAttribute, strands: StrandCount, actual: usize },
    #[error("strand prefix {requested:?} exceeds source population {available:?}")]
    Prefix { requested: StrandCount, available: StrandCount },
}

impl StrandShape {
    pub fn new(strands: StrandCount, points: PointsPerStrand) -> StrandShapeResult<Self> {
        if strands.count() != 0 && points.count() < 2 {
            return Err(StrandShapeError::MissingRootOrTip { points });
        }
        strands.count().checked_mul(points.count())
            .ok_or(StrandShapeError::SampleCount { strands, points })?;
        Ok(Self { strands, points })
    }

    pub fn strands(self) -> StrandCount {
        self.strands
    }

    pub fn points(self) -> PointsPerStrand {
        self.points
    }

    pub fn samples(self) -> StrandSampleCount {
        // The constructor admits this product; shrinking the population preserves it.
        StrandSampleCount(self.strands.count() * self.points.count())
    }

    /// Admit actual strand-major storage at the declared fixed stride.
    pub fn check_positions(self, actual: usize) -> StrandShapeResult<Self> {
        if actual != self.samples().count() {
            return Err(StrandShapeError::PositionCount { shape: self, actual });
        }
        Ok(self)
    }

    pub fn prefix(self, strands: StrandCount) -> StrandShapeResult<Self> {
        if strands > self.strands {
            return Err(StrandShapeError::Prefix { requested: strands, available: self.strands });
        }
        Ok(Self { strands, ..self })
    }

    /// Shrink to a maximum population, preserving the admitted point stride.
    pub fn limited_to(self, maximum: StrandCount) -> Self {
        Self { strands: self.strands.min(maximum), ..self }
    }
}

impl StrandSampleCount {
    /// Host collection indexing or numerical-kernel sample population.
    pub fn count(self) -> usize {
        self.0
    }
}

impl TryFrom<&Strands> for StrandShape {
    type Error = StrandShapeError;

    fn try_from(strands: &Strands) -> StrandShapeResult<Self> {
        let shape = Self::new(
            StrandCount::from_count(strands.len()),
            PointsPerStrand::from_count(strands.points),
        )?.check_positions(strands.positions.len())?;
        for attribute in [StrandAttribute::RootNormal, StrandAttribute::Id,
            StrandAttribute::Group, StrandAttribute::Radius] {
            let actual = match attribute {
                StrandAttribute::RootNormal => strands.root_normal.len(),
                StrandAttribute::Id => strands.id.len(),
                StrandAttribute::Group => strands.group.len(),
                StrandAttribute::Radius => strands.radius.len(),
            };
            if actual != shape.strands.count() {
                return Err(StrandShapeError::AttributeCount {
                    attribute, strands: shape.strands, actual,
                });
            }
        }
        Ok(shape)
    }
}
