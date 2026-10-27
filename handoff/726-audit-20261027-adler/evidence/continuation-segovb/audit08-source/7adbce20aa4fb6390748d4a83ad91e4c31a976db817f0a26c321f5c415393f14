//! A hairdo: follicles, a cut and a style, and the guides they grow.
//!
//! DiffLocks' artists drew 75 base hairstyles by hand in Blender, each about
//! fifty guide strands giving only "the overall shape and direction of the
//! hair"; the randomisation adds everything else. Those guides are not
//! published. Here a base style is three independent choices -- the
//! [`Follicles`] (which way hair grows), the [`Haircut`] (how long it is)
//! and the [`Hairstyle`] (how it is combed) -- and every guide is grown as a
//! strand of hair would lie:
//!
//! * **Out of the scalp.** A strand leaves its follicle the way it is
//!   combed, at the follicle's angle to the scalp (lifted as the style lifts
//!   it), and rises over a few centimetres to its layer: the hair under it.
//!   (Which way the follicle itself points shows in the first centimetre of
//!   every strand, [`crate::ops::Orient`], and in how hair lies where the
//!   style leaves it as it grows.) Hair rooted higher up -- nearer the part
//!   or the crown -- lies over hair rooted lower down, so the layer's height
//!   grows from nothing at the hairline to the style's volume at the top.
//! * **On the head.** Lying on its layer, a strand follows the style's comb
//!   (away from a part, back, forward, to a tie, or as it grows), turned
//!   into the layer's tangent plane, and slides downhill as much as the
//!   hair still to come weighs on it: long hair on the side of the head
//!   runs down it, short hair keeps the way it was combed. In front of the
//!   face, hair keeps to its comb until it is past the face.
//! * **Off the head.** Free -- past the head's widest, or lifted off it -- a
//!   strand is a cantilever: gravity bends it with the curvature a rod's own
//!   weight gives, `(L - s)^2 / (2 L_g^3)` for `L - s` of it still to come
//!   and `L_g` the style's hold (about 5 cm for a strand, more for a styled
//!   or curly lock). Long hair drops straight down where it leaves the
//!   head; short hair stands out.
//! * **Cut.** A strand is as long as the cut makes hair at its root, and
//!   ends where it crosses the cut's line (the hemline, or a fringe's line
//!   at the brows) or the cut's dome.
//! * **Ties.** A ponytail's, bun's or braid's strands run over the scalp to
//!   the tie, then become the tail, the coil or one of the braid's three
//!   lanes.

use crate::identity::PartingGroup;
use crate::identity::StrandId;
use fabelgeist_math::Vec3;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::follicles::{Follicles, GrowthField};
use crate::grid::PointGrid;
use crate::haircut::Haircut;
use crate::hairstyle::{Arrangement, Hairstyle};
use crate::head::Head;
use crate::ops::{Parting, smoothstep};
use crate::rng::{self, Stream};
use crate::strands::{Attachment, Strands, resample_into, rotate, safe_normalize};
use crate::surface::Surface;

mod tied;
pub(crate) use tied::Tied;

/// Follicles, a cut and a style: everything that shapes the guides.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Hairdo {
    pub follicles: Follicles,
    pub haircut: Haircut,
    pub hairstyle: Hairstyle,
}

/// Which way the comb runs.
#[derive(Clone, Copy, Debug)]
enum Flow {
    /// As the hair grows.
    Natural,
    /// Away from a part at `x`.
    Part(f32),
    /// Swept over to one side (`+1` or `-1`) from a deep part.
    Swept(f32),
    Back,
    Forward,
    /// Out from the head's centre.
    Radial,
    /// Down, round the head: hair with no part.
    Down,
    /// Over the scalp to a tie, then as the tie has it.
    Tie(Tie),
}

#[derive(Clone, Copy, Debug)]
enum Tie {
    Tail,
    Bun,
    Braid,
}

impl Hairdo {
    /// `base` with the follicles' hairline: the head to grow this hairdo
    /// on, and to cut its scalp from.
    pub fn head(&self, base: &Head) -> Head {
        self.follicles.head(base)
    }

    /// Whether the cut's fringe hangs over the forehead: it does unless
    /// the style combs it back or up.
    pub fn fringe_falls(&self) -> bool {
        self.haircut.fringe.is_some()
            && !matches!(
                self.hairstyle.arrangement,
                Arrangement::Back | Arrangement::Up
            )
    }

    /// Whether the hair stands out from the head rather than lying on it:
    /// its shape follows each root's own normal all the way.
    pub fn stands(&self) -> bool {
        self.hairstyle.stands()
    }

    /// Whether the hair hangs free, and so is draped. Tied hair is held at
    /// its tie, and styled hair by its product, which the drape knows
    /// nothing of.
    pub fn drapes(&self) -> bool {
        matches!(
            self.hairstyle.arrangement,
            Arrangement::Natural
                | Arrangement::Parted
                | Arrangement::Swept
                | Arrangement::Forward
                | Arrangement::Down
        ) && self.haircut.longest() > 0.2
            && self.haircut.dome == 0.0
    }

    /// How interpolation should keep strands apart: a braid's three lanes,
    /// a fringe from the rest, or either side of a part.
    pub fn parting(&self) -> Parting {
        if self.hairstyle.arrangement == Arrangement::Braid {
            return Parting::Lanes { count: 3 };
        }
        if self.fringe_falls()
            && let Some(fringe) = self.haircut.fringe
        {
            return Parting::Fringe {
                front: fringe.back(),
                half_width: fringe.width,
            };
        }
        match self.hairstyle.arrangement {
            Arrangement::Parted | Arrangement::Swept => Parting::line(self.hairstyle.part, -0.03),
            _ => Parting::None,
        }
    }

    fn flow(&self) -> Flow {
        let style = &self.hairstyle;
        match style.arrangement {
            Arrangement::Natural => Flow::Natural,
            Arrangement::Parted => Flow::Part(style.part),
            Arrangement::Swept => Flow::Swept(style.sweep()),
            Arrangement::Back => Flow::Back,
            Arrangement::Forward => Flow::Forward,
            Arrangement::Down => Flow::Down,
            Arrangement::Up => Flow::Radial,
            Arrangement::Ponytail => Flow::Tie(Tie::Tail),
            Arrangement::Bun => Flow::Tie(Tie::Bun),
            Arrangement::Braid => Flow::Tie(Tie::Braid),
        }
    }

    /// How many guides to place, before the row along the hairline.
    pub(crate) fn guide_count(&self) -> usize {
        if self.haircut.dome > 0.0 {
            160
        } else if self.haircut.longest() < 0.02 {
            140
        } else {
            110
        }
    }

    /// The guides on `head` (the hairdo's own, [`Hairdo::head`]), rooted on
    /// `scalp`, `points` points each. `variation` (0 to 1) jitters their
    /// lengths and directions.
    pub fn guides(
        &self,
        head: &Head,
        scalp: &Surface,
        points: usize,
        variation: f32,
        seed: u64,
    ) -> Strands {
        let count = self.guide_count();
        let key = guide_key(seed);
        let spacing = guide_spacing(scalp.area(), count);
        let mut placed = PointGrid::new(spacing);
        let mut roots = Vec::new();
        // Candidates drawn by index, so the GPU draws the same ones.
        let [a, u, v] = [1, 2, 3].map(|k| rng::hash(key, k));
        for i in 0..(count * 40) as u32 {
            if roots.len() >= count {
                break;
            }
            let p = scalp.sample_at(rng::unit(i, a), rng::unit(i, u), rng::unit(i, v));
            if placed.any_within(p.position, spacing, |_| true) {
                continue;
            }
            placed.insert(p.position, PartingGroup::from_gpu_word(0));
            roots.push(p);
        }
        // And a row along the hairline, closer together. The hairline is
        // where hair is seen most and where combing turns most -- aside at
        // the temples, down at the nape -- and a strand rooted there takes
        // its shape from the guides nearest it: from a guide centimetres up
        // the head it would hang across the face.
        let edge_spacing = edge_spacing(spacing);
        let mut edge = PointGrid::new(edge_spacing);
        for p in scalp.boundary(0.002) {
            if edge.any_within(p, edge_spacing, |_| true)
                || placed.any_within(p, 0.5 * spacing, |_| true)
            {
                continue;
            }
            // Just inside the edge, on the scalp.
            let Some(on) = scalp.closest(p, 0.01) else {
                continue;
            };
            edge.insert(p, PartingGroup::from_gpu_word(0));
            roots.push(on);
        }
        let parting = self.parting();
        let field = self.follicles.field(head);
        let tie = Tied::new(head, self);
        // Every guide on its own, in parallel, each with its own random
        // numbers.
        let paths: Vec<Vec<Vec3>> = roots
            .par_iter()
            .enumerate()
            .map(|(i, root)| {
                let mut rng = Stream::new(comb_key(key, i as u32));
                let path = self.comb(
                    head,
                    &field,
                    &parting,
                    tie.as_ref(),
                    root.position,
                    root.normal,
                    &mut rng,
                    variation,
                );
                let mut positions = vec![Vec3::default(); points];
                resample_into(&path, &mut Vec::new(), &mut positions);
                positions
            })
            .collect();
        let mut strands = Strands::with_capacity(points, roots.len());
        for (i, (root, positions)) in roots.iter().zip(&paths).enumerate() {
            strands.push(
                positions,
                Attachment {
                    uv: root.uv,
                    normal: root.normal,
                    id: StrandId::from_word(rng::hash(i as u32, 0x006D_1DE5)),
                    group: parting.group(root.position),
                    radius: self.follicles.radius,
                },
            );
        }
        strands
    }

    /// Grow one guide: a polyline from `root`, in steps of a millimetre or
    /// two.
    #[allow(clippy::too_many_arguments)]
    fn comb(
        &self,
        head: &Head,
        field: &GrowthField,
        parting: &Parting,
        tie: Option<&Tied>,
        root: Vec3,
        normal: Vec3,
        rng: &mut Stream,
        variation: f32,
    ) -> Vec<Vec3> {
        let (follicles, cut, style) = (&self.follicles, &self.haircut, &self.hairstyle);
        let elevation = head.elevation(root);
        let azimuth = head.azimuth(root);
        let frontness = smoothstep(0.3, 0.95, azimuth.cos()) * (1.0 - elevation).powf(0.5);

        // The follicle: which way it points and at what angle.
        let spread = follicles.direction_spread * rng.gaussian().clamp(-2.5, 2.5);
        let grows = rotate(field.direction(root, normal), normal, spread);
        let follicle_angle =
            follicles.angle + follicles.angle_spread * rng.gaussian().clamp(-2.5, 2.5);

        let mut flow = self.flow();
        let mut stiffness = style.hold;
        let mut angle = follicle_angle + style.lift;
        let mut length = cut.length(head, root);
        let mut tuck = style.tuck;
        let fringed = cut.in_fringe(root) && self.fringe_falls();
        if fringed {
            // A fringe is combed down over the forehead.
            flow = Flow::Forward;
            stiffness = 0.06;
            tuck = 0.0;
        } else if cut.clipped(head, root) {
            // Clipped hair lies as it grows.
            flow = Flow::Natural;
            angle = follicle_angle;
            stiffness = 1.0;
            tuck = 0.0;
        }
        let angle = angle.clamp(0.05, 1.5);
        length *= 1.0 + variation * 0.12 * rng.gaussian().clamp(-2.0, 2.0);
        let wander =
            Vec3::new(rng.gaussian(), rng.gaussian() * 0.3, rng.gaussian()) * (0.12 * variation);

        // The layer this strand lies on, and how far it takes to rise to it.
        let layer = if matches!(flow, Flow::Radial) {
            0.0
        } else {
            style.volume * elevation.powf(0.7)
        };
        let floor = 0.0008;
        let rise = (layer / angle.tan().max(0.05)).max(0.01);
        // The face, from the head itself: skin below the hairline that faces
        // forward, near enough to lie on. No hair grows there, so hair over
        // it has none beneath: a fringe lies a few millimetres off it (a
        // little more for some strands, so it has body), not on the layer it
        // came from.
        let fringe_gap = 0.005 + 0.004 * rng.next_f32();
        let over_face = |p: Vec3| {
            head.above_hairline(p) < 0.0 && head.distance(p) < 0.03 && head.gradient(p).z > 0.35
        };
        let height = |p: Vec3, s: f32| {
            let own = layer * smoothstep(0.0, rise, s) + floor;
            if over_face(p) {
                own.min(fringe_gap)
            } else {
                own
            }
        };

        let part_side = |x: f32| if root.x - x >= 0.0 { 1.0 } else { -1.0 };
        // Down and round the head; at the front, aside and back from the
        // face.
        let down_at = |p: Vec3| {
            let outward = safe_normalize(p - head.cranium.center, Vec3::up());
            if p.z > 0.0 {
                let side = if root.x >= 0.0 { 1.0 } else { -1.0 };
                Vec3::new(side, -0.2, -0.6)
            } else {
                Vec3::new(outward.x, -0.6, outward.z - 0.3)
            }
        };
        // Long hair left as it grows, or combed forward, does not stay
        // that way: it falls round the head, clear of the face, as its
        // own weight takes it.
        let falls = if fringed || !matches!(flow, Flow::Natural | Flow::Forward) {
            0.0
        } else {
            0.85 * smoothstep(0.12, 0.3, length)
        };
        let flow_at = |p: Vec3| -> Vec3 {
            let outward = safe_normalize(p - head.cranium.center, Vec3::up());
            let own = |v: Vec3| {
                safe_normalize(v, Vec3::down()) * (1.0 - falls)
                    + safe_normalize(down_at(p), Vec3::down()) * falls
            };
            match flow {
                Flow::Natural => own(field.direction(p, head.gradient(p))),
                Flow::Part(x) => Vec3::new(
                    part_side(x) * (0.8 + 0.6 * frontness),
                    -0.1,
                    -0.35 * (1.0 - frontness),
                ),
                // Over to the swept side, and down the other from beyond the part.
                Flow::Swept(side) => {
                    let side = if (root.x - style.part) * side < 0.0 {
                        -side
                    } else {
                        side
                    };
                    Vec3::new(side, -0.05, 0.1 * (1.0 - elevation))
                }
                Flow::Back => Vec3::new(0.0, -0.05, -1.0),
                Flow::Forward => own(Vec3::new(0.0, -0.3, 1.0)),
                Flow::Radial => outward,
                Flow::Down => down_at(p),
                Flow::Tie(_) => tie.map_or(Vec3::down(), |t| t.point - p),
            }
        };

        // Out of the follicle the way it is combed, a little off as each
        // follicle is. Which way the follicle itself points near the root is
        // for every strand to take ([`crate::ops::Orient`]): a guide turning
        // from it to the comb over a few millimetres would be a hook
        // interpolation spreads over centimetres.
        let first = rotate(
            safe_normalize(tangent(flow_at(root) + wander, normal), grows),
            normal,
            spread,
        );
        let leaving = first * angle.cos() + normal * angle.sin();

        let mut path = vec![root];
        let mut p = root;
        let mut d = safe_normalize(leaving, normal);
        if cut.dome > 0.0 {
            // Out to the dome, and a little more for the wandering on the way.
            // Centred above and a little in front of the cranium's centre:
            // an afro is fullest over the crown and the temples, and short at
            // the nape.
            let centre = head.cranium.center + Vec3::new(0.0, 0.04, 0.012);
            let o = root - centre;
            let b = d.dot(o);
            let c = o.dot(o) - cut.dome * cut.dome;
            let reach = -b + (b * b - c).max(0.0).sqrt();
            length = reach.max(0.02) * (1.2 + 0.1 * rng.gaussian().clamp(-2.0, 2.0));
        }
        let ds = (length / 300.0).clamp(0.0004, 0.002);
        let steps = (length / ds).ceil() as usize;
        // Where a tied strand is in its bundle, and in its braid lane.
        let mut tied_at: Option<f32> = None;
        let bundle = Vec3::new(rng.gaussian(), rng.gaussian(), 0.0) * 0.004;
        let lane = match parting {
            Parting::Lanes { count } => parting.group(root).lane(*count),
            _ => 0.0,
        };
        let bun_phase = azimuth;
        let is_cut = |p: Vec3| cut.line(head, root, p).is_some_and(|y| p.y < y);
        let to_tie = tie.map_or(0.0, |t| (t.point - root).length());

        for step in 1..=steps {
            let s = step as f32 * ds;
            let remaining = length - s;

            // Past the tie: the tail, the coil or the braid, all strands in
            // one bundle about the tie's axis.
            if let (Some(tie), Some(at)) = (tie, tied_at) {
                let along = s - at;
                p = tie.bundled(along, bundle, lane, bun_phase);
                path.push(p);
                if is_cut(p) {
                    break;
                }
                continue;
            }

            let lift = height(p, s);
            let (distance, n) = (head.distance(p), head.gradient(p));
            let gap = distance - lift;
            // Hair over the forehead rests on it, however high it came from.
            let resting = over_face(p) && gap < 0.03;
            let supported =
                (gap < style.cling || resting) && n.y > -0.3 && !matches!(flow, Flow::Radial);
            let force = Vec3::down();

            if let (Flow::Tie(_), Some(tie)) = (flow, tie) {
                // At the tie -- or, for a strand that cannot reach it from
                // right beneath it, as far as it would have come.
                if (tie.point - p).length() < 0.02 || s > 1.6 * to_tie + 0.03 {
                    tied_at = Some(s);
                    continue;
                }
                // Over the scalp to the tie, held there: no sliding.
                let want = safe_normalize(tangent(tie.point - p, n), d);
                d = safe_normalize(d.lerp(want, (ds / 0.01).min(1.0)), want);
            } else if supported {
                // Lying on the hair beneath: the comb, and gravity down the
                // slope as the hair still to come weighs.
                // A fringe combed forward over the hairline falls straight down
                // the forehead: forward, turned into a curved forehead, would
                // gather it to a point over the nose.
                let comb_to = if over_face(p) && fringed {
                    Vec3::down()
                } else {
                    flow_at(p) + wander
                };
                let combed = safe_normalize(tangent(comb_to, n), d);
                let downhill = tangent(force, n);
                let mut slide = smoothstep(0.0, 4.0 * stiffness, remaining);
                // Hair lying combed on the hair beneath is held there by
                // friction, and slides only where the weight of what hangs
                // from it pulls it off. Over the front of the head that pull
                // is along the comb, to where the hair hangs beside the face:
                // downhill there would be onto the face, which only hair
                // combed that way (a fringe) goes.
                if !fringed && over_face(p + safe_normalize(downhill, d) * 0.02) {
                    slide *= 0.05;
                }
                let want = safe_normalize(combed * (1.0 - slide) + downhill * slide, combed);
                // Rising to its layer near the root, the strand keeps its angle.
                let rising = 1.0 - smoothstep(0.0, rise, s);
                let want = safe_normalize(want * (1.0 - rising) + leaving * rising, want);
                d = safe_normalize(d.lerp(want, (ds / 0.012).min(1.0)), want);
            } else {
                // Free: a cantilever under its own weight -- stiff or short
                // hair standing out barely bends.
                d = cantilever(d, force, remaining, stiffness, ds);
            }
            // Coily hair wanders: a random turn a step, growing with the root of
            // the distance (a random walk in direction).
            if follicles.kink > 0.0 {
                let axis = safe_normalize(
                    d.cross(Vec3::new(rng.gaussian(), rng.gaussian(), rng.gaussian())),
                    crate::strands::orthogonal(d),
                );
                d = rotate(d, axis, rng.gaussian() * follicles.kink * ds.sqrt());
                let outward = safe_normalize(p - head.cranium.center, d);
                d = safe_normalize(d + outward * 0.05, d);
            }
            // The last few centimetres of a tucked cut turn in towards the neck.
            if tuck > 0.0 && remaining < 0.05 {
                let neck = Vec3::new(0.0, p.y, head.neck.a.z);
                let inward = safe_normalize(tangent(neck - p, d), d);
                d = safe_normalize(d + inward * (tuck * ds / 0.05), d);
            }

            let mut next = p + d * ds;
            let floor_here = height(next, s + ds);
            let distance = head.distance(next);
            if distance < floor_here {
                next += head.gradient(next) * (floor_here - distance);
                next = p + safe_normalize(next - p, d) * ds;
            } else if supported && distance > floor_here {
                // Lying on the layer below: settle onto it where it drops away.
                next -= head.gradient(next) * ((distance - floor_here) * 0.15);
                next = p + safe_normalize(next - p, d) * ds;
            }
            d = safe_normalize(next - p, d);
            p = next;
            path.push(p);
            if is_cut(p) {
                break;
            }
        }
        path
    }
}

/// The key every draw of a hairdo's guides derives from.
pub(crate) fn guide_key(seed: u64) -> u32 {
    rng::hash(seed as u32 ^ 0x0571_71E5, (seed >> 32) as u32)
}

/// The key of guide `i`'s own draws as it is combed.
pub(crate) fn comb_key(key: u32, i: u32) -> u32 {
    rng::hash(key ^ 0x0C0B, i)
}

/// How far apart guides are placed, `count` of them on `area`.
pub(crate) fn guide_spacing(area: f32, count: usize) -> f32 {
    0.7 * (area / count as f32).sqrt()
}

/// How far apart the row of guides along the hairline is.
pub(crate) fn edge_spacing(spacing: f32) -> f32 {
    (0.6 * spacing).max(0.008)
}

impl Hairdo {
    /// How the comb runs, as the GPU's guide kernel takes it: a code
    /// (natural, part, swept, back, forward, radial, down, tie) and its one
    /// value (the part's `x`, the sweep's side, the tie: tail, bun or
    /// braid).
    #[cfg_attr(not(feature = "gpu"), allow(dead_code))]
    pub(crate) fn flow_code(&self) -> (u32, f32) {
        match self.flow() {
            Flow::Natural => (0, 0.0),
            Flow::Part(x) => (1, x),
            Flow::Swept(side) => (2, side),
            Flow::Back => (3, 0.0),
            Flow::Forward => (4, 0.0),
            Flow::Radial => (5, 0.0),
            Flow::Down => (6, 0.0),
            Flow::Tie(Tie::Tail) => (7, 0.0),
            Flow::Tie(Tie::Bun) => (7, 1.0),
            Flow::Tie(Tie::Braid) => (7, 2.0),
        }
    }
}

/// Bend `d` towards `force` as a cantilever with `remaining` metres of hair
/// still to hang off it: curvature `remaining^2 / (2 L_g^3)` times the sine
/// of the angle between them, over one step, never past the force.
fn cantilever(d: Vec3, force: Vec3, remaining: f32, stiffness: f32, ds: f32) -> Vec3 {
    let axis = d.cross(force);
    let sin = axis.length();
    if sin < 1e-6 {
        return d;
    }
    let angle = sin.atan2(d.dot(force));
    let curvature = remaining.max(0.0).powi(2) / (2.0 * stiffness.powi(3)) * sin;
    rotate(d, axis * (1.0 / sin), (curvature * ds).min(angle))
}

/// The component of `v` in the plane with normal `n`.
fn tangent(v: Vec3, n: Vec3) -> Vec3 {
    v - n * v.dot(n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::follicles::Growth;
    use crate::haircut::Cut;
    use crate::presets::Preset;

    fn outside(head: &Head, guides: &Strands) -> bool {
        let inside = guides
            .positions
            .iter()
            .filter(|p| head.distance(**p) < -0.002)
            .count();
        inside * 50 < guides.positions.len()
    }

    #[test]
    fn every_preset_grows_guides_outside_the_head() {
        let base = Head::mannequin();
        for preset in Preset::ALL {
            let hairdo = preset.hairdo();
            let head = hairdo.head(&base);
            let scalp = head.scalp(0.008);
            let guides = hairdo.guides(&head, &scalp, 32, 0.5, 1);
            assert!(guides.len() > 30, "{}: {}", preset.name(), guides.len());
            assert!(!guides.has_non_finite(), "{}", preset.name());
            assert!(outside(&head, &guides), "{}", preset.name());
        }
    }

    #[test]
    fn every_cut_style_and_growth_combine() {
        let head = Head::mannequin();
        let scalp = head.scalp(0.01);
        for (i, cut) in Cut::ALL.into_iter().enumerate() {
            for (j, arrangement) in Arrangement::ALL.into_iter().enumerate() {
                let hairdo = Hairdo {
                    follicles: Follicles {
                        growth: Growth::ALL[(i + j) % Growth::ALL.len()],
                        ..Default::default()
                    },
                    haircut: Haircut::new(cut),
                    hairstyle: Hairstyle::new(arrangement),
                };
                let guides = hairdo.guides(&head, &scalp, 16, 0.5, 2);
                assert!(!guides.has_non_finite(), "{cut:?} {arrangement:?}");
                assert!(outside(&head, &guides), "{cut:?} {arrangement:?}");
            }
        }
    }

    #[test]
    fn roots_leave_the_scalp_gently() {
        let base = Head::mannequin();
        let hairdo = Preset::Long.hairdo();
        let head = hairdo.head(&base);
        let scalp = head.scalp(0.008);
        let guides = hairdo.guides(&head, &scalp, 256, 0.0, 3);
        // Rooted on the scalp, and the first few millimetres -- leaving it
        // at the follicle's angle over a curving head, rising to its layer
        // -- within a centimetre: at a steep follicle's 0.7 rad (the angle
        // two spreads up) and some of the layer, 8 mm along is 8 mm up.
        for s in 0..guides.len() {
            let strand = guides.strand(s);
            assert!(head.distance(strand[0]).abs() < 0.002);
            for p in &strand[..6] {
                assert!(head.distance(*p) < 0.01, "{}", head.distance(*p));
            }
        }
    }

    #[test]
    fn short_hair_follows_the_growth_pattern() {
        let head = Head::mannequin();
        let scalp = head.scalp(0.008);
        for growth in [Growth::Forward, Growth::Backward] {
            let hairdo = Hairdo {
                follicles: Follicles {
                    growth,
                    direction_spread: 0.0,
                    ..Default::default()
                },
                haircut: Haircut::new(Cut::Crop),
                hairstyle: Hairstyle::new(Arrangement::Natural),
            };
            let guides = hairdo.guides(&head, &scalp, 16, 0.0, 4);
            // On top, the tips lie towards the face or away from it.
            let (mut sum, mut n) = (0.0, 0);
            for s in 0..guides.len() {
                let strand = guides.strand(s);
                if strand[0].y > 0.1 {
                    sum += (strand[15] - strand[0]).z;
                    n += 1;
                }
            }
            let mean = sum / n.max(1) as f32;
            match growth {
                Growth::Forward => assert!(mean > 0.01, "{mean}"),
                _ => assert!(mean < -0.01, "{mean}"),
            }
        }
    }

    #[test]
    fn long_hair_hangs_short_hair_stands() {
        let straight_down = |d: Vec3| d.dot(Vec3::down());
        let long = cantilever(Vec3::right(), Vec3::down(), 0.4, 0.05, 0.002);
        let short = cantilever(Vec3::right(), Vec3::down(), 0.02, 0.05, 0.002);
        assert!(straight_down(long) > 0.9);
        assert!(straight_down(short) < 0.01);
    }
}
