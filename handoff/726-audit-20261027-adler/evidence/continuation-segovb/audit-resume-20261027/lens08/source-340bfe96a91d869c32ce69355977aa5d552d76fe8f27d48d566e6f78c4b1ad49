//! Follicles: where hair grows, which way it grows, and what it grows as.
//!
//! Hair does not leave the scalp straight out. Each follicle is slanted, and
//! over the scalp the slants line up into a *growth pattern*: in most people
//! a whorl at the crown -- a spiral, usually clockwise seen from above --
//! from which hair runs forward over the top, down the sides and down the
//! back. Some have two whorls, a cowlick at the front hairline, or hair
//! that grows forward or back all over. Short hair shows the pattern as it
//! is; longer hair is combed into a style, but near the root it keeps its
//! follicle's direction for a centimetre or so (a cowlick stands up where
//! the comb goes against it).
//!
//! [`Follicles`] holds the pattern ([`Growth`], placed and shaped by the
//! whorl's position, spiral and handedness) and what else the follicles
//! decide: the angle hair leaves the scalp at and how much it varies, how
//! far the follicle's direction holds against the comb, how coily hair
//! wanders, how thick it is, how evenly it covers the scalp, the hairline's
//! shape and softness, and pattern loss. [`GrowthField`] is the pattern
//! placed on a head: a direction at every point of the scalp, for the guides
//! ([`crate::hairdo`]) and for every strand ([`crate::ops::Orient`]).

use fabelgeist_math::Vec3;
use serde::{Deserialize, Serialize};

use crate::head::Head;
use crate::ops::smoothstep;
use crate::strands::safe_normalize;

/// How the follicles are slanted over the scalp.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Growth {
    /// One whorl at the crown, the most common: hair spirals out of it,
    /// forward over the top, down the sides and the back.
    Whorl,
    /// Two whorls side by side at the crown, turning opposite ways, a ridge
    /// between them.
    DoubleWhorl,
    /// The crown whorl, and a second small one at the front hairline that
    /// lifts the hair there.
    Cowlick,
    /// Straight out from the whorl, without turning.
    Radial,
    /// Forward over the top, towards the face; down at the sides and back.
    Forward,
    /// Back from the face over the top; down at the sides and back.
    Backward,
    /// Down all round, parting naturally at the top.
    Down,
}

impl Growth {
    pub const ALL: [Growth; 7] = [
        Growth::Whorl,
        Growth::DoubleWhorl,
        Growth::Cowlick,
        Growth::Radial,
        Growth::Forward,
        Growth::Backward,
        Growth::Down,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Growth::Whorl => "Crown whorl",
            Growth::DoubleWhorl => "Double whorl",
            Growth::Cowlick => "Whorl and cowlick",
            Growth::Radial => "Radial",
            Growth::Forward => "Forward",
            Growth::Backward => "Backward",
            Growth::Down => "Down",
        }
    }

    /// Whether the pattern has a whorl to place and turn.
    pub fn has_whorl(self) -> bool {
        matches!(
            self,
            Growth::Whorl | Growth::DoubleWhorl | Growth::Cowlick | Growth::Radial
        )
    }
}

/// The hairline's shape, as changes to the head's own.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Hairline {
    /// Raises the hairline (lowers, negative), metres: at the forehead,
    /// over the ears and at the nape.
    pub front: f32,
    pub sides: f32,
    pub nape: f32,
    /// How far the temples recede, metres.
    pub recession: f32,
    /// How far a widow's peak dips in the middle, metres.
    pub peak: f32,
    /// Over how much of the scalp's edge hair thins out to nothing, and
    /// grows finer, metres: 0 is a sharp edge.
    pub softness: f32,
}

/// What the follicles decide.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Follicles {
    pub growth: Growth,
    /// The crown whorl: how far round towards `+x` from the top, and how
    /// far back from the top, radians.
    pub whorl: [f32; 2],
    /// How far hair turns about the whorl as it leaves it, radians.
    pub spiral: f32,
    /// Turning clockwise seen from above (most people's).
    pub clockwise: bool,
    /// The angle hair leaves the scalp at, radians: 0 flat, pi/2 upright.
    pub angle: f32,
    /// How much each follicle's angle and direction stray, radians.
    pub angle_spread: f32,
    pub direction_spread: f32,
    /// How far along a strand its follicle's direction holds against the
    /// comb, metres.
    pub memory: f32,
    /// How much coily hair wanders from its course, radians a root metre.
    pub kink: f32,
    /// Root radius, metres, and how much it varies either way.
    pub radius: f32,
    pub radius_spread: f32,
    /// How unevenly the follicles cover the scalp, 0 to 1: sparser and
    /// denser patches.
    pub patchiness: f32,
    /// Pattern loss, 0 to 1 each: the crown, the temples, the top.
    pub balding: [f32; 3],
    pub hairline: Hairline,
}

impl Default for Follicles {
    fn default() -> Self {
        Self {
            growth: Growth::Whorl,
            whorl: [0.12, 0.55],
            spiral: 1.1,
            clockwise: true,
            angle: 0.35,
            angle_spread: 0.12,
            direction_spread: 0.15,
            memory: 0.008,
            kink: 0.0,
            radius: 35e-6,
            radius_spread: 0.25,
            patchiness: 0.0,
            balding: [0.0; 3],
            hairline: Hairline {
                softness: 0.004,
                ..Default::default()
            },
        }
    }
}

impl Follicles {
    /// `base` with this hairline.
    pub fn head(&self, base: &Head) -> Head {
        let mut head = base.clone();
        let h = &self.hairline;
        head.hairline[0] += h.front;
        head.hairline[1] += h.sides;
        head.hairline[2] += h.nape;
        head.recession += h.recession;
        head.peak += h.peak;
        head
    }

    /// The growth pattern placed on `head`.
    pub fn field(&self, head: &Head) -> GrowthField {
        let hand = if self.clockwise { -1.0 } else { 1.0 };
        let [side, back] = self.whorl;
        let crown = on_cranium(head, side, back);
        let whorls = match self.growth {
            Growth::Whorl => vec![Whorl::new(crown, self.spiral, hand, 0.07)],
            Growth::Radial => vec![Whorl::new(crown, 0.0, hand, 0.07)],
            Growth::DoubleWhorl => vec![
                Whorl::new(on_cranium(head, side - 0.2, back), self.spiral, hand, 0.06),
                Whorl::new(on_cranium(head, side + 0.2, back), self.spiral, -hand, 0.06),
            ],
            Growth::Cowlick => {
                // Just behind the front hairline, a little off the middle.
                let azimuth = 0.25;
                let height = head.hairline_height(azimuth) + 0.012;
                let front = (1..60)
                    .map(|i| on_cranium_front(head, azimuth, i as f32 * 0.025))
                    .min_by(|a, b| (a.y - height).abs().total_cmp(&(b.y - height).abs()))
                    .unwrap_or(crown);
                vec![
                    Whorl::new(crown, self.spiral, hand, 0.07),
                    Whorl::new(front, self.spiral.max(0.6) * 1.4, hand, 0.035),
                ]
            }
            Growth::Forward | Growth::Backward | Growth::Down => Vec::new(),
        };
        GrowthField {
            growth: self.growth,
            center: head.cranium.center,
            whorls,
        }
    }
}

/// The point of `head`'s cranium `side` radians round towards `+x` from
/// the top and `back` radians back.
fn on_cranium(head: &Head, side: f32, back: f32) -> Vec3 {
    let direction = Vec3::new(
        side.sin(),
        back.cos() * side.cos(),
        -back.sin() * side.cos(),
    );
    onto_head(head, direction)
}

/// The point of `head`'s cranium at `azimuth` from the front, `polar`
/// radians down from the top.
fn on_cranium_front(head: &Head, azimuth: f32, polar: f32) -> Vec3 {
    let direction = Vec3::new(
        polar.sin() * azimuth.sin(),
        polar.cos(),
        polar.sin() * azimuth.cos(),
    );
    onto_head(head, direction)
}

fn onto_head(head: &Head, direction: Vec3) -> Vec3 {
    let c = head.cranium;
    let t = 1.0 / (direction / c.radii).length().max(1e-6);
    let mut p = c.center + direction * t;
    for _ in 0..4 {
        p -= head.gradient(p) * head.distance(p);
    }
    p
}

/// One whorl of a growth pattern.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Whorl {
    pub point: Vec3,
    /// How far hair turns about it at its centre, radians.
    pub spiral: f32,
    /// Which way it turns: `1` counter-clockwise seen from outside.
    pub hand: f32,
    /// How far out it turns hair (and, for a second whorl, how far it
    /// reaches), metres.
    pub reach: f32,
}

impl Whorl {
    fn new(point: Vec3, spiral: f32, hand: f32, reach: f32) -> Self {
        Self {
            point,
            spiral,
            hand,
            reach,
        }
    }

    /// Which way hair grows at `p` (normal `n`) for this whorl alone.
    fn direction(&self, p: Vec3, n: Vec3) -> Vec3 {
        let offset = p - self.point;
        let away = tangent(offset, n);
        if away.length_squared() < 1e-10 {
            return safe_normalize(tangent(Vec3::back(), n), Vec3::back());
        }
        let away = away.normalize();
        let around = n.cross(away) * self.hand;
        let turn = self.spiral * (1.0 - smoothstep(0.0, self.reach, offset.length()));
        away * turn.cos() + around * turn.sin()
    }
}

/// A growth pattern placed on a head: which way hair grows anywhere on it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GrowthField {
    pub growth: Growth,
    /// The cranium's centre.
    pub center: Vec3,
    pub whorls: Vec<Whorl>,
}

impl GrowthField {
    /// The direction hair grows at `p` on the scalp, whose normal there is
    /// `n`: a unit vector in the scalp's tangent plane.
    pub fn direction(&self, p: Vec3, n: Vec3) -> Vec3 {
        let down = tangent(Vec3::down(), n);
        let fallback = safe_normalize(down, Vec3::back());
        let v = match (self.growth, self.whorls.as_slice()) {
            (Growth::Whorl | Growth::Radial, [w, ..]) => w.direction(p, n),
            (Growth::DoubleWhorl, [a, b, ..]) => {
                // Each side of the ridge between them follows its own.
                let t = smoothstep(
                    -0.004,
                    0.004,
                    (p - a.point).length() - (p - b.point).length(),
                );
                a.direction(p, n) * (1.0 - t) + b.direction(p, n) * t
            }
            (Growth::Cowlick, [crown, front, ..]) => {
                let t =
                    1.0 - smoothstep(0.5 * front.reach, front.reach, (p - front.point).length());
                crown.direction(p, n) * (1.0 - t) + front.direction(p, n) * t
            }
            (Growth::Forward, _) => {
                let over = smoothstep(0.35, 0.75, n.y);
                tangent(Vec3::new(0.0, -0.15, 1.0), n) * over + down * (1.0 - over)
            }
            (Growth::Backward, _) => {
                let over = smoothstep(0.2, 0.6, n.y + 0.5 * n.z);
                tangent(Vec3::new(0.0, -0.15, -1.0), n) * over + down * (1.0 - over)
            }
            (Growth::Down, _) => {
                // Down, and out from the middle of the top where down is
                // no way at all.
                let q = p - self.center;
                let out = Vec3::new(q.x, 0.0, q.z * 0.3);
                tangent(Vec3::down() + safe_normalize(out, Vec3::right()) * 0.35, n)
            }
            _ => down,
        };
        safe_normalize(tangent(v, n), fallback)
    }
}

/// The component of `v` in the plane with normal `n`.
fn tangent(v: Vec3, n: Vec3) -> Vec3 {
    v - n * v.dot(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn top_normal(head: &Head, p: Vec3) -> Vec3 {
        head.gradient(p)
    }

    #[test]
    fn a_crown_whorl_turns_and_runs_forward_and_down() {
        let head = Head::mannequin();
        let follicles = Follicles::default();
        let field = follicles.field(&head);
        let whorl = field.whorls[0].point;
        assert!(head.distance(whorl).abs() < 1e-3);
        // Near the whorl, hair runs round it; far from it, away from it.
        let near = onto_head(
            &head,
            safe_normalize(
                whorl + Vec3::right() * 0.01 - head.cranium.center,
                Vec3::up(),
            ),
        );
        let n = top_normal(&head, near);
        let d = field.direction(near, n);
        let away = safe_normalize(tangent(near - whorl, n), Vec3::right());
        assert!(d.dot(away).abs() < 0.8, "{}", d.dot(away));
        // The forehead's hair runs forward and down, the nape's down.
        let forehead = onto_head(&head, Vec3::new(0.0, 0.6, 0.8));
        let d = field.direction(forehead, head.gradient(forehead));
        assert!(d.z > 0.2 || d.y < -0.5, "{d}");
        let nape = onto_head(&head, Vec3::new(0.0, -0.3, -0.95));
        assert!(field.direction(nape, head.gradient(nape)).y < -0.7);
    }

    #[test]
    fn every_pattern_gives_a_tangent_direction_everywhere() {
        let head = Head::mannequin();
        let scalp = head.scalp(0.01);
        for growth in Growth::ALL {
            let field = Follicles {
                growth,
                ..Default::default()
            }
            .field(&head);
            for (p, n) in scalp.positions.iter().zip(&scalp.normals) {
                let d = field.direction(*p, *n);
                assert!(d.is_finite(), "{growth:?}");
                assert!((d.length() - 1.0).abs() < 1e-3, "{growth:?}");
                assert!(d.dot(*n).abs() < 1e-3, "{growth:?}");
            }
        }
    }

    #[test]
    fn a_hairline_raised_recedes_and_a_peak_dips() {
        let base = Head::mannequin();
        let follicles = Follicles {
            hairline: Hairline {
                recession: 0.02,
                peak: 0.01,
                ..Default::default()
            },
            ..Default::default()
        };
        let head = follicles.head(&base);
        assert!(head.hairline_height(0.62) > base.hairline_height(0.62) + 0.015);
        assert!(head.hairline_height(0.0) < base.hairline_height(0.0) - 0.008);
    }
}
