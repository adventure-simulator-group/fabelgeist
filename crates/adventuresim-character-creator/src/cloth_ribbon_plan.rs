//! Folded tape follows an authored cloth boundary on every fitted body.
use anyhow::{Context, Result, ensure};
use fabelgeist_armor::gpu::BodyClothLayer;
use fabelgeist_armor::{ArmorComponentRole, Micrometers, Millimeters};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

/// Native tape records: previous/current/next points, inset bits, two interior points.
const CLOTH_RIBBON_WORDS: usize = 6;
/// Directed free edges and authored tape widths per native carrier point.
pub(crate) struct ClothRibbonBoundary {
    pub edges: Vec<[u32; 2]>,
    pub widths: Vec<Millimeters>,
}
/// Reference connectivity determines local surface normals after each refit.
pub(crate) struct ClothRibbonPlan {
    pub words: Vec<u32>,
    pub first: u32,
    pub carrier_points: u32,
    pub carrier_indices: Vec<u32>,
    pub carrier_offset_metres: f32,
    /// The reference realization records cloth-face ownership once; every
    /// subsequent morph evaluates these same triangles and weights.
    pub reference_projection: OnceLock<fabelgeist_compute::QueryHits>,
}
struct RibbonChain {
    points: Vec<u32>,
    closed: bool,
}

impl ClothRibbonPlan {
    pub fn point_count(&self) -> u32 {
        self.words.len() as u32 / CLOTH_RIBBON_WORDS as u32
    }
    pub fn source(&self, first_point: u32, point_words: u32) -> String {
        format!(
            "const FIRST_POINT:u32={first_point}u;\nconst POINT_WORDS:u32={point_words}u;\nconst TAPE_WORDS:u32={CLOTH_RIBBON_WORDS}u;\n{}",
            include_str!("device_cloth_ribbon.wgsl")
        )
    }
    pub fn new(
        boundary: ClothRibbonBoundary,
        thickness: Micrometers,
        first: u32,
        layers: &mut Vec<BodyClothLayer>,
    ) -> Result<Self> {
        let carrier = layers.first().context("binding has no cloth carrier")?;
        let adjacent = carrier
            .indices
            .as_chunks::<3>()
            .0
            .iter()
            .flat_map(|&[a, b, c]| [([a, b], c), ([b, c], a), ([c, a], b)])
            .collect::<BTreeMap<_, _>>();
        let mut words = Vec::new();
        let mut indices = Vec::new();
        for chain in boundary.chains()? {
            let offset = words.len() as u32 / CLOTH_RIBBON_WORDS as u32;
            for (i, &vertex) in chain.points.iter().enumerate() {
                let previous = if i > 0 {
                    chain.points[i - 1]
                } else if chain.closed {
                    chain.points[chain.points.len() - 1]
                } else {
                    vertex
                };
                let next = if i + 1 < chain.points.len() {
                    chain.points[i + 1]
                } else if chain.closed {
                    chain.points[0]
                } else {
                    vertex
                };
                let incoming = adjacent.get(&[previous, vertex]);
                let outgoing = adjacent.get(&[vertex, next]);
                let previous_inside = incoming
                    .or(outgoing)
                    .copied()
                    .context("tape boundary has no incident cloth face")?;
                let next_inside = outgoing
                    .or(incoming)
                    .copied()
                    .context("tape boundary has no incident cloth face")?;
                let width = boundary
                    .widths
                    .get(vertex as usize)
                    .context("binding has no width for its carrier")?
                    .metres();
                for inset in [0_f32, width] {
                    words.extend([
                        previous,
                        vertex,
                        next,
                        inset.to_bits(),
                        previous_inside,
                        next_inside,
                    ]);
                }
            }
            let spans = if chain.closed {
                chain.points.len()
            } else {
                chain.points.len() - 1
            };
            for i in 0..spans {
                let a = offset + i as u32 * 2;
                let b = offset + ((i + 1) % chain.points.len()) as u32 * 2;
                indices.extend([a, b, a + 1, a + 1, b, b + 1]);
            }
        }
        let carrier_points = carrier.point_count;
        let carrier_indices = carrier.indices.clone();
        let carrier_offset_metres = carrier.offset_metres;
        layers.push(BodyClothLayer {
            chart: None,
            point_count: words.len() as u32 / CLOTH_RIBBON_WORDS as u32,
            indices,
            role: ArmorComponentRole::EdgeBinding,
            offset_metres: carrier.offset_metres + thickness.metres(),
            thickness_metres: carrier.thickness_metres + 2. * thickness.metres(),
        });
        Ok(Self {
            words,
            first,
            carrier_points,
            carrier_indices,
            carrier_offset_metres,
            reference_projection: OnceLock::new(),
        })
    }
}
impl ClothRibbonBoundary {
    fn chains(&self) -> Result<Vec<RibbonChain>> {
        let mut edges = self
            .edges
            .iter()
            .map(|&[a, b]| (a, b))
            .collect::<BTreeMap<_, _>>();
        ensure!(
            edges.len() == self.edges.len(),
            "binding has duplicate outgoing edges"
        );
        let mut chains = Vec::new();
        while !edges.is_empty() {
            let incoming = edges.values().copied().collect::<BTreeSet<_>>();
            let start = edges
                .keys()
                .find(|v| !incoming.contains(v))
                .copied()
                .or_else(|| edges.first_key_value().map(|(&v, _)| v));
            let Some(start) = start else {
                break;
            };
            let mut points = vec![start];
            let mut at = start;
            while let Some(next) = edges.remove(&at) {
                points.push(next);
                at = next;
                if at == start {
                    break;
                }
            }
            ensure!(points.len() >= 2, "binding has an empty boundary");
            let closed = points.first() == points.last();
            if closed {
                points.pop();
            }
            chains.push(RibbonChain { points, closed });
        }
        Ok(chains)
    }
}
