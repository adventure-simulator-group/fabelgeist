//! Presets: named hairdos to start from.
//!
//! A preset is follicles, a cut and a style ([`crate::Hairdo`]) that go by
//! one name, and the texture a [`crate::Look`] gives it; every part is free
//! to change after. [`Preset::PAPER`] is one for each kind of hair
//! DiffLocks' figures show -- its 75 base styles are not published:
//!
//! | Preset | Where the paper shows it |
//! | --- | --- |
//! | [`Preset::Bob`] | Fig. 4, the first base: centre part, to the jaw; Fig. 7 and 18, wavy bobs |
//! | [`Preset::Crop`] | Fig. 4, the second base: short, combed forward and up |
//! | [`Preset::Long`] | Fig. 3, 17: long and straight from a centre part |
//! | [`Preset::Medium`] | Fig. 7, 18: shoulder length, side part |
//! | [`Preset::Shag`] | Fig. 7, 8: layered, messy, centre part |
//! | [`Preset::SideSwept`] | Fig. 18: long, swept from a deep side part |
//! | [`Preset::Bangs`] | Fig. 17: long with a fringe |
//! | [`Preset::BowlCut`] | Fig. 17: straight all round, a fringe to the brow |
//! | [`Preset::Ringlets`] | Fig. 3, 17, 18: long curls and waves |
//! | [`Preset::CurlyTop`] | Fig. 4 (randomised), 17: short sides, curly top |
//! | [`Preset::Afro`] | Fig. 1, 3, 9: a dome of coils |
//! | [`Preset::SlickedBack`] | Fig. 17: short, combed back |
//! | [`Preset::Quiff`] | Fig. 3: short, the front pushed up and back |
//! | [`Preset::SweptFringe`] | Fig. 3: short, the fringe swept to one side |
//! | [`Preset::Pixie`] | Fig. 8: short and tousled |
//! | [`Preset::Buzz`] | Fig. 7, 17: clipped close |
//! | [`Preset::Balding`] | Fig. 4 (randomised), 7, 10, 17: thinning crown, receding temples |
//!
//! Curliness, waves, length, volume and density are the randomisation's --
//! a [`crate::Look`] -- not separate presets, as in the paper. The paper
//! names what its data lacks: "braids, accessories, ponytails, and buns";
//! [`Preset::Ponytail`], [`Preset::Bun`] and [`Preset::Braid`] (and the
//! dreadlocks, undercut, mohawk and spiky ones) go beyond it.

use serde::{Deserialize, Serialize};

use crate::follicles::Follicles;
use crate::haircut::{Cut, Fringe, Haircut};
use crate::hairdo::Hairdo;
use crate::hairstyle::{Arrangement, Hairstyle};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Preset {
    Long,
    Medium,
    Bob,
    Bangs,
    SideSwept,
    SlickedBack,
    Ponytail,
    Bun,
    Braid,
    Dreadlocks,
    Undercut,
    Pixie,
    Crop,
    Mohawk,
    Buzz,
    Spiky,
    Afro,
    BowlCut,
    Quiff,
    Shag,
    CurlyTop,
    SweptFringe,
    Ringlets,
    Balding,
}

impl Preset {
    pub const ALL: [Preset; 24] = [
        Preset::Long,
        Preset::Medium,
        Preset::Bob,
        Preset::Bangs,
        Preset::SideSwept,
        Preset::SlickedBack,
        Preset::Ponytail,
        Preset::Bun,
        Preset::Braid,
        Preset::Dreadlocks,
        Preset::Undercut,
        Preset::Pixie,
        Preset::Crop,
        Preset::Mohawk,
        Preset::Buzz,
        Preset::Spiky,
        Preset::Afro,
        Preset::BowlCut,
        Preset::Quiff,
        Preset::Shag,
        Preset::CurlyTop,
        Preset::SweptFringe,
        Preset::Ringlets,
        Preset::Balding,
    ];

    /// One for each kind of hair the paper shows (see the module's table).
    pub const PAPER: [Preset; 17] = [
        Preset::Bob,
        Preset::Crop,
        Preset::Long,
        Preset::Medium,
        Preset::Shag,
        Preset::SideSwept,
        Preset::Bangs,
        Preset::BowlCut,
        Preset::Ringlets,
        Preset::CurlyTop,
        Preset::Afro,
        Preset::SlickedBack,
        Preset::Quiff,
        Preset::SweptFringe,
        Preset::Pixie,
        Preset::Buzz,
        Preset::Balding,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Preset::Long => "Long",
            Preset::Medium => "Medium",
            Preset::Bob => "Bob",
            Preset::Bangs => "Long with bangs",
            Preset::SideSwept => "Side swept",
            Preset::SlickedBack => "Slicked back",
            Preset::Ponytail => "Ponytail",
            Preset::Bun => "Bun",
            Preset::Braid => "Braid",
            Preset::Dreadlocks => "Dreadlocks",
            Preset::Undercut => "Undercut",
            Preset::Pixie => "Pixie",
            Preset::Crop => "Crop",
            Preset::Mohawk => "Mohawk",
            Preset::Buzz => "Buzz cut",
            Preset::Spiky => "Spiky",
            Preset::Afro => "Afro",
            Preset::BowlCut => "Bowl cut",
            Preset::Quiff => "Quiff",
            Preset::Shag => "Shag",
            Preset::CurlyTop => "Curly top",
            Preset::SweptFringe => "Swept fringe",
            Preset::Ringlets => "Ringlets",
            Preset::Balding => "Balding",
        }
    }

    /// Whether the preset's hair is short: a few centimetres or less.
    pub fn is_short(self) -> bool {
        self.hairdo().haircut.longest() < 0.16
    }

    /// The preset's follicles, cut and style.
    pub fn hairdo(self) -> Hairdo {
        let cut = Haircut::new;
        let style = Hairstyle::new;
        let lengths = |cut: Cut, top: f32, front: f32, sides: f32, back: f32| Haircut {
            top,
            front,
            sides,
            back,
            ..Haircut::new(cut)
        };
        let short = |arrangement: Arrangement, lift: f32, volume: f32, hold: f32| Hairstyle {
            lift,
            volume,
            hold,
            cling: 0.03,
            ..Hairstyle::new(arrangement)
        };
        let mut follicles = Follicles::default();
        let (haircut, hairstyle) = match self {
            Preset::Long => (cut(Cut::Long), style(Arrangement::Parted)),
            Preset::Medium => (
                cut(Cut::Shoulder),
                Hairstyle {
                    part: 0.025,
                    tuck: 0.3,
                    ..style(Arrangement::Parted)
                },
            ),
            Preset::Bob => (
                cut(Cut::Bob),
                Hairstyle {
                    part: -0.02,
                    volume: 0.015,
                    hold: 0.07,
                    tuck: 0.9,
                    ..style(Arrangement::Parted)
                },
            ),
            Preset::Bangs => (
                Haircut {
                    hemline: Some(-0.42),
                    fringe: Some(Fringe::default()),
                    ..cut(Cut::Long)
                },
                style(Arrangement::Down),
            ),
            Preset::SideSwept => (
                Haircut {
                    hemline: Some(-0.4),
                    ..lengths(Cut::Long, 0.4, 0.38, 0.39, 0.4)
                },
                Hairstyle {
                    tuck: 0.2,
                    ..style(Arrangement::Swept)
                },
            ),
            Preset::SlickedBack => (cut(Cut::Collar), style(Arrangement::Back)),
            Preset::Ponytail => (
                Haircut {
                    hemline: Some(-0.42),
                    ..lengths(Cut::Long, 0.5, 0.5, 0.5, 0.5)
                },
                style(Arrangement::Ponytail),
            ),
            Preset::Bun => (
                Haircut {
                    hemline: None,
                    ..cut(Cut::Long)
                },
                style(Arrangement::Bun),
            ),
            Preset::Braid => (
                Haircut {
                    hemline: Some(-0.48),
                    ..lengths(Cut::Long, 0.55, 0.55, 0.55, 0.55)
                },
                style(Arrangement::Braid),
            ),
            Preset::Dreadlocks => (
                Haircut {
                    hemline: None,
                    ..lengths(Cut::Shoulder, 0.34, 0.22, 0.3, 0.34)
                },
                Hairstyle {
                    lift: 0.15,
                    volume: 0.02,
                    hold: 0.07,
                    ..style(Arrangement::Down)
                },
            ),
            Preset::Undercut => (
                cut(Cut::Undercut),
                short(Arrangement::Back, 0.1, 0.012, 0.09),
            ),
            Preset::Pixie => (
                cut(Cut::Pixie),
                Hairstyle {
                    part: 0.035,
                    ..short(Arrangement::Swept, 0.05, 0.01, 0.06)
                },
            ),
            Preset::Crop => (
                cut(Cut::Crop),
                short(Arrangement::Forward, 0.25, 0.006, 0.06),
            ),
            Preset::Mohawk => (cut(Cut::Mohawk), short(Arrangement::Up, 0.95, 0.0, 0.25)),
            Preset::Buzz => (cut(Cut::Buzz), short(Arrangement::Natural, 0.0, 0.002, 1.0)),
            Preset::Spiky => (
                lengths(Cut::Crop, 0.06, 0.05, 0.05, 0.06),
                short(Arrangement::Up, 0.75, 0.0, 0.25),
            ),
            Preset::Afro => {
                follicles.kink = 3.0;
                (cut(Cut::Rounded), short(Arrangement::Up, 0.85, 0.0, 0.2))
            }
            // Straight all round to one line at the brow, a fringe in front.
            Preset::BowlCut => (
                cut(Cut::BowlCut),
                Hairstyle {
                    volume: 0.014,
                    hold: 0.07,
                    cling: 0.012,
                    tuck: 0.6,
                    ..style(Arrangement::Down)
                },
            ),
            // Short sides, the front swept up and back.
            Preset::Quiff => (
                lengths(Cut::ShortBackAndSides, 0.1, 0.12, 0.08, 0.08),
                Hairstyle {
                    cling: 0.02,
                    ..short(Arrangement::Back, 0.55, 0.025, 0.11)
                },
            ),
            // Layered, shoulder length, centre part, the front shortest.
            Preset::Shag => (
                cut(Cut::Shag),
                Hairstyle {
                    volume: 0.022,
                    hold: 0.06,
                    tuck: 0.15,
                    ..style(Arrangement::Parted)
                },
            ),
            // Short sides, a crop of curls on top.
            Preset::CurlyTop => (
                lengths(Cut::ShortBackAndSides, 0.08, 0.07, 0.06, 0.06),
                short(Arrangement::Forward, 0.45, 0.02, 0.09),
            ),
            // Short, the fringe swept over to one side.
            Preset::SweptFringe => (
                lengths(Cut::Pixie, 0.07, 0.11, 0.05, 0.05),
                Hairstyle {
                    part: -0.04,
                    ..short(Arrangement::Swept, 0.15, 0.012, 0.07)
                },
            ),
            // Long curls from a slight side part.
            Preset::Ringlets => (
                Haircut {
                    hemline: Some(-0.32),
                    ..lengths(Cut::Long, 0.4, 0.34, 0.38, 0.4)
                },
                Hairstyle {
                    part: 0.02,
                    volume: 0.03,
                    hold: 0.06,
                    ..style(Arrangement::Parted)
                },
            ),
            // Short back and sides; the crown and temples thin.
            Preset::Balding => {
                follicles.balding = [0.9, 0.8, 0.6];
                follicles.hairline.recession = 0.012;
                (
                    lengths(Cut::Crop, 0.03, 0.025, 0.025, 0.03),
                    Hairstyle {
                        cling: 0.02,
                        ..short(Arrangement::Natural, 0.05, 0.004, 0.08)
                    },
                )
            }
        };
        Hairdo {
            follicles,
            haircut,
            hairstyle,
        }
    }
}
