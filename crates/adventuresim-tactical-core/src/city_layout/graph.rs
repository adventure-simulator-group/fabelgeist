//! Shared street topology owns frontage, setbacks, and connections to the market.
use super::*;
use std::collections::{BTreeMap, VecDeque};

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct BlockId(pub u64);
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct StreetNodeId(usize);
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct StreetEdgeId(usize);

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct CityBlock {
    pub(super) id: BlockId,
    pub(super) corners: [ScenePlanPoint; 4],
    pub(super) streets: [StreetClass; 4],
    pub(super) use_role: BlockUse,
}

struct SelectedFrontages {
    edges: BTreeSet<StreetEdgeId>,
    market: CityBlock,
}

#[derive(Clone, Copy)]
struct StreetConnection {
    node: StreetNodeId,
    edge: StreetEdgeId,
}

struct StreetEdge {
    nodes: [StreetNodeId; 2],
    class: StreetClass,
}

#[derive(Default)]
pub(super) struct StreetGraph {
    pub(super) blocks: Vec<CityBlock>,
    nodes: Vec<ScenePlanPoint>,
    edges: Vec<StreetEdge>,
    node_lookup: BTreeMap<(u32, u32), StreetNodeId>,
    edge_lookup: BTreeMap<[StreetNodeId; 2], StreetEdgeId>,
    boundaries: BTreeMap<BlockId, Vec<StreetEdgeId>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BlockUse {
    StreetFrontage,
    Market,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum StreetClass {
    Lane,
    Secondary,
    TradeRoute,
}

impl StreetClass {
    pub(super) fn half_width(
        self,
    ) -> GeometryResult<adventuresim_building_generator::spatial_geometry::PositiveLength> {
        adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(match self {
            Self::Lane => ORDINARY_STREET_HALF_WIDTH_METRES,
            Self::Secondary => SECONDARY_STREET_HALF_WIDTH_METRES,
            Self::TradeRoute => PRIMARY_STREET_HALF_WIDTH_METRES,
        })
    }
    fn surface(self) -> CityStreetSurface {
        match self {
            Self::Lane => CityStreetSurface::CompactedEarth,
            Self::Secondary => CityStreetSurface::Gravel,
            Self::TradeRoute => CityStreetSurface::Fieldstone,
        }
    }
}

impl CityBlock {
    /// Native scene-plane coordinates for subdivision and half-plane kernels.
    pub(super) fn corners_metres(self) -> [Vec2; 4] {
        self.corners.map(ScenePlanPoint::metres)
    }
    pub(super) fn centre(self) -> Vec2 {
        self.corners_metres().into_iter().sum::<Vec2>() * 0.25
    }
    pub(super) fn key(self) -> BlockId {
        self.id
    }
    pub(super) fn is_market(self) -> bool {
        self.use_role == BlockUse::Market
    }
}

impl StreetGraph {
    /// Complete nearby blocks before opening scattered residential frontages.
    pub(super) fn development_order(&self) -> BTreeMap<BlockId, usize> {
        let mut blocks = self.blocks.iter().collect::<Vec<_>>();
        blocks.sort_by(|a, b| {
            a.centre()
                .length_squared()
                .total_cmp(&b.centre().length_squared())
                .then(a.id.cmp(&b.id))
        });
        blocks
            .into_iter()
            .enumerate()
            .map(|(rank, block)| (block.id, rank))
            .collect()
    }

    pub(super) fn segment(
        &mut self,
        points: [ScenePlanPoint; 2],
        class: StreetClass,
        block: Option<BlockId>,
    ) {
        let mut nodes = points.map(|point| {
            let native = point.metres();
            let key = (native.x.to_bits(), native.y.to_bits());
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

    pub(super) fn developed_streets(
        &self,
        developed: &BTreeSet<BlockId>,
    ) -> CityCompileResult<Vec<CityStreetPatch>> {
        if developed.is_empty() {
            return Ok(Vec::new());
        }
        let SelectedFrontages {
            edges: selected,
            market,
        } = self.selected_frontages(developed)?;
        let mut patches = selected
            .into_iter()
            .map(|id| {
                let edge = &self.edges[id.0];
                Ok::<_, GeometryError>(CityStreetPatch::Corridor {
                    start_metres: self.nodes[edge.nodes[0].0],
                    end_metres: self.nodes[edge.nodes[1].0],
                    half_width_metres: edge.class.half_width()?,
                    surface: edge.class.surface(),
                })
            })
            .collect::<GeometryResult<Vec<_>>>()?;
        patches.push(CityStreetPatch::Market {
            corners_metres: market.corners,
            surface: CityStreetSurface::Fieldstone,
        });
        if patches.len() > MAX_CITY_STREET_PATCHES {
            return Err(CityCompileError::Planning(
                CityPlanningIssue::StreetPatchLimit {
                    required: patches.len(),
                    maximum: MAX_CITY_STREET_PATCHES,
                },
            ));
        }
        Ok(patches)
    }

    fn selected_frontages(
        &self,
        developed: &BTreeSet<BlockId>,
    ) -> CityCompileResult<SelectedFrontages> {
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
        let market = self
            .blocks
            .iter()
            .find(|block| block.is_market())
            .ok_or(CityCompileError::Planning(CityPlanningIssue::MissingMarket))?;
        selected.extend(&self.boundaries[&market.id]);
        let mut adjacency = vec![Vec::new(); self.nodes.len()];
        for (index, edge) in self.edges.iter().enumerate() {
            let [a, b] = edge.nodes;
            adjacency[a.0].push(StreetConnection {
                node: b,
                edge: StreetEdgeId(index),
            });
            adjacency[b.0].push(StreetConnection {
                node: a,
                edge: StreetEdgeId(index),
            });
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
            for &StreetConnection { node: next, edge } in &adjacency[node.0] {
                if !reached[next.0] {
                    reached[next.0] = true;
                    parents[next.0] = Some(StreetConnection { node, edge });
                    queue.push_back(next);
                }
            }
        }
        for edge in selected.clone() {
            for mut node in self.edges[edge.0].nodes {
                if !reached[node.0] {
                    return Err(CityCompileError::Planning(
                        CityPlanningIssue::DisconnectedFrontage,
                    ));
                }
                while let Some(StreetConnection {
                    node: parent,
                    edge: connecting,
                }) = parents[node.0]
                {
                    selected.insert(connecting);
                    node = parent;
                }
            }
        }
        Ok(SelectedFrontages {
            edges: selected,
            market: *market,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_quarter_has_shared_t_junctions_and_extension_has_regular_crossings() {
        for seed in [42, 47, 101].map(fabelgeist_determinism::Seed::from_u64) {
            let graph = CitySite::central_german_market_town()
                .unwrap()
                .street_graph(
                    seed,
                    DevelopmentExtent::for_population(ResidentCount::new(40_000)).unwrap(),
                )
                .unwrap();
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
                .filter(|(point, degree)| {
                    point.metres().x < 0.0 && point.metres().x > -1000.0 && **degree == 3
                })
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
                    .filter(|(p, _)| p.metres().x > 450.0
                        && p.metres().x < 1000.0
                        && p.metres().y.abs() < 700.0)
                    .all(|(_, degree)| *degree == 4)
            );
            assert_eq!(
                graph
                    .blocks
                    .iter()
                    .filter(|block| block.is_market())
                    .count(),
                1
            );
            for block in &graph.blocks {
                for side in 0..4 {
                    let a = block.corners_metres()[side];
                    let b = block.corners_metres()[(side + 1) % 4];
                    let c = block.corners_metres()[(side + 2) % 4];
                    assert!((b - a).perp_dot(c - b) > 0.0, "inverted or concave block");
                }
            }
            for edge in &graph.edges {
                let [a, b] = edge.nodes.map(|id| graph.nodes[id.0].metres());
                let delta = b - a;
                for (index, point) in graph.nodes.iter().enumerate() {
                    if edge.nodes.contains(&StreetNodeId(index)) {
                        continue;
                    }
                    let progress = (point.metres() - a).dot(delta) / delta.length_squared();
                    assert!(
                        progress <= 0.001
                            || progress >= 0.999
                            || point.metres().distance(a + delta * progress) > 0.01,
                        "junction terminates inside an unsplit street edge"
                    );
                }
            }
        }
    }

    #[test]
    fn surveyed_anchor_inside_market_range_and_near_approaches_remains_valid() {
        let site = CitySite::from_trade_route(
            [
                Vec2::new(-1400.0, -120.0),
                Vec2::new(-450.0, -80.0),
                Vec2::new(75.0, 0.0),
                Vec2::new(450.0, 50.0),
                Vec2::new(1400.0, 100.0),
            ]
            .map(|point| ScenePlanPoint::try_from(point).unwrap()),
        )
        .unwrap();
        for seed in [42, 101].map(fabelgeist_determinism::Seed::from_u64) {
            let city = site
                .generate(
                    seed,
                    adventuresim_core::settlement_property::ResidentCount::new(900),
                    &super::super::tests::economy(),
                )
                .unwrap();
            assert_eq!(city.unhoused_population, ResidentCount::ZERO);
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
        let city = CitySite::from_trade_route(
            anchors.map(|point| ScenePlanPoint::try_from(point).unwrap()),
        )
        .unwrap()
        .generate(
            (42).into(),
            adventuresim_core::settlement_property::ResidentCount::new(900),
            &super::super::tests::economy(),
        )
        .unwrap();
        assert!(city.streets.iter().all(|patch| patch.is_valid()));
        let endpoints = city
            .streets
            .iter()
            .filter_map(|patch| match patch {
                CityStreetPatch::Corridor {
                    start_metres,
                    end_metres,
                    ..
                } => {
                    let start_metres = start_metres.metres();
                    let end_metres = end_metres.metres();
                    Some([start_metres, end_metres])
                }
                _ => None,
            })
            .flatten()
            .collect::<Vec<_>>();
        assert!(anchors.iter().all(|anchor| endpoints.contains(anchor)));
    }
}
