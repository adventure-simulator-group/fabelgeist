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

use serde::{Deserialize, Serialize};

use crate::follicles::Growth;
use crate::groom::Groom;
use crate::hairdo::Hairdo;
use crate::hairstyle::Arrangement;
use crate::head::Head;
use crate::ops::{
    Clump, Context, Curl, DensityMask, Frizz, Interpolate, Noise, Op, Orient, Profile, Settle,
    Shrinkwrap, Trim, Wave,
};
use crate::presets::Preset;
use crate::rng::Rng;
use crate::strands::Strands;
use crate::surface::Surface;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Look {
    /// The preset this look started from.
    pub preset: Preset,
    /// The follicles, the cut and the style.
    pub hairdo: Hairdo,
    /// Strands grown from the guides.
    pub strands: usize,
    /// Points per strand.
    pub points: usize,
    /// Guide jitter, 0 to 1.
    pub variation: f32,
    /// Neighbours blended by interpolation.
    pub neighbors: usize,
    /// Share of length taken off at random: how ragged the ends are.
    pub length_variation: f32,
    /// Overall clumping, 0 to 1.
    pub clump: f32,
    /// Coarse lock size at the root, metres.
    pub clump_size: f32,
    /// Twist of the coarse locks, radians a metre.
    pub twist: f32,
    /// Locks clumped all along into ropes: dreadlocks.
    #[serde(default)]
    pub locs: bool,
    /// How much the strands curl, 0 to 1.
    pub curl: f32,
    /// Curl radius, metres.
    pub curl_radius: f32,
    /// Curl turns a metre.
    pub curl_turns: f32,
    /// Curl lock size at the root, metres.
    pub curl_size: f32,
    /// Where along the strand curls start, 0 to 1.
    pub curl_start: f32,
    /// Waves either side, metres, and crest to crest.
    #[serde(default)]
    pub wave: f32,
    #[serde(default = "default_wavelength")]
    pub wavelength: f32,
    /// Coherent waviness, metres.
    pub noise: f32,
    /// Noise cells a metre along a strand.
    pub noise_scale: f32,
    /// How far flyaway strands wander, metres.
    pub frizz: f32,
    /// Share of strands that fly away.
    #[serde(default = "default_flyaways")]
    pub flyaways: f32,
    /// Drape under gravity and wind.
    pub drape: bool,
    pub gravity: f32,
    pub wind: [f32; 3],
    pub seed: i32,
}

fn default_wavelength() -> f32 {
    0.08
}

fn default_flyaways() -> f32 {
    0.06
}

impl Look {
    /// A tidy default for a preset.
    pub fn new(preset: Preset) -> Self {
        let hairdo = preset.hairdo();
        let mut look = Self {
            preset,
            hairdo,
            strands: 80_000,
            points: 48,
            variation: 0.4,
            neighbors: 4,
            length_variation: 0.15,
            clump: 0.6,
            clump_size: 0.025,
            twist: 0.0,
            locs: false,
            curl: 0.0,
            curl_radius: 0.008,
            curl_turns: 12.0,
            curl_size: 0.008,
            curl_start: 0.15,
            wave: 0.004,
            wavelength: 0.09,
            noise: 0.006,
            noise_scale: 5.0,
            frizz: 0.003,
            flyaways: 0.04,
            drape: hairdo.drapes(),
            gravity: 9.81,
            wind: [0.0; 3],
            seed: 1,
        };
        match preset {
            Preset::Dreadlocks => {
                look.points = 64;
                look.locs = true;
                look.clump = 1.0;
                look.clump_size = 0.013;
                look.twist = 30.0;
                look.wave = 0.006;
                look.frizz = 0.002;
                look.flyaways = 0.25;
                look.length_variation = 0.25;
            }
            Preset::Braid => {
                look.frizz = 0.0015;
                look.points = 128;
                look.clump = 0.15;
                look.wave = 0.0;
                look.length_variation = 0.05;
                look.flyaways = 0.03;
            }
            Preset::Ponytail => {
                look.points = 64;
                look.clump = 0.3;
                look.wave = 0.003;
            }
            Preset::Bun => {
                look.frizz = 0.001;
                look.points = 128;
                look.clump = 0.2;
                look.wave = 0.0;
                look.length_variation = 0.05;
            }
            Preset::Undercut | Preset::Mohawk => {
                look.strands = 90_000;
                look.points = 24;
                look.clump = 0.5;
                look.clump_size = 0.012;
                look.wave = 0.0;
                look.noise = 0.0015;
                look.frizz = 0.002;
            }
            // Tight coils in small locks, wandering: a cloud, not rays.
            Preset::Afro => {
                look.strands = 80_000;
                look.points = 160;
                look.curl = 1.0;
                look.curl_radius = 0.0035;
                look.curl_turns = 110.0;
                look.curl_size = 0.003;
                look.curl_start = 0.0;
                look.clump = 0.25;
                look.clump_size = 0.01;
                look.noise = 0.012;
                look.noise_scale = 12.0;
                look.frizz = 0.003;
                look.flyaways = 0.3;
                look.wave = 0.0;
            }
            Preset::CurlyTop => {
                look.strands = 100_000;
                look.points = 96;
                look.curl = 1.0;
                look.curl_radius = 0.004;
                look.curl_turns = 70.0;
                look.curl_size = 0.004;
                look.curl_start = 0.05;
                look.clump = 0.3;
                look.clump_size = 0.01;
                look.wave = 0.0;
                look.noise = 0.004;
                look.noise_scale = 15.0;
            }
            Preset::Ringlets => {
                look.points = 160;
                look.curl = 1.0;
                look.curl_radius = 0.007;
                look.curl_turns = 18.0;
                look.curl_size = 0.01;
                look.curl_start = 0.12;
                look.clump = 0.4;
                look.wave = 0.0;
            }
            Preset::Shag => {
                look.clump = 0.7;
                look.wave = 0.01;
                look.wavelength = 0.08;
                look.noise = 0.012;
                look.noise_scale = 6.0;
                look.frizz = 0.004;
                look.flyaways = 0.1;
            }
            // A fringe is cut blunt: its tips must not gather into points.
            Preset::Bangs => {
                look.clump = 0.3;
                look.length_variation = 0.08;
            }
            Preset::BowlCut => {
                look.clump = 0.3;
                look.wave = 0.0;
                look.length_variation = 0.05;
            }
            Preset::Quiff | Preset::SweptFringe => {
                look.strands = 90_000;
                look.points = 32;
                look.clump = 0.6;
                look.clump_size = 0.015;
                look.wave = 0.0;
                look.noise = 0.004;
                look.noise_scale = 10.0;
                look.frizz = 0.002;
            }
            Preset::Balding => {
                look.strands = 100_000;
                look.points = 12;
                look.clump = 0.2;
                look.wave = 0.0;
                look.noise = 0.001;
                look.frizz = 0.0;
            }
            Preset::Buzz => {
                look.strands = 120_000;
                look.points = 8;
                look.clump = 0.0;
                look.noise = 0.0003;
                look.frizz = 0.0;
                look.wave = 0.0;
                look.length_variation = 0.3;
            }
            Preset::Crop | Preset::Spiky => {
                look.strands = 100_000;
                look.points = 16;
                look.clump = if preset == Preset::Crop { 0.3 } else { 0.55 };
                look.clump_size = 0.012;
                look.noise = 0.004;
                look.noise_scale = 20.0;
                look.wave = 0.0;
                look.frizz = 0.002;
            }
            Preset::Pixie => {
                look.strands = 80_000;
                look.points = 24;
                look.clump_size = 0.015;
                look.wave = 0.002;
                look.frizz = 0.002;
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
        look.seed = rng.next_u32() as i32 & 0x7FFF_FFFF;
        look.variation = rng.range(0.2, 0.8);
        let hairdo = &mut look.hairdo;

        // The follicles: most grow from one clockwise crown whorl.
        let f = &mut hairdo.follicles;
        let mut pick = rng.index(100) as i32;
        f.growth = Growth::Whorl;
        for (growth, weight) in [
            (Growth::Whorl, 55),
            (Growth::Cowlick, 12),
            (Growth::DoubleWhorl, 8),
            (Growth::Radial, 5),
            (Growth::Forward, 8),
            (Growth::Backward, 7),
            (Growth::Down, 5),
        ] {
            pick -= weight;
            if pick < 0 {
                f.growth = growth;
                break;
            }
        }
        f.whorl = [rng.range(-0.3, 0.3), rng.range(0.3, 0.8)];
        f.spiral = rng.range(0.6, 1.6);
        f.clockwise = rng.chance(0.75);
        f.angle = (f.angle * rng.range(0.7, 1.5)).clamp(0.1, 1.2);
        f.angle_spread = rng.range(0.05, 0.2);
        f.direction_spread = rng.range(0.05, 0.3);
        f.memory = rng.log_range(0.003, 0.015);
        f.radius = rng.range(25e-6, 50e-6);
        f.radius_spread = rng.range(0.15, 0.35);
        f.patchiness = if rng.chance(0.3) {
            rng.range(0.0, 0.5)
        } else {
            0.0
        };
        let h = &mut f.hairline;
        h.front = rng.range(-0.008, 0.01);
        h.sides = rng.range(-0.008, 0.008);
        h.nape = rng.range(-0.01, 0.01);
        if rng.chance(0.25) {
            h.recession += rng.range(0.0, 0.02);
        }
        h.peak = if rng.chance(0.15) {
            rng.range(0.0, 0.012)
        } else {
            0.0
        };
        h.softness = rng.range(0.002, 0.008);

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

        look.neighbors = 3 + rng.index(4);
        look.length_variation = rng.range(0.0, 0.35);
        look.clump = rng.range(0.1, 0.9);
        look.clump_size = rng.log_range(0.01, 0.05);
        look.twist = if rng.chance(0.3) {
            rng.range(-8.0, 8.0)
        } else {
            0.0
        };
        look.noise = rng.log_range(0.001, 0.012);
        look.noise_scale = rng.log_range(2.0, 15.0);
        look.frizz = rng.log_range(0.001, 0.006);
        look.flyaways = rng.log_range(0.01, 0.2);
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
        look.wave = 0.0;
        look.curl = 0.0;
        match kind {
            0 => look.wave = rng.range(0.0, 0.004),
            1 => {
                look.wave = rng.range(0.005, 0.016);
                look.wavelength = rng.range(0.05, 0.11);
            }
            2 => {
                look.curl = rng.range(0.5, 1.0);
                look.curl_radius = rng.range(0.006, 0.014);
                look.curl_turns = rng.range(6.0, 12.0);
                look.curl_size = rng.log_range(0.008, 0.025);
                look.curl_start = rng.range(0.1, 0.35);
            }
            3 => {
                look.curl = rng.range(0.8, 1.0);
                look.curl_radius = rng.range(0.004, 0.009);
                look.curl_turns = rng.range(15.0, 40.0);
                look.curl_size = rng.log_range(0.004, 0.012);
                look.curl_start = rng.range(0.03, 0.2);
            }
            _ => {
                look.curl = 1.0;
                look.curl_radius = rng.range(0.002, 0.005);
                look.curl_turns = rng.range(60.0, 140.0);
                look.curl_size = rng.log_range(0.002, 0.006);
                look.curl_start = rng.range(0.0, 0.06);
                // Coily hair's follicles are curved: it wanders.
                look.hairdo.follicles.kink = rng.range(1.0, 3.0);
            }
        }
        if preset == Preset::Afro {
            look.noise = rng.range(0.008, 0.02);
            look.noise_scale = rng.range(8.0, 16.0);
            look.curl_size = rng.range(0.002, 0.005);
            look.flyaways = rng.range(0.15, 0.4);
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
            look.clump = rng.range(0.9, 1.0);
            look.clump_size = rng.range(0.01, 0.02);
            look.twist = rng.range(15.0, 40.0);
        }
        // Enough points for about eight a turn, within the dataset's 256.
        if look.curl > 0.0 && !short {
            let length = 0.5;
            let turns = look.curl_turns * length;
            look.points = ((turns * 8.0) as usize).clamp(look.points, 256);
        }
        look.drape = look.hairdo.drapes() && rng.chance(0.85);
        look.gravity = rng.range(7.0, 12.0);
        let wind_angle = rng.range(0.0, std::f32::consts::TAU);
        let wind = if rng.chance(0.5) {
            rng.range(0.0, 4.0)
        } else {
            0.0
        };
        look.wind = [wind_angle.sin() * wind, 0.0, wind_angle.cos() * wind];
        look
    }

    /// `base` with this look's hairline: the head to grow it on, and to cut
    /// its scalp from ([`Head::scalp`]).
    pub fn head(&self, base: &Head) -> Head {
        self.hairdo.head(base)
    }

    /// The guides this look grows from, on its `head` ([`Look::head`]).
    pub fn guides(&self, head: &Head, scalp: &Surface) -> Strands {
        self.hairdo
            .guides(head, scalp, 64, self.variation, self.seed as u64)
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

    /// The stack this look expands to, on its `head` ([`Look::head`]).
    pub fn groom(&self, head: &Head) -> Groom {
        let seed = self.seed;
        let hairdo = &self.hairdo;
        let follicles = &hairdo.follicles;
        let mut ops = Vec::new();
        if self.drape {
            ops.push(Op::Settle(Settle {
                gravity: self.gravity,
                wind: self.wind,
                ..Default::default()
            }));
            ops.push(Op::Shrinkwrap(Shrinkwrap {
                offset: 0.003,
                smoothing: 4,
                preserve_length: true,
                ..Default::default()
            }));
        }
        let [crown, temples, top] = follicles.balding;
        ops.push(Op::Interpolate(Interpolate {
            count: self.strands,
            neighbors: self.neighbors,
            parting: hairdo.parting(),
            // Where neighbouring guides are combed apart, a strand follows
            // one of them: the hair parts, it does not fill the gap.
            divergence: 0.5,
            // Hair that stands out follows its own root's normal all the
            // way; hair that lies on the head only leaves the scalp so.
            follow_length: if hairdo.stands() { 1.0 } else { 0.05 },
            density: if crown + temples + top > 0.0 {
                DensityMask::Balding {
                    crown,
                    temples,
                    top,
                }
            } else {
                DensityMask::Uniform
            },
            patchiness: follicles.patchiness,
            edge: follicles.hairline.softness,
            radius: follicles.radius,
            seed,
            ..Default::default()
        }));
        ops.push(Op::Orient(Orient {
            field: follicles.field(head),
            memory: follicles.memory,
            direction_spread: follicles.direction_spread,
            angle_spread: follicles.angle_spread,
            seed: seed + 9,
        }));
        if self.length_variation > 0.0 {
            ops.push(Op::Trim(Trim {
                length: 1.0,
                random: self.length_variation,
                max_length: 0.0,
                seed: seed + 1,
            }));
        }
        if self.clump > 0.0 {
            // Dreadlocks are clumped all along, into ropes.
            let rope = self.locs;
            ops.push(Op::Clump(Clump {
                guide_distance: self.clump_size,
                factor: if rope {
                    0.93 * self.clump
                } else {
                    self.clump * 0.8
                },
                // A rope gathers within a few centimetres of the scalp.
                shape: if rope { 0.05 } else { 0.3 },
                twist: self.twist,
                twist_random: 0.5,
                tip_spread: self.clump_size * 0.15,
                clump_offset: self.clump_size * 0.2,
                preserve_length: true,
                seed: seed + 2,
                min_length: 0.015,
                ..Default::default()
            }));
        }
        if self.curl > 0.0 {
            ops.push(Op::Curl(Curl {
                guide_distance: self.curl_size,
                reuse_guide_map: false,
                factor: self.curl,
                curl_start: self.curl_start,
                radius: self.curl_radius,
                factor_start: 0.7,
                factor_end: 1.0,
                frequency: self.curl_turns / 3.0,
                random_offset: 0.15,
                seed: seed + 3,
                ..Default::default()
            }));
        }
        if self.wave > 0.0 {
            ops.push(Op::Wave(Wave {
                amplitude: self.wave,
                wavelength: self.wavelength,
                seed: seed + 8,
                ..Default::default()
            }));
        }
        if self.clump > 0.0 && !self.locs {
            ops.push(Op::Clump(Clump {
                guide_distance: (self.clump_size * 0.3).max(0.003),
                factor: self.clump * 0.6,
                shape: 0.75,
                tip_spread: 0.001,
                preserve_length: true,
                seed: seed + 4,
                min_length: 0.015,
                ..Default::default()
            }));
        }
        if self.noise > 0.0 {
            ops.push(Op::Noise(Noise {
                // Offsets, not a walk: Blender's cumulative noise sums a
                // point's offset per point, so its reach grows with the
                // point count. Here `distance` is how far hair strays.
                cumulative: false,
                distance: self.noise,
                scale: 6.0,
                scale_along_curve: self.noise_scale / 5.0,
                offset_per_curve: 0.02,
                shape: 0.6,
                preserve_length: true,
                seed: seed + 5,
                ..Default::default()
            }));
        }
        if self.frizz > 0.0 {
            ops.push(Op::Frizz(Frizz {
                distance: self.frizz,
                shape: 0.5,
                cumulative: true,
                seed: seed + 6,
                share: self.flyaways,
                // Every strand wanders a little on its own, so that no two
                // lie quite parallel: the highlight breaks into strands.
                calm: 0.12,
                frequency: 15.0,
                ..Default::default()
            }));
        }
        ops.push(Op::Shrinkwrap(Shrinkwrap {
            offset: 0.0015,
            smoothing: 3,
            preserve_length: false,
            ..Default::default()
        }));
        ops.push(Op::Profile(Profile {
            radius: follicles.radius,
            random: follicles.radius_spread,
            seed: seed + 7,
            relative_to: Some(follicles.radius),
        }));
        Groom {
            points: self.points,
            ops,
        }
    }
}
