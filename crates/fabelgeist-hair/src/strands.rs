//! Strands: the curves every operation reads and writes.
//!
//! Blender keeps hair as a `Curves` geometry: points in curve order, with
//! attributes on the point and on the curve domain. [`Strands`] is the same
//! with one simplification the DiffLocks dataset makes too: every strand has
//! the same number of points (256 in the dataset). A fixed count makes every
//! operation a loop over equal slices, lets them run a strand per task, and
//! is what the GPU wants to draw -- a strand is an instance, a point is a
//! vertex pair.
//!
//! The curve-domain attributes are the ones the hair nodes pass between them:
//! where a strand is attached on the scalp ([`Strands::root_uv`], Blender's
//! `surface_uv_coordinate`), the scalp's normal there, a stable [`id`] that
//! keys each strand's randomness, the parting [`group`] it was interpolated
//! in, and its radius at the root.
//!
//! The functions at the bottom are Blender's curve field inputs for a single
//! strand: the spline parameter (arc length), tangents, minimum-twist
//! normals, and sampling at a factor or a length.
//!
//! [`id`]: Strands::id
//! [`group`]: Strands::group

use fabelgeist_math::Vec3;

/// A set of strands with the same number of points each.
#[derive(Clone, Debug, Default)]
pub struct Strands {
    /// Points per strand, root first. At least two.
    pub points: usize,
    /// Strand-major: strand `s`'s points are `positions[s * points..][..points]`.
    pub positions: Vec<Vec3>,
    /// Where each strand is attached, in the scalp's UV map.
    pub root_uv: Vec<[f32; 2]>,
    /// The scalp's normal at each strand's root.
    pub root_normal: Vec<Vec3>,
    /// Stable per-strand ID: every random choice about a strand is keyed by
    /// it, so a strand keeps its curl phase when others are added.
    pub id: Vec<u32>,
    /// The parting group a strand was interpolated in; `-1` belongs to none.
    pub group: Vec<i32>,
    /// Radius at the root, metres. The profile along the strand is the
    /// groom's ([`crate::ops::Profile`]).
    pub radius: Vec<f32>,
    /// Which strand leads each one's clump, once an operation has chosen
    /// (Blender's `guide_curve_index`); later operations may reuse it, so
    /// that curls follow the clumps.
    pub guide_map: Option<crate::ops::GuideMap>,
}

impl Strands {
    pub fn new(points: usize) -> Self {
        assert!(points >= 2, "a strand needs a root and a tip");
        Self {
            points,
            ..Default::default()
        }
    }

    pub fn with_capacity(points: usize, strands: usize) -> Self {
        let mut s = Self::new(points);
        s.positions.reserve(points * strands);
        s.root_uv.reserve(strands);
        s.root_normal.reserve(strands);
        s.id.reserve(strands);
        s.group.reserve(strands);
        s.radius.reserve(strands);
        s
    }

    /// Number of strands.
    pub fn len(&self) -> usize {
        self.root_uv.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn strand(&self, s: usize) -> &[Vec3] {
        &self.positions[s * self.points..(s + 1) * self.points]
    }

    pub fn strand_mut(&mut self, s: usize) -> &mut [Vec3] {
        &mut self.positions[s * self.points..(s + 1) * self.points]
    }

    pub fn root(&self, s: usize) -> Vec3 {
        self.positions[s * self.points]
    }

    /// Append a strand; `positions` must hold [`Strands::points`] points.
    pub fn push(&mut self, positions: &[Vec3], attachment: Attachment) {
        assert_eq!(positions.len(), self.points);
        self.guide_map = None;
        self.positions.extend_from_slice(positions);
        self.root_uv.push(attachment.uv);
        self.root_normal.push(attachment.normal);
        self.id.push(attachment.id);
        self.group.push(attachment.group);
        self.radius.push(attachment.radius);
    }

    /// The attributes of strand `s`, without its points.
    pub fn attachment(&self, s: usize) -> Attachment {
        Attachment {
            uv: self.root_uv[s],
            normal: self.root_normal[s],
            id: self.id[s],
            group: self.group[s],
            radius: self.radius[s],
        }
    }

    /// Every strand resampled to `points` points spaced evenly by arc length.
    pub fn resampled(&self, points: usize) -> Strands {
        if points == self.points {
            return self.clone();
        }
        let mut out = Strands::with_capacity(points, self.len());
        let mut buffer = vec![Vec3::default(); points];
        let mut lengths = Vec::new();
        for s in 0..self.len() {
            resample_into(self.strand(s), &mut lengths, &mut buffer);
            out.push(&buffer, self.attachment(s));
        }
        out.guide_map = self.guide_map.clone();
        out
    }

    /// Only the strands `keep` names, in that order.
    pub fn select(&self, keep: &[usize]) -> Strands {
        let mut out = Strands::with_capacity(self.points, keep.len());
        for &s in keep {
            out.push(self.strand(s), self.attachment(s));
        }
        out
    }

    /// Append every strand of `other`, resampling it to this point count.
    pub fn extend(&mut self, other: &Strands) {
        let other = other.resampled(self.points);
        self.guide_map = None;
        self.positions.extend_from_slice(&other.positions);
        self.root_uv.extend_from_slice(&other.root_uv);
        self.root_normal.extend_from_slice(&other.root_normal);
        self.id.extend_from_slice(&other.id);
        self.group.extend_from_slice(&other.group);
        self.radius.extend_from_slice(&other.radius);
    }

    /// The box around every point.
    pub fn bounds(&self) -> [Vec3; 2] {
        let mut lo = Vec3::splat(f32::INFINITY);
        let mut hi = Vec3::splat(f32::NEG_INFINITY);
        for &p in &self.positions {
            lo = lo.min(p);
            hi = hi.max(p);
        }
        [lo, hi]
    }

    /// Each strand's length.
    pub fn lengths(&self) -> Vec<f32> {
        (0..self.len()).map(|s| length(self.strand(s))).collect()
    }

    /// Whether any position is not finite.
    pub fn has_non_finite(&self) -> bool {
        self.positions.iter().any(|p| !p.is_finite())
    }
}

/// A strand's curve-domain attributes.
#[derive(Clone, Copy, Debug, Default)]
pub struct Attachment {
    pub uv: [f32; 2],
    pub normal: Vec3,
    pub id: u32,
    pub group: i32,
    pub radius: f32,
}

// ---------------------------------------------------------------------------
// One strand's field inputs.

/// Cumulative arc length at each point: zero at the root, the strand's length
/// at the tip.
pub fn arc_lengths(strand: &[Vec3], out: &mut Vec<f32>) {
    out.clear();
    out.reserve(strand.len());
    let mut total = 0.0;
    out.push(0.0);
    for pair in strand.windows(2) {
        total += (pair[1] - pair[0]).length();
        out.push(total);
    }
}

pub fn length(strand: &[Vec3]) -> f32 {
    strand.windows(2).map(|p| (p[1] - p[0]).length()).sum()
}

/// Blender's poly-curve tangent: along the first and last segments at the
/// ends, and between the neighbours inside.
pub fn tangents(strand: &[Vec3], out: &mut Vec<Vec3>) {
    out.clear();
    let n = strand.len();
    for i in 0..n {
        let d = if i == 0 {
            strand[1] - strand[0]
        } else if i == n - 1 {
            strand[n - 1] - strand[n - 2]
        } else {
            (strand[i + 1] - strand[i]).normalize() + (strand[i] - strand[i - 1]).normalize()
        };
        out.push(safe_normalize(d, Vec3::up()));
    }
}

/// A vector perpendicular to `v`, as Blender's `math::orthogonal` chooses
/// one: crossing with the axis `v` has least of.
pub fn orthogonal(v: Vec3) -> Vec3 {
    let a = v.abs();
    let o = if a.x <= a.y && a.x <= a.z {
        Vec3::new(0.0, -v.z, v.y)
    } else if a.y <= a.z {
        Vec3::new(-v.z, 0.0, v.x)
    } else {
        Vec3::new(-v.y, v.x, 0.0)
    };
    safe_normalize(o, Vec3::right())
}

/// Minimum-twist normals: the first perpendicular to the first tangent, each
/// next one the last carried along by the rotation between their tangents
/// (parallel transport).
pub fn normals(tangents: &[Vec3], out: &mut Vec<Vec3>) {
    out.clear();
    let Some(&first) = tangents.first() else {
        return;
    };
    let mut normal = orthogonal(first);
    out.push(normal);
    for pair in tangents.windows(2) {
        normal = transport(normal, pair[0], pair[1]);
        out.push(normal);
    }
}

/// Carry `v` by the smallest rotation taking unit `from` to unit `to`.
pub fn transport(v: Vec3, from: Vec3, to: Vec3) -> Vec3 {
    let axis = from.cross(to);
    let s = axis.length();
    let c = from.dot(to);
    if s < 1e-7 {
        return if c > 0.0 { v } else { -v };
    }
    rotate(v, axis * (1.0 / s), s.atan2(c))
}

/// Rodrigues' rotation of `v` by `angle` about unit `axis`.
pub fn rotate(v: Vec3, axis: Vec3, angle: f32) -> Vec3 {
    let (sin, cos) = angle.sin_cos();
    v * cos + axis.cross(v) * sin + axis * (axis.dot(v) * (1.0 - cos))
}

pub fn safe_normalize(v: Vec3, fallback: Vec3) -> Vec3 {
    let l = v.length();
    if l > 1e-12 && l.is_finite() {
        v * (1.0 / l)
    } else {
        fallback
    }
}

/// Resample by arc length into `out.len()` evenly spaced points.
pub fn resample_into(strand: &[Vec3], lengths: &mut Vec<f32>, out: &mut [Vec3]) {
    arc_lengths(strand, lengths);
    let total = *lengths.last().unwrap_or(&0.0);
    let n = out.len();
    let mut segment = 0;
    for (i, point) in out.iter_mut().enumerate() {
        let target = total * i as f32 / (n - 1).max(1) as f32;
        while segment + 2 < lengths.len() && lengths[segment + 1] < target {
            segment += 1;
        }
        let span = lengths[segment + 1] - lengths[segment];
        let t = if span > 1e-12 {
            ((target - lengths[segment]) / span).clamp(0.0, 1.0)
        } else {
            0.0
        };
        *point = strand[segment].lerp(strand[segment + 1], t);
    }
}

/// A strand's positions, arc lengths, tangents and normals, ready to be
/// sampled at any factor or length -- Blender's `Sample Curve`.
#[derive(Clone, Debug, Default)]
pub struct CurveSampler {
    pub positions: Vec<Vec3>,
    pub lengths: Vec<f32>,
    pub tangents: Vec<Vec3>,
    pub normals: Vec<Vec3>,
}

/// What [`CurveSampler`] returns.
#[derive(Clone, Copy, Debug)]
pub struct Sample {
    pub position: Vec3,
    pub tangent: Vec3,
    pub normal: Vec3,
}

impl Sample {
    /// The third axis of the frame: tangent cross normal.
    pub fn binormal(&self) -> Vec3 {
        self.tangent.cross(self.normal)
    }
}

impl CurveSampler {
    pub fn new(strand: &[Vec3]) -> Self {
        let mut sampler = Self::default();
        sampler.set(strand);
        sampler
    }

    pub fn set(&mut self, strand: &[Vec3]) {
        self.positions.clear();
        self.positions.extend_from_slice(strand);
        arc_lengths(strand, &mut self.lengths);
        tangents(strand, &mut self.tangents);
        normals(&self.tangents, &mut self.normals);
    }

    pub fn length(&self) -> f32 {
        *self.lengths.last().unwrap_or(&0.0)
    }

    /// At `factor` of the arc length, clamped to the strand.
    pub fn at_factor(&self, factor: f32) -> Sample {
        self.at_length(factor * self.length())
    }

    /// At arc length `s` from the root, clamped to the strand.
    pub fn at_length(&self, s: f32) -> Sample {
        let n = self.positions.len();
        let s = s.clamp(0.0, self.length());
        // The segment holding `s`: the last point at or before it.
        let upper = self.lengths.partition_point(|&l| l <= s).clamp(1, n - 1);
        let i = upper - 1;
        let span = self.lengths[upper] - self.lengths[i];
        let t = if span > 1e-12 {
            ((s - self.lengths[i]) / span).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let tangent = safe_normalize(
            self.tangents[i].lerp(self.tangents[upper], t),
            self.tangents[i],
        );
        let normal = self.normals[i].lerp(self.normals[upper], t);
        // Keep the frame orthonormal after interpolation.
        let normal = safe_normalize(normal - tangent * normal.dot(tangent), self.normals[i]);
        Sample {
            position: self.positions[i].lerp(self.positions[upper], t),
            tangent,
            normal,
        }
    }
}

/// Blur a per-point value along a strand: `iterations` passes of averaging
/// each interior point with its neighbours, weighted by `weight` (Blender's
/// `Blur Attribute` on the point domain of a curve). The root and tip keep
/// their values.
pub fn blur(values: &mut [Vec3], iterations: usize, weight: &[f32]) {
    let n = values.len();
    if n < 3 {
        return;
    }
    let mut next = values.to_vec();
    for _ in 0..iterations {
        for i in 1..n - 1 {
            let w = weight.get(i).copied().unwrap_or(1.0).clamp(0.0, 1.0);
            let average = (values[i - 1] + values[i] + values[i + 1]) * (1.0 / 3.0);
            next[i] = values[i].lerp(average, w);
        }
        values[1..n - 1].copy_from_slice(&next[1..n - 1]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(n: usize) -> Vec<Vec3> {
        (0..n)
            .map(|i| Vec3::new(0.0, i as f32 * 0.1, 0.0))
            .collect()
    }

    #[test]
    fn sampling_by_factor_walks_the_arc() {
        let sampler = CurveSampler::new(&line(11));
        assert!((sampler.length() - 1.0).abs() < 1e-5);
        let mid = sampler.at_factor(0.55);
        assert!((mid.position.y - 0.55).abs() < 1e-5);
        assert!((mid.tangent.y - 1.0).abs() < 1e-5);
        assert!(mid.normal.dot(mid.tangent).abs() < 1e-5);
    }

    #[test]
    fn resampling_spaces_points_evenly() {
        let uneven = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.9, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        ];
        let mut out = vec![Vec3::default(); 5];
        resample_into(&uneven, &mut Vec::new(), &mut out);
        for (i, p) in out.iter().enumerate() {
            assert!((p.y - i as f32 * 0.25).abs() < 1e-5, "{i}: {p}");
        }
    }

    #[test]
    fn transported_normals_stay_perpendicular() {
        let helix: Vec<Vec3> = (0..64)
            .map(|i| {
                let a = i as f32 * 0.3;
                Vec3::new(a.cos(), i as f32 * 0.05, a.sin())
            })
            .collect();
        let mut t = Vec::new();
        let mut n = Vec::new();
        tangents(&helix, &mut t);
        normals(&t, &mut n);
        for (t, n) in t.iter().zip(&n) {
            assert!(t.dot(*n).abs() < 1e-3);
            assert!((n.length() - 1.0).abs() < 1e-3);
        }
    }
}
