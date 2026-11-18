//! Columns of a neckline band around a closed cut boundary, in the canonical
//! unposed reference in metres.
use super::cloth::ReferenceCloth;
use crate::cloth_opening::ClothSurfaceSample;
use anyhow::{Context, Result, ensure};
use bevy::math::Vec3;

/// Boundary edges shorter than this are merged into their neighbour, in
/// metres. A cut corner can leave edges of a millimetre whose directions
/// say nothing about the neckline's shape.
const TRIM_MIN_EDGE_M: f32 = 0.004;
/// Cosine between neighbouring edge perpendiculars below which the band is
/// mitred. A gentler bend turns the band smoothly instead.
const MITRE_TURN_COSINE: f32 = 0.9;
/// Least cosine of half a mitred turn; a sharper corner doubles back.
const MITRE_HALF_TURN_COSINE: f32 = 0.5;
/// Reference arc length to each side over which a column's direction across
/// the band is averaged. Single cut vertices then cannot kink a deep band.
const TRIM_DIRECTION_REACH_M: f32 = 0.04;

/// One column across the band, before its rows are laid on the cloth.
#[derive(Clone, Copy)]
pub(super) struct TrimColumn {
    pub edge: ClothSurfaceSample,
    pub point: Vec3,
    /// Offset in the cloth per metre of inset; longer than unit at a mitre.
    pub reach: Vec3,
    /// Reference arc length along the neckline, in metres.
    pub arc: f32,
    /// Chart travel along the neckline per metre of inset. At a mitre the
    /// two sides keep their own straight charts up to the shared diagonal.
    pub shear: f32,
    pub joins_next: bool,
}

/// The band's columns in order around the neckline and its reference length.
pub(super) struct TrimOutline {
    pub columns: Vec<TrimColumn>,
    pub length: f32,
}

/// Frames of a closed boundary. Edge `i` runs from vertex `i` to the next.
struct NecklineEdges<'a> {
    neckline: &'a [u32],
    points: Vec<Vec3>,
    /// Unit direction of each edge.
    along: Vec<Vec3>,
    /// Reference length of each edge, in metres.
    lengths: Vec<f32>,
    /// Unit perpendicular of each edge in the cloth, away from the opening.
    across: Vec<Vec3>,
    /// Whether the band is mitred at each vertex.
    sharp: Vec<bool>,
    /// Direction across the band at each vertex.
    inward: Vec<Vec3>,
}

impl TrimOutline {
    /// Long cut edges get interior columns, so the band's free edge follows
    /// the cloth and not chords between sparse cut corners. A sharp corner
    /// gets two columns on its diagonal, one closing each side. The last
    /// column closes the loop on the first.
    pub fn new(cloth: &ReferenceCloth<'_>, neckline: &[u32], spacing: f32) -> Result<Self> {
        let mut kept: Vec<u32> = Vec::with_capacity(neckline.len());
        for &vertex in neckline {
            let point = cloth.points[vertex as usize];
            let clear = |other: Option<&u32>| {
                other
                    .is_none_or(|&v| (cloth.points[v as usize] - point).length() >= TRIM_MIN_EDGE_M)
            };
            if clear(kept.last()) && clear(kept.first().filter(|_| kept.len() > 1)) {
                kept.push(vertex);
            }
        }
        ensure!(kept.len() >= 3, "gown neckline is degenerate");
        let neckline = kept.as_slice();
        let edges = NecklineEdges::new(cloth, neckline)?;
        let count = neckline.len();
        let mut columns = Vec::new();
        let mut travelled = 0.;
        for i in 0..count {
            let next = (i + 1) % count;
            let [arriving, leaving] = edges.corner(i)?;
            if i > 0 && edges.sharp[i] {
                columns.push(TrimColumn {
                    arc: travelled,
                    ..arriving
                });
            }
            columns.push(TrimColumn {
                arc: travelled,
                ..leaving
            });
            let length = edges.lengths[i];
            let steps = (length / spacing).ceil().max(1.) as usize;
            // A mitred corner keeps its side straight up to the diagonal.
            let side = |vertex: usize| {
                if edges.sharp[vertex] {
                    edges.across[i]
                } else {
                    edges.inward[vertex]
                }
            };
            for step in 1..steps {
                let share = step as f32 / steps as f32;
                columns.push(TrimColumn {
                    edge: ClothSurfaceSample {
                        vertices: [neckline[i], neckline[next], neckline[next]].map(|v| v as usize),
                        weights: [1. - share, share, 0.],
                    },
                    point: edges.points[i].lerp(edges.points[next], share),
                    reach: side(i)
                        .lerp(side(next), share)
                        .try_normalize()
                        .context("neckline trim direction reverses along an edge")?,
                    arc: travelled + length * share,
                    shear: 0.,
                    joins_next: true,
                });
            }
            travelled += length;
        }
        let [closing, _] = edges.corner(0)?;
        columns.push(TrimColumn {
            arc: travelled,
            joins_next: false,
            ..closing
        });
        Ok(Self {
            columns,
            length: travelled,
        })
    }
}

impl<'a> NecklineEdges<'a> {
    fn new(cloth: &ReferenceCloth<'_>, neckline: &'a [u32]) -> Result<Self> {
        let count = neckline.len();
        let vertex_inward = cloth.inward_directions(neckline)?;
        let points = neckline
            .iter()
            .map(|&v| cloth.points[v as usize])
            .collect::<Vec<_>>();
        let mut along = Vec::with_capacity(count);
        let mut lengths = Vec::with_capacity(count);
        let mut across = Vec::with_capacity(count);
        for i in 0..count {
            let next = (i + 1) % count;
            let edge = points[next] - points[i];
            let normal =
                cloth.normals[neckline[i] as usize] + cloth.normals[neckline[next] as usize];
            let side = normal
                .cross(edge)
                .try_normalize()
                .context("neckline has a stationary edge")?;
            let toward = vertex_inward[i] + vertex_inward[next];
            across.push(if side.dot(toward) < 0. { -side } else { side });
            lengths.push(edge.length());
            along.push(edge.normalize());
        }
        let sharp = (0..count)
            .map(|i| across[(i + count - 1) % count].dot(across[i]) < MITRE_TURN_COSINE)
            .collect();
        let mut edges = Self {
            neckline,
            points,
            along,
            lengths,
            across,
            sharp,
            inward: vertex_inward,
        };
        for vertex in 0..count {
            if !edges.sharp[vertex] {
                edges.inward[vertex] = edges.averaged(vertex)?;
            }
        }
        Ok(edges)
    }

    fn before(&self, index: usize) -> usize {
        (index + self.neckline.len() - 1) % self.neckline.len()
    }

    /// Length-weighted edge perpendiculars to each side of a vertex, stopping
    /// at a mitre so each side of a corner stays straight.
    fn averaged(&self, vertex: usize) -> Result<Vec3> {
        let count = self.neckline.len();
        let mut sum = Vec3::ZERO;
        for forward in [true, false] {
            let mut remaining = TRIM_DIRECTION_REACH_M;
            let mut edge = if forward { vertex } else { self.before(vertex) };
            for _ in 0..count {
                let length = self.lengths[edge].min(remaining);
                sum += self.across[edge] * length;
                remaining -= length;
                let end = if forward { (edge + 1) % count } else { edge };
                if remaining <= 0. || self.sharp[end] {
                    break;
                }
                edge = if forward { end } else { self.before(edge) };
            }
        }
        sum.try_normalize()
            .context("neckline trim direction cancels along the edge")
    }

    /// The columns that arrive at and leave a vertex, without an arc. They
    /// differ only at a mitre, where they share the diagonal.
    fn corner(&self, vertex: usize) -> Result<[TrimColumn; 2]> {
        let column = |reach: Vec3, shear: f32, joins_next: bool| TrimColumn {
            edge: ClothSurfaceSample {
                vertices: [self.neckline[vertex] as usize; 3],
                weights: [1., 0., 0.],
            },
            point: self.points[vertex],
            reach,
            arc: 0.,
            shear,
            joins_next,
        };
        if !self.sharp[vertex] {
            return Ok([column(self.inward[vertex], 0., true); 2]);
        }
        let before = self.before(vertex);
        let diagonal = (self.across[before] + self.across[vertex])
            .try_normalize()
            .context("neckline doubles back on itself")?;
        let half_turn = diagonal.dot(self.across[vertex]);
        ensure!(
            half_turn > MITRE_HALF_TURN_COSINE,
            "neckline corner is too sharp for its trim"
        );
        let reach = diagonal / half_turn;
        Ok([
            column(reach, reach.dot(self.along[before]), false),
            column(reach, reach.dot(self.along[vertex]), true),
        ])
    }
}
