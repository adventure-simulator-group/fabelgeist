//! Hairstyles: what is done with the hair once it is cut -- which way it is
//! combed, how full and how lifted, how firmly it holds, and whether it is
//! tied.
//!
//! [`Arrangement`] names the ways; [`Hairstyle`] is one with its numbers.
//! The cut ([`crate::Haircut`]) says how long the hair is, the follicles
//! ([`crate::Follicles`]) which way it grows; a style combs over both, so
//! any style goes with any cut and any growth.

use serde::{Deserialize, Serialize};

/// Which way a style arranges the hair.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Arrangement {
    /// As it grows: the follicles' pattern, then gravity.
    Natural,
    /// Away from a part either side.
    Parted,
    /// Over to one side from a deep part.
    Swept,
    /// Combed straight back.
    Back,
    /// Combed forward.
    Forward,
    /// Down all round, no part.
    Down,
    /// Stood up and out from the head.
    Up,
    /// Over the scalp to a tie, then a tail.
    Ponytail,
    /// Over the scalp to a tie, then coiled round it.
    Bun,
    /// Over the scalp to a tie, then three lanes plaited.
    Braid,
}

impl Arrangement {
    pub const ALL: [Arrangement; 10] = [
        Arrangement::Natural,
        Arrangement::Parted,
        Arrangement::Swept,
        Arrangement::Back,
        Arrangement::Forward,
        Arrangement::Down,
        Arrangement::Up,
        Arrangement::Ponytail,
        Arrangement::Bun,
        Arrangement::Braid,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Arrangement::Natural => "Natural",
            Arrangement::Parted => "Parted",
            Arrangement::Swept => "Side swept",
            Arrangement::Back => "Slicked back",
            Arrangement::Forward => "Combed forward",
            Arrangement::Down => "Down, no part",
            Arrangement::Up => "Stood up",
            Arrangement::Ponytail => "Ponytail",
            Arrangement::Bun => "Bun",
            Arrangement::Braid => "Braid",
        }
    }

    /// Whether the hair is gathered to a tie.
    pub fn tied(self) -> bool {
        matches!(
            self,
            Arrangement::Ponytail | Arrangement::Bun | Arrangement::Braid
        )
    }
}

/// A hairstyle.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Hairstyle {
    pub arrangement: Arrangement,
    /// The part, metres towards `+x` from the middle: where a parted style
    /// parts, and what a swept one sweeps away from.
    pub part: f32,
    /// How far the hair stands off the head at the crown, metres: the hair
    /// beneath lifts what lies over it.
    pub volume: f32,
    /// Added to the angle the follicles give, radians: product and blow
    /// drying lift the roots.
    pub lift: f32,
    /// How firmly the hair holds its shape: its gravity length, metres
    /// (about 5 cm for a loose strand, more for a styled or curly lock).
    pub hold: f32,
    /// How far off the head hair may be and still lie on it, metres: long
    /// hair leaves the head where it curves away, short or styled hair
    /// keeps to it.
    pub cling: f32,
    /// How far the last few centimetres turn under, radians.
    pub tuck: f32,
    /// How high the tie is, radians up the back of the head from level
    /// with its centre.
    pub tie: f32,
}

impl Hairstyle {
    pub fn new(arrangement: Arrangement) -> Self {
        let base = Self {
            arrangement,
            part: 0.0,
            volume: 0.012,
            lift: 0.0,
            hold: 0.05,
            cling: 0.004,
            tuck: 0.0,
            tie: 0.25,
        };
        match arrangement {
            Arrangement::Natural | Arrangement::Parted | Arrangement::Down => base,
            Arrangement::Swept => Self {
                part: -0.045,
                volume: 0.016,
                ..base
            },
            Arrangement::Back => Self {
                volume: 0.007,
                lift: -0.1,
                hold: 0.07,
                cling: 0.03,
                ..base
            },
            Arrangement::Forward => Self {
                volume: 0.006,
                hold: 0.06,
                cling: 0.03,
                ..base
            },
            Arrangement::Up => Self {
                volume: 0.0,
                lift: 0.85,
                hold: 0.25,
                cling: 0.03,
                ..base
            },
            Arrangement::Ponytail => Self {
                volume: 0.004,
                lift: -0.15,
                hold: 0.06,
                ..base
            },
            Arrangement::Bun => Self {
                volume: 0.004,
                lift: -0.15,
                tie: 0.8,
                ..base
            },
            Arrangement::Braid => Self {
                volume: 0.004,
                lift: -0.15,
                hold: 0.08,
                tie: -0.48,
                ..base
            },
        }
    }

    /// Whether the hair stands out from the head rather than lying on it.
    pub fn stands(&self) -> bool {
        self.arrangement == Arrangement::Up
    }

    /// Which way a swept style goes: away from its part.
    pub fn sweep(&self) -> f32 {
        if self.part > 0.0 { -1.0 } else { 1.0 }
    }
}
