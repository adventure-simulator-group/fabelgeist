//! A geometry range references a shared list of building placements.
use super::*;

#[derive(Clone, ShaderType)]
pub(in crate::presentation::buildings::gpu) struct DrawRange {
    pub geometry: UVec4,
    pub instances: UVec4,
}

pub(super) struct InstanceRanges {
    pub ranges: Vec<DrawRange>,
    pub owners: Vec<u32>,
    pub capacity: u32,
}

/// Group identities stay compact even when thousands of pieces share thousands
/// of owners. Expand an owner list only once per distinct combination of groups.
#[derive(Default)]
pub(super) struct RangeGroups {
    owners: Vec<Arc<[u32]>>,
    identities: HashMap<*const u32, usize>,
    ranges: HashMap<(u32, u32, u32), Vec<usize>>,
}

impl RangeGroups {
    pub fn push(&mut self, range: (u32, u32, u32), owners: Arc<[u32]>) {
        let index = *self.identities.entry(owners.as_ptr()).or_insert_with(|| {
            let index = self.owners.len();
            self.owners.push(owners);
            index
        });
        self.ranges.entry(range).or_default().push(index);
    }

    pub fn finish(self) -> InstanceRanges {
        let mut ranges: Vec<_> = self.ranges.into_iter().collect();
        ranges.sort_unstable_by_key(|(key, _)| *key);
        let mut lists = HashMap::<Vec<usize>, (u32, u32)>::new();
        let mut result = InstanceRanges {
            ranges: Vec::new(),
            owners: Vec::new(),
            capacity: 0,
        };
        for ((start, count, flags), mut groups) in ranges {
            groups.sort_unstable();
            let (owner_start, length) = *lists.entry(groups.clone()).or_insert_with(|| {
                let mut owners: Vec<_> = groups
                    .iter()
                    .flat_map(|&group| self.owners[group].iter().copied())
                    .collect();
                owners.sort_unstable();
                let start = result.owners.len() as u32;
                let length = owners.len() as u32;
                result.owners.extend(owners);
                (start, length)
            });
            let clusters = count.div_ceil(VERTICES_PER_CLUSTER);
            assert!(
                (owner_start + length).checked_mul(clusters).is_some(),
                "bounded city draw address"
            );
            result.capacity = result
                .capacity
                .checked_add(clusters * length)
                .expect("bounded city draw capacity");
            result.ranges.push(DrawRange {
                geometry: UVec4::new(owner_start, start, count, flags),
                instances: UVec4::new(length, clusters, VERTICES_PER_CLUSTER, 0),
            });
        }
        result
    }
}

impl InstanceRanges {
    pub fn unshared_ranges(&self) -> usize {
        self.ranges
            .iter()
            .map(|range| range.instances.x as usize)
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlapping_prototypes_and_duplicate_components_keep_every_draw() {
        let first: Arc<[u32]> = vec![2, 0].into();
        let second: Arc<[u32]> = vec![1].into();
        let mut groups = RangeGroups::default();
        groups.push((0, 12, 1), first.clone());
        groups.push((0, 12, 1), second.clone());
        groups.push((0, 12, 1), first.clone());
        groups.push((12, 12, 2), first);
        groups.push((12, 12, 2), second);
        let grouped = groups.finish();
        let owners = |range: &DrawRange| {
            &grouped.owners
                [range.geometry.x as usize..(range.geometry.x + range.instances.x) as usize]
        };
        assert_eq!(owners(&grouped.ranges[0]), &[0, 0, 1, 2, 2]);
        assert_eq!(owners(&grouped.ranges[1]), &[0, 1, 2]);
        assert_eq!(grouped.capacity, 8);
        assert_eq!(grouped.unshared_ranges(), 8);
    }

    #[test]
    fn many_placements_share_ranges_and_owner_lists_without_losing_draws() {
        let owners: Arc<[u32]> = (0..1000).collect();
        let mut groups = RangeGroups::default();
        groups.push((0, 12, 1), owners.clone());
        groups.push((12, 384, 2), owners);
        let grouped = groups.finish();
        assert_eq!(grouped.ranges.len(), 2);
        assert_eq!(grouped.owners.len(), 1000);
        assert_eq!(grouped.capacity, 3000);
        for range in &grouped.ranges {
            assert_eq!(range.geometry.x, 0);
            assert_eq!(range.instances.x, 1000);
            for placement in 0..range.instances.x {
                for cluster in 0..range.instances.y {
                    let encoded = placement * range.instances.y + cluster;
                    assert_eq!(
                        grouped.owners[(encoded / range.instances.y) as usize],
                        placement
                    );
                    assert_eq!(encoded % range.instances.y, cluster);
                }
            }
        }
    }
}
