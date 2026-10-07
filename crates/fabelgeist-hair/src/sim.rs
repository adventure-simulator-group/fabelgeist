//! Real-time physics: hair that moves with the head and the wind.
//!
//! A hundred thousand strands cannot each be simulated in a frame. As in
//! AMD's TressFX and Unreal's grooms, a few thousand of them are -- here the
//! first strands of the groom's shuffled order, which are an even sample of
//! the head -- and every drawn strand follows the motion of the simulated
//! strands nearest its root ([`SimBinding`]), blended on the GPU
//! ([`crate::render::HairRenderer::set_simulated`]).
//!
//! Each simulated strand is an elastic rod with its fibres' real mass and
//! stiffness ([`crate::rod`]): it bends and barely stretches as hair does,
//! swings under gravity, is dragged by the air and damped by its own
//! viscoelasticity, and is clamped in its follicle, which the head carries.
//! A substep is a step of implicit Euler, solved exactly a strand at a time,
//! so stiff hair costs no more than soft. Nothing pulls a strand back to the
//! groom: the groom is instead the rods' equilibrium under gravity, solved
//! for when the simulation starts ([`SimSettings::sag_free`]), so it holds
//! as groomed until something moves it, and then moves as physics has it.
//! The head and body push the strands out, with Coulomb friction, in the
//! head's frame so that they move with it; the head moves rigidly
//! ([`Rigid`]).
//!
//! On the GPU ([`crate::gpu::GpuSim`]) the strands also meet each other:
//!
//! * **Volume.** The hair's density is splatted to a grid in the head's
//!   frame every substep; where it is denser than the groom had it at rest
//!   (or than the head's typical density, where the groom had none) the
//!   strands are pushed down the excess's gradient, so hair pressed against
//!   hair keeps its volume instead of collapsing through it.
//! * **Friction.** The same grid carries the hair's momentum; each point's
//!   velocity is drawn towards the grid's there, so hair moving through
//!   hair drags it along (Petrovic et al. 2005, McAdams et al. 2009).
//! * **Collision.** Every substep, points sampled along the simulated
//!   segments are sorted into a grid, and wherever two strands' segments
//!   are nearer than their wisps' radius -- and nearer than the groom had
//!   them -- they are moved apart, back to the side they were on a substep
//!   before, and their closing velocity is stopped, with Coulomb friction
//!   along the contact. Hair lying on hair at rest stays as it is. Up to a
//!   thousand strands collide; a finer level's others meet through the
//!   volume and friction.
//!
//! How many strands are simulated, and with how many points, is a level of
//! detail of its own ([`SimLevel`]), chosen like the drawing's by how large
//! the head is on screen ([`crate::LodPolicy::sim_level`]); switching levels
//! carries the motion across.

use fabelgeist_math::Vec3;
use rayon::prelude::*;

use crate::collider::Collider;
use crate::grid::PointGrid;
use crate::head::Head;
use crate::lod::HairLods;
use crate::rod::{Rod, Substep, frames_along, rest_shape};
use crate::strands::safe_normalize;

/// A rotation and a translation: where the head is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rigid {
    /// Columns of the rotation.
    pub axes: [Vec3; 3],
    pub translation: Vec3,
}

impl Default for Rigid {
    fn default() -> Self {
        Self {
            axes: [Vec3::right(), Vec3::up(), Vec3::new(0.0, 0.0, 1.0)],
            translation: Vec3::default(),
        }
    }
}

impl Rigid {
    /// Turned by `yaw` about the vertical, then `pitch` about the side axis
    /// and `roll` about the front, about `pivot` (the neck), and moved by
    /// `offset`.
    pub fn from_angles(yaw: f32, pitch: f32, roll: f32, pivot: Vec3, offset: Vec3) -> Self {
        let (sy, cy) = yaw.sin_cos();
        let (sp, cp) = pitch.sin_cos();
        let (sr, cr) = roll.sin_cos();
        // R = Ry * Rx * Rz, as columns.
        let ry = [Vec3::new(cy, 0.0, -sy), Vec3::up(), Vec3::new(sy, 0.0, cy)];
        let rx = [
            Vec3::right(),
            Vec3::new(0.0, cp, sp),
            Vec3::new(0.0, -sp, cp),
        ];
        let rz = [
            Vec3::new(cr, sr, 0.0),
            Vec3::new(-sr, cr, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        ];
        let apply = |m: &[Vec3; 3], v: Vec3| m[0] * v.x + m[1] * v.y + m[2] * v.z;
        let axes = [0, 1, 2].map(|k| apply(&ry, apply(&rx, rz[k])));
        let rotated_pivot = apply(&axes, pivot);
        Self {
            axes,
            translation: pivot - rotated_pivot + offset,
        }
    }

    pub fn rotate(&self, v: Vec3) -> Vec3 {
        self.axes[0] * v.x + self.axes[1] * v.y + self.axes[2] * v.z
    }

    pub fn apply(&self, p: Vec3) -> Vec3 {
        self.rotate(p) + self.translation
    }

    /// From the world into the head's frame.
    pub fn unapply(&self, p: Vec3) -> Vec3 {
        let q = p - self.translation;
        Vec3::new(
            self.axes[0].dot(q),
            self.axes[1].dot(q),
            self.axes[2].dot(q),
        )
    }

    /// Column-major, as WGSL's `mat4x4<f32>` reads it.
    pub fn matrix(&self) -> [[f32; 4]; 4] {
        let [a, b, c] = self.axes;
        let t = self.translation;
        [
            [a.x, a.y, a.z, 0.0],
            [b.x, b.y, b.z, 0.0],
            [c.x, c.y, c.z, 0.0],
            [t.x, t.y, t.z, 1.0],
        ]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SimSettings {
    /// Strands simulated.
    pub guides: usize,
    /// Points a simulated strand, resampled from the groom.
    pub points: usize,
    /// The hair fibre's Young's modulus, pascals: about 3.5 GPa dry.
    pub youngs_modulus: f32,
    /// The fibre's radius, metres: 20 to 50 micrometres.
    pub fibre_radius: f32,
    /// The fibre's density, kilograms a cubic metre.
    pub density: f32,
    /// How many fibres a simulated strand's clump holds together as one
    /// rod. Friction, oils and cohesion bind a lock's fibres, so it bends
    /// stiffer than the fibres would apart -- a bound bundle's stiffness
    /// grows as the square of its fibres, theirs apart as the count -- each
    /// fibre this many times a lone fibre's; and the air drags on the clump
    /// as one cylinder.
    pub clump: f32,
    /// The bending's viscous loss, seconds: its force per unit of the
    /// elastic force's rate.
    pub internal_damping: f32,
    /// The air's drag, as a share of a smooth cylinder's the clump's width:
    /// 0 in a vacuum.
    pub air_drag: f32,
    /// Metres a second squared.
    pub gravity: f32,
    /// The air's velocity, metres a second.
    pub wind: [f32; 3],
    /// How much the wind gusts, relative to its mean.
    pub turbulence: f32,
    /// Substeps a frame; on the GPU the strands meet each other at each.
    pub substeps: usize,
    /// The rods' implicit steps a substep.
    pub solver_substeps: usize,
    /// Metres kept off the head.
    pub collision_offset: f32,
    /// Coulomb friction against the head and body.
    pub head_friction: f32,
    /// The groom is the rods' equilibrium under gravity, so it holds as
    /// groomed until moved; otherwise the rods rest as groomed, unloaded,
    /// and the hair sags under its weight when the simulation starts.
    pub sag_free: bool,
    /// How strongly hair denser than at rest is pushed apart, 0 to 1 a
    /// substep (GPU only).
    pub volume: f32,
    /// How much denser than at rest hair may get before it is, a share.
    pub volume_tolerance: f32,
    /// How far each point's velocity is drawn to the hair's around it, 0
    /// to 1 a substep (GPU only).
    pub friction: f32,
    /// Strands stop where they close on each other (GPU only).
    pub strand_collision: bool,
    /// The distance strands keep, metres: a simulated strand stands for a
    /// wisp this wide. 0 sizes it to the strands' spacing.
    pub collision_radius: f32,
    /// Coulomb friction where strands meet.
    pub collision_friction: f32,
    /// At most how many strands collide -- the first, an even sample of
    /// the head -- the rest meeting through the volume and friction alone:
    /// collisions cost more than the strands' count grows.
    pub collision_strands: usize,
    /// Collide every this many substeps: hair swinging moves a good share
    /// of a wisp's width in a frame, and would pass through between checks
    /// a frame apart.
    pub collision_interval: usize,
    /// The density grid's cell, metres.
    pub grid_cell: f32,
}

impl SimSettings {
    /// These settings at `level`'s strands and points.
    pub fn at(self, level: SimLevel) -> Self {
        Self {
            guides: level.strands,
            points: level.points,
            ..self
        }
    }

    /// A fibre's mass a metre, kilograms.
    pub fn linear_density(&self) -> f32 {
        self.density * std::f32::consts::PI * self.fibre_radius * self.fibre_radius
    }

    /// A simulated strand's bending stiffness per fibre, newton square
    /// metres: a lone fibre's, `E π r⁴ / 4`, times the clump.
    pub fn bending_stiffness(&self) -> f32 {
        self.clump.max(1.0) * self.youngs_modulus * std::f32::consts::PI * self.fibre_radius.powi(4)
            / 4.0
    }

    /// Without the strands meeting each other: the solver alone, as the
    /// host's [`HairSim`] runs it.
    pub fn alone(self) -> Self {
        Self {
            volume: 0.0,
            friction: 0.0,
            strand_collision: false,
            ..self
        }
    }
}

/// How finely hair is simulated: how many strands, of how many points.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SimLevel {
    pub strands: usize,
    pub points: usize,
}

impl SimLevel {
    pub const HIGH: Self = Self {
        strands: 8192,
        points: 32,
    };
    pub const MEDIUM: Self = Self {
        strands: 2048,
        points: 24,
    };
    pub const LOW: Self = Self {
        strands: 512,
        points: 12,
    };
    /// Finest first.
    pub const ALL: [Self; 3] = [Self::HIGH, Self::MEDIUM, Self::LOW];

    pub fn name(&self) -> &'static str {
        match *self {
            Self::HIGH => "high",
            Self::MEDIUM => "medium",
            Self::LOW => "low",
            _ => "custom",
        }
    }
}

impl Default for SimSettings {
    fn default() -> Self {
        Self {
            guides: 2048,
            points: 24,
            youngs_modulus: 3.5e9,
            fibre_radius: 40e-6,
            density: 1300.0,
            clump: 50.0,
            internal_damping: 0.01,
            air_drag: 1.0,
            gravity: 9.81,
            wind: [0.0; 3],
            turbulence: 0.5,
            substeps: 4,
            solver_substeps: 4,
            collision_offset: 0.002,
            head_friction: 0.3,
            sag_free: true,
            volume: 0.35,
            volume_tolerance: 0.15,
            friction: 0.08,
            strand_collision: true,
            collision_radius: 0.0,
            collision_friction: 0.3,
            collision_strands: 1024,
            collision_interval: 1,
            grid_cell: 0.01,
        }
    }
}

/// Which simulated strands each drawn strand follows, and how much.
pub struct SimBinding {
    /// Per drawn strand (in the groom's shuffled order): four simulated
    /// strands and their weights, summing to one.
    pub guides: Vec<[u32; 4]>,
    pub weights: Vec<[f32; 4]>,
    /// Simulated strands, and points each.
    pub count: usize,
    pub points: usize,
}

pub struct HairSim {
    pub settings: SimSettings,
    points: usize,
    /// Groomed positions, in the head's frame.
    rest: Vec<Vec3>,
    /// Each strand's rod's rest ([`rest_shape`]), two slots a point, and
    /// the material and choice it was solved for.
    shape: Vec<[f32; 4]>,
    shaped: Option<(Rod, bool)>,
    /// World space; `previous` is a substep's velocity behind, the substep
    /// `h_last` seconds. Each segment's frame and its angular velocity, two
    /// slots a point.
    position: Vec<Vec3>,
    previous: Vec<Vec3>,
    frames: Vec<[f32; 4]>,
    h_last: f32,
    head: Rigid,
    time: f32,
}

impl HairSim {
    /// The point count of the level a simulation is made from: the one of
    /// `levels` nearest `wanted`.
    pub fn level_points(levels: impl IntoIterator<Item = usize>, wanted: usize) -> Option<usize> {
        levels
            .into_iter()
            .min_by_key(|&p| (p as i64 - wanted as i64).abs())
    }

    /// Simulate the first `settings.guides` strands of `lods`, at the
    /// level whose point count is nearest `settings.points`; the binding
    /// ties every drawn strand to the four simulated ones nearest its root.
    pub fn new(lods: &HairLods, settings: SimSettings) -> (Self, SimBinding) {
        let points = Self::level_points(lods.levels.iter().map(|l| l.points), settings.points)
            .expect("a groom has a level");
        let level = lods
            .levels
            .iter()
            .find(|l| l.points == points)
            .expect("chosen above");
        Self::from_level(points, &level.positions, settings)
    }

    /// Simulate the first `settings.guides` strands of a level of `points`
    /// points a strand (`positions` as [`crate::lod::StrandLevel`] holds
    /// them, in the groom's shuffled order) -- a level read back from the
    /// GPU as well as one built on the host. A level's strands start at
    /// their roots whatever its point count, so it binds every strand.
    pub fn from_level(
        points: usize,
        positions: &[[f32; 4]],
        settings: SimSettings,
    ) -> (Self, SimBinding) {
        let total = positions.len() / points.max(1);
        let count = settings.guides.min(total).max(1.min(total));
        let rest: Vec<Vec3> = positions[..count * points]
            .iter()
            .map(|p| Vec3::new(p[0], p[1], p[2]))
            .collect();

        // Bind every drawn strand to the simulated roots nearest its own.
        let root = |s: usize| {
            let p = positions[s * points];
            Vec3::new(p[0], p[1], p[2])
        };
        let roots: Vec<(Vec3, i32)> = (0..count).map(|g| (rest[g * points], 0)).collect();
        let spacing = 0.2 / (count as f32).sqrt().max(1.0);
        let grid = PointGrid::build(&roots, spacing.max(0.003));
        let (guides, weights): (Vec<[u32; 4]>, Vec<[f32; 4]>) = (0..total)
            .into_par_iter()
            .map(|s| {
                if s < count {
                    return ([s as u32, 0, 0, 0], [1.0, 0.0, 0.0, 0.0]);
                }
                let nearest = grid.nearest_k(root(s), 4, f32::INFINITY, |_| true);
                let mut ids = [0u32; 4];
                let mut w = [0.0f32; 4];
                let mut total = 0.0;
                for (k, &(g, d)) in nearest.iter().enumerate() {
                    ids[k] = g;
                    w[k] = 1.0 / (d * d + 1e-8);
                    total += w[k];
                }
                for v in w.iter_mut() {
                    *v /= total.max(1e-12);
                }
                (ids, w)
            })
            .unzip();

        let sim = Self {
            settings,
            points,
            position: rest.clone(),
            previous: rest.clone(),
            shape: vec![[0.0; 4]; 2 * rest.len()],
            shaped: None,
            frames: vec![[0.0; 4]; 2 * rest.len()],
            rest,
            h_last: 1.0 / 240.0,
            head: Rigid::default(),
            time: 0.0,
        };
        let binding = SimBinding {
            guides,
            weights,
            count,
            points,
        };
        (sim, binding)
    }

    /// Strands simulated.
    pub fn strands(&self) -> usize {
        self.rest.len() / self.points
    }

    /// Points per simulated strand.
    pub fn points(&self) -> usize {
        self.points
    }

    pub fn head(&self) -> Rigid {
        self.head
    }

    /// Back to the groom, the head where it is.
    pub fn reset(&mut self, head: Rigid) {
        self.shape_rods();
        self.head = head;
        for (p, r) in self.position.iter_mut().zip(&self.rest) {
            *p = head.apply(*r);
        }
        self.previous.clone_from(&self.position);
        self.align_frames();
    }

    /// The rods' rest solved for the settings, if they changed.
    fn shape_rods(&mut self) {
        let s = self.settings;
        let rod = Rod::new(&s);
        if self.shaped != Some((rod, s.sag_free)) {
            self.rest
                .par_chunks(self.points)
                .zip(self.shape.par_chunks_mut(2 * self.points))
                .for_each(|(rest, shape)| rest_shape(&rod, rest, s.sag_free, shape));
            self.shaped = Some((rod, s.sag_free));
        }
    }

    /// Each segment's frame onto where its strand now is, not spinning.
    pub(crate) fn align_frames(&mut self) {
        let points = self.points;
        let head = self.head;
        self.rest
            .par_chunks(points)
            .zip(self.shape.par_chunks(2 * points))
            .zip(self.position.par_chunks(points))
            .zip(self.frames.par_chunks_mut(2 * points))
            .for_each(|(((rest, shape), x), frames)| frames_along(&head, rest, shape, x, frames));
    }

    /// Step `dt` seconds with the head moved to `head`, kept out of
    /// `collider` (in the head's frame) if there is one.
    pub fn step(&mut self, dt: f32, head: Rigid, collider: Option<&Head>) {
        let dt = dt.min(1.0 / 20.0);
        if dt.is_nan() || dt <= 0.0 {
            return;
        }
        if self.shaped.is_none() {
            let head = self.head;
            self.reset(head);
        }
        self.shape_rods();
        let s = self.settings;
        let rod = Rod::new(&s);
        let outer = s.substeps.max(1);
        let inner = s.solver_substeps.max(1);
        let h = dt / (outer * inner) as f32;
        let from = self.head;
        let collider = collider.map(|c| c as &(dyn Collider + Sync));
        for k in 0..outer {
            // The head moves through the frame's substeps.
            let a = blend(&from, &head, k as f32 / outer as f32);
            let b = blend(&from, &head, (k + 1) as f32 / outer as f32);
            for m in 0..inner {
                let step = Substep {
                    rod: &rod,
                    h,
                    h_last: self.h_last,
                    from: blend(&a, &b, m as f32 / inner as f32),
                    to: blend(&a, &b, (m + 1) as f32 / inner as f32),
                    collider,
                    offset: s.collision_offset,
                    friction: s.head_friction,
                };
                let time = self.time;
                let points = self.points;
                self.position
                    .par_chunks_mut(points)
                    .zip(self.previous.par_chunks_mut(points))
                    .zip(self.frames.par_chunks_mut(2 * points))
                    .zip(self.rest.par_chunks(points))
                    .zip(self.shape.par_chunks(2 * points))
                    .for_each(|((((x, previous), frames), rest), shape)| {
                        let air = air(&s, rest[0], time);
                        step.strand(rest, shape, air, x, previous, frames);
                    });
                self.h_last = h;
                self.time += h;
            }
        }
        self.head = head;
    }

    /// The simulated strands' points, world space, `w` unused.
    pub fn positions(&self) -> Vec<[f32; 4]> {
        self.position.iter().map(|p| [p.x, p.y, p.z, 1.0]).collect()
    }

    /// The groomed points, in the head's frame.
    pub fn rest(&self) -> Vec<[f32; 4]> {
        self.rest.iter().map(|p| [p.x, p.y, p.z, 1.0]).collect()
    }
}

/// The air's velocity at a strand rooted at `root` (the head's frame):
/// the wind, gusting by noise over the head and time.
fn air(s: &SimSettings, root: Vec3, time: f32) -> Vec3 {
    let gust = if s.turbulence > 0.0 {
        let q = root * 3.0 + Vec3::new(time * 0.9, time * 0.4, 0.0);
        1.0 + s.turbulence * crate::noise::perlin(q) * 2.0
    } else {
        1.0
    };
    Vec3::from_array(s.wind) * gust
}

/// Part of the way from one pose to another (rotation by normalised lerp
/// of the axes, which is close enough within a frame).
pub(crate) fn blend(a: &Rigid, b: &Rigid, t: f32) -> Rigid {
    let x = safe_normalize(a.axes[0].lerp(b.axes[0], t), b.axes[0]);
    let y0 = a.axes[1].lerp(b.axes[1], t);
    let y = safe_normalize(y0 - x * y0.dot(x), b.axes[1]);
    Rigid {
        axes: [x, y, x.cross(y)],
        translation: a.translation.lerp(b.translation, t),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lod::LodSettings;
    use crate::{Look, Preset};

    fn groom() -> (Head, HairLods) {
        let mut look = Look::new(Preset::Long);
        look.strands = 3000;
        look.drape = false;
        let head = look.head(&Head::mannequin());
        let scalp = head.scalp(0.008);
        let hair = look.grow(&head, &scalp);
        let settings = LodSettings {
            cards: Vec::new(),
            ..Default::default()
        };
        let lods = HairLods::build(&hair, head.cranium.center, Some(&head), &settings);
        (head, lods)
    }

    #[test]
    fn a_still_head_keeps_the_groom_as_it_is() {
        let (head, lods) = groom();
        let (mut sim, binding) = HairSim::new(
            &lods,
            SimSettings {
                guides: 300,
                ..Default::default()
            },
        );
        assert_eq!(binding.guides.len(), lods.strand_count());
        for _ in 0..120 {
            sim.step(1.0 / 60.0, Rigid::default(), Some(&head));
        }
        let rest = sim.rest();
        let now = sim.positions();
        let p = sim.points;
        let mut worst: f32 = 0.0;
        for g in 0..binding.count {
            for i in 1..p {
                let a = Vec3::new(now[g * p + i][0], now[g * p + i][1], now[g * p + i][2]);
                let b = Vec3::new(
                    now[g * p + i - 1][0],
                    now[g * p + i - 1][1],
                    now[g * p + i - 1][2],
                );
                let ra = Vec3::new(rest[g * p + i][0], rest[g * p + i][1], rest[g * p + i][2]);
                let rb = Vec3::new(
                    rest[g * p + i - 1][0],
                    rest[g * p + i - 1][1],
                    rest[g * p + i - 1][2],
                );
                assert!(((a - b).length() - (ra - rb).length()).abs() < 1e-3);
            }
            let tip = Vec3::new(
                now[g * p + p - 1][0],
                now[g * p + p - 1][1],
                now[g * p + p - 1][2],
            );
            let rest_tip = Vec3::new(
                rest[g * p + p - 1][0],
                rest[g * p + p - 1][1],
                rest[g * p + p - 1][2],
            );
            worst = worst.max((tip - rest_tip).length());
        }
        // The groom is the rods' rest under gravity, and hair lying on the
        // scalp stays as it lies: nothing moves it.
        assert!(worst < 1e-4, "{worst}");
    }

    #[test]
    fn turning_the_head_carries_the_roots_and_the_tips_lag() {
        let (head, lods) = groom();
        let (mut sim, binding) = HairSim::new(
            &lods,
            SimSettings {
                guides: 200,
                ..Default::default()
            },
        );
        let turned = Rigid::from_angles(0.6, 0.0, 0.0, Vec3::new(0.0, -0.1, 0.0), Vec3::default());
        sim.step(1.0 / 60.0, turned, Some(&head));
        let p = sim.points;
        let now = sim.positions();
        let rest = sim.rest();
        for g in 0..binding.count {
            let root = Vec3::new(now[g * p][0], now[g * p][1], now[g * p][2]);
            let want = turned.apply(Vec3::new(rest[g * p][0], rest[g * p][1], rest[g * p][2]));
            assert!((root - want).length() < 1e-5);
        }
        assert!(
            (turned.unapply(turned.apply(Vec3::new(0.1, 0.2, 0.3))) - Vec3::new(0.1, 0.2, 0.3))
                .length()
                < 1e-5
        );
    }

    /// One strand of `points`, simulated alone.
    fn strand(points: &[Vec3], settings: SimSettings) -> HairSim {
        let positions: Vec<[f32; 4]> = points.iter().map(|p| [p.x, p.y, p.z, 1.0]).collect();
        HairSim::from_level(
            points.len(),
            &positions,
            SimSettings {
                guides: 1,
                ..settings
            },
        )
        .0
    }

    /// `n` points evenly along `length` from `from` in direction `d`.
    fn line(from: Vec3, d: Vec3, length: f32, n: usize) -> Vec<Vec3> {
        (0..n)
            .map(|i| from + d * (length * i as f32 / (n - 1) as f32))
            .collect()
    }

    fn tip(sim: &HairSim) -> Vec3 {
        *sim.position.last().expect("a strand")
    }

    /// Still air and no loss: the rod alone.
    fn lossless() -> SimSettings {
        SimSettings {
            internal_damping: 0.0,
            air_drag: 0.0,
            turbulence: 0.0,
            sag_free: false,
            ..Default::default()
        }
    }

    /// The mean period of `samples`, `dt` apart, by its rising crossings
    /// of its mean.
    fn period(samples: &[f32], dt: f32) -> f32 {
        let mean = samples.iter().sum::<f32>() / samples.len() as f32;
        let crossings: Vec<f32> = samples
            .windows(2)
            .enumerate()
            .filter(|(_, w)| w[0] < mean && w[1] >= mean)
            .map(|(k, w)| (k as f32 + (mean - w[0]) / (w[1] - w[0])) * dt)
            .collect();
        assert!(crossings.len() >= 3, "{} crossings", crossings.len());
        (crossings[crossings.len() - 1] - crossings[0]) / (crossings.len() - 1) as f32
    }

    #[test]
    fn a_cantilever_sags_as_beam_theory_says() {
        // Ten centimetres held out level, stiff enough to bend a little: its
        // tip settles w L⁴ / 8 EI below the root.
        let length = 0.1;
        let settings = SimSettings {
            clump: 60.0,
            internal_damping: 0.05,
            turbulence: 0.0,
            sag_free: false,
            ..Default::default()
        };
        let mut sim = strand(&line(Vec3::default(), Vec3::right(), length, 32), settings);
        for _ in 0..240 {
            sim.step(1.0 / 60.0, Rigid::default(), None);
        }
        let expected = settings.linear_density() * settings.gravity * length.powi(4)
            / (8.0 * settings.bending_stiffness());
        let sag = -tip(&sim).y;
        assert!(
            (sag / expected - 1.0).abs() < 0.03,
            "sags {sag} m, beam theory {expected} m"
        );
    }

    #[test]
    fn a_cantilever_rings_at_its_natural_frequency() {
        // Let go straight, it rings about where it sags, at
        // 1.8751² / 2π sqrt(EI / λ L⁴) hertz.
        let length = 0.1;
        let settings = SimSettings {
            clump: 60.0,
            substeps: 1,
            solver_substeps: 4,
            ..lossless()
        };
        let mut sim = strand(&line(Vec3::default(), Vec3::right(), length, 32), settings);
        let dt = 1.0 / 1000.0;
        let heights: Vec<f32> = (0..1000)
            .map(|_| {
                sim.step(dt, Rigid::default(), None);
                tip(&sim).y
            })
            .collect();
        let expected = 1.8751f32.powi(2) / std::f32::consts::TAU
            * (settings.bending_stiffness() / (settings.linear_density() * length.powi(4))).sqrt();
        let rings = 1.0 / period(&heights, dt);
        assert!(
            (rings / expected - 1.0).abs() < 0.03,
            "rings at {rings} Hz, beam theory {expected} Hz"
        );
    }

    #[test]
    fn a_hanging_strand_swings_as_a_chain_does() {
        // Barely stiff, a hanging strand is a heavy chain, held up by its
        // tension: shaped J0(2.405 sqrt(z / L)), z up from the tip, it
        // swings at 2.405 / 2 sqrt(g / L) radians a second. A strand that
        // only followed its root, without the tension, would not.
        let length = 0.3;
        let settings = SimSettings {
            youngs_modulus: 3.5e7,
            clump: 1.0,
            substeps: 1,
            solver_substeps: 4,
            ..lossless()
        };
        let n = 32;
        let hanging = line(Vec3::default(), Vec3::down(), length, n);
        let mut sim = strand(&hanging, settings);
        sim.reset(Rigid::default());
        let j0 = |x: f32| {
            let (mut sum, mut term) = (0.0f32, 1.0f32);
            for k in 1..20 {
                sum += term;
                term *= -(x * x / 4.0) / (k * k) as f32;
            }
            sum
        };
        for (i, p) in hanging.iter().enumerate() {
            let z = length * (1.0 - i as f32 / (n - 1) as f32);
            let swung = *p + Vec3::right() * (0.01 * j0(2.405 * (z / length).sqrt()));
            let swung = if i == 0 { *p } else { swung };
            sim.position[i] = swung;
            sim.previous[i] = swung;
        }
        sim.align_frames();
        let dt = 1.0 / 500.0;
        let sideways: Vec<f32> = (0..1500)
            .map(|_| {
                sim.step(dt, Rigid::default(), None);
                tip(&sim).x
            })
            .collect();
        let expected = 2.405 / 2.0 * (settings.gravity / length).sqrt() / std::f32::consts::TAU;
        let swings = 1.0 / period(&sideways, dt);
        assert!(
            (swings / expected - 1.0).abs() < 0.03,
            "swings at {swings} Hz, a chain at {expected} Hz"
        );
    }

    #[test]
    fn a_groom_rests_as_groomed_under_its_weight() {
        // Twenty centimetres held out level, and a curl hanging: under their
        // weight both would sag far, but their rods rest so that the groom
        // is the equilibrium.
        let level = line(Vec3::default(), Vec3::right(), 0.2, 32);
        let curl: Vec<Vec3> = (0..32)
            .map(|i| {
                let a = i as f32 * 0.45;
                Vec3::new(0.01 * a.cos() - 0.01, -0.006 * i as f32, 0.01 * a.sin())
            })
            .collect();
        // Unloaded, the level strand droops; the curl, stiffer for its
        // coils, stretches a little.
        for (groom, sags) in [(level, 0.02), (curl, 1e-3)] {
            let settings = SimSettings {
                turbulence: 0.0,
                ..Default::default()
            };
            let drift = |settings: SimSettings| {
                let mut sim = strand(&groom, settings);
                for _ in 0..180 {
                    sim.step(1.0 / 60.0, Rigid::default(), None);
                }
                (tip(&sim) - *groom.last().expect("points")).length()
            };
            let held = drift(settings);
            assert!(held < 1e-4, "the groom moves {held} m");
            let unloaded = drift(SimSettings {
                sag_free: false,
                ..settings
            });
            assert!(unloaded > sags, "unloaded, it sags only {unloaded} m");
        }
    }
}
