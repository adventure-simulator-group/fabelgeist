//! Border ornaments drawn procedurally, as an alternative to a relief image.
//!
//! An ornament fills one engraving cell, which repeats like an image would:
//! along a trim band, the cell's x runs along the edge and its y in from it.
//! Its lines are cut to the engraving's depth, with antialiased edges, and
//! drawn on the device at the bake's own resolution, so they stay sharp at
//! any cell size.
use serde::{Deserialize, Serialize};

/// A procedural ornament: a motif repeated across the cell, between optional
/// fillet lines along its two long sides.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ornament {
    pub motif: Motif,
    /// Motif repeats across one cell.
    pub repeats: u32,
    /// Width of every cut line, as a fraction of the cell.
    pub line: f32,
    /// Whether a fillet line runs along each long side of the cell.
    pub fillets: bool,
}

/// The repeated figure. Heights are fractions of the room left between the
/// fillets, and lengths fractions of one repeat.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Motif {
    /// A running wave.
    Wave { amplitude: f32 },
    /// A running zigzag.
    Zigzag { amplitude: f32 },
    /// Interlaced waves out of phase with each other.
    Guilloche { amplitude: f32, strands: u32 },
    /// A twisted cord: grooves slanting across the band.
    Rope { slant: f32 },
    /// A row of beads.
    Beads { radius: f32 },
    /// A wave with a leaf in each of its bays.
    Vine { amplitude: f32, leaf: f32 },
}

impl Motif {
    /// Every motif at its default proportions, in the order a picker offers
    /// them.
    pub const ALL: [Motif; 6] = [
        Motif::Wave { amplitude: 0.8 },
        Motif::Zigzag { amplitude: 0.8 },
        Motif::Guilloche {
            amplitude: 0.8,
            strands: 3,
        },
        Motif::Rope { slant: 0.8 },
        Motif::Beads { radius: 0.7 },
        Motif::Vine {
            amplitude: 0.6,
            leaf: 0.6,
        },
    ];

    pub const MAX_STRANDS: u32 = 6;

    pub fn name(self) -> &'static str {
        match self {
            Motif::Wave { .. } => "Wave",
            Motif::Zigzag { .. } => "Zigzag",
            Motif::Guilloche { .. } => "Guilloche",
            Motif::Rope { .. } => "Rope",
            Motif::Beads { .. } => "Beads",
            Motif::Vine { .. } => "Vine",
        }
    }

    /// Repeats across a square cell that read well: a cord needs many
    /// strands, a wave only a couple of swells.
    pub fn suggested_repeats(self) -> u32 {
        match self {
            Motif::Wave { .. } | Motif::Guilloche { .. } | Motif::Vine { .. } => 2,
            Motif::Zigzag { .. } => 3,
            Motif::Beads { .. } => 4,
            Motif::Rope { .. } => 6,
        }
    }

    /// The motif's code in the ornament kernel, then its two shape
    /// parameters and strand count.
    pub(crate) fn words(self) -> (u32, f32, f32, u32) {
        match self {
            Motif::Wave { amplitude } => (0, amplitude, 0.0, 1),
            Motif::Zigzag { amplitude } => (1, amplitude, 0.0, 1),
            Motif::Guilloche { amplitude, strands } => (2, amplitude, 0.0, strands),
            Motif::Rope { slant } => (3, slant, 0.0, 1),
            Motif::Beads { radius } => (4, radius, 0.0, 1),
            Motif::Vine { amplitude, leaf } => (5, amplitude, leaf, 1),
        }
    }

    fn valid(self) -> bool {
        let unit = |x: f32| x.is_finite() && (0.0..=1.0).contains(&x);
        match self {
            Motif::Wave { amplitude } | Motif::Zigzag { amplitude } => unit(amplitude),
            Motif::Guilloche { amplitude, strands } => {
                unit(amplitude) && (1..=Self::MAX_STRANDS).contains(&strands)
            }
            Motif::Rope { slant } => slant.is_finite() && (0.1..=3.0).contains(&slant),
            Motif::Beads { radius } => radius.is_finite() && (0.1..=1.0).contains(&radius),
            Motif::Vine { amplitude, leaf } => unit(amplitude) && unit(leaf),
        }
    }
}

impl Default for Ornament {
    /// A running wave between two fillets.
    fn default() -> Self {
        Self {
            motif: Motif::ALL[0],
            repeats: Motif::ALL[0].suggested_repeats(),
            line: 0.06,
            fillets: true,
        }
    }
}

impl Ornament {
    pub const MAX_REPEATS: u32 = 8;
    pub const MIN_LINE: f32 = 0.01;
    pub const MAX_LINE: f32 = 0.2;

    pub fn validate(&self) -> Result<(), String> {
        if !(1..=Self::MAX_REPEATS).contains(&self.repeats)
            || !self.line.is_finite()
            || !(Self::MIN_LINE..=Self::MAX_LINE).contains(&self.line)
            || !self.motif.valid()
        {
            return Err("Invalid ornament parameters".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_default_motif_is_valid_and_named_apart() {
        for motif in Motif::ALL {
            let ornament = Ornament {
                motif,
                ..Ornament::default()
            };
            assert!(ornament.validate().is_ok(), "{motif:?}");
        }
        let names = Motif::ALL
            .map(Motif::name)
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(names.len(), Motif::ALL.len());
    }

    #[test]
    fn out_of_range_ornaments_are_rejected() {
        let default = Ornament::default();
        for invalid in [
            Ornament {
                repeats: 0,
                ..default.clone()
            },
            Ornament {
                line: 0.5,
                ..default.clone()
            },
            Ornament {
                motif: Motif::Guilloche {
                    amplitude: 0.5,
                    strands: 0,
                },
                ..default.clone()
            },
            Ornament {
                motif: Motif::Wave {
                    amplitude: f32::NAN,
                },
                ..default
            },
        ] {
            assert!(invalid.validate().is_err(), "{invalid:?}");
        }
    }
}
