//! Wall pieces stay fixed while roof weathering is emitted. Index them once
//! rather than searching the growing geometry for every contact station.
use std::collections::{BTreeMap, BTreeSet};

use crate::{GeometryOwnerId, ResolvedBounds, ResolvedSolid, WallAssembly};

const POSITIVE_CONTACT_DEPTH_METRES: f32 = 0.025;

pub(super) struct RoofContacts(BTreeMap<GeometryOwnerId, Vec<ResolvedBounds>>);

impl RoofContacts {
    pub(super) fn new(walls: &[WallAssembly], solids: &[ResolvedSolid]) -> Self {
        let mut owners: BTreeMap<_, Vec<_>> =
            walls.iter().map(|wall| (wall.owner, Vec::new())).collect();
        for solid in solids {
            if let Some(bounds) = owners.get_mut(&solid.owner) {
                bounds.push(solid.yaw_bounds());
            }
        }
        Self(owners)
    }

    pub(super) fn touching(&self, weathering: &[ResolvedSolid]) -> BTreeSet<GeometryOwnerId> {
        let weather: Vec<_> = weathering.iter().map(ResolvedSolid::yaw_bounds).collect();
        self.0
            .iter()
            .filter_map(|(owner, hosts)| {
                hosts
                    .iter()
                    .any(|host| {
                        weather.iter().any(|weather| {
                            let min = host.min.max(weather.min);
                            let max = host.max.min(weather.max);
                            (max - min).min_element() > POSITIVE_CONTACT_DEPTH_METRES
                        })
                    })
                    .then_some(*owner)
            })
            .collect()
    }
}
