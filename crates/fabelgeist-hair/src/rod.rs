//! The elastic rods hair is simulated as ([`crate::sim`]): the solver
//! [`crate::sim::HairSim`] runs on the host, which the GPU's kernels
//! ([`crate::gpu::GpuSim`]) mirror step for step.
//!
//! A simulated strand is a Cosserat rod (Pai 2002, Spillmann and Teschner
//! 2007): point masses -- its fibres' mass, lumped half a segment each way
//! -- and, for each segment, a material frame with the cross-section's
//! rotational inertia. Two energies hold it together, in real units:
//!
//! * **Stretch and shear.** Each segment is to run along its frame's axis,
//!   at its length, with the fibre's Young's modulus: hair barely stretches.
//! * **Bend and twist.** Each frame is to keep its turn from the one before
//!   (the Darboux vector, as a quaternion's vector part), with the fibre's
//!   bending stiffness times how many fibres its clump bends as one
//!   ([`SimSettings::clump`]), and twisting stiffness as an isotropic
//!   rod's. The root's frame turns from the follicle's, which the head
//!   carries. Frames are degrees of freedom, so the energy is local, and
//!   curls and twist come out as they are.
//!
//! Gravity pulls, the air drags on the clump as on a cylinder (by White's
//! correlation for its drag coefficient), and the bending is viscoelastic
//! (stiffness-proportional damping). Each substep is one step of implicit
//! Euler, Newton-linearised (Baraff and Witkin 1998) in the points'
//! velocities and the frames' angular velocities: grouping each frame with
//! the point at its segment's end, every group couples only to the groups
//! beside it, so the strand's system is block tridiagonal in 6x6 blocks and
//! is solved exactly, by block LDLᵀ, in one pass down the strand and one
//! back. Stiffness costs nothing, and no iteration count sets how stiff the
//! hair looks.
//!
//! **At rest under gravity.** A groom is drawn as it hangs, so the shape the
//! rod rests at is solved for so that the groom is its equilibrium
//! ([`rest_shape`]): each segment carries the weight below it, which sets
//! its rest length and shear, and each joint the moment of that weight
//! about it, which sets its rest bend and twist (Derouet-Jourdan et al.
//! 2013, Hsu et al. 2023). The groom then neither sags when the simulation
//! starts nor springs back to it when moved: it moves only as forces move
//! it.

use std::f32::consts::PI;
use std::ops::{Add, Mul, Neg, Sub};

use fabelgeist_math::Vec3;

use crate::collider::{Collider, push_out};
use crate::sim::{Rigid, SimSettings};
use crate::strands::{orthogonal, safe_normalize, transport};

/// Air at room temperature: density, kilograms a cubic metre, and
/// kinematic viscosity, square metres a second.
pub const AIR_DENSITY: f32 = 1.2;
pub const AIR_VISCOSITY: f32 = 1.5e-5;
/// How much of a clump's cross-section its fibres fill.
const PACKING: f32 = 0.6;
/// Keratin's Poisson ratio: an isotropic rod twists `1 / (1 + ν)` as
/// stiffly as it bends.
const POISSON: f32 = 0.4;

/// The rod's material, per unit of its mass per length: what the solver
/// needs of [`SimSettings`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rod {
    /// Young's modulus over density, square metres a second squared: a
    /// segment's stretch stiffness times its length.
    pub stretch: f32,
    /// Bending and twisting stiffness over mass per length, metres to the
    /// fourth a second squared.
    pub bend: f32,
    pub twist: f32,
    /// The clump's radius squared: its frames' inertia per unit of mass is
    /// a half of it about the axis, a quarter about a diameter.
    pub gyration: f32,
    /// Seconds: viscous bending force over elastic, per unit of rate.
    pub damping: f32,
    /// The air's drag a metre of strand, per unit of speed squared, over
    /// mass per length; and the viscous term, which slow strands feel.
    pub drag: f32,
    pub drag_viscous: f32,
    pub gravity: Vec3,
}

impl Rod {
    pub fn new(s: &SimSettings) -> Self {
        let r = s.fibre_radius.max(1e-7);
        let density = s.density.max(1.0);
        let per_length = density * PI * r * r;
        let clump = s.clump.max(1.0);
        let diameter = 2.0 * r * (clump / PACKING).sqrt();
        let bend = clump * s.youngs_modulus.max(1.0) * r * r / (4.0 * density);
        Self {
            stretch: s.youngs_modulus.max(1.0) / density,
            bend,
            twist: bend / (1.0 + POISSON),
            gyration: 0.25 * diameter * diameter,
            damping: s.internal_damping.max(0.0),
            drag: s.air_drag.max(0.0) * 0.5 * AIR_DENSITY * diameter / (clump * per_length),
            drag_viscous: 10.0 * (AIR_VISCOSITY / diameter).powf(2.0 / 3.0),
            gravity: Vec3::new(0.0, -s.gravity, 0.0),
        }
    }

    /// The stiffness of the joint before segment `s`, in its first frame's
    /// axes: a segment's frame is the strand's at its middle, so the joint
    /// spans from the middle of the segment before (the follicle, for the
    /// root's) to the middle of this one. Four times, as the Darboux vector
    /// is half the turn.
    fn joint(&self, lengths: &[f32], s: usize) -> Vec3 {
        let span = if s == 0 {
            0.5 * lengths[0]
        } else {
            0.5 * (lengths[s - 1] + lengths[s])
        };
        Vec3::new(self.bend, self.bend, self.twist) * (4.0 / span)
    }
}

/// Point `i`'s mass over mass per length: half of each segment beside it.
fn point_mass(lengths: &[f32], i: usize) -> f32 {
    0.5 * (lengths[i - 1] + lengths.get(i).copied().unwrap_or(0.0))
}

/// A 3x3 matrix, by columns.
#[derive(Clone, Copy, Debug)]
pub struct M3(pub [Vec3; 3]);

impl M3 {
    pub const ZERO: Self = Self(
        [Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }; 3],
    );

    pub fn diagonal(s: f32) -> Self {
        Self::scaling(Vec3::splat(s))
    }

    pub fn scaling(s: Vec3) -> Self {
        Self([
            Vec3::new(s.x, 0.0, 0.0),
            Vec3::new(0.0, s.y, 0.0),
            Vec3::new(0.0, 0.0, s.z),
        ])
    }

    /// `a bᵀ`.
    pub fn outer(a: Vec3, b: Vec3) -> Self {
        Self([a * b.x, a * b.y, a * b.z])
    }

    /// `[a]×`: `a ×` as a matrix.
    pub fn skew(a: Vec3) -> Self {
        Self([
            Vec3::new(0.0, a.z, -a.y),
            Vec3::new(-a.z, 0.0, a.x),
            Vec3::new(a.y, -a.x, 0.0),
        ])
    }

    pub fn transpose(&self) -> Self {
        let [c0, c1, c2] = self.0;
        Self([
            Vec3::new(c0.x, c1.x, c2.x),
            Vec3::new(c0.y, c1.y, c2.y),
            Vec3::new(c0.z, c1.z, c2.z),
        ])
    }

    pub fn inverse(&self) -> Self {
        let [a, b, c] = self.0;
        // Rows of the inverse are the columns' pairwise crosses over the
        // determinant.
        let (r0, r1, r2) = (b.cross(c), c.cross(a), a.cross(b));
        let det = a.dot(r0);
        let inv = if det.abs() > 1e-30 { 1.0 / det } else { 0.0 };
        Self([r0 * inv, r1 * inv, r2 * inv]).transpose()
    }
}

impl Add for M3 {
    type Output = Self;
    fn add(self, o: Self) -> Self {
        Self([self.0[0] + o.0[0], self.0[1] + o.0[1], self.0[2] + o.0[2]])
    }
}

impl Sub for M3 {
    type Output = Self;
    fn sub(self, o: Self) -> Self {
        Self([self.0[0] - o.0[0], self.0[1] - o.0[1], self.0[2] - o.0[2]])
    }
}

impl Neg for M3 {
    type Output = Self;
    fn neg(self) -> Self {
        self * -1.0
    }
}

impl Mul<f32> for M3 {
    type Output = Self;
    fn mul(self, s: f32) -> Self {
        Self([self.0[0] * s, self.0[1] * s, self.0[2] * s])
    }
}

impl Mul<Vec3> for M3 {
    type Output = Vec3;
    fn mul(self, v: Vec3) -> Vec3 {
        self.0[0] * v.x + self.0[1] * v.y + self.0[2] * v.z
    }
}

impl Mul for M3 {
    type Output = Self;
    fn mul(self, o: Self) -> Self {
        Self([self * o.0[0], self * o.0[1], self * o.0[2]])
    }
}

/// A 6x6 matrix of 3x3 blocks, `[[a, b], [c, d]]`, acting on a frame's
/// angular velocity then a point's velocity.
#[derive(Clone, Copy, Debug)]
struct M6 {
    a: M3,
    b: M3,
    c: M3,
    d: M3,
}

/// A frame's angular velocity and a point's velocity.
type V6 = (Vec3, Vec3);

impl M6 {
    const ZERO: Self = Self {
        a: M3::ZERO,
        b: M3::ZERO,
        c: M3::ZERO,
        d: M3::ZERO,
    };

    fn mul(&self, o: &Self) -> Self {
        Self {
            a: self.a * o.a + self.b * o.c,
            b: self.a * o.b + self.b * o.d,
            c: self.c * o.a + self.d * o.c,
            d: self.c * o.b + self.d * o.d,
        }
    }

    fn apply(&self, (p, q): V6) -> V6 {
        (self.a * p + self.b * q, self.c * p + self.d * q)
    }

    fn transpose(&self) -> Self {
        Self {
            a: self.a.transpose(),
            b: self.c.transpose(),
            c: self.b.transpose(),
            d: self.d.transpose(),
        }
    }

    fn sub(&self, o: &Self) -> Self {
        Self {
            a: self.a - o.a,
            b: self.b - o.b,
            c: self.c - o.c,
            d: self.d - o.d,
        }
    }

    /// By the Schur complement of `a`.
    fn inverse(&self) -> Self {
        let ai = self.a.inverse();
        let aib = ai * self.b;
        let cai = self.c * ai;
        let si = (self.d - self.c * aib).inverse();
        Self {
            a: ai + aib * si * cai,
            b: -(aib * si),
            c: -(si * cai),
            d: si,
        }
    }
}

/// A unit quaternion: `v` its vector part, `w` its scalar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quat {
    pub v: Vec3,
    pub w: f32,
}

impl Mul for Quat {
    type Output = Self;

    /// The rotation `o`, then this one.
    fn mul(self, o: Self) -> Self {
        Self {
            v: o.v * self.w + self.v * o.w + self.v.cross(o.v),
            w: self.w * o.w - self.v.dot(o.v),
        }
    }
}

impl Quat {
    pub const IDENTITY: Self = Self {
        v: Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        w: 1.0,
    };

    pub fn conj(self) -> Self {
        Self {
            v: -self.v,
            w: self.w,
        }
    }

    pub fn rotate(self, p: Vec3) -> Vec3 {
        let t = self.v.cross(p) * 2.0;
        p + t * self.w + self.v.cross(t)
    }

    pub fn matrix(self) -> M3 {
        M3([
            self.rotate(Vec3::new(1.0, 0.0, 0.0)),
            self.rotate(Vec3::new(0.0, 1.0, 0.0)),
            self.rotate(Vec3::new(0.0, 0.0, 1.0)),
        ])
    }

    pub fn normalize(self) -> Self {
        let l = (self.v.length_squared() + self.w * self.w).sqrt();
        if l > 1e-12 {
            Self {
                v: self.v * (1.0 / l),
                w: self.w / l,
            }
        } else {
            Self::IDENTITY
        }
    }

    /// The rotation by `theta` (axis times angle).
    pub fn exp(theta: Vec3) -> Self {
        let angle = theta.length();
        if angle < 1e-12 {
            return Self::IDENTITY;
        }
        let half = 0.5 * angle;
        Self {
            v: theta * (half.sin() / angle),
            w: half.cos(),
        }
    }

    /// The rotation whose matrix is `m`, by Shepperd's method.
    pub fn from_matrix(m: M3) -> Self {
        let [c0, c1, c2] = m.0;
        let (m00, m11, m22) = (c0.x, c1.y, c2.z);
        let trace = m00 + m11 + m22;
        let q = if trace > 0.0 {
            let s = (trace + 1.0).sqrt() * 2.0;
            Self {
                v: Vec3::new(c1.z - c2.y, c2.x - c0.z, c0.y - c1.x) * (1.0 / s),
                w: 0.25 * s,
            }
        } else if m00 > m11 && m00 > m22 {
            let s = (1.0 + m00 - m11 - m22).sqrt() * 2.0;
            Self {
                v: Vec3::new(0.25 * s, (c1.x + c0.y) / s, (c2.x + c0.z) / s),
                w: (c1.z - c2.y) / s,
            }
        } else if m11 > m22 {
            let s = (1.0 + m11 - m00 - m22).sqrt() * 2.0;
            Self {
                v: Vec3::new((c1.x + c0.y) / s, 0.25 * s, (c2.y + c1.z) / s),
                w: (c2.x - c0.z) / s,
            }
        } else {
            let s = (1.0 + m22 - m00 - m11).sqrt() * 2.0;
            Self {
                v: Vec3::new((c2.x + c0.z) / s, (c2.y + c1.z) / s, 0.25 * s),
                w: (c0.y - c1.x) / s,
            }
        };
        q.normalize()
    }

    /// The frame whose axes are `d1`, `d3 × d1` and `d3`.
    pub fn from_frame(d1: Vec3, d3: Vec3) -> Self {
        Self::from_matrix(M3([d1, d3.cross(d1), d3]))
    }

    pub fn to_array(self) -> [f32; 4] {
        [self.v.x, self.v.y, self.v.z, self.w]
    }

    pub fn from_array(a: [f32; 4]) -> Self {
        Self {
            v: Vec3::new(a[0], a[1], a[2]),
            w: a[3],
        }
    }
}

/// The head's rotation as a quaternion.
pub fn rigid_rotation(pose: &Rigid) -> Quat {
    Quat::from_matrix(M3(pose.axes))
}

/// The turn from frame `a` to frame `b`, `a⁻¹ b`, signed as near as can be
/// to `groomed` (the groom's turn there), so that it changes smoothly
/// however far the joint bends from rest, short of a half turn: its vector
/// part, the Darboux vector, in `a`'s axes; how that changes with `a`'s
/// rotation in `a`'s axes (with `b`'s in `b`'s axes, this times the turn's
/// matrix, negated); and the turn's matrix.
fn darboux(a: Quat, b: Quat, groomed: Quat) -> (Vec3, M3, M3) {
    let mut q = a.conj().mul(b);
    // The groom's turn the short way round, as the rest was solved for.
    let side = (q.v.dot(groomed.v) + q.w * groomed.w) * groomed.w;
    if side < 0.0 || (side == 0.0 && q.w < 0.0) {
        q = Quat { v: -q.v, w: -q.w };
    }
    let change = (M3::skew(q.v) - M3::diagonal(q.w)) * 0.5;
    (q.v, change, q.matrix())
}

/// The segments' lengths and directions, groomed.
fn segments(points: &[Vec3]) -> (Vec<f32>, Vec<Vec3>) {
    points
        .windows(2)
        .map(|w| {
            let d = w[1] - w[0];
            (d.length().max(1e-6), safe_normalize(d, Vec3::down()))
        })
        .unzip()
}

/// The rod a strand rests as, two slots a point: each segment's frame (the
/// head's frame), then the Darboux vector of the joint before it and the
/// length it rests at; the last point's slots hold the follicle's frame. If
/// `sag_free`, the groom `rest` (in the head's frame, the head upright) is
/// the rod's equilibrium under gravity; otherwise the rod rests as groomed
/// and hangs from there.
pub fn rest_shape(rod: &Rod, rest: &[Vec3], sag_free: bool, shape: &mut [[f32; 4]]) {
    let n = rest.len();
    if n < 2 {
        shape.fill([0.0; 4]);
        return;
    }
    let segs = n - 1;
    let (lengths, tangents) = segments(rest);
    // What each segment carries: the weight below it.
    let mut carried = vec![Vec3::splat(0.0); segs];
    if sag_free {
        let mut below = Vec3::splat(0.0);
        for j in (0..segs).rev() {
            below += rod.gravity * point_mass(&lengths, j + 1);
            carried[j] = below;
        }
    }
    // Frames carried from the follicle's, twist-free, each axis sheared off
    // its segment as far as the weight it carries strains it.
    let normal = orthogonal(tangents[0]);
    let follicle = Quat::from_frame(normal, tangents[0]);
    let mut frames = Vec::with_capacity(segs);
    let mut rest_lengths = Vec::with_capacity(segs);
    let mut strains = Vec::with_capacity(segs);
    let (mut t, mut u) = (tangents[0], normal);
    for j in 0..segs {
        u = transport(u, t, tangents[j]);
        t = tangents[j];
        let strain = carried[j] * (lengths[j] / rod.stretch);
        let chord = tangents[j] * lengths[j] - strain;
        let length = chord.length().max(1e-9);
        let axis = chord * (1.0 / length);
        let d1 = safe_normalize(transport(u, t, axis), u);
        frames.push(Quat::from_frame(d1, axis));
        rest_lengths.push(length);
        strains.push(strain);
    }
    // Each joint rests bent and twisted by the moment it carries: the
    // weights' below it, through the segments as levers.
    let mut moment = Vec3::splat(0.0);
    for s in (0..segs).rev() {
        let a = if s == 0 { follicle } else { frames[s - 1] };
        let (turn, change, _) = darboux(a, frames[s], Quat::IDENTITY);
        let mut rest_turn = turn;
        if sag_free {
            let axis = frames[s].rotate(Vec3::new(0.0, 0.0, 1.0));
            let stretch = rod.stretch / lengths[s];
            moment += axis.cross(strains[s]) * (stretch * rest_lengths[s]);
            // The joint's torque on its second frame, `Gᵀ K (turn - rest)`
            // in the first frame's axes, holds the moment below it.
            let k = rod.joint(&lengths, s);
            let wanted = change.transpose().inverse() * (a.conj().rotate(moment) * -1.0);
            rest_turn = turn - Vec3::new(wanted.x / k.x, wanted.y / k.y, wanted.z / k.z);
        }
        shape[2 * s] = frames[s].to_array();
        shape[2 * s + 1] = [rest_turn.x, rest_turn.y, rest_turn.z, rest_lengths[s]];
    }
    shape[2 * segs] = follicle.to_array();
    shape[2 * segs + 1] = [0.0; 4];
}

/// One substep of `h` seconds, the head going from `from` to `to`.
pub struct Substep<'a> {
    pub rod: &'a Rod,
    pub h: f32,
    /// Seconds the strands' last substep took: their velocities are their
    /// moves over it.
    pub h_last: f32,
    pub from: Rigid,
    pub to: Rigid,
    pub collider: Option<&'a (dyn Collider + Sync)>,
    pub offset: f32,
    pub friction: f32,
}

/// A joint as a substep sees it, in its first frame's axes: with `G` how
/// its turn changes with that frame's rotation, its stiffness `Gᵀ K G`
/// against it, the torque `Gᵀ K (turn - rest)` that frame feels the
/// opposite of (the second, the same in its own axes), and the turn's
/// matrix, which takes the second frame's axes to the first's.
#[derive(Clone, Copy)]
struct Joint {
    stiffness: M3,
    torque: Vec3,
    turn: M3,
    /// The Darboux vector's curvature against either frame's rotation,
    /// `-(K (turn - rest)) · turn / 4` alike about every axis, where it
    /// stiffens: Gauss-Newton misses it, and a joint bent far from rest
    /// would overshoot without it.
    curving: f32,
}

impl Joint {
    /// The stiffness against the first frame's rotation, in its own axes.
    fn first(&self) -> M3 {
        self.stiffness + M3::diagonal(self.curving)
    }

    /// The stiffness against the second frame's rotation, in its own axes.
    fn second(&self) -> M3 {
        self.turn.transpose() * self.stiffness * self.turn + M3::diagonal(self.curving)
    }

    /// Against the second frame's rotation and the first's.
    fn across(&self) -> M3 {
        -(self.turn.transpose() * self.stiffness)
    }
}

/// A segment as a substep sees it: its frame's axes, how far it is from
/// running along the third at its rest length, its stiffness and that
/// length.
#[derive(Clone, Copy)]
struct Segment {
    axes: M3,
    strain: Vec3,
    stiffness: f32,
    length: f32,
}

impl Segment {
    fn axis(&self) -> Vec3 {
        self.axes.0[2]
    }

    /// How the strain changes with the frame's rotation in its own axes,
    /// transposed: `L [e3]×ᵀ Rᵀ`.
    fn lever(&self) -> M3 {
        M3::skew(Vec3::new(0.0, 0.0, 1.0)).transpose() * self.axes.transpose() * self.length
    }
}

impl Substep<'_> {
    /// Step one strand: `rest` its groomed points (the head's frame),
    /// `shape` its rod's rest ([`rest_shape`]), `air` the air's velocity
    /// about it; `x` and `previous` its points now and one substep's
    /// velocity behind, and `frames` each segment's frame and angular
    /// velocity (world space), two slots a point, which it leaves likewise.
    /// A frame's angular velocity is solved for in its own axes, where its
    /// twist, light and soft beside its stretch, stands apart exactly.
    pub fn strand(
        &self,
        rest: &[Vec3],
        shape: &[[f32; 4]],
        air: Vec3,
        x: &mut [Vec3],
        previous: &mut [Vec3],
        frames: &mut [[f32; 4]],
    ) {
        let n = x.len();
        if n < 2 {
            return;
        }
        let segs = n - 1;
        let rod = self.rod;
        let h = self.h;
        let (lengths, _) = segments(rest);
        // The root rides the head, its follicle turning with it.
        let root_from = self.from.apply(rest[0]);
        let root_to = self.to.apply(rest[0]);
        let root_velocity = (root_to - root_from) * (1.0 / h);
        x[0] = root_from;
        let head_from = rigid_rotation(&self.from);
        let follicle = head_from.mul(Quat::from_array(shape[2 * segs]));
        let follicle_spin = follicle
            .conj()
            .rotate(spin(head_from, rigid_rotation(&self.to), h));
        let frame = |j: usize| Quat::from_array(frames[2 * j]);

        let segments: Vec<Segment> = (0..segs)
            .map(|j| {
                let axes = frame(j).matrix();
                let length = shape[2 * j + 1][3];
                Segment {
                    axes,
                    strain: x[j + 1] - x[j] - axes.0[2] * length,
                    stiffness: rod.stretch / lengths[j],
                    length,
                }
            })
            .collect();
        // Joint s is the one before segment s.
        let joints: Vec<Joint> = (0..segs)
            .map(|s| {
                let a = if s == 0 { follicle } else { frame(s - 1) };
                let groomed = Quat::from_array(shape[2 * if s == 0 { segs } else { s - 1 }])
                    .conj()
                    .mul(Quat::from_array(shape[2 * s]));
                let (turn, change, matrix) = darboux(a, frame(s), groomed);
                let r = shape[2 * s + 1];
                let off = turn - Vec3::new(r[0], r[1], r[2]);
                let k = rod.joint(&lengths, s);
                let gk = change.transpose() * M3::scaling(k);
                let held = Vec3::new(k.x * off.x, k.y * off.y, k.z * off.z);
                Joint {
                    stiffness: gk * change,
                    torque: gk * off,
                    turn: matrix,
                    curving: (-0.25 * held.dot(turn)).max(0.0),
                }
            })
            .collect();

        // Implicit Euler for each frame's and point's new velocities, a
        // group (frame g, in its axes, and point g + 1) a row, factored as
        // it goes: (M + h D + (h² + h β) H_bend + h² H_stretch) v' = M v
        // + h F + h D air, the root's and follicle's known velocities moved
        // right.
        let elastic = h * h;
        let viscous = h * h + h * rod.damping;
        let identity = M3::diagonal(1.0);
        let mut factors: Vec<(M6, V6)> = Vec::with_capacity(segs);
        let mut d_last = M6::ZERO;
        let mut y_last = (Vec3::splat(0.0), Vec3::splat(0.0));
        for g in 0..segs {
            let i = g + 1;
            let seg = segments[g];
            let next = segments.get(g + 1);
            let after = joints.get(g + 1);
            let m = point_mass(&lengths, i);
            let velocity = (x[i] - previous[i]) * (1.0 / self.h_last);
            let w = frames[2 * g + 1];
            let spinning = seg.axes.transpose() * Vec3::new(w[0], w[1], w[2]);
            // The air, across the point's tangent.
            let mut tangent = seg.strain + seg.axis() * seg.length;
            if let Some(next) = next {
                tangent += next.strain + next.axis() * next.length;
            }
            let tangent = safe_normalize(tangent, seg.axis());
            let relative = velocity - air;
            let across = relative - tangent * relative.dot(tangent);
            let speed = across.length();
            let c = rod.drag * (speed + rod.drag_viscous * speed.cbrt());
            let drag = (identity - M3::outer(tangent, tangent)) * (m * c);

            let inertia = M3::scaling(Vec3::new(0.25, 0.25, 0.5)) * (lengths[g] * rod.gyration);
            let lever = seg.lever() * seg.stiffness;
            let local_strain = seg.axes.transpose() * seg.strain;
            let mut bend = joints[g].second();
            let mut torque = Vec3::new(0.0, 0.0, 1.0).cross(local_strain)
                * (seg.stiffness * seg.length)
                + joints[g].turn.transpose() * joints[g].torque;
            if let Some(after) = after {
                bend = bend + after.first();
                torque -= after.torque;
            }
            let mut force = rod.gravity * m - seg.strain * seg.stiffness;
            let mut pull = seg.stiffness;
            if let Some(next) = next {
                force += next.strain * next.stiffness;
                pull += next.stiffness;
            }
            // Turning the frame across its axis: the shear's stiffness, and
            // the tension's along it (which the Gauss-Newton part misses,
            // and without which a stretched segment's light frame
            // overshoots).
            let across_axis = seg.length * (seg.length + local_strain.z.max(0.0));
            let diagonal = M6 {
                a: inertia
                    + M3::scaling(Vec3::new(1.0, 1.0, 0.0))
                        * (elastic * seg.stiffness * across_axis)
                    + bend * viscous,
                b: lever * elastic,
                c: lever.transpose() * elastic,
                d: identity * (m + elastic * pull) + drag * h,
            };
            let left = M6 {
                a: joints[g].across() * viscous,
                b: -(lever * elastic),
                c: M3::ZERO,
                d: -(identity * (elastic * seg.stiffness)),
            };
            let mut rhs = (
                inertia * spinning + torque * h,
                velocity * m + force * h + drag * air * h,
            );
            let left = if g == 0 {
                let (p, q) = left.apply((follicle_spin, root_velocity));
                rhs = (rhs.0 - p, rhs.1 - q);
                M6::ZERO
            } else {
                left
            };
            // Block LDLᵀ: this row's L against the last row's D.
            let l = left.mul(&d_last);
            let d = diagonal.sub(&l.mul(&left.transpose()));
            let d_inverse = d.inverse();
            let ly = l.apply(y_last);
            let y = (rhs.0 - ly.0, rhs.1 - ly.1);
            factors.push((l, d_inverse.apply(y)));
            d_last = d_inverse;
            y_last = y;
        }

        // Back up the strand: each group's new velocities, then where its
        // point goes, the head's push, and its frame's turn.
        x[0] = root_to;
        previous[0] = root_from;
        let mut later = (Vec3::splat(0.0), Vec3::splat(0.0));
        for g in (0..segs).rev() {
            let mut u = factors[g].1;
            if g + 1 < segs {
                let lu = factors[g + 1].0.transpose().apply(later);
                u = (u.0 - lu.0, u.1 - lu.1);
            }
            later = u;
            let (w, v) = u;
            let i = g + 1;
            let (p, v) = self.contact(x[i] + v * h, v, rest[i]);
            x[i] = p;
            previous[i] = p - v * h;
            let w = segments[g].axes * w;
            let turned = Quat::exp(w * h)
                .mul(Quat::from_array(frames[2 * g]))
                .normalize();
            frames[2 * g] = turned.to_array();
            frames[2 * g + 1] = [w.x, w.y, w.z, 0.0];
        }
    }

    /// Off the head, if `p` is in it: out to the offset -- or as near as
    /// the groom has the point (`groomed`, the head's frame; even a little
    /// in, where the groom's head and the collider's differ), hair lying on
    /// the scalp staying as it lies -- the velocity into it stopped and the
    /// sliding slowed by Coulomb friction, relative to the head's surface
    /// there.
    fn contact(&self, p: Vec3, v: Vec3, groomed: Vec3) -> (Vec3, Vec3) {
        let Some(collider) = self.collider else {
            return (p, v);
        };
        let local = self.to.unapply(p);
        if collider.distance(local) >= self.offset {
            return (p, v);
        }
        let offset = self.offset.min(collider.distance(groomed));
        if collider.distance(local) >= offset {
            return (p, v);
        }
        let out = push_out(collider, local, offset);
        let normal = self.to.rotate(collider.query(out).1);
        let p = self.to.apply(out);
        // The head's own velocity there.
        let surface = (p - self.from.apply(self.to.unapply(p))) * (1.0 / self.h);
        let mut relative = v - surface;
        let into = relative.dot(normal);
        if into < 0.0 {
            relative -= normal * into;
            let sliding = relative.length();
            if sliding > 1e-9 {
                relative *= (1.0 - self.friction * -into / sliding).max(0.0);
            }
        }
        (p, relative + surface)
    }
}

/// The angular velocity turning `from` to `to` in `h` seconds.
pub fn spin(from: Quat, to: Quat, h: f32) -> Vec3 {
    let mut q = to.mul(from.conj());
    if q.w < 0.0 {
        q = Quat { v: -q.v, w: -q.w };
    }
    let s = q.v.length();
    if s < 1e-9 {
        return q.v * (2.0 / h);
    }
    q.v * (2.0 * s.atan2(q.w) / (s * h))
}

/// Each segment's frame for a strand at `x` whose rod rests as `shape`,
/// groomed as `rest`, the head at `head`: the head-carried rest frame turned
/// the least way from the groomed segment onto the segment now, not
/// spinning; two slots a point.
pub fn frames_along(
    head: &Rigid,
    rest: &[Vec3],
    shape: &[[f32; 4]],
    x: &[Vec3],
    frames: &mut [[f32; 4]],
) {
    let q = rigid_rotation(head);
    for j in 0..x.len().saturating_sub(1) {
        let carried = q.mul(Quat::from_array(shape[2 * j]));
        let groomed = head.rotate(safe_normalize(rest[j + 1] - rest[j], Vec3::down()));
        let along = safe_normalize(x[j + 1] - x[j], groomed);
        let turned = least_turn(groomed, along).mul(carried).normalize();
        frames[2 * j] = turned.to_array();
        frames[2 * j + 1] = [0.0; 4];
    }
}

/// The least rotation taking unit `a` to unit `b`.
fn least_turn(a: Vec3, b: Vec3) -> Quat {
    let c = a.dot(b);
    if c < -0.999_999 {
        return Quat {
            v: orthogonal(a),
            w: 0.0,
        };
    }
    Quat {
        v: a.cross(b),
        w: 1.0 + c,
    }
    .normalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The implicit step is only as good as its derivatives: the Darboux
    /// vector changes with either frame's turn as `darboux` says.
    #[test]
    fn the_turn_changes_as_its_derivative_says() {
        let a = Quat::exp(Vec3::new(0.3, -0.2, 0.5));
        let b = Quat::exp(Vec3::new(-0.6, 0.9, 0.4)).mul(a);
        let (turn, change, matrix) = darboux(a, b, Quat::IDENTITY);
        let e = 1e-3;
        for k in 0..3 {
            let mut d = Vec3::splat(0.0);
            d.set_axis(k, e);
            let (by_a, _, _) = darboux(a.mul(Quat::exp(d)), b, Quat::IDENTITY);
            let (by_b, _, _) = darboux(a, b.mul(Quat::exp(d)), Quat::IDENTITY);
            let off_a = (by_a - turn) - change * d;
            let off_b = (by_b - turn) + change * matrix * d;
            assert!(
                off_a.length() < 1e-5 && off_b.length() < 1e-5,
                "{off_a:?} {off_b:?}"
            );
        }
    }

    /// A 6x6 block solve undoes its matrix.
    #[test]
    fn a_block_inverts() {
        let m = M6 {
            a: M3([
                Vec3::new(4.0, 1.0, 0.5),
                Vec3::new(1.0, 3.0, 0.2),
                Vec3::new(0.5, 0.2, 2.0),
            ]),
            b: M3([
                Vec3::new(0.3, 0.1, 0.0),
                Vec3::new(0.2, 0.4, 0.1),
                Vec3::new(0.0, 0.1, 0.5),
            ]),
            c: M3::ZERO,
            d: M3::diagonal(5.0),
        };
        let m = M6 {
            c: m.b.transpose(),
            ..m
        };
        let v = (Vec3::new(1.0, 2.0, 3.0), Vec3::new(-1.0, 0.5, 2.0));
        let (p, q) = m.inverse().apply(m.apply(v));
        assert!((p - v.0).length() + (q - v.1).length() < 1e-5);
    }
}
