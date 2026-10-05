//! Propagate continuous free intervals before selecting another translation.
use super::*;

pub(super) struct BlockSearch<'a> {
    pub(super) domains: &'a [PlacementDomain],
    states: usize,
    rejection: Option<CityCompileError>,
}
impl<'a> BlockSearch<'a> {
    pub fn new(domains: &'a [PlacementDomain]) -> Self {
        Self {
            domains,
            states: 0,
            rejection: None,
        }
    }
    pub fn solve(mut self) -> Result<Vec<PropertyTranslation>, CityCompileError> {
        let mut accepted = Vec::new();
        if self.search(&mut accepted)? {
            return accepted
                .into_iter()
                .map(|selected| {
                    let domain = &self.domains[selected.domain.index()];
                    Ok(PropertyTranslation {
                        domain: selected.domain,
                        displacement: domain
                            .delta_at(selected.frontage_displacement)
                            .map_err(|issue| domain.packing_error(issue))?,
                    })
                })
                .collect();
        }
        #[cfg(test)]
        self.dump_rejection();
        super::coupled::solve(
            self.domains,
            MAX_BLOCK_PACKING_SEARCH_STATES.saturating_sub(self.states),
        )
    }
    fn free_intervals(
        &self,
        index: PackingDomainIndex,
        accepted: &[FrontageSelection],
    ) -> Vec<FrontageInterval> {
        let domain = &self.domains[index.index()];
        let mut allowed = Some(domain.allowed);
        for selected in accepted {
            let other = selected.domain;
            let frontage_displacement = selected.frontage_displacement;
            allowed = allowed.and_then(|range| {
                self.preserve_frontage_order(index, other, frontage_displacement, range)
            });
        }
        let mut free = allowed.into_iter().collect::<Vec<_>>();
        for selected in accepted {
            let other = selected.domain;
            let frontage_displacement = selected.frontage_displacement;
            let occupied = &self.domains[other.index()];
            let delta = occupied.frontage.tangent().as_dvec2() * frontage_displacement.metres();
            for forbidden in domain.proposed.forbidden_displacements(
                &occupied.proposed,
                domain.frontage.tangent(),
                delta,
            ) {
                free = free
                    .into_iter()
                    .flat_map(|interval| interval.without(forbidden))
                    .collect();
            }
        }
        free
    }
    fn preserve_frontage_order(
        &self,
        index: PackingDomainIndex,
        other: PackingDomainIndex,
        frontage_displacement: FrontageDisplacement,
        range: FrontageInterval,
    ) -> Option<FrontageInterval> {
        let first = &self.domains[index.index()];
        let second = &self.domains[other.index()];
        if first.frontage.edge != second.frontage.edge {
            return Some(range);
        }
        let tangent = first.frontage.tangent().as_dvec2();
        let projection = |geometry: &ParcelGeometry| {
            geometry
                .reservation
                .corners()
                .into_iter()
                .map(|point| point.as_dvec2().dot(tangent))
                .fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), p| {
                    (a.min(p), b.max(p))
                })
        };
        let (a, b) = projection(&first.proposed);
        let (c, d) = projection(&second.proposed);
        let rate = tangent.length_squared();
        if (a + b)
            .total_cmp(&(c + d))
            .then(first.owner.cmp(&second.owner))
            .is_lt()
        {
            range.with_half_plane(
                c + frontage_displacement.metres() * rate
                    - b
                    - CityPlotBounds::COORDINATE_TOLERANCE_METRES,
                -rate,
            )
        } else {
            range.with_half_plane(
                a - d
                    - frontage_displacement.metres() * rate
                    - CityPlotBounds::COORDINATE_TOLERANCE_METRES,
                rate,
            )
        }
    }
    pub(super) fn propagate_frontage_capacity(&self, candidates: &mut [DomainChoices]) {
        for edge in 0..4 {
            let tangent = self.domains[0].frontage.block.corners[(edge + 1) % 4]
                - self.domains[0].frontage.block.corners[edge];
            let mut row: Vec<_> = (0..candidates.len())
                .filter(|&i| self.domains[candidates[i].domain.index()].frontage.edge == edge)
                .collect();
            row.sort_by(|&a, &b| {
                let projection = |index: usize| {
                    self.domains[candidates[index].domain.index()]
                        .proposed
                        .reservation
                        .centre_metres
                        .as_dvec2()
                        .dot(tangent.as_dvec2())
                };
                projection(a).total_cmp(&projection(b)).then(
                    self.domains[candidates[a].domain.index()]
                        .owner
                        .cmp(&self.domains[candidates[b].domain.index()].owner),
                )
            });
            for (rank, &left) in row.iter().enumerate() {
                for &right in &row[rank + 1..] {
                    let gap = self.row_gap(candidates[left].domain, candidates[right].domain);
                    if let Some(first) = candidates[left].intervals.first() {
                        let minimum = first.minimum_metres + gap;
                        candidates[right].intervals = candidates[right]
                            .intervals
                            .iter()
                            .filter_map(|range| range.with_half_plane(-minimum, 1.0))
                            .collect();
                    }
                }
            }
            for (rank, &right) in row.iter().enumerate().rev() {
                for &left in &row[..rank] {
                    let gap = self.row_gap(candidates[left].domain, candidates[right].domain);
                    if let Some(last) = candidates[right].intervals.last() {
                        let maximum = last.maximum_metres - gap;
                        candidates[left].intervals = candidates[left]
                            .intervals
                            .iter()
                            .filter_map(|range| range.with_half_plane(maximum, -1.0))
                            .collect();
                    }
                }
            }
        }
    }
    fn row_gap(&self, first: PackingDomainIndex, second: PackingDomainIndex) -> f64 {
        let a = &self.domains[first.index()];
        let b = &self.domains[second.index()];
        a.proposed
            .forbidden_displacements(&b.proposed, a.frontage.tangent(), DVec2::ZERO)
            .iter()
            .map(|forbidden| -forbidden.minimum_metres)
            .fold(f64::NEG_INFINITY, f64::max)
    }
    fn next_domain(&mut self, accepted: &[FrontageSelection]) -> Option<DomainChoices> {
        let mut candidates = Vec::new();
        for (ordinal, domain) in self.domains.iter().enumerate() {
            let index = PackingDomainIndex::new(ordinal);
            if accepted.iter().any(|selected| selected.domain == index) {
                continue;
            }
            let free = self.free_intervals(index, accepted);
            if free.is_empty() {
                self.rejection = Some(CityCompileError::Packing {
                    property: domain.owner,
                    issue: CityPackingIssue::NoFreeFrontage {
                        block: domain.frontage.block.id.0,
                        envelope: domain.proposed.reservation,
                        available_displacement_metres: Some(domain.allowed),
                        blocking_properties: accepted
                            .iter()
                            .map(|selected| self.domains[selected.domain.index()].owner)
                            .collect(),
                    },
                });
                return None;
            }
            candidates.push(DomainChoices {
                domain: index,
                intervals: free,
            });
        }
        self.propagate_domains(&mut candidates);
        if let Some(candidate) = candidates
            .iter()
            .find(|candidate| candidate.intervals.is_empty())
        {
            let domain = &self.domains[candidate.domain.index()];
            self.rejection = Some(CityCompileError::Packing {
                property: domain.owner,
                issue: CityPackingIssue::NoFreeFrontage {
                    block: domain.frontage.block.id.0,
                    envelope: domain.proposed.reservation,
                    available_displacement_metres: Some(domain.allowed),
                    blocking_properties: self
                        .domains
                        .iter()
                        .filter(|other| {
                            other.frontage.edge == domain.frontage.edge
                                && other.owner != domain.owner
                        })
                        .map(|other| other.owner)
                        .collect(),
                },
            });
            return None;
        }
        candidates.into_iter().min_by(|a, b| {
            let room = |intervals: &[FrontageInterval]| {
                intervals
                    .iter()
                    .map(|i| i.maximum_metres - i.minimum_metres)
                    .sum::<f64>()
            };
            self.domains[a.domain.index()]
                .frontage
                .priority()
                .cmp(&self.domains[b.domain.index()].frontage.priority())
                .then_with(|| room(&a.intervals).total_cmp(&room(&b.intervals)))
                .then(
                    self.domains[a.domain.index()]
                        .owner
                        .cmp(&self.domains[b.domain.index()].owner),
                )
        })
    }
    fn candidates(
        &self,
        index: PackingDomainIndex,
        free: &[FrontageInterval],
        accepted: &[FrontageSelection],
    ) -> Vec<f64> {
        let domain = &self.domains[index.index()];
        let mut candidates: Vec<_> = free
            .iter()
            .flat_map(|interval| {
                [
                    interval.nearest_origin(),
                    interval.minimum_metres,
                    interval.maximum_metres,
                ]
            })
            .collect();
        // Future original poses supply contact boundaries, allowing an earlier
        // service to move just enough rather than consuming a later home's slot.
        for (other, remaining) in self.domains.iter().enumerate() {
            if other == index.index()
                || accepted
                    .iter()
                    .any(|selected| selected.domain.index() == other)
            {
                continue;
            }
            for forbidden in domain.proposed.forbidden_displacements(
                &remaining.proposed,
                domain.frontage.tangent(),
                DVec2::ZERO,
            ) {
                candidates.extend([forbidden.minimum_metres, forbidden.maximum_metres]);
            }
        }
        candidates.retain(|value| {
            value.is_finite()
                && free.iter().any(|interval| {
                    *value >= interval.minimum_metres && *value <= interval.maximum_metres
                })
        });
        candidates.sort_by(|a, b| a.abs().total_cmp(&b.abs()).then(a.total_cmp(b)));
        candidates.dedup_by(|a, b| (*a - *b).abs() <= CityPlotBounds::COORDINATE_TOLERANCE_METRES);
        candidates
    }
    fn search(&mut self, accepted: &mut Vec<FrontageSelection>) -> Result<bool, CityCompileError> {
        if accepted.len() == self.domains.len() {
            let mut geometry = self
                .domains
                .iter()
                .map(PlacementDomain::geometry_at_zero)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|issue| self.domains[0].packing_error(issue))?;
            for selected in accepted.iter() {
                let index = selected.domain;
                let frontage_displacement = selected.frontage_displacement;
                let domain = &self.domains[index.index()];
                geometry[index.index()] = domain
                    .proposed
                    .translated(
                        domain
                            .delta_at(frontage_displacement)
                            .map_err(|issue| domain.packing_error(issue))?,
                        domain.frontage.tangent(),
                    )
                    .map_err(|issue| domain.packing_error(domain.geometry_error(issue)))?;
            }
            // Candidate pruning uses land and building intervals. Complete
            // rounded garden geometry remains an acceptance constraint, so
            // valid authored poses need no extra numerical garden setback.
            return Ok(geometry
                .iter()
                .all(|owner| geometry.iter().all(|other| owner.garden_clears(other))));
        }
        // Preserve the inexpensive authored search allocation independently of
        // the coupled solver's measured block-search budget.
        if self.states >= MAX_AUTHORED_PACKING_SEARCH_STATES {
            return Ok(false);
        }
        let Some(choices) = self.next_domain(accepted) else {
            return Ok(false);
        };
        for displacement in self.candidates(choices.domain, &choices.intervals, accepted) {
            self.states += 1;
            accepted.push(FrontageSelection {
                domain: choices.domain,
                frontage_displacement: FrontageDisplacement::from_metres(displacement).ok_or_else(
                    || {
                        self.domains[choices.domain.index()]
                            .packing_error(CoupledPackingIssue::NumericalFailure)
                    },
                )?,
            });
            if self.search(accepted)? {
                return Ok(true);
            }
            accepted.pop();
            if self.states >= MAX_AUTHORED_PACKING_SEARCH_STATES {
                break;
            }
        }
        Ok(false)
    }
}

#[cfg(test)]
impl BlockSearch<'_> {
    fn dump_rejection(&mut self) {
        let Some(path) = std::env::var_os("FABELGEIST_SUPPORT_DIAGNOSTIC_DIR") else {
            return;
        };
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(path);
        std::fs::create_dir_all(&path).unwrap();
        let candidates: Vec<_> = self
            .domains
            .iter()
            .enumerate()
            .map(|(index, _)| DomainChoices {
                domain: PackingDomainIndex::new(index),
                intervals: self.free_intervals(PackingDomainIndex::new(index), &[]),
            })
            .collect();
        let mut propagated = candidates.clone();
        self.propagate_frontage_capacity(&mut propagated);
        let domains: Vec<_> = self.domains.iter().enumerate().map(|(index, domain)| serde_json::json!({
            "owner":domain.owner,"edge":domain.frontage.edge,"tangent":domain.frontage.tangent(),
            "allowed":domain.allowed,"reservation":domain.proposed.reservation,
            "buildings":domain.proposed.buildings,"bearings":domain.proposed.bearings,
            "free":candidates[index].intervals,"propagated":propagated[index].intervals,
            "pairs":self.domains.iter().enumerate().filter(|(i,_)|*i!=index).map(|(_,other)|serde_json::json!({"other":other.owner,"forbidden":domain.proposed.forbidden_displacements(&other.proposed,domain.frontage.tangent(),DVec2::ZERO)})).collect::<Vec<_>>()
        })).collect();
        std::fs::write(path.join(format!("packing-block-{}.json", self.domains[0].frontage.block.id.0)), serde_json::to_vec_pretty(&serde_json::json!({"states":self.states,"domains":domains,"block":format!("{:?}",self.domains[0].frontage.block)})).unwrap()).unwrap();
    }
}
