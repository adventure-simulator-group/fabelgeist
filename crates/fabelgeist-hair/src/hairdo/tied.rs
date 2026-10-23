//! Geometry of tails, buns and braids. Native vectors are metres in the head frame.
use super::{Flow, Hairdo, Tie, cantilever, safe_normalize, smoothstep, tangent};
use crate::head::Head;
use fabelgeist_math::Vec3;

/// Where a tied style's strands meet, and the path the tie hangs along.
pub(crate) struct Tied {
    kind: Tie,
    /// The tie, off the scalp.
    pub(crate) point: Vec3,
    /// Out of the head at the tie.
    pub(crate) out: Vec3,
    /// The tail's axis from the tie, a polyline two millimetres a step.
    pub(crate) axis: Vec<Vec3>,
}

impl Tied {
    pub(crate) fn new(head: &Head, hairdo: &Hairdo) -> Option<Self> {
        let Flow::Tie(kind) = hairdo.flow() else {
            return None;
        };
        let c = head.cranium.center;
        let r = head.cranium.radii;
        // Up the back of the head: high for a tail, on the crown's back for
        // a bun, low at the nape for a braid.
        let a = hairdo.hairstyle.tie;
        let on_scalp = c + Vec3::new(0.0, r.y * a.sin(), -r.z * a.cos());
        let out = head.gradient(on_scalp);
        let point = on_scalp + out * (0.012 - head.distance(on_scalp));
        // The tail leaves the tie out of the head and bends down as a
        // cantilever -- stiffer than one strand: it is a bundle.
        let mut axis = vec![point];
        if !matches!(kind, Tie::Bun) {
            // As long as the longest hair: clippers leave the back short.
            let length = hairdo.haircut.longest();
            let stiffness = hairdo.hairstyle.hold * 2.0;
            let mut p = point;
            let mut d = safe_normalize(out + Vec3::down() * 0.4, Vec3::down());
            let ds = 0.002;
            let mut s = 0.0;
            while s < length {
                d = cantilever(d, Vec3::down(), length - s, stiffness, ds);
                let mut next = p + d * ds;
                let distance = head.distance(next);
                if distance < 0.02 {
                    next += head.gradient(next) * (0.02 - distance);
                    next = p + safe_normalize(next - p, d) * ds;
                }
                d = safe_normalize(next - p, d);
                p = next;
                axis.push(p);
                s += ds;
            }
        }
        if axis.len() < 2 {
            axis.push(point + out * 0.002);
        }
        Some(Self {
            kind,
            point,
            out,
            axis,
        })
    }

    /// The axis at arc length `s` from the tie, its direction and a frame
    /// across it (the second axis points out of the head).
    fn frame(&self, s: f32) -> (Vec3, Vec3, Vec3, Vec3) {
        let f = (s / 0.002).max(0.0);
        let i = (f as usize).min(self.axis.len().saturating_sub(2));
        let t = (f - i as f32).clamp(0.0, 1.0);
        let p = self.axis[i].lerp(self.axis[i + 1], t);
        let d = safe_normalize(self.axis[i + 1] - self.axis[i], Vec3::down());
        let across = safe_normalize(d.cross(self.out), Vec3::right());
        let depth = across.cross(d);
        (p, d, across, depth)
    }

    /// Where a strand is `along` metres past the tie: in the tail, coiled
    /// in the bun or plaited in its lane of the braid, `bundle` off the
    /// middle.
    pub(super) fn bundled(&self, along: f32, bundle: Vec3, lane: f32, phase: f32) -> Vec3 {
        use std::f32::consts::TAU;
        match self.kind {
            Tie::Tail => {
                let (c, _, across, depth) = self.frame(along);
                let spread = 1.0 + 0.6 * smoothstep(0.1, 0.4, along);
                c + (across * bundle.x + depth * bundle.y) * spread
            }
            Tie::Braid => {
                let (c, _, across, depth) = self.frame(along);
                let phase = along / 0.035 * TAU + lane * TAU / 3.0;
                let width = 0.016 * (1.0 - 0.45 * smoothstep(0.0, 0.45, along));
                let taper = 1.0 - smoothstep(0.38, 0.5, along);
                c + across * (phase.sin() * width * taper + bundle.x * 0.6)
                    + depth * ((2.0 * phase).sin() * width * 0.35 * taper + bundle.y * 0.6)
            }
            Tie::Bun => {
                // Coiled round the tie, out from the head as it winds.
                let turns = along / 0.11;
                let a = phase + turns * TAU;
                let r = 0.018 + 0.01 * (turns * 0.5).min(1.0) + bundle.x;
                let u = safe_normalize(tangent(Vec3::up(), self.out), Vec3::up());
                let v = self.out.cross(u);
                self.point
                    + (u * a.cos() + v * a.sin()) * r
                    + self.out * (0.004 + 0.012 * (turns * 0.4).min(1.0) + bundle.y)
            }
        }
    }
}
