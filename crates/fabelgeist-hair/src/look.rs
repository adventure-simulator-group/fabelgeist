//! A look: the few knobs a person turns, expanded into a full stack.
//!
//! DiffLocks draws 110 parameters per sample -- geometry and material -- each
//! from a range chosen by eye, and runs them through a fixed pipeline: guides
//! interpolated to ~100K strands, then clumping, curling and noise "repeated
//! multiple times with various intensities", shrinkwrap "at various
//! stages", the material, and a drape under random wind and gravity. A
//! [`Look`] is that pipeline with its knobs gathered into a handful that
//! mean something to a person (how clumped, how curly, how frizzy, how long
//! the drape) and [`Look::groom`] is the stack they expand to, in
//! DiffLocks' order:
//!
//! 1. the drape, on the guides (cheap there, and the full groom inherits it):
//!    the change gravity and wind make from standard conditions;
//! 2. interpolation, with the style's part, the balding mask, the uneven
//!    cover and the hairline's thinning, then every root turned the way its
//!    follicle points ([`crate::ops::Orient`]);
//! 3. length variation;
//! 4. coarse clumps -- the large locks;
//! 5. curls, following their own locks, or waves rippling through them;
//! 6. fine clumps at the tips;
//! 7. noise, then frizz on a few flyaway strands;
//! 8. shrinkwrap off the body -- the face and shoulders included, which
//!    hair reaching them rests on;
//! 9. the strands' radii.
//!
//! What the hair grows from, how it is cut and how it is styled are the
//! look's [`Hairdo`]; the rest of the look is the hair's texture.
//! [`Look::random`] draws one the way the dataset does, from a preset.

use crate::parameters::Draping;
use crate::parameters::GuideNeighborCount;
use crate::parameters::HairSeed;
use crate::parameters::LockFormation;
use crate::parameters::PointsPerStrand;
use crate::parameters::StrandCount;
use fabelgeist_math::units::Acceleration;
use fabelgeist_math::units::AccelerationVector;
use fabelgeist_math::units::Metres;
use fabelgeist_math::units::PerMetre;
use fabelgeist_math::units::RadiansPerMetre;
use fabelgeist_math::units::Ratio;
use fabelgeist_math::units::TurnsPerMetre;
use serde::{Deserialize, Serialize};

use crate::hairdo::Hairdo;
use crate::hairstyle::Arrangement;
use crate::head::Head;
use crate::ops::Context;
use crate::presets::Preset;
use crate::rng::Rng;
use crate::strands::Strands;
use crate::surface::Surface;
mod groom;
mod random;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Look {
    /// The preset this look started from.
    pub preset: Preset,
    /// The follicles, the cut and the style.
    pub hairdo: Hairdo,
    /// Strands grown from the guides.
    pub strands: StrandCount,
    /// Points per strand.
    pub points: PointsPerStrand,
    /// Guide jitter, 0 to 1.
    pub variation: Ratio,
    /// Neighbours blended by interpolation.
    pub neighbors: GuideNeighborCount,
    /// Share of length taken off at random: how ragged the ends are.
    pub length_variation: Ratio,
    /// Overall clumping, 0 to 1.
    pub clump: Ratio,
    /// Coarse lock size at the root, metres.
    pub clump_size: Metres,
    /// Twist of the coarse locks, radians a metre.
    pub twist: RadiansPerMetre,
    /// Locks clumped all along into ropes: dreadlocks.
    #[serde(default)]
    pub locs: LockFormation,
    /// How much the strands curl, 0 to 1.
    pub curl: Ratio,
    /// Curl radius, metres.
    pub curl_radius: Metres,
    /// Curl turns a metre.
    pub curl_turns: TurnsPerMetre,
    /// Curl lock size at the root, metres.
    pub curl_size: Metres,
    /// Where along the strand curls start, 0 to 1.
    pub curl_start: Ratio,
    /// Waves either side, metres, and crest to crest.
    #[serde(default)]
    pub wave: Metres,
    #[serde(default = "default_wavelength")]
    pub wavelength: Metres,
    /// Coherent waviness, metres.
    pub noise: Metres,
    /// Noise cells a metre along a strand.
    pub noise_scale: PerMetre,
    /// How far flyaway strands wander, metres.
    pub frizz: Metres,
    /// Share of strands that fly away.
    #[serde(default = "default_flyaways")]
    pub flyaways: Ratio,
    /// Drape under gravity and wind.
    pub drape: Draping,
    pub gravity: Acceleration,
    pub wind: AccelerationVector,
    pub seed: HairSeed,
}

fn default_wavelength() -> Metres {
    Metres::from_metres(0.08)
}

fn default_flyaways() -> Ratio {
    Ratio::from_ratio(0.06)
}

impl Look {
    /// A tidy default for a preset.
    pub fn new(preset: Preset) -> Self {
        let hairdo = preset.hairdo();
        let mut look = Self {
            preset,
            hairdo,
            strands: StrandCount::from_count(80_000),
            points: PointsPerStrand::from_count(48),
            variation: Ratio::from_ratio(0.4),
            neighbors: GuideNeighborCount::from_count(4),
            length_variation: Ratio::from_ratio(0.15),
            clump: Ratio::from_ratio(0.6),
            clump_size: Metres::from_metres(0.025),
            twist: RadiansPerMetre::from_radians_per_metre(0.0),
            locs: LockFormation::from_enabled(false),
            curl: Ratio::from_ratio(0.0),
            curl_radius: Metres::from_metres(0.008),
            curl_turns: TurnsPerMetre::from_turns_per_metre(12.0),
            curl_size: Metres::from_metres(0.008),
            curl_start: Ratio::from_ratio(0.15),
            wave: Metres::from_metres(0.004),
            wavelength: Metres::from_metres(0.09),
            noise: Metres::from_metres(0.006),
            noise_scale: PerMetre::from_per_metre(5.0),
            frizz: Metres::from_metres(0.003),
            flyaways: Ratio::from_ratio(0.04),
            drape: Draping::from_enabled(hairdo.drapes()),
            gravity: Acceleration::from_metres_per_second_squared(9.81),
            wind: AccelerationVector::from_components([0.0; 3]),
            seed: HairSeed::from_word(1),
        };
        match preset {
            Preset::Dreadlocks => {
                look.points = PointsPerStrand::from_count(64);
                look.locs = LockFormation::from_enabled(true);
                look.clump = Ratio::from_ratio(1.0);
                look.clump_size = Metres::from_metres(0.013);
                look.twist = RadiansPerMetre::from_radians_per_metre(30.0);
                look.wave = Metres::from_metres(0.006);
                look.frizz = Metres::from_metres(0.002);
                look.flyaways = Ratio::from_ratio(0.25);
                look.length_variation = Ratio::from_ratio(0.25);
            }
            Preset::Braid => {
                look.frizz = Metres::from_metres(0.0015);
                look.points = PointsPerStrand::from_count(128);
                look.clump = Ratio::from_ratio(0.15);
                look.wave = Metres::from_metres(0.0);
                look.length_variation = Ratio::from_ratio(0.05);
                look.flyaways = Ratio::from_ratio(0.03);
            }
            Preset::Ponytail => {
                look.points = PointsPerStrand::from_count(64);
                look.clump = Ratio::from_ratio(0.3);
                look.wave = Metres::from_metres(0.003);
            }
            Preset::Bun => {
                look.frizz = Metres::from_metres(0.001);
                look.points = PointsPerStrand::from_count(128);
                look.clump = Ratio::from_ratio(0.2);
                look.wave = Metres::from_metres(0.0);
                look.length_variation = Ratio::from_ratio(0.05);
            }
            Preset::Undercut | Preset::Mohawk => {
                look.strands = StrandCount::from_count(90_000);
                look.points = PointsPerStrand::from_count(24);
                look.clump = Ratio::from_ratio(0.5);
                look.clump_size = Metres::from_metres(0.012);
                look.wave = Metres::from_metres(0.0);
                look.noise = Metres::from_metres(0.0015);
                look.frizz = Metres::from_metres(0.002);
            }
            // Tight coils in small locks, wandering: a cloud, not rays.
            Preset::Afro => {
                look.strands = StrandCount::from_count(80_000);
                look.points = PointsPerStrand::from_count(160);
                look.curl = Ratio::from_ratio(1.0);
                look.curl_radius = Metres::from_metres(0.0035);
                look.curl_turns = TurnsPerMetre::from_turns_per_metre(110.0);
                look.curl_size = Metres::from_metres(0.003);
                look.curl_start = Ratio::from_ratio(0.0);
                look.clump = Ratio::from_ratio(0.25);
                look.clump_size = Metres::from_metres(0.01);
                look.noise = Metres::from_metres(0.012);
                look.noise_scale = PerMetre::from_per_metre(12.0);
                look.frizz = Metres::from_metres(0.003);
                look.flyaways = Ratio::from_ratio(0.3);
                look.wave = Metres::from_metres(0.0);
            }
            Preset::CurlyTop => {
                look.strands = StrandCount::from_count(100_000);
                look.points = PointsPerStrand::from_count(96);
                look.curl = Ratio::from_ratio(1.0);
                look.curl_radius = Metres::from_metres(0.004);
                look.curl_turns = TurnsPerMetre::from_turns_per_metre(70.0);
                look.curl_size = Metres::from_metres(0.004);
                look.curl_start = Ratio::from_ratio(0.05);
                look.clump = Ratio::from_ratio(0.3);
                look.clump_size = Metres::from_metres(0.01);
                look.wave = Metres::from_metres(0.0);
                look.noise = Metres::from_metres(0.004);
                look.noise_scale = PerMetre::from_per_metre(15.0);
            }
            Preset::Ringlets => {
                look.points = PointsPerStrand::from_count(160);
                look.curl = Ratio::from_ratio(1.0);
                look.curl_radius = Metres::from_metres(0.007);
                look.curl_turns = TurnsPerMetre::from_turns_per_metre(18.0);
                look.curl_size = Metres::from_metres(0.01);
                look.curl_start = Ratio::from_ratio(0.12);
                look.clump = Ratio::from_ratio(0.4);
                look.wave = Metres::from_metres(0.0);
            }
            Preset::Shag => {
                look.clump = Ratio::from_ratio(0.7);
                look.wave = Metres::from_metres(0.01);
                look.wavelength = Metres::from_metres(0.08);
                look.noise = Metres::from_metres(0.012);
                look.noise_scale = PerMetre::from_per_metre(6.0);
                look.frizz = Metres::from_metres(0.004);
                look.flyaways = Ratio::from_ratio(0.1);
            }
            // A fringe is cut blunt: its tips must not gather into points.
            Preset::Bangs => {
                look.clump = Ratio::from_ratio(0.3);
                look.length_variation = Ratio::from_ratio(0.08);
            }
            Preset::BowlCut => {
                look.clump = Ratio::from_ratio(0.3);
                look.wave = Metres::from_metres(0.0);
                look.length_variation = Ratio::from_ratio(0.05);
            }
            Preset::Quiff | Preset::SweptFringe => {
                look.strands = StrandCount::from_count(90_000);
                look.points = PointsPerStrand::from_count(32);
                look.clump = Ratio::from_ratio(0.6);
                look.clump_size = Metres::from_metres(0.015);
                look.wave = Metres::from_metres(0.0);
                look.noise = Metres::from_metres(0.004);
                look.noise_scale = PerMetre::from_per_metre(10.0);
                look.frizz = Metres::from_metres(0.002);
            }
            Preset::Balding => {
                look.strands = StrandCount::from_count(100_000);
                look.points = PointsPerStrand::from_count(12);
                look.clump = Ratio::from_ratio(0.2);
                look.wave = Metres::from_metres(0.0);
                look.noise = Metres::from_metres(0.001);
                look.frizz = Metres::from_metres(0.0);
            }
            Preset::Buzz => {
                look.strands = StrandCount::from_count(120_000);
                look.points = PointsPerStrand::from_count(8);
                look.clump = Ratio::from_ratio(0.0);
                look.noise = Metres::from_metres(0.0003);
                look.frizz = Metres::from_metres(0.0);
                look.wave = Metres::from_metres(0.0);
                look.length_variation = Ratio::from_ratio(0.3);
            }
            Preset::Crop | Preset::Spiky => {
                look.strands = StrandCount::from_count(100_000);
                look.points = PointsPerStrand::from_count(16);
                look.clump = Ratio::from_ratio(if preset == Preset::Crop { 0.3 } else { 0.55 });
                look.clump_size = Metres::from_metres(0.012);
                look.noise = Metres::from_metres(0.004);
                look.noise_scale = PerMetre::from_per_metre(20.0);
                look.wave = Metres::from_metres(0.0);
                look.frizz = Metres::from_metres(0.002);
            }
            Preset::Pixie => {
                look.strands = StrandCount::from_count(80_000);
                look.points = PointsPerStrand::from_count(24);
                look.clump_size = Metres::from_metres(0.015);
                look.wave = Metres::from_metres(0.002);
                look.frizz = Metres::from_metres(0.002);
            }
            _ => {}
        }
        look
    }

    /// A look drawn the way the dataset draws one: every knob from a range
    /// that reads as real hair for the preset -- its follicles, cut and
    /// style as much as its texture.
    pub fn random(preset: Preset, rng: &mut Rng) -> Self {
        let mut look = Self::new(preset);
        look.seed = HairSeed::from_word(rng.next_u32() as i32 & 0x7FFF_FFFF);
        look.variation = Ratio::from_ratio(rng.range(0.2, 0.8));
        let hairdo = &mut look.hairdo;

        random::follicles(&mut hairdo.follicles, rng);

        // The cut, longer or shorter; the style fuller or flatter, lifted
        // more or less, parted elsewhere -- and now and then combed another
        // way.
        hairdo.haircut = hairdo.haircut.scaled(rng.range(0.75, 1.25));
        let style = &mut hairdo.hairstyle;
        style.volume *= rng.log_range(0.6, 2.2);
        style.lift *= rng.range(0.7, 1.5);
        style.part += rng.range(-0.03, 0.03);
        let loose = [
            Arrangement::Natural,
            Arrangement::Parted,
            Arrangement::Swept,
            Arrangement::Down,
        ];
        if loose.contains(&style.arrangement) && rng.chance(0.25) {
            style.arrangement = loose[rng.index(loose.len())];
            if style.arrangement == Arrangement::Swept && style.part.abs() < 0.02 {
                style.part = if rng.chance(0.5) { 0.045 } else { -0.045 };
            }
        }

        look.neighbors = GuideNeighborCount::from_count(3 + rng.index(4));
        look.length_variation = Ratio::from_ratio(rng.range(0.0, 0.35));
        look.clump = Ratio::from_ratio(rng.range(0.1, 0.9));
        look.clump_size = Metres::from_metres(rng.log_range(0.01, 0.05));
        look.twist = RadiansPerMetre::from_radians_per_metre(if rng.chance(0.3) {
            rng.range(-8.0, 8.0)
        } else {
            0.0
        });
        look.noise = Metres::from_metres(rng.log_range(0.001, 0.012));
        look.noise_scale = PerMetre::from_per_metre(rng.log_range(2.0, 15.0));
        look.frizz = Metres::from_metres(rng.log_range(0.001, 0.006));
        look.flyaways = Ratio::from_ratio(rng.log_range(0.01, 0.2));
        let short = preset.is_short();
        let tied = look.hairdo.hairstyle.arrangement.tied();
        // Texture: straight, wavy, loose curls, curly or coily -- weighted
        // as hair is, most of it straight or wavy.
        let kind = match preset {
            Preset::Afro => 4,
            Preset::CurlyTop => [3, 4][rng.index(2)],
            Preset::Ringlets => [2, 3][rng.index(2)],
            Preset::Dreadlocks | Preset::Buzz | Preset::Balding => 0,
            _ if short => [0, 0, 1, 3][rng.index(4)],
            _ if tied => [0, 0, 1, 1, 2][rng.index(5)],
            _ => [0, 0, 0, 1, 1, 1, 2, 2, 3, 4][rng.index(10)],
        };
        look.wave = Metres::from_metres(0.0);
        look.curl = Ratio::from_ratio(0.0);
        match kind {
            0 => look.wave = Metres::from_metres(rng.range(0.0, 0.004)),
            1 => {
                look.wave = Metres::from_metres(rng.range(0.005, 0.016));
                look.wavelength = Metres::from_metres(rng.range(0.05, 0.11));
            }
            2 => {
                look.curl = Ratio::from_ratio(rng.range(0.5, 1.0));
                look.curl_radius = Metres::from_metres(rng.range(0.006, 0.014));
                look.curl_turns = TurnsPerMetre::from_turns_per_metre(rng.range(6.0, 12.0));
                look.curl_size = Metres::from_metres(rng.log_range(0.008, 0.025));
                look.curl_start = Ratio::from_ratio(rng.range(0.1, 0.35));
            }
            3 => {
                look.curl = Ratio::from_ratio(rng.range(0.8, 1.0));
                look.curl_radius = Metres::from_metres(rng.range(0.004, 0.009));
                look.curl_turns = TurnsPerMetre::from_turns_per_metre(rng.range(15.0, 40.0));
                look.curl_size = Metres::from_metres(rng.log_range(0.004, 0.012));
                look.curl_start = Ratio::from_ratio(rng.range(0.03, 0.2));
            }
            _ => {
                look.curl = Ratio::from_ratio(1.0);
                look.curl_radius = Metres::from_metres(rng.range(0.002, 0.005));
                look.curl_turns = TurnsPerMetre::from_turns_per_metre(rng.range(60.0, 140.0));
                look.curl_size = Metres::from_metres(rng.log_range(0.002, 0.006));
                look.curl_start = Ratio::from_ratio(rng.range(0.0, 0.06));
                // Coily hair's follicles are curved: it wanders.
                look.hairdo.follicles.kink = rng.range(1.0, 3.0);
            }
        }
        if preset == Preset::Afro {
            look.noise = Metres::from_metres(rng.range(0.008, 0.02));
            look.noise_scale = PerMetre::from_per_metre(rng.range(8.0, 16.0));
            look.curl_size = Metres::from_metres(rng.range(0.002, 0.005));
            look.flyaways = Ratio::from_ratio(rng.range(0.15, 0.4));
            look.hairdo.follicles.kink = rng.range(2.0, 4.0);
        }
        let follicles = &mut look.hairdo.follicles;
        if preset == Preset::Balding {
            follicles.balding = [
                rng.range(0.5, 1.0),
                rng.range(0.4, 1.0),
                rng.range(0.3, 0.8),
            ];
        } else if rng.chance(0.15) {
            follicles.balding = [
                rng.range(0.0, 1.0),
                rng.range(0.0, 1.0),
                rng.range(0.0, 0.8),
            ];
        }
        if preset == Preset::Dreadlocks {
            look.clump = Ratio::from_ratio(rng.range(0.9, 1.0));
            look.clump_size = Metres::from_metres(rng.range(0.01, 0.02));
            look.twist = RadiansPerMetre::from_radians_per_metre(rng.range(15.0, 40.0));
        }
        // Enough points for about eight a turn, within the dataset's 256.
        if look.curl.ratio() > 0.0 && !short {
            let length = 0.5;
            let turns = look.curl_turns.turns_per_metre() * length;
            look.points = PointsPerStrand::from_count(
                ((turns * 8.0) as usize).clamp(look.points.count(), 256),
            );
        }
        look.drape = Draping::from_enabled(look.hairdo.drapes() && rng.chance(0.85));
        look.gravity = Acceleration::from_metres_per_second_squared(rng.range(7.0, 12.0));
        let wind_angle = rng.range(0.0, std::f32::consts::TAU);
        let wind = if rng.chance(0.5) {
            rng.range(0.0, 4.0)
        } else {
            0.0
        };
        look.wind = AccelerationVector::from_components([
            wind_angle.sin() * wind,
            0.0,
            wind_angle.cos() * wind,
        ]);
        look
    }

    /// `base` with this look's hairline: the head to grow it on, and to cut
    /// its scalp from ([`Head::scalp`]).
    pub fn head(&self, base: &Head) -> Head {
        self.hairdo.head(base)
    }

    /// The guides this look grows from, on its `head` ([`Look::head`]).
    pub fn guides(&self, head: &Head, scalp: &Surface) -> Strands {
        self.hairdo.guides(
            head,
            scalp,
            64,
            self.variation.ratio(),
            self.seed.word() as u64,
        )
    }

    /// The whole groom on its `head` ([`Look::head`]), rooted on `scalp`:
    /// the guides, and the stack run over them.
    pub fn grow(&self, head: &Head, scalp: &Surface) -> Strands {
        let context = Context {
            scalp,
            collider: head,
        };
        self.groom(head).grow(&self.guides(head, scalp), &context)
    }
}
