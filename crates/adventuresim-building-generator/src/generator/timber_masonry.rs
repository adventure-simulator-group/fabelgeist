//! Ground-storey masonry contacts for upper timber sills and jetty braces.
use super::*;

impl TimberFrameBuilder<'_> {
    pub(super) fn resolve_masonry_bearings(
        &mut self,
        walls: &[crate::WallAssembly],
        storey_height_metres: f32,
    ) -> Vec<ResolvedItemId> {
        let mut masonry_bearing_interfaces = Vec::new();
        if walls.iter().any(|wall| {
            wall.storey_level == 0 && wall.material == crate::WallMaterialClass::CivilianMasonry
        }) {
            let sill_contacts = self
                .members
                .iter()
                .filter(|member| {
                    member.role == crate::TimberMemberRole::Sill
                        && (member.start.y - storey_height_metres).abs() <= 0.01
                })
                .flat_map(|member| {
                    [
                        (member.start_node, member.support_interfaces[0]),
                        (member.end_node, member.support_interfaces[1]),
                    ]
                })
                .chain(
                    self.members
                        .iter()
                        .filter(|member| member.role == crate::TimberMemberRole::Knagge)
                        .map(|member| (member.start_node, member.support_interfaces[0])),
                )
                .collect::<Vec<_>>();
            for (node_id, interface_id) in sill_contacts {
                let Some(interface) = self
                    .geometry
                    .support_interfaces
                    .iter()
                    .find(|interface| interface.id == interface_id)
                    .copied()
                else {
                    continue;
                };
                let masonry_support = walls
                    .iter()
                    .filter(|wall| {
                        wall.storey_level == 0
                            && wall.material == crate::WallMaterialClass::CivilianMasonry
                    })
                    .find(|wall| {
                        wall.host_solids.iter().any(|id| {
                            self.geometry
                                .solids
                                .iter()
                                .find(|solid| solid.id == *id)
                                .is_some_and(|solid| {
                                    let half = solid.size * 0.5 + Vec3::splat(0.01);
                                    let min = solid.centre - half;
                                    let max = solid.centre + half;
                                    interface.bounds.max.cmpge(min).all()
                                        && interface.bounds.min.cmple(max).all()
                                })
                        })
                    })
                    .map(|wall| wall.support_node);
                if let Some(support) = masonry_support
                    && let Some(node) = self
                        .geometry
                        .structural_nodes
                        .iter_mut()
                        .find(|node| node.id == node_id)
                {
                    node.supported_by.push(support);
                    node.supported_by.sort_unstable();
                    node.supported_by.dedup();
                    masonry_bearing_interfaces.push(interface_id);
                }
            }
        }

        masonry_bearing_interfaces
    }
}
