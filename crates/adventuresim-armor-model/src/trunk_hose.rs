//! Body-fitted trunk hose joining separate leg garments across the pelvis.

use serde::{Deserialize, Serialize};

use crate::{GenerateError, Millimeters, Permille, TextileColor};

/// A continuous breeches carrier with a waist opening and two leg openings.
///
/// The character creator cuts this garment from the wearer's pelvic surface so
/// the crotch bridge and seat remain continuous on different body proportions.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrunkHoseDesign {
    pub clearance: Millimeters,
    pub thickness: Millimeters,
    /// Reach from the waist toward the knees relative to the pelvic fit span.
    pub length: Permille,
    /// Number of vertical panes distributed around the waist.
    pub panel_count: u8,
    pub primary_color: TextileColor,
    pub secondary_color: TextileColor,
}

impl TrunkHoseDesign {
    pub fn validate(&self) -> Result<(), GenerateError> {
        if !(1..=10).contains(&self.clearance.0)
            || !(1..=8).contains(&self.thickness.0)
            || !(900..=1_200).contains(&self.length.0)
            || !(4..=16).contains(&self.panel_count)
            || !self.panel_count.is_multiple_of(2)
        {
            return Err(crate::DesignError::ParametricParameters.into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alternating_panes_require_an_even_closed_ring() {
        let mut design = TrunkHoseDesign {
            clearance: Millimeters(6),
            thickness: Millimeters(2),
            length: Permille(1_000),
            panel_count: 8,
            primary_color: TextileColor([150, 33, 29]),
            secondary_color: TextileColor([48, 78, 35]),
        };
        assert!(design.validate().is_ok());
        design.panel_count = 7;
        assert!(design.validate().is_err());
    }
}
