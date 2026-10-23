//! Haircuts: how long the hair is, everywhere on the head, and where its
//! ends are cut.
//!
//! A cut is the same whatever the hair then does -- a bob is a bob parted,
//! swept back or tied -- so it is kept apart from the style
//! ([`crate::Hairstyle`]). [`Haircut`] gives a length for every follicle,
//! from the lengths at the top, the front hairline, over the ears and at the
//! back, and then what a hairdresser does to that:
//!
//! * a **hemline**, the line the ends are cut straight along, level or
//!   sloping down towards the face (an A-line) or away from it;
//! * **clippers** up the back and sides to a height, with a hard line (an
//!   undercut) or a fade;
//! * a **strip** down the middle kept long and the rest clipped (a mohawk);
//! * a **fringe**, the front cut to the brows;
//! * a **dome**, the ends cut round to a sphere (an afro's shape).
//!
//! [`Cut`] names the usual cuts; [`Haircut::new`] makes one, whose numbers
//! are then free to change.

use serde::{Deserialize, Serialize};

use crate::head::{Head, around};
use crate::ops::smoothstep;
use fabelgeist_math::Vec3;

/// The usual cuts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cut {
    /// One length, well past the shoulders.
    Long,
    /// One length to the shoulders.
    Shoulder,
    /// Long, the top and front cut shorter in layers.
    Layered,
    /// Shoulder length, heavily layered, the front shortest.
    Shag,
    /// One length to the collar.
    Collar,
    /// To the jaw, one length.
    Bob,
    /// A bob longer in front than behind.
    AngledBob,
    /// Straight all round at the brow, a fringe in front.
    BowlCut,
    /// Short, a little longer on top.
    Pixie,
    /// A few centimetres all over.
    Crop,
    /// Clipped back and sides under a longer top, faded.
    ShortBackAndSides,
    /// Clipped back and sides to a hard line under a long top.
    Undercut,
    /// Clipped almost to the skin low, growing longer up the head.
    Fade,
    /// A strip down the middle, the rest clipped.
    Mohawk,
    /// Clipped close all over.
    Buzz,
    /// The ends cut round to a dome.
    Rounded,
}

impl Cut {
    pub const ALL: [Cut; 16] = [
        Cut::Long,
        Cut::Shoulder,
        Cut::Layered,
        Cut::Shag,
        Cut::Collar,
        Cut::Bob,
        Cut::AngledBob,
        Cut::BowlCut,
        Cut::Pixie,
        Cut::Crop,
        Cut::ShortBackAndSides,
        Cut::Undercut,
        Cut::Fade,
        Cut::Mohawk,
        Cut::Buzz,
        Cut::Rounded,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Cut::Long => "Long",
            Cut::Shoulder => "Shoulder length",
            Cut::Layered => "Long layers",
            Cut::Shag => "Shag",
            Cut::Collar => "Collar length",
            Cut::Bob => "Bob",
            Cut::AngledBob => "Angled bob",
            Cut::BowlCut => "Bowl cut",
            Cut::Pixie => "Pixie",
            Cut::Crop => "Crop",
            Cut::ShortBackAndSides => "Short back and sides",
            Cut::Undercut => "Undercut",
            Cut::Fade => "Fade",
            Cut::Mohawk => "Mohawk",
            Cut::Buzz => "Buzz",
            Cut::Rounded => "Rounded",
        }
    }
}

/// A fringe: the front of the scalp cut short, to hang over the forehead.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Fringe {
    /// Half its width, metres.
    pub width: f32,
    /// How far back from the front of the head it reaches, metres.
    pub depth: f32,
    /// How far below the front hairline its ends are, metres (about 5 cm
    /// is the brows).
    pub drop: f32,
}

impl Default for Fringe {
    fn default() -> Self {
        Self {
            width: 0.066,
            depth: 0.07,
            drop: 0.048,
        }
    }
}

impl Fringe {
    /// Whether a root belongs to the fringe.
    pub fn holds(&self, root: Vec3) -> bool {
        root.z > FRONT - self.depth && root.x.abs() < self.width
    }

    /// The `z` behind which the fringe ends.
    pub fn back(&self) -> f32 {
        FRONT - self.depth
    }
}

/// About where the front of the head is, `z`.
const FRONT: f32 = 0.1;

/// A haircut: a length for every follicle, and where the ends are cut.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Haircut {
    /// The cut this one started from.
    pub cut: Cut,
    /// Lengths, metres: at the top, at the front hairline, over the ears
    /// and at the back.
    pub top: f32,
    pub front: f32,
    pub sides: f32,
    pub back: f32,
    /// The height the ends are cut straight along behind, `y` metres, if
    /// they are.
    pub hemline: Option<f32>,
    /// How much lower the hemline is a metre towards the face: positive is
    /// an A-line, longer in front.
    pub slope: f32,
    /// Clippers up to this height on the scalp (0 the hairline, 1 the
    /// crown); below it hair is `clipper` long. The change takes `fade` of
    /// the height: 0 is a hard line.
    pub clip: f32,
    pub clipper: f32,
    pub fade: f32,
    /// A strip this wide either side of the middle kept long, the rest
    /// clipped, metres (0: none).
    pub strip: f32,
    pub fringe: Option<Fringe>,
    /// The ends cut to a dome this far from the head's centre, metres (0:
    /// none).
    pub dome: f32,
}

impl Haircut {
    pub fn new(cut: Cut) -> Self {
        let long = Self {
            cut,
            top: 0.45,
            front: 0.4,
            sides: 0.43,
            back: 0.45,
            hemline: None,
            slope: 0.0,
            clip: 0.0,
            clipper: 0.006,
            fade: 0.0,
            strip: 0.0,
            fringe: None,
            dome: 0.0,
        };
        let short = |top: f32, front: f32, sides: f32, back: f32| Self {
            top,
            front,
            sides,
            back,
            ..long
        };
        match cut {
            Cut::Long => Self {
                hemline: Some(-0.45),
                ..long
            },
            Cut::Shoulder => Self {
                top: 0.32,
                front: 0.26,
                sides: 0.3,
                back: 0.32,
                hemline: Some(-0.27),
                ..long
            },
            Cut::Layered => Self {
                top: 0.24,
                front: 0.2,
                sides: 0.34,
                back: 0.45,
                hemline: Some(-0.4),
                ..long
            },
            Cut::Shag => Self {
                top: 0.2,
                front: 0.15,
                sides: 0.24,
                back: 0.3,
                hemline: Some(-0.24),
                ..long
            },
            Cut::Collar => Self {
                top: 0.2,
                front: 0.2,
                sides: 0.2,
                back: 0.2,
                hemline: Some(-0.12),
                ..long
            },
            Cut::Bob => Self {
                top: 0.26,
                front: 0.2,
                sides: 0.23,
                back: 0.26,
                hemline: Some(-0.09),
                ..long
            },
            Cut::AngledBob => Self {
                top: 0.28,
                front: 0.24,
                sides: 0.26,
                back: 0.26,
                hemline: Some(-0.06),
                slope: 0.45,
                ..long
            },
            Cut::BowlCut => Self {
                top: 0.16,
                front: 0.13,
                sides: 0.15,
                back: 0.16,
                hemline: Some(-0.01),
                fringe: Some(Fringe::default()),
                ..long
            },
            Cut::Pixie => short(0.08, 0.11, 0.05, 0.05),
            Cut::Crop => short(0.045, 0.04, 0.035, 0.04),
            Cut::ShortBackAndSides => Self {
                clip: 0.45,
                clipper: 0.012,
                fade: 0.12,
                ..short(0.1, 0.12, 0.08, 0.08)
            },
            Cut::Undercut => Self {
                clip: 0.5,
                clipper: 0.008,
                ..short(0.14, 0.15, 0.14, 0.14)
            },
            Cut::Fade => Self {
                clip: 0.55,
                clipper: 0.001,
                fade: 0.45,
                ..short(0.06, 0.07, 0.05, 0.05)
            },
            Cut::Mohawk => Self {
                strip: 0.022,
                clipper: 0.006,
                ..short(0.12, 0.1, 0.12, 0.12)
            },
            Cut::Buzz => short(0.009, 0.007, 0.008, 0.009),
            Cut::Rounded => Self {
                dome: 0.18,
                ..short(0.14, 0.1, 0.12, 0.12)
            },
        }
    }

    /// The cut, every length times `factor`, the hemline as far from the
    /// crown as the hair is long.
    pub fn scaled(mut self, factor: f32) -> Self {
        for length in [
            &mut self.top,
            &mut self.front,
            &mut self.sides,
            &mut self.back,
        ] {
            *length *= factor;
        }
        let crown = 0.115;
        self.hemline = self.hemline.map(|y| crown - (crown - y) * factor);
        self.dome = if self.dome > 0.0 {
            0.09 + (self.dome - 0.09) * factor
        } else {
            0.0
        };
        self
    }

    /// The longest hair anywhere, metres.
    pub fn longest(&self) -> f32 {
        self.top.max(self.front).max(self.sides).max(self.back)
    }

    /// Whether a root is clipped short (under the clippers, or outside a
    /// mohawk's strip).
    pub fn clipped(&self, head: &Head, root: Vec3) -> bool {
        (self.clip > 0.0 && head.elevation(root) < self.clip - 0.5 * self.fade)
            || (self.strip > 0.0 && root.x.abs() > self.strip)
    }

    /// Whether a root is in the fringe.
    pub fn in_fringe(&self, root: Vec3) -> bool {
        self.fringe.is_some_and(|f| f.holds(root))
    }

    /// How long the hair rooted at `root` is, metres (before any hemline
    /// or dome).
    pub fn length(&self, head: &Head, root: Vec3) -> f32 {
        if let Some(fringe) = self.fringe
            && fringe.holds(root)
        {
            // Long enough to reach the fringe's line.
            return 0.1 + fringe.drop;
        }
        let e = head.elevation(root);
        let a = head.azimuth(root);
        // Round the head by where the root is near the hairline, the top's
        // length towards the crown.
        let ring = around(self.front, self.sides, self.back, a);
        let mut length = ring + (self.top - ring) * smoothstep(0.25, 0.85, e);
        if self.clip > 0.0 {
            let half = 0.5 * self.fade + 1e-3;
            let t = smoothstep(self.clip - half, self.clip + half, e);
            length = self.clipper + (length - self.clipper) * t;
        }
        if self.strip > 0.0 {
            // Clipped outside the strip, the full length a little inside.
            let t = 1.0 - smoothstep(self.strip - 0.006, self.strip, root.x.abs());
            length = self.clipper + (length - self.clipper) * t;
        }
        length.max(0.0005)
    }

    /// The height a strand rooted at `root` is cut at where it is at `p`,
    /// if it is cut on a line: the fringe's line for the fringe, the
    /// hemline for the rest.
    pub fn line(&self, head: &Head, root: Vec3, p: Vec3) -> Option<f32> {
        if let Some(fringe) = self.fringe
            && fringe.holds(root)
        {
            return Some(head.hairline_height(0.0) - fringe.drop);
        }
        self.hemline.map(|y| y - self.slope * p.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clippers_cut_the_sides_and_leave_the_top() {
        let head = Head::mannequin();
        let cut = Haircut::new(Cut::Undercut);
        let side = head.cranium.center + Vec3::new(head.cranium.radii.x, 0.0, 0.0);
        let top = head.crown();
        assert!(cut.length(&head, side) < 0.01);
        assert!(cut.length(&head, top) > 0.1);
        assert!(cut.clipped(&head, side) && !cut.clipped(&head, top));
    }

    #[test]
    fn an_angled_bob_is_longer_in_front() {
        let head = Head::mannequin();
        let cut = Haircut::new(Cut::AngledBob);
        let root = head.crown();
        let front = cut.line(&head, root, Vec3::new(0.0, 0.0, 0.08)).unwrap();
        let back = cut.line(&head, root, Vec3::new(0.0, 0.0, -0.08)).unwrap();
        assert!(front < back - 0.05);
    }

    #[test]
    fn scaling_keeps_every_cut_positive() {
        let head = Head::mannequin();
        for cut in Cut::ALL {
            let haircut = Haircut::new(cut).scaled(0.7);
            for p in [head.crown(), Vec3::new(0.07, 0.03, 0.0)] {
                assert!(haircut.length(&head, p) > 0.0, "{cut:?}");
            }
        }
    }
}
