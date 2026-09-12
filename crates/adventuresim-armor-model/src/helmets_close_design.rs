//! Controls for a plain three-plate close helmet and its pierced visor.
use serde::{Deserialize, Serialize};

use super::HelmetFit;
use crate::{DesignError, Millimeters, Permille, VisorBreaths};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CloseHelmetDesign {
    pub bellows: Option<crate::VisorBellows>,
    /// Requested maximum extension below the jaw. Anatomical fitting raises
    /// the hem when the wearer or underlying plates leave less room.
    pub neck_length: Millimeters,
    pub sight_ledge: Millimeters,
    /// Visor overlap above the skull's brow, independent of eye-slot height.
    pub brow_overlap: Millimeters,
    /// Additional rise of the visor's upper edge at the forehead center.
    pub brow_peak: Millimeters,
    pub crown: crate::HelmetCrown,
    pub visor_fluting: Option<crate::PlateFluting>,
    pub fit: HelmetFit,
    /// Skin clearance at the jaw and neck, independent of cranial lining space.
    pub face_clearance: Millimeters,
    /// Lateral lining space at the ears, independent of crown padding.
    pub temple_clearance: Millimeters,
    /// Minimum jaw and neck widths relative to the temple envelope.
    /// Anatomical clearance can require wider sections.
    pub jaw_width: Permille,
    pub neck_width: Permille,
    pub visor_projection: Millimeters,
    pub sight_gap: Millimeters,
    pub sight_span: Millimeters,
    pub sight_bridge: Millimeters,
    pub comb_height: Millimeters,
    pub throat_flare: Millimeters,
    /// Rear neck flange, independent of the throat's terminal flare.
    pub back_flare: Millimeters,
    pub back_edge_lift: Millimeters,
    /// Length and outward sweep of the three overlapping nape lames.
    pub nape_length: Millimeters,
    pub nape_flare: Millimeters,
    pub chin_projection: Millimeters,
    /// Position of the projecting face ridge, from visor brow to lower edge.
    pub ridge_height: Permille,
    /// Zero rounds the ridge; one thousand retains the angular profile.
    pub ridge_sharpness: Permille,
    pub breaths: VisorBreaths,
}

impl Default for CloseHelmetDesign {
    fn default() -> Self {
        Self {
            bellows: None,
            neck_length: Millimeters(0),
            sight_ledge: Millimeters(0),
            brow_overlap: Millimeters(30),
            brow_peak: Millimeters(25),
            crown: crate::HelmetCrown::default(),
            visor_fluting: None,
            fit: HelmetFit {
                clearance: Millimeters(10),
                crown_height: Permille(1000),
                ..HelmetFit::default()
            },
            face_clearance: Millimeters(5),
            temple_clearance: Millimeters(5),
            jaw_width: Permille(840),
            neck_width: Permille(800),
            visor_projection: Millimeters(12),
            sight_gap: Millimeters(6),
            sight_span: Millimeters(170),
            sight_bridge: Millimeters(5),
            comb_height: Millimeters(6),
            throat_flare: Millimeters(3),
            back_flare: Millimeters(3),
            back_edge_lift: Millimeters(24),
            nape_length: Millimeters(50),
            nape_flare: Millimeters(70),
            chin_projection: Millimeters(0),
            ridge_height: Permille(350),
            ridge_sharpness: Permille(950),
            breaths: VisorBreaths::default(),
        }
    }
}

impl CloseHelmetDesign {
    pub(super) fn validate_shape(&self) -> Result<(), DesignError> {
        if let Some(bellows) = &self.bellows {
            bellows.validate()?;
        }
        self.crown.validate()?;
        if let Some(pattern) = &self.visor_fluting {
            pattern.validate()?;
        }
        self.breaths.validate(450..=850)?;
        // The pierced lifting plate uses the same gauge as the bowl. Larger
        // gauges require a different visor construction and opening treatment.
        let valid = self.neck_length.0 <= 45
            && self.sight_ledge.0 <= 12
            && (8..=40).contains(&self.brow_overlap.0)
            && self.brow_peak.0 <= 30
            && self.fit.wall_thickness.0 <= 4
            && (3..=20).contains(&self.face_clearance.0)
            && (3..=15).contains(&self.temple_clearance.0)
            && (700..=1000).contains(&self.jaw_width.0)
            && (650..=1000).contains(&self.neck_width.0)
            && (12..=50).contains(&self.visor_projection.0)
            && (3..=12).contains(&self.sight_gap.0)
            && (70..=190).contains(&self.sight_span.0)
            && self.sight_bridge.0 <= 12
            && self.comb_height.0 <= 40
            && self.throat_flare.0 <= 15
            && self.back_flare.0 <= 20
            && (10..=45).contains(&self.back_edge_lift.0)
            && self.nape_length.0 <= 80
            && self.nape_flare.0 <= 90
            && self.chin_projection.0 <= 25
            && (300..=550).contains(&self.ridge_height.0)
            && self.ridge_sharpness.0 <= 1000;
        if !valid {
            return Err(DesignError::ParametricParameters);
        }
        Ok(())
    }
}
