//! Measured contact between frame endpoints and containing roof or wall hosts.
use super::*;
struct EndpointBearing {
    node: StructuralNodeId,
    interface: ResolvedItemId,
    role: crate::TimberMemberRole,
}
impl TimberFrameBuilder<'_> {
    pub(super) fn member_interface(
        &mut self,
        node: StructuralNodeId,
        point: Vec3,
        section: Vec2,
    ) -> Result<ResolvedItemId, GenerationError> {
        let interface = ResolvedItemId(
            (4_u64 << 60) | (u64::from(self.owner.0) << 32) | 0x100_000 | self.next_interface,
        );
        self.next_interface += 1;
        let half = Vec3::new(section.x, section.x.min(section.y), section.y) * 0.5;
        self.geometry
            .support_interfaces
            .push(crate::SupportInterface::new(
                interface,
                self.owner,
                node,
                SpatialBounds::<Architectural>::from_metres(point - half, point + half)?,
            ));
        Ok::<_, crate::GenerationError>(interface)
    }
    pub(super) fn bind_child_bearings(
        &mut self,
        bays: &[crate::TimberFrameBay],
        walls: &[crate::WallAssembly],
        roof_assemblies: &[RoofAssembly],
    ) {
        // Bind dormer curbs and child fronts to the authoritative Stage 4 roof
        // framing / Stage 3 child-wall hosts only where an endpoint interface has
        // positive physical contact. This intentionally replaces the former
        // ground-to-dormer posts, which pierced the parent roof and drainage.
        let endpoint_contacts = self
            .members
            .iter()
            .filter(|member| {
                member.role == crate::TimberMemberRole::DormerTrimmer
                    || bays.iter().any(|bay| {
                        bay.member_ids.contains(&member.id)
                            && bay.wall.is_some_and(|wall_id| {
                                walls.iter().any(|wall| {
                                    wall.id == wall_id
                                        && matches!(
                                            wall.source,
                                            crate::WallSourceId::RoofChildFront { .. }
                                        )
                                })
                            })
                    })
            })
            .flat_map(|member| {
                [
                    EndpointBearing {
                        node: member.start_node,
                        interface: member.support_interfaces[0],
                        role: member.role,
                    },
                    EndpointBearing {
                        node: member.end_node,
                        interface: member.support_interfaces[1],
                        role: member.role,
                    },
                ]
            })
            .collect::<Vec<_>>();
        for bearing in endpoint_contacts {
            self.bind_child_bearing(bearing, walls, roof_assemblies);
        }
    }
    fn bind_child_bearing(
        &mut self,
        bearing: EndpointBearing,
        walls: &[crate::WallAssembly],
        roof_assemblies: &[RoofAssembly],
    ) {
        let Some(interface) = self
            .geometry
            .support_interfaces
            .iter()
            .find(|interface| interface.id == bearing.interface)
            .cloned()
        else {
            return;
        };
        let overlaps = |solid: &ResolvedSolid| {
            let half = solid.size.metres() * 0.5;
            let min = solid.centre.metres() - half;
            let max = solid.centre.metres() + half;
            let overlap =
                interface.bounds.max().metres().min(max) - interface.bounds.min().metres().max(min);
            overlap.cmpgt(Vec3::splat(0.001)).all()
        };
        let mut external_supports = self
            .geometry
            .solids
            .iter()
            .filter(|solid| {
                matches!(
                    solid.role,
                    SolidRole::RoofFace | SolidRole::RoofFraming | SolidRole::RoofPlate
                ) && overlaps(solid)
            })
            .flat_map(|solid| solid.supported_by.iter().copied())
            .collect::<Vec<_>>();
        if bearing.role == crate::TimberMemberRole::DormerTrimmer {
            let node_position = self
                .geometry
                .structural_nodes
                .iter()
                .find(|node| node.id == bearing.node)
                .map(|node| node.position);
            if let Some(position) = node_position {
                let plan = Vec2::new(position.metres().x, position.metres().z);
                for parent_roof in roof_assemblies.iter().filter(|roof| roof.parent.is_none()) {
                    let on_parent_plane = parent_roof.faces.iter().any(|face| {
                        let outline = face
                            .polygon
                            .iter()
                            .map(|point| Vec2::new(point.x, point.z))
                            .collect::<Vec<_>>();
                        let inside_face = plan_point_in_polygon(plan, &outline)
                            && !face.cutouts.iter().any(|cutout| {
                                let cutout = cutout
                                    .iter()
                                    .map(|point| Vec2::new(point.x, point.z))
                                    .collect::<Vec<_>>();
                                plan_point_in_polygon(plan, &cutout)
                            });
                        let underside = face.underside_height_at(plan);
                        inside_face && (underside - position.metres().y).abs() <= 0.03
                    });
                    if on_parent_plane {
                        external_supports.extend(parent_roof.support_nodes.iter().copied());
                    }
                }
            }
        }
        for wall in walls.iter().filter(|wall| {
            matches!(wall.source, crate::WallSourceId::RoofChildFront { .. })
                && wall.host_solids.iter().any(|host| {
                    self.geometry
                        .solids
                        .iter()
                        .find(|solid| solid.id == *host)
                        .is_some_and(&overlaps)
                })
        }) {
            external_supports.push(wall.support_node);
        }
        external_supports.sort_unstable();
        external_supports.dedup();
        if let Some(node) = self
            .geometry
            .structural_nodes
            .iter_mut()
            .find(|node| node.id == bearing.node)
        {
            node.supported_by.extend(external_supports);
            node.supported_by.sort_unstable();
            node.supported_by.dedup();
        }
    }
}
