//! Measuring a body mesh into the measurements a pattern is drawn from.
//!
//! A GarmentCode pattern is made to measure: every panel's size comes from a
//! [`Body`], the twenty-six numbers a tailor would take. The bundled bodies
//! are averages, so drawing a pattern from one and then putting it on somebody
//! else's body is a garment made for a different person -- which is what
//! scaling a body model to the pattern's height amounts to. This module goes
//! the other way: it takes the measurements off the mesh, so the pattern is
//! drawn for the body it will be worn by.
//!
//! # What the mesh has to be
//!
//! * A closed triangle mesh of a standing body, in **centimetres**, the unit
//!   GarmentCode measures in.
//! * `+y` up, `+x` to the body's **left**, `+z` in **front** of it. The floor
//!   is wherever the lowest vertex is; nothing has to be centred.
//! * Standing, legs apart enough to be separate below the crotch, arms away
//!   from the body far enough to be separate below the armpit. Any A-pose or
//!   T-pose qualifies; arms straight down the sides do not, because the armpit
//!   is found by looking for where the arms leave the torso.
//!
//! # What the caller has to supply
//!
//! Six pairs of joint positions, in [`Landmarks`]. Everything else is taken
//! off the surface. The joints are needed because some measurements are about
//! the skeleton rather than the skin -- the arm's length, the shoulder line a
//! sleeve is aligned to, the angle the arm is held at -- and because they say
//! where to look for the rest: which cut is a thigh and which is a calf.
//!
//! # How the levels are found
//!
//! Nothing is at a fixed fraction of the height. Each level is the answer to a
//! question about the surface, so a short broad body and a tall narrow one are
//! each measured at their own waist rather than at the average person's:
//!
//! * **crotch** -- the highest cut at which the legs are still two rings.
//! * **hips** -- the widest cut between the crotch and the waist.
//! * **waist** -- the narrowest cut between the hip joints and the ribs.
//! * **armpit** -- the highest cut at which the arms are still their own rings.
//! * **bust** -- the widest cut between the waist and the armpit.
//! * **underbust** -- the narrowest cut between the waist and the bust.
//! * **neck base** -- the lowest cut above the shoulders that is one ring and
//!   narrower than half the shoulders.
//!
//! The one measurement that is not taken off the surface is `head_l`, the nape
//! to the top of the head, which is Euclidean because GarmentCode's own
//! documentation says so.

use anyhow::{Result, bail};

use crate::Body;

mod section;

pub use section::BodyMesh;
use section::{Axes, Ring, back_width, cross_section, flatten, flatten_on_plane, hull, perimeter};

/// The joints a measurement needs from whichever rig posed the mesh.
///
/// Each pair is `[left, right]`, in the same centimetre frame as the mesh.
/// Which is which only matters for the sign of `x`; every measurement that
/// uses a pair either averages them or takes one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Landmarks {
    /// The shoulder joints -- where the upper arm turns in its socket. The
    /// line between them is `shoulder_w`, which is what a sleeve is aligned
    /// to.
    pub shoulders: [[f32; 3]; 2],
    pub elbows: [[f32; 3]; 2],
    pub wrists: [[f32; 3]; 2],
    /// The hip joints -- where the thigh turns. Used to say where a thigh is,
    /// and as the bottom of the range the waist is looked for in.
    pub hips: [[f32; 3]; 2],
    pub knees: [[f32; 3]; 2],
    /// The base of the neck, roughly. Only used to say where to start looking
    /// for the neck on the surface, so it does not have to be exact.
    pub neck: [f32; 3],
}

/// How finely the body is scanned when a level is looked for, as a fraction of
/// the body's height.
///
/// A body is scanned a few hundred times over, so this is the cost of a
/// measurement; two thousandths of a height is about 3 mm on an adult, which
/// is finer than the levels themselves are defined.
const SCAN_STEP: f32 = 0.002;

/// Take the measurements off a body mesh.
pub fn measure(mesh: BodyMesh<'_>, landmarks: &Landmarks) -> Result<Body> {
    if mesh.vertices.is_empty() || mesh.faces.is_empty() {
        bail!("the body mesh is empty");
    }

    let measurer = Measurer::new(mesh, landmarks)?;
    measurer.body()
}

struct Measurer<'a> {
    mesh: BodyMesh<'a>,
    marks: Landmarks,
    floor: f32,
    top: f32,
    /// The body's own middle, side to side and front to back, from the torso
    /// rather than from the whole mesh: outstretched arms would drag the
    /// average sideways and a foot forwards.
    axis: [f32; 2],
    step: f32,
}

/// A level found on the body, with the section that was found there.
struct Level {
    y: f32,
    girth: f32,
    hull: Vec<[f32; 2]>,
    ring: Ring,
}

impl<'a> Measurer<'a> {
    fn new(mesh: BodyMesh<'a>, landmarks: &Landmarks) -> Result<Self> {
        let mut floor = f32::INFINITY;
        let mut top = f32::NEG_INFINITY;
        for vertex in mesh.vertices {
            floor = floor.min(vertex[1]);
            top = top.max(vertex[1]);
        }
        // GarmentCode measures in centimetres, and a mesh in metres is the
        // mistake this will be made with: it would measure without complaint
        // and produce a pattern for a doll.
        let height = top - floor;
        if !height.is_finite() || height <= 50.0 {
            bail!(
                "the body mesh is {:.3} tall; measurements are in centimetres,                  so a standing body should be over 50",
                height
            );
        }

        // The hip joints are inside the pelvis, so their middle is the body's
        // axis whatever the arms and feet are doing.
        let hips = landmarks.hips;
        let axis = [
            (hips[0][0] + hips[1][0]) / 2.0,
            (hips[0][2] + hips[1][2]) / 2.0,
        ];

        Ok(Self {
            mesh,
            marks: *landmarks,
            floor,
            top,
            axis,
            step: (top - floor) * SCAN_STEP,
        })
    }

    fn height(&self) -> f32 {
        self.top - self.floor
    }

    /// The rings of a horizontal cut.
    fn cut(&self, y: f32) -> Vec<Ring> {
        cross_section(self.mesh, [0.0, y, 0.0], [0.0, 1.0, 0.0])
    }

    /// The torso's ring at a height, and its girth.
    fn level(&self, y: f32) -> Option<Level> {
        let rings = self.cut(y);
        let ring = section::ring_at(&rings, Axes::Horizontal, self.axis)?;
        let outline = hull(&flatten(ring, Axes::Horizontal));
        Some(Level {
            y,
            girth: perimeter(&outline),
            hull: outline,
            ring: ring.clone(),
        })
    }

    /// Scan a range of heights and keep the level that best answers `better`.
    fn scan(&self, from: f32, to: f32, better: fn(f32, f32) -> bool) -> Option<Level> {
        let (from, to) = (from.min(to), from.max(to));
        let mut best: Option<Level> = None;
        let mut y = from;
        while y <= to {
            if let Some(level) = self.level(y)
                && level.girth > 0.0
                && best.as_ref().is_none_or(|b| better(level.girth, b.girth))
            {
                best = Some(level);
            }
            y += self.step;
        }
        best
    }

    /// How many rings a horizontal cut makes, ignoring specks.
    ///
    /// A cut can catch a sliver of surface where it grazes tangentially --
    /// the top of a shoulder, the outside of a heel -- and counting those as
    /// limbs would put the armpit at the wrong height. A ring smaller than a
    /// fiftieth of the body's height around is one of those.
    fn ring_count(&self, y: f32) -> usize {
        let floor = self.height() / 50.0;
        self.cut(y)
            .iter()
            .filter(|ring| perimeter(&hull(&flatten(ring, Axes::Horizontal))) > floor)
            .count()
    }

    /// Scan down from `from` for the highest cut at which the arms have come
    /// away from the torso -- three rings rather than one.
    fn split_below(&self, from: f32, to: f32) -> Option<f32> {
        let mut y = from;
        while y > to {
            if self.ring_count(y) >= 3 {
                return Some(y);
            }
            y -= self.step;
        }
        None
    }

    fn body(&self) -> Result<Body> {
        let height = self.height();
        let hip_joint = (self.marks.hips[0][1] + self.marks.hips[1][1]) / 2.0;
        let shoulder_joint = (self.marks.shoulders[0][1] + self.marks.shoulders[1][1]) / 2.0;

        // --- the levels, from the bottom up.

        // Below the crotch there are two legs, above it one torso. The scan
        // starts at the hip joints, which are always above the crotch, and the
        // first cut that finds two rings has passed it.
        let crotch_y = self
            .split_two_legs(hip_joint)
            .unwrap_or(self.floor + height * 0.46);

        // The waist is the narrowest place between the ribs and the hips. The
        // range is generous: from the hip joints themselves up to a fifth of
        // the body above them, which covers a long torso and a short one.
        let waist = self
            .scan(
                hip_joint + height * 0.01,
                hip_joint + height * 0.18,
                |a, b| a < b,
            )
            .ok_or_else(|| anyhow::anyhow!("no waist: the torso has no closed section"))?;

        // The hips are the widest place between the crotch and the waist.
        let hips = self
            .scan(crotch_y + height * 0.01, waist.y - height * 0.02, |a, b| {
                a > b
            })
            .ok_or_else(|| anyhow::anyhow!("no hipline between the crotch and the waist"))?;

        // Below the armpit the arms are their own rings; above it they have
        // merged into the shoulders.
        let armpit_y = self
            .split_below(shoulder_joint, waist.y)
            .unwrap_or(shoulder_joint - height * 0.07);

        let bust = self
            .scan(waist.y + height * 0.02, armpit_y - height * 0.01, |a, b| {
                a > b
            })
            .ok_or_else(|| anyhow::anyhow!("no bustline between the waist and the armpit"))?;
        let underbust = self.underbust(&bust, &waist);

        let neck = self.neck_base(shoulder_joint)?;

        // --- the sagittal profile, for the tape measurements down the body.

        // The profile is hulled over a band a little taller than the tape
        // that will run down it. A hull closes across the top and the bottom
        // of whatever it is given, and those chords lie across the body rather
        // than down it; cutting the band flush with the tape leaves them at
        // its ends, and the depth of the body gets added to a length measured
        // down it. Keeping them outside the tape's own range is what stops
        // that, and it costs nothing -- a tape does not stop abruptly either.
        let margin = height * 0.04;
        let profile = self.profile(self.axis[0], waist.y - margin, self.top);
        let nape = self.nape(&profile, neck.y);
        let head_l = distance3(nape, self.crown());

        let waist_line = self.tape(&profile, waist.y, neck.y, Side::Back);
        let bust_points = self.peak_separation(&bust.ring, Side::Front);
        let bum_points = self.peak_separation(&hips.ring, Side::Back);

        // The line over the bust is taken in the plane through a bust point,
        // not down the middle: that is where a tape measure would run.
        let bust_plane = if bust_points > 1.0 {
            self.axis[0] + bust_points / 2.0
        } else {
            self.axis[0]
        };
        let bust_profile = self.profile(bust_plane, waist.y - margin, neck.y + margin);
        let waist_over_bust_line = self.tape(&bust_profile, waist.y, neck.y, Side::Front);
        let bust_line = self.tape(&bust_profile, bust.y, neck.y, Side::Front);

        // --- the limbs.

        let arm_length = distance3(self.marks.shoulders[0], self.marks.elbows[0])
            + distance3(self.marks.elbows[0], self.marks.wrists[0]);
        let arm_pose_angle = self.arm_pose();
        // The thigh is measured below the crotch. Above it the two legs are
        // one pelvis, and the widest cut along the bone would be the hips --
        // which is how a thigh comes out a metre around.
        let hip_to_knee = self.marks.hips[0][1] - self.marks.knees[0][1];
        let below_crotch = if hip_to_knee > 1.0 {
            ((self.marks.hips[0][1] - crotch_y + height * 0.01) / hip_to_knee).clamp(0.05, 0.5)
        } else {
            0.10
        };
        let leg_circ = self.limb_girth(
            self.marks.hips[0],
            self.marks.knees[0],
            below_crotch,
            (below_crotch + 0.45).min(0.75),
            |a, b| a > b,
        );
        let wrist = self.limb_girth(
            self.marks.elbows[0],
            self.marks.wrists[0],
            0.80,
            1.00,
            |a, b| a < b,
        );

        // --- the shoulders.

        let shoulder_w = distance3(self.marks.shoulders[0], self.marks.shoulders[1]);
        let neck_w = back_width(&neck.hull);
        let shoulder_incl = self.shoulder_inclination(neck_w, shoulder_w, armpit_y);
        let shoulder_top = self.shoulder_top(shoulder_w, armpit_y);

        let mut body = Body::default();
        let mut set = |name: &str, value: f32| body.insert(name, value as f64);

        set("height", height);
        set("head_l", head_l);

        set("bust", bust.girth);
        set("underbust", underbust.girth);
        set("waist", waist.girth);
        set("hips", hips.girth);
        set("leg_circ", leg_circ);
        set("wrist", wrist);

        set("back_width", back_width(&bust.hull));
        set("waist_back_width", back_width(&waist.hull));
        set("hip_back_width", back_width(&hips.hull));
        set("neck_w", neck_w);

        set("shoulder_w", shoulder_w);
        set("shoulder_incl", shoulder_incl);
        set("armscye_depth", shoulder_top - armpit_y);
        set("arm_length", arm_length);
        set("arm_pose_angle", arm_pose_angle);

        set("bust_points", bust_points);
        set("bum_points", bum_points);
        set("waist_line", waist_line);
        set("waist_over_bust_line", waist_over_bust_line);
        set("bust_line", bust_line);
        set("vert_bust_line", nape[1] - bust.y);
        set("hips_line", waist.y - hips.y);
        set("crotch_hip_diff", hips.y - crotch_y);
        set("hip_inclination", self.hip_inclination(&waist, &hips));

        body.eval_dependencies();
        Ok(body)
    }

    /// The girth right below the bust.
    ///
    /// Not an extremum of the body the way the other levels are: from the
    /// waist upwards a body gets steadily wider, so the narrowest cut below
    /// the bust is the waist itself. What marks the underbust is the *dip*
    /// under a bust that overhangs -- so the scan runs down from the bust and
    /// stops at the first cut that is narrower than the one below it.
    ///
    /// A body with no such dip has no underbust to find, and the fallback is
    /// stated rather than measured: two fifths of the way from the waist up to
    /// the bust, which is where it sits on the bodies GarmentCode ships.
    fn underbust(&self, bust: &Level, waist: &Level) -> Level {
        let mut y = bust.y - self.step;
        let mut last = bust.girth;
        while y > waist.y + self.step {
            let Some(level) = self.level(y) else { break };
            if level.girth > last {
                return level;
            }
            last = level.girth;
            y -= self.step;
        }

        let fallback = waist.y + (bust.y - waist.y) * 0.4;
        self.level(fallback).unwrap_or(Level {
            y: waist.y,
            girth: waist.girth,
            hull: waist.hull.clone(),
            ring: waist.ring.clone(),
        })
    }

    /// Scan down from the hip joints for the crotch: the highest cut at which
    /// the legs are two rings rather than one pelvis.
    fn split_two_legs(&self, from: f32) -> Option<f32> {
        let mut y = from;
        let bottom = self.floor + self.height() * 0.30;
        while y > bottom {
            if self.ring_count(y) >= 2 {
                return Some(y);
            }
            y -= self.step;
        }
        None
    }

    /// The base of the neck: where the neck column meets the shoulders.
    ///
    /// Found in two passes, because neither alone gets it. Rising from the
    /// shoulders the body narrows steeply through the trapezius and then
    /// hardly at all up the neck, so the first pass finds the narrowest cut --
    /// which is unambiguously *on* the neck, but partway up it. The second
    /// comes back down to the lowest cut still within a sixth of that width,
    /// which is where the neck stops being a column and starts being a
    /// shoulder. Stopping at the first cut under some fraction of the
    /// shoulders would land wherever on the slope that fraction happened to
    /// fall.
    fn neck_base(&self, shoulder_joint: f32) -> Result<Level> {
        let ceiling = self.marks.neck[1] + self.height() * 0.06;
        let mut narrowest: Option<Level> = None;
        let mut y = shoulder_joint;
        while y < ceiling {
            if self.ring_count(y) == 1
                && let Some(level) = self.level(y)
                && narrowest
                    .as_ref()
                    .is_none_or(|best| span(&level.hull, 0) < span(&best.hull, 0))
            {
                narrowest = Some(level);
            }
            y += self.step;
        }
        let narrowest = narrowest.ok_or_else(|| {
            anyhow::anyhow!("no neck found above the shoulders; is the mesh y-up and standing?")
        })?;

        let column = span(&narrowest.hull, 0) * 7.0 / 6.0;
        let mut y = narrowest.y;
        let mut base = None;
        while y > shoulder_joint {
            let next = y - self.step;
            match self.level(next) {
                Some(level) if self.ring_count(next) == 1 && span(&level.hull, 0) <= column => {
                    base = Some(level);
                    y = next;
                }
                _ => break,
            }
        }
        Ok(base.unwrap_or(narrowest))
    }

    /// The highest point of the body -- the top of the head.
    fn crown(&self) -> [f32; 3] {
        let mut crown = self.mesh.vertices[0];
        for vertex in self.mesh.vertices {
            if vertex[1] > crown[1] {
                crown = *vertex;
            }
        }
        crown
    }

    /// The body's outline seen from the side, cut down the plane `x` and kept
    /// between two heights, as a convex hull in `(z, y)`.
    ///
    /// The band matters. A hull of the whole side of a body runs from the back
    /// of the head straight to the back of the calves, and a tape measured
    /// along it would miss every shape in between. Hulling only the stretch a
    /// measurement covers is what makes the result a tape over that stretch --
    /// over the bust, down the back -- rather than a chord past it.
    fn profile(&self, x: f32, low: f32, high: f32) -> Vec<[f32; 2]> {
        let rings = cross_section(self.mesh, [x, 0.0, 0.0], [1.0, 0.0, 0.0]);
        let mut points = Vec::new();
        for ring in &rings {
            points.extend(
                flatten(ring, Axes::Sagittal)
                    .into_iter()
                    .filter(|point| point[1] >= low && point[1] <= high),
            );
        }
        hull(&points)
    }

    /// The nape: the back of the neck, where a tape measure starts.
    fn nape(&self, profile: &[[f32; 2]], neck_y: f32) -> [f32; 3] {
        let mut back = f32::INFINITY;
        for point in profile {
            if (point[1] - neck_y).abs() < self.height() * 0.02 {
                back = back.min(point[0]);
            }
        }
        if !back.is_finite() {
            back = self.axis[1];
        }
        [self.axis[0], neck_y, back]
    }

    /// The length of a tape run down one side of the body's profile, between
    /// two heights.
    ///
    /// The profile is already a convex hull, so this follows what a tape
    /// measure would: over the bust rather than into the hollow beneath it,
    /// down the back rather than into the small of it.
    fn tape(&self, profile: &[[f32; 2]], from_y: f32, to_y: f32, side: Side) -> f32 {
        // The band is clamped to the profile's own height first. A convex hull
        // is closed across its top and its bottom, and those two chords lie
        // across the body rather than down it -- a tape does not run over the
        // top of a shoulder from the front to the back. Asking for a band that
        // reaches even a millimetre past the top of the profile leaves the top
        // chord inside it, and the width of the body gets added to a length
        // measured down it.
        let mut low = f32::INFINITY;
        let mut high = f32::NEG_INFINITY;
        for point in profile {
            low = low.min(point[1]);
            high = high.max(point[1]);
        }
        let low = low.max(from_y.min(to_y));
        let high = high.min(from_y.max(to_y));
        if high <= low || !(high - low).is_finite() {
            return 0.0;
        }
        let middle = section::centroid(profile);
        let mut total = 0.0;
        for index in 0..profile.len() {
            let a = profile[index];
            let b = profile[(index + 1) % profile.len()];
            let facing = (a[0] + b[0]) / 2.0;
            let wanted = match side {
                Side::Front => facing > middle[0],
                Side::Back => facing < middle[0],
            };
            if !wanted {
                continue;
            }
            let Some((a, b)) = clip_to_band(a, b, low, high) else {
                continue;
            };
            total += ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt();
        }
        total
    }

    /// The distance between the two peaks of a section -- the bust points on
    /// the front, the bum points on the back.
    fn peak_separation(&self, ring: &Ring, side: Side) -> f32 {
        let flat = flatten(ring, Axes::Horizontal);
        let middle = section::centroid(&flat);
        let mut peaks = [None::<[f32; 2]>; 2];
        for point in &flat {
            let ahead = match side {
                Side::Front => point[1] > middle[1],
                Side::Back => point[1] < middle[1],
            };
            if !ahead {
                continue;
            }
            let half = usize::from(point[0] < middle[0]);
            let better = match (peaks[half], side) {
                (None, _) => true,
                (Some(best), Side::Front) => point[1] > best[1],
                (Some(best), Side::Back) => point[1] < best[1],
            };
            if better {
                peaks[half] = Some(*point);
            }
        }
        match peaks {
            [Some(a), Some(b)] => ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt(),
            _ => 0.0,
        }
    }

    /// The girth of a limb, scanned along the bone between two joints.
    ///
    /// The cuts are square to the bone rather than to the floor. A thigh is
    /// close enough to vertical that it hardly matters, but a forearm held out
    /// at an angle cut horizontally gives an ellipse, and its wrist would come
    /// out several centimetres too big.
    fn limb_girth(
        &self,
        from: [f32; 3],
        to: [f32; 3],
        start: f32,
        end: f32,
        better: fn(f32, f32) -> bool,
    ) -> f32 {
        let axis = [to[0] - from[0], to[1] - from[1], to[2] - from[2]];
        let length = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
        if length < 1e-3 {
            return 0.0;
        }
        let normal = [axis[0] / length, axis[1] / length, axis[2] / length];

        let mut best = 0.0f32;
        let steps = 24;
        for step in 0..=steps {
            let t = start + (end - start) * step as f32 / steps as f32;
            let at = [
                from[0] + axis[0] * t,
                from[1] + axis[1] * t,
                from[2] + axis[2] * t,
            ];
            let rings = cross_section(self.mesh, at, normal);
            let mut nearest: Option<(f32, f32)> = None;
            for ring in &rings {
                let flat = flatten_on_plane(ring, normal);
                let girth = perimeter(&hull(&flat));
                if girth <= 0.0 {
                    continue;
                }
                // The ring around the limb is the one whose middle is nearest
                // the bone; a cut across a thigh also catches the other one.
                let middle = section::centroid(&flat);
                let bone = flatten_on_plane(&[at], normal)[0];
                let distance = (middle[0] - bone[0]).powi(2) + (middle[1] - bone[1]).powi(2);
                if nearest.is_none_or(|(best, _)| distance < best) {
                    nearest = Some((distance, girth));
                }
            }
            let Some((_, girth)) = nearest else { continue };
            if best == 0.0 || better(girth, best) {
                best = girth;
            }
        }
        best
    }

    /// The angle the upper arm is held at, below the horizontal.
    ///
    /// GarmentCode rotates a sleeve by this, so a T-pose is zero and an A-pose
    /// held at forty degrees is forty.
    fn arm_pose(&self) -> f32 {
        let mut total = 0.0;
        for side in 0..2 {
            let shoulder = self.marks.shoulders[side];
            let elbow = self.marks.elbows[side];
            let out = ((elbow[0] - shoulder[0]).powi(2) + (elbow[2] - shoulder[2]).powi(2)).sqrt();
            let down = shoulder[1] - elbow[1];
            total += down.atan2(out).to_degrees();
        }
        total / 2.0
    }

    /// The top of the shoulder, above the shoulder joint.
    fn shoulder_top(&self, shoulder_w: f32, armpit_y: f32) -> f32 {
        let band = self.height() * 0.02;
        let target = shoulder_w / 2.0;
        let mut top = armpit_y;
        for vertex in self.mesh.vertices {
            if ((vertex[0] - self.axis[0]).abs() - target).abs() < band && vertex[1] > armpit_y {
                top = top.max(vertex[1]);
            }
        }
        top
    }

    /// The slope of the shoulder line, in degrees below the horizontal.
    ///
    /// Fitted across the whole of the outer shoulder rather than taken between
    /// two points: the surface there is a smooth fall from the neck to the
    /// joint, and a pair of points on it is at the mercy of wherever the mesh
    /// happened to put a vertex.
    fn shoulder_inclination(&self, neck_w: f32, shoulder_w: f32, armpit_y: f32) -> f32 {
        let inner = neck_w / 2.0;
        let outer = shoulder_w / 2.0;
        if outer <= inner + 1.0 || !(outer - inner).is_finite() {
            return 0.0;
        }
        // Only the band of the body around the shoulder joints: further front
        // is the collarbone and further back the shoulder blade, and both fall
        // away from the line a garment sits on.
        let plane = (self.marks.shoulders[0][2] + self.marks.shoulders[1][2]) / 2.0;
        let depth = self.height() * 0.05;
        // A sagittal cut near the neck also intersects the head. Its top is
        // not a shoulder sample; including it makes narrow-necked bodies
        // report a shoulder slope of sixty degrees or more.
        let shoulder_ceiling = self.marks.neck[1] + self.height() * 0.04;

        // Sampled by cutting rather than by sorting the vertices into bins.
        // The stretch between the neck and the shoulder is only a few
        // centimetres wide, and on a body model at a working level of detail
        // most bins across it hold no vertex at all -- which is how a shoulder
        // ends up with no measurable slope. A cut has a top wherever it is
        // taken.
        const SAMPLES: usize = 8;
        let mut points: Vec<(f32, f32)> = Vec::with_capacity(SAMPLES);
        for index in 0..SAMPLES {
            let across = inner + (outer - inner) * (index as f32 + 0.5) / SAMPLES as f32;
            let mut top = f32::NEG_INFINITY;
            for sign in [1.0f32, -1.0] {
                let x = self.axis[0] + sign * across;
                for ring in cross_section(self.mesh, [x, 0.0, 0.0], [1.0, 0.0, 0.0]) {
                    for point in ring {
                        if point[1] > armpit_y
                            && point[1] <= shoulder_ceiling
                            && (point[2] - plane).abs() <= depth
                        {
                            top = top.max(point[1]);
                        }
                    }
                }
            }
            if top.is_finite() {
                points.push((across, top));
            }
        }
        let samples = points;
        if samples.len() < 3 {
            return 0.0;
        }

        let count = samples.len() as f32;
        let mean_x = samples.iter().map(|s| s.0).sum::<f32>() / count;
        let mean_y = samples.iter().map(|s| s.1).sum::<f32>() / count;
        let covariance: f32 = samples
            .iter()
            .map(|(x, y)| (x - mean_x) * (y - mean_y))
            .sum();
        let variance: f32 = samples.iter().map(|(x, _)| (x - mean_x).powi(2)).sum();
        if variance <= 0.0 {
            return 0.0;
        }
        (-covariance / variance).atan().to_degrees().max(0.0)
    }

    /// The angle of the body's side between the waist and the hips, from the
    /// vertical.
    fn hip_inclination(&self, waist: &Level, hips: &Level) -> f32 {
        let drop = waist.y - hips.y;
        if drop <= 0.0 {
            return 0.0;
        }
        let out = span(&hips.hull, 0) / 2.0 - span(&waist.hull, 0) / 2.0;
        out.atan2(drop).to_degrees().max(0.0)
    }
}

/// Which face of the body a tape runs down.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Side {
    Front,
    Back,
}

/// The extent of a polygon along one of its axes.
fn span(polygon: &[[f32; 2]], axis: usize) -> f32 {
    let mut low = f32::INFINITY;
    let mut high = f32::NEG_INFINITY;
    for point in polygon {
        low = low.min(point[axis]);
        high = high.max(point[axis]);
    }
    if high > low { high - low } else { 0.0 }
}

fn distance3(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2)).sqrt()
}

/// Trim a `(z, y)` segment to the part of it between two heights.
fn clip_to_band(a: [f32; 2], b: [f32; 2], low: f32, high: f32) -> Option<([f32; 2], [f32; 2])> {
    let (mut a, mut b) = (a, b);
    if a[1] > b[1] {
        std::mem::swap(&mut a, &mut b);
    }
    if b[1] <= low || a[1] >= high {
        return None;
    }
    let at = |y: f32| {
        let t = (y - a[1]) / (b[1] - a[1]);
        [a[0] + (b[0] - a[0]) * t, y]
    };
    let start = if a[1] < low { at(low) } else { a };
    let end = if b[1] > high { at(high) } else { b };
    Some((start, end))
}

#[cfg(test)]
mod tests;
