//! `Interpolate Hair Curves V2`: from a few guides to a full head of hair.
//!
//! The node group scatters points over the scalp -- Poisson-disk, as dense
//! as `Density` asks, thinned by a density mask (DiffLocks' balding patterns)
//! -- and grows a strand from each by `Interpolate Curves`: the shapes of the
//! nearest guides (`Interpolation Guides` of them, only those in the same
//! parting group), each taken relative to its own root and blended by
//! distance, placed at the new root. With `Follow Surface Normal` each
//! guide's shape is first turned from its root's normal to the new root's,
//! so a shape combed on the crown still lies along the head at the temple.
//! Strands closer than `Distance to Guides` to a guide's root are dropped.
//!
//! Where it departs: the node group takes a density per unit area; here the
//! stack asks for a strand count (the dataset's "~100K strands") and the
//! Poisson radius follows from it and the scalp's area. And it blends only
//! guides that agree ([`Interpolate::divergence`]): the mean of a guide
//! combed left and one combed right is a strand combed straight down
//! between them -- across the face, from the front hairline -- where real
//! hair would part and go one way or the other.

use crate::identity::PartingGroup;
use crate::identity::StrandId;
use fabelgeist_math::Vec3;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::grid::PointGrid;
use crate::ops::Context;
use crate::rng::{self, Rng};
use crate::strands::{Attachment, Strands, rotate, safe_normalize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Interpolate {
    /// How many strands to grow (before the density mask thins them).
    pub count: usize,
    /// Blend the nearest this many guides (`Interpolation Guides`).
    pub neighbors: usize,
    /// Drop new strands closer than this to a guide's root (0 keeps all).
    pub distance_to_guides: f32,
    /// Space roots out (`Poisson Disk Distribution`) rather than scatter
    /// them independently.
    pub poisson: bool,
    /// Turn each guide's shape from its root normal to the new root's.
    pub follow_surface_normal: bool,
    /// Along how much of the strand the turn fades out, metres. Blender
    /// turns the whole strand; on a curved scalp that swings long hair's
    /// tips out by the angle between neighbouring roots.
    pub follow_length: f32,
    /// Keep interpolation within parting groups (`Part by Texture`).
    pub parting: Parting,
    /// Blend only guides whose shapes agree with the nearest one's: a
    /// guide is left out where its course strays from the nearest guide's
    /// by more than this share of how far that one has come (0: blend all).
    /// Where combing diverges, strands follow one side or the other, and
    /// the hair parts there as hair does.
    #[serde(default)]
    pub divergence: f32,
    /// How likely a root is to be kept, over the scalp.
    pub density: DensityMask,
    /// How unevenly roots cover the scalp, 0 to 1: patches a few
    /// centimetres across thinned by up to this much.
    #[serde(default)]
    pub patchiness: f32,
    /// Over how far in from the scalp's edge -- the hairline -- roots thin
    /// out and strands grow finer, metres (0: a sharp edge).
    #[serde(default)]
    pub edge: f32,
    /// Keep the guides themselves among the strands.
    pub keep_guides: bool,
    /// Root radius of the new strands, metres.
    pub radius: f32,
    pub seed: i32,
}

impl Default for Interpolate {
    fn default() -> Self {
        Self {
            count: 30_000,
            neighbors: 4,
            distance_to_guides: 0.0,
            poisson: true,
            follow_surface_normal: true,
            follow_length: 0.05,
            parting: Parting::None,
            divergence: 0.0,
            density: DensityMask::Uniform,
            patchiness: 0.0,
            edge: 0.0,
            keep_guides: false,
            radius: 35e-6,
            seed: 0,
        }
    }
}

/// Where hair parts: a plane through the head (the part line is where it
/// meets the scalp) over the front of the head. Roots on either side are
/// kept apart; behind `until` (a `z`, towards the back) the part has ended
/// and roots take from both sides.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum Parting {
    #[default]
    None,
    Plane {
        normal: [f32; 3],
        offset: f32,
        until: f32,
    },
    /// Sectors round the head, the front one in the middle: a braid's
    /// lanes, gathered from the left, the top and the right.
    Lanes { count: u32 },
    /// A fringe: the front of the scalp (in front of `front`, within
    /// `half_width` of the middle) apart from the rest, whose front is
    /// split left from right -- hair combed to either side of a fringe must
    /// not be averaged into hair combed down the face.
    Fringe { front: f32, half_width: f32 },
}

impl Parting {
    /// A part along the head from front to crown, `offset` metres to the
    /// side (0 in the middle).
    pub fn line(offset: f32, until: f32) -> Self {
        Parting::Plane {
            normal: [1.0, 0.0, 0.0],
            offset,
            until,
        }
    }

    /// The group of a root: 0 or 1 on either side of the part, -1 where
    /// there is none.
    pub fn group(&self, p: Vec3) -> PartingGroup {
        let word = match self {
            Parting::None => -1,
            Parting::Plane {
                normal,
                offset,
                until,
            } => {
                if p.z < *until {
                    -1
                } else if Vec3::from_array(*normal).dot(p) > *offset {
                    1
                } else {
                    0
                }
            }
            Parting::Fringe { front, half_width } => {
                if p.z > *front && p.x.abs() < *half_width {
                    1
                } else if p.z > -0.03 {
                    if p.x >= 0.0 { 2 } else { 0 }
                } else {
                    -1
                }
            }
            Parting::Lanes { count } => {
                let a = p.x.atan2(p.z + 0.02);
                let t = (a + std::f32::consts::PI) / std::f32::consts::TAU;
                ((t * *count as f32) as i32).clamp(0, *count as i32 - 1)
            }
        };
        PartingGroup::from_gpu_word(word)
    }
}

/// Whether a strand of group `own` may take its shape from a guide of group
/// `guide`: the same side, or either has no side.
pub fn compatible(own: PartingGroup, guide: PartingGroup) -> bool {
    own.is_unassigned() || guide.is_unassigned() || own == guide
}

/// How likely a root is to be kept, over the scalp. Patterns read the scalp
/// UV map as [`crate::Head`] lays it out: the crown in the middle, the face
/// towards `+v`, the hairline near the rim.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum DensityMask {
    #[default]
    Uniform,
    /// Male-pattern loss: a thinned crown, temples receding, the top
    /// thinning between them. Each amount from 0 (none) to 1.
    Balding { crown: f32, temples: f32, top: f32 },
    /// A mask image over the UV square, row-major from `v = 0`, values 0 to 1.
    Image {
        width: usize,
        height: usize,
        values: Vec<f32>,
    },
}

impl DensityMask {
    pub fn at(&self, uv: [f32; 2]) -> f32 {
        match self {
            DensityMask::Uniform => 1.0,
            DensityMask::Balding {
                crown,
                temples,
                top,
            } => {
                let (x, y) = (uv[0] - 0.5, uv[1] - 0.5);
                let r = (x * x + y * y).sqrt() * 2.0;
                // The crown whorl sits a little behind the top.
                let cx = x;
                let cy = y + 0.08;
                let crown_r = (cx * cx + cy * cy).sqrt() * 2.0;
                let crown_loss =
                    crown * (1.0 - super::smoothstep(0.1, 0.1 + 0.35 * crown, crown_r));
                // Temples: towards the front at either side.
                let angle = x.atan2(y);
                let temple = (-((angle.abs() - 0.55) / 0.3).powi(2)).exp();
                let temple_loss =
                    temples * temple * super::smoothstep(0.62 - 0.35 * temples, 0.8, r);
                // The top: a band from the front hairline back to the crown.
                let band = (-(x / 0.16).powi(2)).exp() * super::smoothstep(-0.2, 0.3, y);
                let top_loss = top * band * super::smoothstep(0.0, 0.5, r);
                (1.0 - crown_loss.max(temple_loss).max(top_loss)).clamp(0.0, 1.0)
            }
            DensityMask::Image {
                width,
                height,
                values,
            } => {
                if *width == 0 || *height == 0 {
                    return 1.0;
                }
                let x = (uv[0].clamp(0.0, 1.0) * (*width - 1) as f32).max(0.0);
                let y = (uv[1].clamp(0.0, 1.0) * (*height - 1) as f32).max(0.0);
                let (x0, y0) = (x as usize, y as usize);
                let (x1, y1) = ((x0 + 1).min(width - 1), (y0 + 1).min(height - 1));
                let (fx, fy) = (x - x0 as f32, y - y0 as f32);
                let v = |x: usize, y: usize| values[y * width + x];
                let top = v(x0, y0) * (1.0 - fx) + v(x1, y0) * fx;
                let bottom = v(x0, y1) * (1.0 - fx) + v(x1, y1) * fx;
                top * (1.0 - fy) + bottom * fy
            }
        }
    }
}

impl Interpolate {
    pub fn apply(&self, guides: &Strands, context: &Context) -> Strands {
        let points = guides.points;
        let scalp = context.scalp;
        let mut rng = Rng::new(rng::seed(self.seed, 0x1A7E) as u64);

        // Roots: area-sampled, spaced out, thinned by the mask.
        let spacing = 0.55 * (scalp.area() / self.count.max(1) as f32).sqrt();
        let mut placed = PointGrid::new(spacing.max(1e-5));
        let mut roots = Vec::with_capacity(self.count);
        let attempts = if self.poisson {
            self.count * 2
        } else {
            self.count
        };
        for _ in 0..attempts {
            if roots.len() >= self.count {
                break;
            }
            let point = scalp.sample(&mut rng);
            if self.poisson && placed.any_within(point.position, spacing, |_| true) {
                continue;
            }
            placed.insert(point.position, PartingGroup::from_gpu_word(0));
            roots.push(point);
        }
        let seed = rng::seed(self.seed, 0xD3_5E);
        let edge = self.edge_distance(scalp);
        let offset = rng::signed_vec3(seed, 0x9A7C) * 50.0;
        let inset = |p: Vec3| {
            if self.edge > 0.0 {
                let d = edge
                    .nearest(p, self.edge, |_| true)
                    .map_or(self.edge, |(_, d)| d);
                super::smoothstep(0.0, self.edge, d)
            } else {
                1.0
            }
        };
        roots.retain(|r| {
            let mut keep = self.density.at(r.uv);
            if self.patchiness > 0.0 {
                let n = crate::noise::fractal(r.position * 30.0 + offset, 3, 0.5);
                keep *= 1.0 - self.patchiness * super::smoothstep(0.45, 0.7, n);
            }
            // Thinning out to the edge, a few fine hairs right at it.
            keep *= 0.08 + 0.92 * inset(r.position).powf(1.5);
            keep >= 1.0 || rng::unit(r.position.x.to_bits() ^ r.position.z.to_bits(), seed) < keep
        });

        // The guides, by root, in their parting groups.
        let guide_groups: Vec<PartingGroup> = (0..guides.len())
            .map(|g| match self.parting {
                Parting::None => PartingGroup::Unassigned,
                _ => self.parting.group(guides.root(g)),
            })
            .collect();
        let grid = PointGrid::build(
            &(0..guides.len())
                .map(|g| (guides.root(g), guide_groups[g]))
                .collect::<Vec<_>>(),
            spacing.max(0.01),
        );
        let near_guide = |p: Vec3| {
            self.distance_to_guides > 0.0 && grid.any_within(p, self.distance_to_guides, |_| true)
        };

        let neighbors = self.neighbors.max(1);
        let grown: Vec<Option<(Vec<Vec3>, Attachment)>> = roots
            .par_iter()
            .enumerate()
            .map(|(i, root)| {
                if near_guide(root.position) {
                    return None;
                }
                let group = self.parting.group(root.position);
                let nearest = grid.nearest_k(root.position, neighbors, f32::INFINITY, |g| {
                    compatible(group, g)
                });
                if nearest.is_empty() {
                    return None;
                }
                let nearest = if self.divergence > 0.0 && points >= 3 {
                    let reference = guides.strand(nearest[0].0 as usize);
                    let samples = [points / 3, 2 * points / 3, points - 1];
                    nearest
                        .into_iter()
                        .filter(|&(g, _)| {
                            let strand = guides.strand(g as usize);
                            samples.iter().all(|&j| {
                                let own = reference[j] - reference[0];
                                let theirs = strand[j] - strand[0];
                                (own - theirs).length() <= self.divergence * own.length().max(0.02)
                            })
                        })
                        .collect()
                } else {
                    nearest
                };
                // Inverse-square weights; a root on top of a guide takes it.
                let mut weights: Vec<f32> = nearest
                    .iter()
                    .map(|&(_, d)| 1.0 / (d * d + 1e-10))
                    .collect();
                let total: f32 = weights.iter().sum();
                weights.iter_mut().for_each(|w| *w /= total);

                let mut shape = vec![Vec3::default(); points];
                for (&(g, _), &w) in nearest.iter().zip(&weights) {
                    let g = g as usize;
                    let strand = guides.strand(g);
                    let guide_root = strand[0];
                    let turn = if self.follow_surface_normal {
                        Some((guides.root_normal[g], root.normal))
                    } else {
                        None
                    };
                    let mut along = 0.0;
                    for (j, p) in strand.iter().enumerate() {
                        if j > 0 {
                            along += (*p - strand[j - 1]).length();
                        }
                        let mut offset = *p - guide_root;
                        if let Some((from, to)) = turn {
                            // Turned fully at the root, not at all past
                            // `follow_length`: the strand leaves the scalp
                            // as the scalp there faces, and hangs as its
                            // guide hangs.
                            let share = 1.0 - super::smoothstep(0.0, self.follow_length, along);
                            offset = turn_between(offset, from, to, share);
                        }
                        shape[j] += offset * w;
                    }
                }
                let positions: Vec<Vec3> = shape.iter().map(|o| root.position + *o).collect();
                let id = rng::hash(i as u32, rng::seed(self.seed, 0x51D));
                // Hair at the hairline is finer.
                let radius = self.radius * (0.45 + 0.55 * inset(root.position));
                Some((
                    positions,
                    Attachment {
                        uv: root.uv,
                        normal: root.normal,
                        id: StrandId::from_word(id),
                        group,
                        radius,
                    },
                ))
            })
            .collect();

        let mut out = Strands::with_capacity(points, grown.len() + guides.len());
        if self.keep_guides {
            for (g, &group) in guide_groups.iter().enumerate() {
                let mut a = guides.attachment(g);
                a.group = group;
                a.radius = self.radius;
                out.push(guides.strand(g), a);
            }
        }
        for (positions, attachment) in grown.into_iter().flatten() {
            out.push(&positions, attachment);
        }
        out
    }
}

impl Interpolate {
    /// Points along the scalp's open edges, a millimetre apart, to measure
    /// how far in from the hairline a root is.
    fn edge_distance(&self, scalp: &crate::surface::Surface) -> PointGrid {
        let mut grid = PointGrid::new(self.edge.max(0.002));
        if self.edge <= 0.0 {
            return grid;
        }
        for p in scalp.boundary(0.001) {
            grid.insert(p, PartingGroup::from_gpu_word(0));
        }
        grid
    }
}

/// Turn `v` by `share` of the rotation taking direction `from` to `to`.
fn turn_between(v: Vec3, from: Vec3, to: Vec3, share: f32) -> Vec3 {
    let from = safe_normalize(from, Vec3::up());
    let to = safe_normalize(to, Vec3::up());
    let axis = from.cross(to);
    let s = axis.length();
    let c = from.dot(to);
    if s < 1e-6 {
        return v;
    }
    rotate(v, axis * (1.0 / s), s.atan2(c) * share)
}
