//! Rings and a continuous cord route along a real garment opening.
use crate::{
    ClothClosures, DesignError, GenerateError, Micrometers, Millimeters, Permille,
    TextileAppearance, TextileColor,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LaceRouting {
    Crossed,
    Spiral,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClothLacing {
    /// Ordered ring stations, from the opening's lower to upper end.
    pub positions: Vec<Permille>,
    pub routing: LaceRouting,
    pub ring_radius: Millimeters,
    pub eyelet: LacingEyelet,
    pub cord: Micrometers,
    /// Distance into the cloth from each physical edge.
    pub inset: Millimeters,
    /// Ring center behind the cloth face and cord distance in front of it.
    pub ring_seat: Millimeters,
    pub cord_lift: Millimeters,
    pub fabric: TextileAppearance,
}
/// Metal rings sit behind an opening; hand-sewn eyelets require real holes
/// through the cloth stack, with a thread outline rather than metal fittings.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum LacingEyelet {
    Metal {
        wire: Micrometers,
        color: TextileColor,
    },
    Sewn {
        thread: Micrometers,
        fabric: TextileAppearance,
    },
}
impl LacingEyelet {
    pub fn gauge(&self) -> Micrometers {
        match self {
            Self::Metal { wire, .. } => *wire,
            Self::Sewn { thread, .. } => *thread,
        }
    }
    pub fn gauge_mut(&mut self) -> &mut Micrometers {
        match self {
            Self::Metal { wire, .. } => wire,
            Self::Sewn { thread, .. } => thread,
        }
    }
    fn validate(&self) -> Result<(), GenerateError> {
        if let Self::Sewn { fabric, .. } = self {
            fabric.validate()?;
        }
        if !ClothLacing::WIRE_RANGE.contains(&self.gauge().0) {
            return Err(DesignError::ParametricParameters.into());
        }
        Ok(())
    }
}
impl ClothLacing {
    // Millimetre eyelets need an aperture, while thread-scale roundness is
    // shading detail. Reserve the mesh budget for the enclosing cloth shape.
    pub const RING_CURVE_SEGMENTS: usize = 4;
    pub const COUNT_RANGE: std::ops::RangeInclusive<usize> = 2..=12;
    pub const RADIUS_RANGE: std::ops::RangeInclusive<u16> = 1..=4;
    pub const WIRE_RANGE: std::ops::RangeInclusive<u16> = 200..=1000;
    pub const CORD_RANGE: std::ops::RangeInclusive<u16> = 500..=3000;
    pub const INSET_RANGE: std::ops::RangeInclusive<u16> = 1..=10;
    pub const SEAT_RANGE: std::ops::RangeInclusive<u16> = 0..=10;
    pub const LIFT_RANGE: std::ops::RangeInclusive<u16> = 1..=60;
    pub fn cord_range(&self) -> std::ops::RangeInclusive<u16> {
        // The polygon's inscribed aperture is narrower than the ideal circle.
        // Subtract the whole wire gauge conservatively before admitting cord.
        let inscribed = (f32::from(self.ring_radius.0)
            * 2.
            * (std::f32::consts::PI / Self::RING_CURVE_SEGMENTS as f32).cos()
            * Micrometers::PER_MILLIMETRE as f32)
            .floor() as u32;
        let aperture = inscribed.saturating_sub(u32::from(self.eyelet.gauge().0));
        let maximum = aperture
            .saturating_sub(1)
            .min(u32::from(*Self::CORD_RANGE.end())) as u16;
        *Self::CORD_RANGE.start()..=maximum
    }
    pub fn validate(&self) -> Result<(), GenerateError> {
        self.fabric.validate()?;
        self.eyelet.validate()?;
        if !Self::COUNT_RANGE.contains(&self.positions.len())
            || self
                .positions
                .iter()
                .any(|p| !ClothClosures::POSITION_RANGE.contains(&p.0))
            || self.positions.windows(2).any(|p| p[0].0 >= p[1].0)
            || !Self::RADIUS_RANGE.contains(&self.ring_radius.0)
            || !Self::WIRE_RANGE.contains(&self.eyelet.gauge().0)
            || !Self::CORD_RANGE.contains(&self.cord.0)
            || !Self::INSET_RANGE.contains(&self.inset.0)
            || !Self::SEAT_RANGE.contains(&self.ring_seat.0)
            || !Self::LIFT_RANGE.contains(&self.cord_lift.0)
            || (matches!(self.eyelet, LacingEyelet::Sewn { .. }) && self.ring_seat.0 != 0)
            || self.inset.0 < self.ring_radius.0
        {
            return Err(DesignError::ParametricParameters.into());
        }
        if !self.cord_range().contains(&self.cord.0) {
            return Err(DesignError::ParametricParameters.into());
        }
        Ok(())
    }
}
