use super::*;

const OWNER: GeometryOwnerId = GeometryOwnerId(96_000);
const NODE_BASE: u64 = 9_600_000;
const CONTACT_METRES: f32 = 0.025;

pub(super) fn assemble(
    d: Dimensions,
    pitch: f32,
    walls: &mut Vec<crate::WallAssembly>,
    geometry: &mut ResolvedGeometry,
) -> SmallChurchPlan {
    let initial_walls = walls.len();
    let mut builder = Fittings {
        geometry,
        walls,
        ids: Vec::new(),
    };
    builder.part(
        Vec3::new(d.width() * 0.5, 0.08, d.nave_depth() * 0.5),
        Vec3::new(d.width() - 0.5, 0.16, d.nave_depth() - 0.5),
        SolidRole::ChurchFloor,
    );
    if d.chancel_cells > 0 {
        builder.part(
            Vec3::new(d.width() * 0.5, 0.08, (d.depth() + d.nave_depth()) * 0.5),
            Vec3::new(d.width() - 3.5, 0.16, d.depth() - d.nave_depth() - 0.5),
            SolidRole::ChurchFloor,
        );
        let span = 3.0 * CELL_SIZE_METRES;
        builder.part(
            Vec3::new(d.width() * 0.5, 0.08, d.nave_depth()),
            Vec3::new(span, 0.16, 0.5),
            SolidRole::ChurchFloor,
        );
        for x in [-1.0, 1.0] {
            builder.part(
                Vec3::new(
                    d.width() * 0.5 + x * (span * 0.5 - 0.1),
                    1.425,
                    d.nave_depth(),
                ),
                Vec3::new(0.2, 2.85, 0.5),
                SolidRole::ChurchPier,
            );
        }
        builder.part(
            Vec3::new(d.width() * 0.5, 3.0, d.nave_depth()),
            Vec3::new(span, 0.3, 0.5),
            SolidRole::BeamJoist,
        );
        builder.band(
            Vec2::new(d.width() * 0.5, d.nave_depth()),
            Vec2::X,
            span,
            3.15,
            d.nave_eave - 3.15,
            crate::WallMaterialClass::RubbleMasonry,
        );
    }
    belfry(&mut builder, d, pitch);
    SmallChurchPlan {
        kind: d.kind,
        size: d.size,
        nave_bays: d.nave_cells / 2,
        nave_depth_metres: d.nave_depth(),
        chancel_eave_metres: (d.chancel_cells > 0).then_some(d.chancel_eave),
        belfry_stage: ResolvedBounds {
            min: Vec3::new(
                d.bell_centre().x - 1.05,
                d.bell_floor(pitch),
                d.bell_centre().y - 1.05,
            ),
            max: Vec3::new(
                d.bell_centre().x + 1.05,
                d.bell_floor(pitch) + 1.65,
                d.bell_centre().y + 1.05,
            ),
        },
        fittings: builder.ids,
        bearing_walls: builder.walls[initial_walls..]
            .iter()
            .map(|wall| wall.id)
            .collect(),
        public_route: ResolvedBounds {
            min: Vec3::new(d.width() * 0.5 - 0.6, 0.18, 0.3),
            max: Vec3::new(d.width() * 0.5 + 0.6, 2.35, d.depth() - 0.4),
        },
    }
}

fn belfry(builder: &mut Fittings<'_>, d: Dimensions, pitch: f32) {
    let centre = d.bell_centre();
    let floor = d.bell_floor(pitch);
    let cap = floor + 1.65;
    // Four continuous posts carry the bell frame to the ground, clear of the entrance aisle.
    for x in [-0.85, 0.85] {
        for z in [-0.85, 0.85] {
            builder.part(
                Vec3::new(centre.x + x, cap * 0.5, centre.y + z),
                Vec3::new(0.18, cap, 0.18),
                SolidRole::BeamJoist,
            );
        }
    }
    for z in [-0.85, 0.85] {
        builder.part(
            Vec3::new(centre.x, floor - 0.10, centre.y + z),
            Vec3::new(1.88, 0.20, 0.22),
            SolidRole::BeamJoist,
        );
        builder.band(
            centre + Vec2::Y * z,
            Vec2::X,
            2.1,
            cap - 0.2,
            0.2,
            crate::WallMaterialClass::InternalTimber,
        );
    }
    for x in [-0.85, 0.85] {
        builder.band(
            centre + Vec2::X * x,
            Vec2::Y,
            2.1,
            cap - 0.2,
            0.2,
            crate::WallMaterialClass::InternalTimber,
        );
    }
    for x in [-0.85, 0.85] {
        builder.part(
            Vec3::new(centre.x + x, floor + 1.25, centre.y),
            Vec3::new(0.2, 0.2, 1.88),
            SolidRole::BeamJoist,
        );
    }
    builder.part(
        Vec3::new(centre.x, floor + 1.25, centre.y),
        Vec3::new(1.88, 0.2, 0.2),
        SolidRole::BeamJoist,
    );
    builder.part(
        Vec3::new(centre.x, floor + 0.92, centre.y),
        Vec3::new(0.25, 0.48, 0.25),
        SolidRole::ChurchBell,
    );
    builder.part(
        Vec3::new(centre.x, floor + 0.62, centre.y),
        Vec3::new(0.62, 0.18, 0.62),
        SolidRole::ChurchBell,
    );
    // Open sound stage: the space between slats is geometry, not a painted black panel.
    for level in 0..4 {
        let y = floor + 0.2 + level as f32 * 0.3;
        for z in [-0.85, 0.85] {
            builder.part(
                Vec3::new(centre.x, y, centre.y + z),
                Vec3::new(1.7, 0.10, 0.18),
                SolidRole::BeamJoist,
            );
        }
        for x in [-0.85, 0.85] {
            builder.part(
                Vec3::new(centre.x + x, y, centre.y),
                Vec3::new(0.18, 0.10, 1.7),
                SolidRole::BeamJoist,
            );
        }
    }
}

struct Fittings<'a> {
    geometry: &'a mut ResolvedGeometry,
    walls: &'a mut Vec<crate::WallAssembly>,
    ids: Vec<ResolvedItemId>,
}

impl Fittings<'_> {
    fn part(&mut self, centre: Vec3, size: Vec3, role: SolidRole) -> ResolvedItemId {
        let slot = self.ids.len() as u64 + 1;
        let id = StructuralNodeId(NODE_BASE + slot);
        let min = centre - size * 0.5;
        let max = centre + size * 0.5;
        let parents = self
            .geometry
            .solids
            .iter()
            .filter(|solid| {
                let extent = solid.size * 0.5;
                let overlap =
                    (max.min(solid.centre + extent) - min.max(solid.centre - extent)).min_element();
                overlap >= -CONTACT_METRES
            })
            .flat_map(|solid| solid.supported_by.iter().copied())
            .collect();
        self.geometry.structural_nodes.push(StructuralNode {
            id,
            owner: OWNER,
            kind: StructuralNodeKind::WallBearing,
            position: Vec3::new(centre.x, min.y, centre.z),
            supported_by: parents,
            grounded: min.y <= CONTACT_METRES,
        });
        let solid = wall_solid(
            self.geometry,
            OWNER,
            slot,
            centre,
            size,
            role,
            crate::ResolvedSolidShape::Cuboid,
            id,
        );
        self.ids.push(solid);
        solid
    }

    fn band(
        &mut self,
        origin: Vec2,
        tangent: Vec2,
        length: f32,
        base: f32,
        height: f32,
        material: crate::WallMaterialClass,
    ) {
        let thickness = if material == crate::WallMaterialClass::InternalTimber {
            0.18
        } else {
            0.5
        };
        let size = if tangent.x == 1.0 {
            Vec3::new(length, height, thickness)
        } else {
            Vec3::new(thickness, height, length)
        };
        let solid = self.part(
            Vec3::new(origin.x, base + height * 0.5, origin.y),
            size,
            SolidRole::WallHost,
        );
        let node = self.geometry.solids.last().unwrap().supported_by[0];
        let index = self.walls.len() as u64;
        self.walls.push(crate::WallAssembly {
            id: crate::WallAssemblyId(NODE_BASE + index),
            owner: OWNER,
            source: crate::WallSourceId::ChurchExterior {
                range: crate::ChurchRange::Nave,
                side: Direction::North,
                bay: index as u8,
            },
            material,
            storey_level: 0,
            frame: crate::WallLocalFrame {
                origin,
                tangent,
                outward: Vec2::new(-tangent.y, tangent.x),
                inside_room: Some(0),
                outside_room: None,
            },
            radial_frame: None,
            length_metres: length,
            height_metres: height,
            base_elevation_metres: base,
            thickness_metres: thickness,
            structural_role: crate::WallStructuralRole::LoadBearing,
            support_node: node,
            host_solids: vec![solid],
            opening_ids: Vec::new(),
            replaced_by_owner: None,
        });
    }
}
