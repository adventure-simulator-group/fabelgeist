//! Parametric rounded breath slots shared by pierced helmet plates.
use crate::{DesignError, Millimeters, Permille};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum VentSides {
    Both,
    Left,
    Right,
}

/// Slot rotation from vertical in the authored visor surface.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SlotInclination(pub i16);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct VisorBreaths {
    pub count_per_row: u8,
    pub rows: u8,
    pub width: Millimeters,
    pub length: Millimeters,
    /// Total width of each cheek pattern, including its first and last slots.
    pub span: Millimeters,
    pub row_spacing: Millimeters,
    /// Lateral distance from the face center to each pattern center.
    pub center_offset: Millimeters,
    /// Vertical location on the authored visor, measured from brow to chin.
    pub height: Permille,
    /// Fraction of the maximum corner radius. Zero produces square corners.
    pub rounding: Permille,
    pub inclination: SlotInclination,
    pub sides: VentSides,
}

impl Default for VisorBreaths {
    fn default() -> Self {
        Self {
            count_per_row: 2,
            rows: 1,
            width: Millimeters(3),
            length: Millimeters(8),
            span: Millimeters(26),
            row_spacing: Millimeters(22),
            center_offset: Millimeters(30),
            height: Permille(820),
            rounding: Permille(500),
            inclination: SlotInclination(90),
            sides: VentSides::Both,
        }
    }
}

impl VisorBreaths {
    /// An upper row of vertical slots on a separate burgonet face defense.
    pub fn buffe() -> Self {
        Self {
            count_per_row: 4,
            width: Millimeters(3),
            length: Millimeters(12),
            span: Millimeters(54),
            center_offset: Millimeters(36),
            height: Permille(160),
            inclination: SlotInclination(0),
            ..Self::default()
        }
    }

    pub(crate) fn validate(
        &self,
        heights: std::ops::RangeInclusive<u16>,
    ) -> Result<(), DesignError> {
        let b = self;
        if b.count_per_row > 8
            || !(1..=4).contains(&b.rows)
            || !(2..=6).contains(&b.width.0)
            || !(8..=20).contains(&b.length.0)
            || !(20..=60).contains(&b.span.0)
            || !(14..=25).contains(&b.row_spacing.0)
            || !(10..=100).contains(&b.center_offset.0)
            || !heights.contains(&b.height.0)
            || b.rounding.0 > 1000
            || !(-90..=90).contains(&b.inclination.0)
        {
            return Err(DesignError::ParametricParameters);
        }
        if b.count_per_row == 0 {
            return Ok(());
        }
        let angle = f32::from(b.inclination.0).to_radians();
        let width = f32::from(b.width.0) * angle.cos() + f32::from(b.length.0) * angle.sin().abs();
        let height = f32::from(b.length.0) * angle.cos() + f32::from(b.width.0) * angle.sin().abs();
        let web = crate::pierced_plate_domain::MINIMUM_WEB_MM as f32;
        let required_span =
            width * f32::from(b.count_per_row) + web * f32::from(b.count_per_row - 1);
        if required_span > f32::from(b.span.0)
            || (b.sides == VentSides::Both
                && 2.0 * f32::from(b.center_offset.0) - f32::from(b.span.0) < web)
            || (b.rows > 1 && height + web > f32::from(b.row_spacing.0))
        {
            return Err(DesignError::VisorOpeningSpacing);
        }
        Ok(())
    }

    pub(crate) fn openings(&self, chart_height: f64) -> Vec<Vec<[f64; 2]>> {
        let b = self;
        let mut holes = Vec::new();
        let angle = f64::from(b.inclination.0).to_radians();
        let width = f64::from(b.width.0);
        let length = f64::from(b.length.0);
        let projected_width = width * angle.cos() + length * angle.sin().abs();
        for side in [-1.0, 1.0] {
            if matches!(b.sides, VentSides::Left) && side < 0.0
                || matches!(b.sides, VentSides::Right) && side > 0.0
            {
                continue;
            }
            for row in 0..b.rows {
                for slot in 0..b.count_per_row {
                    let offset = if b.count_per_row == 1 {
                        0.0
                    } else {
                        (f64::from(slot) / f64::from(b.count_per_row - 1) - 0.5)
                            * (f64::from(b.span.0) - projected_width)
                    };
                    let center = [
                        side * (f64::from(b.center_offset.0) + offset),
                        f64::from(b.height.0) * chart_height / 1000.0
                            + (f64::from(row) - f64::from(b.rows - 1) * 0.5)
                                * f64::from(b.row_spacing.0),
                    ];
                    holes.push(crate::pierced_plate_domain::rounded_slot(
                        center,
                        width,
                        length,
                        angle * side,
                        f64::from(b.rounding.0) / 1000.0,
                    ));
                }
            }
        }
        holes
    }
}
