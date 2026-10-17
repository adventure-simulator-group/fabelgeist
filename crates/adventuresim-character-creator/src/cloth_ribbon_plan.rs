//! Folded tape follows an authored cloth boundary on every fitted body.
use anyhow::{Result, ensure};
use fabelgeist_armor::gpu::BodyClothLayer;
use fabelgeist_armor::{ArmorComponentRole, ClothBinding, Millimeters};
use std::collections::{BTreeMap, BTreeSet};

/// Directed free edges and authored tape widths per native carrier point.
pub(crate) struct ClothRibbonBoundary {
    pub edges: Vec<[u32; 2]>,
    pub widths: Vec<Millimeters>,
}

/// Native GPU records: previous/current/next carrier indices and inset metres.
pub(crate) struct ClothRibbonPlan {
    pub words: Vec<u32>,
    pub first: u32,
}

impl ClothRibbonPlan {
    pub fn new(
        boundary: ClothRibbonBoundary,
        binding: &ClothBinding,
        first: u32,
        layers: &mut Vec<BodyClothLayer>,
    ) -> Result<Self> {
        let mut edges = boundary
            .edges
            .into_iter()
            .map(|[a, b]| (a, b))
            .collect::<BTreeMap<_, _>>();
        let mut words = Vec::new();
        let mut indices = Vec::new();
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
            let mut chain = vec![start];
            let mut at = start;
            while let Some(next) = edges.remove(&at) {
                chain.push(next);
                at = next;
                if at == start {
                    break;
                }
            }
            ensure!(chain.len() >= 2, "binding has an empty boundary");
            let closed = chain.first() == chain.last();
            if closed {
                chain.pop();
            }
            let offset = words.len() as u32 / 4;
            for (i, &vertex) in chain.iter().enumerate() {
                let previous = if i > 0 {
                    chain[i - 1]
                } else if closed {
                    chain[chain.len() - 1]
                } else {
                    vertex
                };
                let next = if i + 1 < chain.len() {
                    chain[i + 1]
                } else if closed {
                    chain[0]
                } else {
                    vertex
                };
                let width = boundary
                    .widths
                    .get(vertex as usize)
                    .ok_or_else(|| anyhow::anyhow!("binding has no width for its carrier"))?
                    .metres();
                for inset in [0_f32, width] {
                    words.extend([previous, vertex, next, inset.to_bits()]);
                }
            }
            let spans = if closed { chain.len() } else { chain.len() - 1 };
            for i in 0..spans {
                let a = offset + i as u32 * 2;
                let b = offset + ((i + 1) % chain.len()) as u32 * 2;
                indices.extend([a, b, a + 1, a + 1, b, b + 1]);
            }
        }
        let carrier = &layers[0];
        layers.push(BodyClothLayer {
            chart: None,
            point_count: words.len() as u32 / 4,
            indices,
            role: ArmorComponentRole::EdgeBinding,
            offset_metres: carrier.offset_metres + binding.thickness.metres(),
            thickness_metres: carrier.thickness_metres + 2. * binding.thickness.metres(),
        });
        Ok(Self { words, first })
    }
}
