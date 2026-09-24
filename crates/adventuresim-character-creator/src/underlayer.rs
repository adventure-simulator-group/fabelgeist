//! Fitted textile garments and detachable mail patches cut from the source
//! body.
mod pattern;
pub use pattern::{RegionFrame, regions};

use anyhow::{Result, ensure};
use fabelgeist_armor::{Millimeters, Permille, TextileColor};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnderlayerKind {
    ArmingDoublet,
    PaddedHose,
    MailVoiders,
    MailBrayette,
    MailKneeVoider,
    MailStandard,
}

impl UnderlayerKind {
    pub fn is_mail(self) -> bool {
        matches!(
            self,
            Self::MailVoiders | Self::MailBrayette | Self::MailKneeVoider | Self::MailStandard
        )
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnderlayerDesign {
    pub kind: UnderlayerKind,
    pub clearance: Millimeters,
    pub thickness: Millimeters,
    pub color: TextileColor,
    pub length: Permille,
    pub sleeve_length: Permille,
    pub patch_width: Millimeters,
    /// Additional Boolean subtraction boxes, in the reference body's metre space.
    pub cuts: Vec<SurfaceBox>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SurfaceBox {
    pub minimum: ReferencePoint,
    pub maximum: ReferencePoint,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReferencePoint(pub [f32; 3]);

pub const CLEARANCE_MM: std::ops::RangeInclusive<u16> = 1..=10;
pub const THICKNESS_MM: std::ops::RangeInclusive<u16> = 1..=8;
pub const DOUBLET_LENGTH: std::ops::RangeInclusive<u16> = 700..=1200;
pub const HOSE_LENGTH: std::ops::RangeInclusive<u16> = 700..=1000;
pub const BRAYETTE_LENGTH: std::ops::RangeInclusive<u16> = 900..=1200;
pub const SLEEVE_LENGTH: std::ops::RangeInclusive<u16> = 500..=1000;
pub const PATCH_WIDTH_MM: std::ops::RangeInclusive<u16> = 35..=100;

impl UnderlayerDesign {
    pub fn length_range(&self) -> std::ops::RangeInclusive<u16> {
        match self.kind {
            UnderlayerKind::PaddedHose => HOSE_LENGTH,
            UnderlayerKind::MailBrayette => BRAYETTE_LENGTH,
            _ => DOUBLET_LENGTH,
        }
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(
            CLEARANCE_MM.contains(&self.clearance.0),
            "underlayer clearance must be 1..10 mm"
        );
        ensure!(
            THICKNESS_MM.contains(&self.thickness.0),
            "underlayer thickness must be 1..8 mm"
        );
        ensure!(
            self.length_range().contains(&self.length.0),
            "underlayer length is outside its construction range"
        );
        ensure!(
            SLEEVE_LENGTH.contains(&self.sleeve_length.0),
            "underlayer sleeve length must be 500..1000 permille"
        );
        ensure!(
            PATCH_WIDTH_MM.contains(&self.patch_width.0),
            "mail patch width must be 35..100 mm"
        );
        for cut in &self.cuts {
            ensure!(
                (0..3).all(|i| cut.minimum.0[i].is_finite()
                    && cut.maximum.0[i].is_finite()
                    && cut.minimum.0[i] < cut.maximum.0[i]),
                "invalid underlayer subtraction box"
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn design() -> UnderlayerDesign {
        UnderlayerDesign {
            kind: UnderlayerKind::ArmingDoublet,
            clearance: Millimeters(4),
            thickness: Millimeters(1),
            color: TextileColor([220, 205, 170]),
            length: Permille(1000),
            sleeve_length: Permille(1000),
            patch_width: Millimeters(80),
            cuts: vec![],
        }
    }

    #[test]
    fn invalid_cut_and_construction_ranges_are_rejected() {
        let mut candidate = design();
        candidate.kind = UnderlayerKind::PaddedHose;
        candidate.length = Permille(1100);
        assert!(candidate.validate().is_err());
        candidate.length = Permille(700);
        assert!(candidate.validate().is_ok());
        candidate.cuts.push(SurfaceBox {
            minimum: ReferencePoint([0.; 3]),
            maximum: ReferencePoint([1., f32::NAN, 1.]),
        });
        assert!(candidate.validate().is_err());
        candidate.cuts[0].maximum = ReferencePoint([0., 1., 1.]);
        assert!(candidate.validate().is_err());
    }
}
