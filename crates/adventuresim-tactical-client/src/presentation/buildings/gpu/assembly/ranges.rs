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

impl InstanceRanges {
    pub fn new(jobs: Vec<UVec4>) -> Self {
        let mut groups = HashMap::<(u32, u32, u32), Vec<u32>>::new();
        for job in jobs {
            groups.entry((job.y, job.z, job.w)).or_default().push(job.x);
        }
        // Stable ordering makes uploads and cache products deterministic.
        let mut groups: Vec<_> = groups.into_iter().collect();
        groups.sort_unstable_by_key(|(key, _)| *key);
        let mut lists = HashMap::<Vec<u32>, u32>::new();
        let mut result = Self {
            ranges: Vec::new(),
            owners: Vec::new(),
            capacity: 0,
        };
        for ((start, count, flags), mut owners) in groups {
            owners.sort_unstable();
            let length = owners.len() as u32;
            let owner_start = *lists.entry(owners.clone()).or_insert_with(|| {
                let start = result.owners.len() as u32;
                result.owners.extend(owners);
                start
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn many_placements_share_ranges_and_owner_lists_without_losing_draws() {
        let jobs = (0..1000)
            .flat_map(|owner| [UVec4::new(owner, 0, 12, 1), UVec4::new(owner, 12, 384, 2)])
            .collect();
        let grouped = InstanceRanges::new(jobs);
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
