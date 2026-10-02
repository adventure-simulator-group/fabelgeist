//! Shared street topology owns frontage, setbacks, and connections to the market.
use super::*;
use std::collections::{BTreeMap, VecDeque};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct BlockId(pub u64);
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct StreetNodeId(usize);
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct StreetEdgeId(usize);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum StreetClass {
    Lane,
    Secondary,
    TradeRoute,
}

impl StreetClass {
    pub(super) fn half_width(self) -> f32 {
        match self {
            Self::Lane => ORDINARY_STREET_HALF_WIDTH_METRES,
            Self::Secondary => SECONDARY_STREET_HALF_WIDTH_METRES,
            Self::TradeRoute => PRIMARY_STREET_HALF_WIDTH_METRES,
        }
    }
    fn surface(self) -> CityStreetSurface {
        match self {
            Self::Lane => CityStreetSurface::CompactedEarth,
            Self::Secondary => CityStreetSurface::Gravel,
            Self::TradeRoute => CityStreetSurface::Fieldstone,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct CityBlock {
    pub(super) id: BlockId,
    pub(super) corners: [Vec2; 4],
    pub(super) streets: [StreetClass; 4],
    pub(super) market: bool,
}

impl CityBlock {
    pub(super) fn centre(self) -> Vec2 {
        self.corners.into_iter().sum::<Vec2>() * 0.25
    }
    pub(super) fn key(self) -> BlockId {
        self.id
    }
    pub(super) fn is_market(self) -> bool {
        self.market
    }
}

struct StreetEdge {
    nodes: [StreetNodeId; 2],
    class: StreetClass,
}

#[derive(Default)]
pub(super) struct StreetGraph {
    pub(super) blocks: Vec<CityBlock>,
    nodes: Vec<Vec2>,
    edges: Vec<StreetEdge>,
    node_lookup: BTreeMap<(u32, u32), StreetNodeId>,
    edge_lookup: BTreeMap<[StreetNodeId; 2], StreetEdgeId>,
    boundaries: BTreeMap<BlockId, Vec<StreetEdgeId>>,
}

impl StreetGraph {
    pub(super) fn segment(
        &mut self,
        points: [Vec2; 2],
        class: StreetClass,
        block: Option<BlockId>,
    ) {
        let mut nodes = points.map(|point| {
            let key = (point.x.to_bits(), point.y.to_bits());
            *self.node_lookup.entry(key).or_insert_with(|| {
                let id = StreetNodeId(self.nodes.len());
                self.nodes.push(point);
                id
            })
        });
        nodes.sort_unstable();
        let id = *self.edge_lookup.entry(nodes).or_insert_with(|| {
            let id = StreetEdgeId(self.edges.len());
            self.edges.push(StreetEdge { nodes, class });
            id
        });
        self.edges[id.0].class = self.edges[id.0].class.max(class);
        if let Some(block) = block {
            self.boundaries.entry(block).or_default().push(id);
        }
    }

    pub(super) fn developed_streets(&self, developed: &BTreeSet<BlockId>) -> Vec<CityStreetPatch> {
        if developed.is_empty() {
            return Vec::new();
        }
        let mut selected = self
            .edges
            .iter()
            .enumerate()
            .filter_map(|(index, edge)| {
                (edge.class == StreetClass::TradeRoute).then_some(StreetEdgeId(index))
            })
            .collect::<BTreeSet<_>>();
        for block in developed {
            selected.extend(&self.boundaries[block]);
        }
        let market = self.blocks.iter().find(|block| block.market).unwrap();
        selected.extend(&self.boundaries[&market.id]);
        let mut adjacency = vec![Vec::new(); self.nodes.len()];
        for (index, edge) in self.edges.iter().enumerate() {
            let [a, b] = edge.nodes;
            adjacency[a.0].push((b, StreetEdgeId(index)));
            adjacency[b.0].push((a, StreetEdgeId(index)));
        }
        let mut parents = vec![None; self.nodes.len()];
        let mut reached = vec![false; self.nodes.len()];
        let mut queue = VecDeque::new();
        for node in self
            .edges
            .iter()
            .filter(|edge| edge.class == StreetClass::TradeRoute)
            .flat_map(|edge| edge.nodes)
        {
            if !reached[node.0] {
                reached[node.0] = true;
                queue.push_back(node);
            }
        }
        while let Some(node) = queue.pop_front() {
            for &(next, edge) in &adjacency[node.0] {
                if !reached[next.0] {
                    reached[next.0] = true;
                    parents[next.0] = Some((node, edge));
                    queue.push_back(next);
                }
            }
        }
        for edge in selected.clone() {
            for mut node in self.edges[edge.0].nodes {
                assert!(
                    reached[node.0],
                    "developed frontage is disconnected from the trade route"
                );
                while let Some((parent, connecting)) = parents[node.0] {
                    selected.insert(connecting);
                    node = parent;
                }
            }
        }
        let mut patches = selected
            .into_iter()
            .map(|id| {
                let edge = &self.edges[id.0];
                CityStreetPatch::Corridor {
                    start_metres: self.nodes[edge.nodes[0].0],
                    end_metres: self.nodes[edge.nodes[1].0],
                    half_width_metres: edge.class.half_width(),
                    surface: edge.class.surface(),
                }
            })
            .collect::<Vec<_>>();
        patches.push(CityStreetPatch::Market {
            corners_metres: self
                .blocks
                .iter()
                .find(|block| block.market)
                .unwrap()
                .corners,
            surface: CityStreetSurface::Fieldstone,
        });
        assert!(patches.len() <= MAX_CITY_STREET_PATCHES);
        patches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_quarter_has_shared_t_junctions_and_extension_has_regular_crossings() {
        for seed in [42, 47, 101] {
            let graph = CitySite::central_german_market_town()
                .street_graph(seed, DevelopmentExtent::for_population(40_000));
            let mut degrees = vec![0; graph.nodes.len()];
            for edge in &graph.edges {
                assert_ne!(edge.nodes[0], edge.nodes[1]);
                for node in edge.nodes {
                    degrees[node.0] += 1;
                }
            }
            let old_t = graph
                .nodes
                .iter()
                .zip(&degrees)
                .filter(|(point, degree)| point.x < 0.0 && point.x > -1000.0 && **degree == 3)
                .count();
            assert!(
                old_t > 50,
                "old quarter still has only four-way intersections"
            );
            assert!(
                graph
                    .nodes
                    .iter()
                    .zip(&degrees)
                    .filter(|(p, _)| p.x > 450.0 && p.x < 1000.0 && p.y.abs() < 700.0)
                    .all(|(_, degree)| *degree == 4)
            );
            assert_eq!(graph.blocks.iter().filter(|block| block.market).count(), 1);
            for block in &graph.blocks {
                for side in 0..4 {
                    let a = block.corners[side];
                    let b = block.corners[(side + 1) % 4];
                    let c = block.corners[(side + 2) % 4];
                    assert!((b - a).perp_dot(c - b) > 0.0, "inverted or concave block");
                }
            }
            for edge in &graph.edges {
                let [a, b] = edge.nodes.map(|id| graph.nodes[id.0]);
                let delta = b - a;
                for (index, point) in graph.nodes.iter().enumerate() {
                    if edge.nodes.contains(&StreetNodeId(index)) {
                        continue;
                    }
                    let progress = (*point - a).dot(delta) / delta.length_squared();
                    assert!(
                        progress <= 0.001
                            || progress >= 0.999
                            || point.distance(a + delta * progress) > 0.01,
                        "junction terminates inside an unsplit street edge"
                    );
                }
            }
        }
    }

    #[test]
    fn surveyed_anchor_inside_market_range_and_near_approaches_remains_valid() {
        let site = CitySite::from_trade_route([
            Vec2::new(-1400.0, -120.0),
            Vec2::new(-450.0, -80.0),
            Vec2::new(75.0, 0.0),
            Vec2::new(450.0, 50.0),
            Vec2::new(1400.0, 100.0),
        ])
        .unwrap();
        for seed in [42, 101] {
            let city = site.generate(seed, 900, &super::super::tests::economy());
            assert_eq!(city.unhoused_population, 0);
            assert!(city.unplaced_services.is_empty());
            assert!(city.streets.iter().all(|street| street.is_valid()));
            assert_eq!(
                city.streets
                    .iter()
                    .filter(|s| matches!(s, CityStreetPatch::Market { .. }))
                    .count(),
                1
            );
        }
    }

    #[test]
    fn short_surveyed_bends_and_external_anchors_survive_street_generation() {
        let anchors = [
            Vec2::new(-1800.0, -200.0),
            Vec2::new(-1500.0, 150.0),
            Vec2::new(0.5, 0.0),
            Vec2::new(1500.0, -150.0),
            Vec2::new(1800.0, 200.0),
        ];
        let city = CitySite::from_trade_route(anchors).unwrap().generate(
            42,
            900,
            &super::super::tests::economy(),
        );
        assert!(city.streets.iter().all(|patch| patch.is_valid()));
        let endpoints = city
            .streets
            .iter()
            .filter_map(|patch| match patch {
                CityStreetPatch::Corridor {
                    start_metres,
                    end_metres,
                    ..
                } => Some([*start_metres, *end_metres]),
                _ => None,
            })
            .flatten()
            .collect::<Vec<_>>();
        assert!(anchors.iter().all(|anchor| endpoints.contains(anchor)));
    }
}
