//! Rectangular fixed roof-wall windows; attachment policy belongs to the caller.
use super::*;
mod surfaces;

#[derive(Clone)]
pub(super) struct RectangularRoofWindow {
    pub wall_id: crate::WallAssemblyId,
    pub opening_id: crate::OpeningAssemblyId,
    pub owner: GeometryOwnerId,
    pub source: crate::WallSourceId,
    pub origin: Vec2,
    pub tangent: Vec2,
    pub outward: Vec2,
    pub base: f32,
    pub width: f32,
    pub height: f32,
    pub thickness: f32,
    pub opening_width: f32,
    pub clear_height: f32,
    pub sill_height: f32,
    pub head_height: f32,
    pub head_member: Option<crate::TimberFrameMember>,
    pub wall_node: StructuralNodeId,
    pub storey_level: u16,
    pub ornamental_frame: bool,
}

struct WindowNodes {
    jamb_nodes: [StructuralNodeId; 2],
    head_node: StructuralNodeId,
    spandrel_node: StructuralNodeId,
}
struct WindowSolids {
    host_solids: Vec<ResolvedItemId>,
    jamb_solids: [ResolvedItemId; 2],
    sill_solid: ResolvedItemId,
    head_solid: ResolvedItemId,
    spandrel_solid: ResolvedItemId,
}
impl RectangularRoofWindow {
    pub(super) fn build(
        self,
        geometry: &mut ResolvedGeometry,
    ) -> (crate::WallAssembly, crate::OpeningAssembly) {
        let nodes = self.nodes(geometry);
        let solids = self.solids(&nodes, geometry);
        let opening = self.opening(&nodes, &solids, geometry);
        (self.wall(solids), opening)
    }
    fn local_size(&self, width: f32, height: f32, depth: f32) -> Vec3 {
        if self.tangent.x.abs() > 0.5 {
            Vec3::new(width, height, depth)
        } else {
            Vec3::new(depth, height, width)
        }
    }
    fn nodes(&self, geometry: &mut ResolvedGeometry) -> WindowNodes {
        let Self {
            owner,
            origin,
            tangent,
            base,
            width,
            opening_width,
            sill_height,
            clear_height,
            head_height,
            height,
            wall_node,
            ..
        } = self.clone();
        let top = base + height;
        let jamb_nodes = [
            StructuralNodeId(wall_node.0 + 1),
            StructuralNodeId(wall_node.0 + 2),
        ];
        for (side, node) in [-1.0_f32, 1.0].into_iter().zip(jamb_nodes) {
            geometry.structural_nodes.push(StructuralNode {
                id: node,
                owner,
                kind: StructuralNodeKind::OpeningJamb,
                position: Vec3::new(
                    origin.x + tangent.x * side * (width + opening_width) * 0.25,
                    base,
                    origin.y + tangent.y * side * (width + opening_width) * 0.25,
                ),
                supported_by: vec![wall_node],
                grounded: false,
            });
        }
        let head_node = StructuralNodeId(wall_node.0 + 3);
        let spandrel_node = StructuralNodeId(wall_node.0 + 4);
        geometry.structural_nodes.push(StructuralNode {
            id: head_node,
            owner,
            kind: StructuralNodeKind::OpeningHead,
            position: Vec3::new(
                origin.x,
                base + sill_height + clear_height + head_height * 0.5,
                origin.y,
            ),
            supported_by: jamb_nodes.to_vec(),
            grounded: false,
        });
        geometry.structural_nodes.push(StructuralNode {
            id: spandrel_node,
            owner,
            kind: StructuralNodeKind::OpeningSpandrel,
            position: Vec3::new(origin.x, top, origin.y),
            supported_by: vec![head_node],
            grounded: false,
        });

        WindowNodes {
            jamb_nodes,
            head_node,
            spandrel_node,
        }
    }
    fn solids(&self, nodes: &WindowNodes, geometry: &mut ResolvedGeometry) -> WindowSolids {
        let Self {
            owner,
            origin,
            tangent,
            base,
            width,
            height,
            thickness,
            opening_width,
            clear_height,
            sill_height,
            head_height,
            head_member,
            wall_node,
            ..
        } = self.clone();
        let top = base + height;
        let head_bottom = base + sill_height + clear_height;
        let side_width = (width - opening_width) * 0.5;
        let WindowNodes {
            jamb_nodes,
            head_node,
            spandrel_node,
        } = *nodes;
        let mut host_solids = Vec::new();
        let mut jamb_solids = [ResolvedItemId::default(); 2];
        for (slot, side, node, target) in [
            (0_u64, -1.0_f32, jamb_nodes[0], 0_usize),
            (1, 1.0, jamb_nodes[1], 1),
        ] {
            let plan = origin + tangent * side * (opening_width + side_width) * 0.5;
            let solid = wall_solid(
                geometry,
                owner,
                slot,
                Vec3::new(plan.x, base + height * 0.5, plan.y),
                self.local_size(side_width, height, thickness),
                SolidRole::OpeningJamb,
                crate::ResolvedSolidShape::Cuboid,
                node,
            );
            host_solids.push(solid);
            jamb_solids[target] = solid;
        }
        let sill_solid = wall_solid(
            geometry,
            owner,
            2,
            Vec3::new(origin.x, base + sill_height * 0.5, origin.y),
            self.local_size(opening_width, sill_height, thickness),
            SolidRole::OpeningSill,
            crate::ResolvedSolidShape::Cuboid,
            wall_node,
        );
        host_solids.push(sill_solid);
        let head_solid = if let Some(member) = &head_member {
            member.solid
        } else {
            wall_solid(
                geometry,
                owner,
                3,
                Vec3::new(origin.x, head_bottom + head_height * 0.5, origin.y),
                self.local_size(opening_width + 0.12, head_height, thickness),
                SolidRole::OpeningHead,
                crate::ResolvedSolidShape::Cuboid,
                head_node,
            )
        };
        if head_member.is_none() {
            host_solids.push(head_solid);
        }
        let spandrel_height = (top - (head_bottom + head_height) + 0.025).max(0.08);
        let spandrel_solid = wall_solid(
            geometry,
            owner,
            4,
            Vec3::new(origin.x, top - spandrel_height * 0.5, origin.y),
            self.local_size(opening_width, spandrel_height, thickness),
            SolidRole::OpeningSpandrel,
            crate::ResolvedSolidShape::Cuboid,
            spandrel_node,
        );
        host_solids.push(spandrel_solid);
        host_solids.extend(self.ornaments(geometry));
        WindowSolids {
            host_solids,
            jamb_solids,
            sill_solid,
            head_solid,
            spandrel_solid,
        }
    }
    fn ornaments(&self, geometry: &mut ResolvedGeometry) -> Vec<ResolvedItemId> {
        let Self {
            owner,
            origin,
            tangent,
            outward,
            base,
            width,
            height,
            wall_node,
            ornamental_frame,
            ..
        } = self.clone();
        let top = base + height;
        let mut host_solids = Vec::new();
        // Non-Fachwerk fixtures retain the compact Stage-3 child-front frame.
        // The five accepted civilian programs instead receive their opening-
        // first members from `TimberFrameAssembly`, so duplicating this legacy
        // four-piece overlay would create two competing structural authorities.
        for (slot, plan, centre_y, frame_size) in (ornamental_frame)
            .then_some([
                (
                    100_u64,
                    origin - tangent * (width * 0.5 - 0.055),
                    base + height * 0.5,
                    self.local_size(0.11, height, 0.08),
                ),
                (
                    101,
                    origin + tangent * (width * 0.5 - 0.055),
                    base + height * 0.5,
                    self.local_size(0.11, height, 0.08),
                ),
                (
                    102,
                    origin,
                    base + 0.055,
                    self.local_size(width, 0.11, 0.08),
                ),
                (103, origin, top - 0.055, self.local_size(width, 0.11, 0.08)),
            ])
            .into_iter()
            .flatten()
        {
            host_solids.push(wall_solid(
                geometry,
                owner,
                slot,
                Vec3::new(plan.x, centre_y, plan.y) + Vec3::new(outward.x, 0.0, outward.y) * 0.12,
                frame_size,
                SolidRole::FrameMember,
                crate::ResolvedSolidShape::Cuboid,
                wall_node,
            ));
        }

        host_solids
    }
    fn opening(
        &self,
        nodes: &WindowNodes,
        solids: &WindowSolids,
        geometry: &mut ResolvedGeometry,
    ) -> crate::OpeningAssembly {
        let Self {
            wall_id,
            opening_id,
            owner,
            source,
            origin,
            tangent,
            outward,
            base,
            opening_width,
            clear_height,
            sill_height,
            head_member,
            ..
        } = self.clone();
        let sill_elevation = base + sill_height;
        let WindowNodes {
            jamb_nodes,
            head_node,
            spandrel_node,
        } = *nodes;
        let WindowSolids {
            jamb_solids,
            sill_solid,
            head_solid,
            spandrel_solid,
            ..
        } = *solids;
        let void_id = self.void(geometry);
        let reveal_surfaces = self.reveals(geometry);
        let closure_solids = self.closures(nodes, geometry);
        let (bearing_ids, wall_above_interface) = self.bearings(nodes, geometry);
        crate::OpeningAssembly {
            id: opening_id,
            owner,
            host_wall: wall_id,
            host_source: source,
            frame: crate::WallLocalFrame {
                origin,
                tangent,
                outward,
                inside_room: None,
                outside_room: None,
            },
            use_kind: crate::OpeningUse::Window,
            profile: crate::OpeningProfile::Rectangular {
                width_metres: opening_width,
                height_metres: clear_height,
            },
            sill_elevation_metres: sill_elevation,
            closure: fixed_window_closure_policy(),
            head_kind: head_member.map_or(crate::OpeningHeadKind::TimberLintel, |member| {
                crate::OpeningHeadKind::TimberFrameMember { member: member.id }
            }),
            void_id,
            jamb_solids,
            sill_solid: Some(sill_solid),
            head_solid,
            spandrel_solid,
            reveal_surfaces,
            closure_solids,
            jamb_nodes,
            head_node,
            spandrel_node,
            tracery_node: None,
            stance_surface: None,
            mount_solid: None,
            ray_indices: Vec::new(),
            sectional_void: (0..=8)
                .map(|slice| crate::OpeningVoidSlice {
                    depth_fraction: slice as f32 / 8.0,
                    width_metres: opening_width,
                    height_metres: clear_height,
                })
                .collect(),
            head_bearing_interfaces: bearing_ids,
            wall_above_interface,
        }
    }
    fn wall(&self, solids: WindowSolids) -> crate::WallAssembly {
        let Self {
            wall_id,
            opening_id,
            owner,
            source,
            origin,
            tangent,
            outward,
            base,
            width,
            height,
            thickness,
            wall_node,
            storey_level,
            ..
        } = self.clone();
        let host_solids = solids.host_solids;
        crate::WallAssembly {
            id: wall_id,
            owner,
            source,
            material: crate::WallMaterialClass::TimberInfill,
            storey_level,
            frame: crate::WallLocalFrame {
                origin,
                tangent,
                outward,
                inside_room: None,
                outside_room: None,
            },
            radial_frame: None,
            length_metres: width,
            height_metres: height,
            base_elevation_metres: base,
            thickness_metres: thickness,
            structural_role: crate::WallStructuralRole::LoadBearing,
            support_node: wall_node,
            host_solids,
            opening_ids: vec![opening_id],
            replaced_by_owner: None,
        }
    }
}
