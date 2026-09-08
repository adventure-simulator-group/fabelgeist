use super::*;
use crate::*;

const WORKPLACE_OWNER: GeometryOwnerId = GeometryOwnerId(95_000);
const WORKPLACE_WALL_BASE: u64 = 95_000;
const BOARD_WALL_THICKNESS_METRES: f32 = 0.18;
const MASONRY_WALL_THICKNESS_METRES: f32 = 0.45;
const WORKPLACE_NODE_BASE: u64 = 95_000_000;
const BEARING_DEPTH_METRES: f32 = 0.04;

pub(super) struct Assembly<'a> {
    pub plan: WorkplacePlan,
    pub geometry: &'a mut ResolvedGeometry,
    pub walls: &'a mut Vec<WallAssembly>,
}

impl Assembly<'_> {
    /// Every part owns a bearing node and records contact with existing support geometry.
    pub fn part(
        &mut self,
        feature: WorkplaceFeature,
        material: WorkplaceMaterial,
        centre: Vec3,
        size: Vec3,
        silhouette: bool,
    ) -> ResolvedItemId {
        let slot = self.plan.parts.len() as u64 + 1;
        let id = ResolvedItemId((1_u64 << 60) | (u64::from(WORKPLACE_OWNER.0) << 32) | slot);
        let node = StructuralNodeId(WORKPLACE_NODE_BASE + slot);
        let bottom = centre.y - size.y * 0.5;
        let supported_by = self
            .geometry
            .solids
            .iter()
            .filter(|other| {
                let overlap = (other.centre + other.size * 0.5).min(centre + size * 0.5)
                    - (other.centre - other.size * 0.5).max(centre - size * 0.5);
                overlap.min_element() >= -BEARING_DEPTH_METRES
            })
            .flat_map(|other| other.supported_by.iter().copied())
            .collect();
        self.geometry.structural_nodes.push(StructuralNode {
            id: node,
            owner: WORKPLACE_OWNER,
            kind: StructuralNodeKind::WallBearing,
            position: Vec3::new(centre.x, bottom, centre.z),
            supported_by,
            grounded: bottom <= BEARING_DEPTH_METRES,
        });
        self.geometry.solids.push(ResolvedSolid {
            id,
            owner: WORKPLACE_OWNER,
            centre,
            size,
            yaw_radians: 0.0,
            crossfall_radians: 0.0,
            longfall_radians: 0.0,
            role: SolidRole::WorkplacePart,
            shape: ResolvedSolidShape::Cuboid,
            supported_by: vec![node],
        });
        self.geometry.support_interfaces.push(SupportInterface {
            id: ResolvedItemId((4_u64 << 60) | (u64::from(WORKPLACE_OWNER.0) << 32) | slot),
            owner: WORKPLACE_OWNER,
            node,
            bounds: ResolvedBounds {
                min: centre - size * 0.5,
                max: Vec3::new(
                    centre.x + size.x * 0.5,
                    bottom + BEARING_DEPTH_METRES,
                    centre.z + size.z * 0.5,
                ),
            },
        });
        self.plan.parts.push(WorkplacePart {
            solid: id,
            feature,
            material,
            silhouette,
        });
        id
    }

    pub fn wall(
        &mut self,
        start: Vec2,
        end: Vec2,
        outward: Vec2,
        base: f32,
        height: f32,
        timber: bool,
    ) {
        let tangent = if end.x == start.x {
            Vec2::Y * (end.y - start.y).signum()
        } else {
            Vec2::X * (end.x - start.x).signum()
        };
        let length = start.distance(end);
        let thickness = if timber {
            BOARD_WALL_THICKNESS_METRES
        } else {
            MASONRY_WALL_THICKNESS_METRES
        };
        let centre = (start + end) * 0.5;
        let size = if tangent.x.abs() > 0.5 {
            Vec3::new(length, height, thickness)
        } else {
            Vec3::new(thickness, height, length)
        };
        let material = if timber {
            WorkplaceMaterial::Timber
        } else {
            WorkplaceMaterial::Masonry
        };
        let solid = self.part(
            WorkplaceFeature::Wall,
            material,
            Vec3::new(centre.x, base + height * 0.5, centre.y),
            size,
            true,
        );
        let index = self.plan.walls.len() as u32;
        let id = WallAssemblyId(WORKPLACE_WALL_BASE + u64::from(index));
        let support_node = self.geometry.solids.last().unwrap().supported_by[0];
        self.walls.push(WallAssembly {
            id,
            owner: WORKPLACE_OWNER,
            source: WallSourceId::WorkplaceWall { index },
            material: if timber {
                WallMaterialClass::TimberInfill
            } else {
                WallMaterialClass::CivilianMasonry
            },
            storey_level: 0,
            frame: WallLocalFrame {
                origin: centre,
                tangent,
                outward,
                inside_room: Some(0),
                outside_room: None,
            },
            radial_frame: None,
            length_metres: length,
            height_metres: height,
            base_elevation_metres: base,
            thickness_metres: thickness,
            structural_role: WallStructuralRole::LoadBearing,
            support_node,
            host_solids: vec![solid],
            opening_ids: vec![],
            replaced_by_owner: None,
        });
        self.plan.walls.push(id);
    }

    pub fn passage(&mut self, min: Vec3, max: Vec3) {
        self.plan.passages.push(WorkplacePassage { min, max });
    }
}

pub(crate) fn resolve_workplace(
    program: &BuildingProgram,
    walls: &mut Vec<WallAssembly>,
    geometry: &mut ResolvedGeometry,
) -> Option<WorkplacePlan> {
    let kind = program.workplace_kind()?;
    let mut assembly = Assembly {
        plan: WorkplacePlan {
            kind,
            size: program.workplace_size?,
            plot_dimensions_metres: program.plot_dimensions_metres(),
            walls: vec![],
            parts: vec![],
            passages: vec![],
        },
        geometry,
        walls,
    };
    super::envelope::build_envelope(&mut assembly, program);
    super::equipment::fit_workplace(&mut assembly, program);
    Some(assembly.plan)
}
