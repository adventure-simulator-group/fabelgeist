//! Candidate furniture changes only the affected navigation nodes and edges.
use super::*;
use crate::interior::InteriorResult as Result;
use crate::spatial_geometry::GeometryResult;
use crate::spatial_geometry::PlanExtents;

pub(in crate::interior) struct Occupancy<'a> {
    nav: &'a Navigation,
    nodes: Vec<u32>,
    edges: Vec<Vec<u32>>,
    swept: Vec<Vec<Option<Rect>>>,
}

pub(in crate::interior) struct Change {
    nodes: Vec<usize>,
    edges: Vec<(usize, usize)>,
}

impl<'a> Occupancy<'a> {
    pub fn new(nav: &'a Navigation) -> Result<Self> {
        let swept = nav
            .edges
            .iter()
            .enumerate()
            .map(|(index, edges)| {
                let a = nav.nodes[index];
                edges
                    .iter()
                    .map(|&next| {
                        let b = nav.nodes[next];
                        (a.storey == b.storey)
                            .then(|| {
                                Rect::new(
                                    ArchitecturalPlanPoint::from_metres(
                                        (a.position_metres.metres() + b.position_metres.metres())
                                            * 0.5,
                                    )?,
                                    PlanExtents::from_metres(
                                        (a.position_metres.metres() - b.position_metres.metres())
                                            .abs()
                                            * 0.5
                                            + Vec2::splat(PERSON_RADIUS),
                                    )?,
                                )
                            })
                            .transpose()
                    })
                    .collect::<GeometryResult<Vec<_>>>()
            })
            .collect::<GeometryResult<Vec<_>>>()?;
        Ok(Self {
            nav,
            nodes: vec![0; nav.nodes.len()],
            edges: nav.edges.iter().map(|edges| vec![0; edges.len()]).collect(),
            swept,
        })
    }

    pub fn add(&mut self, placements: &[InteriorPlacement]) -> Result<Change> {
        let mut change = Change {
            nodes: Vec::new(),
            edges: Vec::new(),
        };
        for placement in placements {
            let footprint = placement.footprint()?;
            let expanded = footprint.expanded(
                crate::spatial_geometry::SignedLength::from_metres(PERSON_RADIUS)?,
            )?;
            for (index, node) in self.nav.nodes.iter().enumerate() {
                if node.storey != placement.storey {
                    continue;
                }
                if expanded.contains(node.position_metres) {
                    self.nodes[index] += 1;
                    change.nodes.push(index);
                }
                for (edge, swept) in self.swept[index].iter().enumerate() {
                    if swept.is_some_and(|swept| footprint.overlaps(swept)) {
                        self.edges[index][edge] += 1;
                        change.edges.push((index, edge));
                    }
                }
            }
        }
        Ok(change)
    }

    pub fn remove(&mut self, change: Change) {
        for index in change.nodes {
            self.nodes[index] -= 1;
        }
        for (index, edge) in change.edges {
            self.edges[index][edge] -= 1;
        }
    }

    pub fn flood(&self) -> Flood {
        let mut parents = vec![None; self.nodes.len()];
        let mut queue = VecDeque::new();
        if self.nodes[self.nav.entry] == 0 {
            parents[self.nav.entry] = Some(self.nav.entry);
            queue.push_back(self.nav.entry);
        }
        while let Some(index) = queue.pop_front() {
            for (&next, &blocked) in self.nav.edges[index].iter().zip(&self.edges[index]) {
                if self.nodes[next] != 0 || blocked != 0 || parents[next].is_some() {
                    continue;
                }
                parents[next] = Some(index);
                queue.push_back(next);
            }
        }
        Flood {
            parents,
            entry: self.nav.entry,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reference(nav: &Navigation, placements: &[InteriorPlacement]) -> Vec<Option<usize>> {
        let mut parents = vec![None; nav.nodes.len()];
        let blocked = |index: usize| {
            placements.iter().any(|p| {
                p.storey == nav.nodes[index].storey
                    && p.footprint()
                        .unwrap()
                        .expanded(
                            crate::spatial_geometry::SignedLength::from_metres(PERSON_RADIUS)
                                .unwrap(),
                        )
                        .unwrap()
                        .contains(nav.nodes[index].position_metres)
            })
        };
        let mut queue = VecDeque::new();
        if !blocked(nav.entry) {
            parents[nav.entry] = Some(nav.entry);
            queue.push_back(nav.entry);
        }
        while let Some(index) = queue.pop_front() {
            for &next in &nav.edges[index] {
                if blocked(next) || parents[next].is_some() {
                    continue;
                }
                let a = nav.nodes[index];
                let b = nav.nodes[next];
                let swept = Rect::new(
                    ArchitecturalPlanPoint::from_metres(
                        (a.position_metres.metres() + b.position_metres.metres()) * 0.5,
                    )
                    .unwrap(),
                    PlanExtents::from_metres(
                        (a.position_metres.metres() - b.position_metres.metres()).abs() * 0.5
                            + Vec2::splat(PERSON_RADIUS),
                    )
                    .unwrap(),
                )
                .unwrap();
                if a.storey == b.storey
                    && placements
                        .iter()
                        .any(|p| p.storey == a.storey && p.footprint().unwrap().overlaps(swept))
                {
                    continue;
                }
                parents[next] = Some(index);
                queue.push_back(next);
            }
        }
        parents
    }

    #[test]
    fn incremental_candidates_preserve_paths_and_rejected_overlap_restores_graph() {
        use adventuresim_world_schema::settlement_buildings::BuildingUse;
        let program = crate::BuildingProgram::validated_settlement(
            crate::settlement_archetype(BuildingUse::Hospital),
            BuildingUse::Hospital,
            42,
            None,
        )
        .unwrap();
        let plan = crate::generate(&program).unwrap();
        let layout = crate::interior::furnish(&plan, &program).unwrap();
        let nav = Navigation::new(&plan).unwrap();
        let mut occupancy = Occupancy::new(&nav).unwrap();
        for length in 1..=layout.placements.len() {
            let candidate = &layout.placements[length - 1..length];
            occupancy.add(candidate).unwrap();
            let expected = reference(&nav, &layout.placements[..length]);
            assert_eq!(occupancy.flood().parents, expected);
            let duplicate = occupancy.add(candidate).unwrap();
            occupancy.remove(duplicate);
            assert_eq!(occupancy.flood().parents, expected);
        }
        let mut obstruction = layout.placements[0].clone();
        obstruction.storey = nav.nodes[nav.entry].storey;
        obstruction.centre_metres = nav.nodes[nav.entry].position_metres;
        let change = occupancy.add(&[obstruction]).unwrap();
        assert!(occupancy.flood().parents.iter().all(Option::is_none));
        occupancy.remove(change);
        assert!(occupancy.flood().parents[nav.entry].is_some());
    }
}
