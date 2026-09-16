//! Optional attic windows selected against the generated end-truss volumes.
use super::*;
use geo::{Area, BooleanOps};

const CLEAR_WIDTH_METRES: f32 = 0.70;
const CLEAR_HEIGHT_METRES: f32 = 1.00;
const BAY_WIDTH_METRES: f32 = 1.10;
const MINIMUM_SILL_HEIGHT_METRES: f32 = 0.10;
const SPANDREL_HEIGHT_METRES: f32 = 0.12;
const BAY_DEPTH_METRES: f32 = 0.20;
const CANDIDATE_STEP_METRES: f32 = 0.025;
const CONTACT_TOLERANCE_METRES: f32 = 0.004;
const FRAME_DEPTH_METRES: f32 = 0.15;
const GABLE_WALL_ID_BASE: u64 = 2_000_000;
const GABLE_OWNER_ID_BASE: u32 = 90_000;
const GABLE_MEMBER_ID_BASE: u64 = 1_000_000;
const GABLE_NODE_ID_BASE: u64 = 40_000_000;
const GABLE_INTERFACE_ID_BASE: u64 = 0x0200_0000;
const IDS_PER_GABLE: u64 = 16;
const JAMB_WIDTH_METRES: f32 = (BAY_WIDTH_METRES - CLEAR_WIDTH_METRES) * 0.5;

struct Candidate {
    frame: crate::WallLocalFrame,
    base: f32,
    tie: crate::TimberFrameMember,
    head: crate::TimberFrameMember,
    height: f32,
}

impl Candidate {
    fn find(builder: &TimberFrameBuilder<'_>, face: &RoofEnclosureFace) -> Option<Self> {
        let normal = face.normal();
        let outward = Vec2::new(normal.x, normal.z).round();
        let tangent = if outward.x.abs() > 0.5 {
            Vec2::Y
        } else {
            Vec2::X
        };
        let plane = face.polygon[0].dot(normal);
        let tie = builder
            .members
            .iter()
            .find(|member| {
                member.role == crate::TimberMemberRole::GableTie
                    && (member.start.dot(normal) - plane).abs() < BAY_DEPTH_METRES
                    && (member.end - member.start)
                        .normalize()
                        .dot(Vec3::new(tangent.x, 0.0, tangent.y))
                        .abs()
                        > 0.99
            })?
            .clone();
        let midpoint = (tie.start + tie.end) * 0.5;
        let centre = Vec2::new(midpoint.x, midpoint.z);
        let origin = centre + outward * (plane - centre.dot(outward) - BAY_DEPTH_METRES * 0.5);
        let base = midpoint.y + tie.section_metres.x * 0.5 - CONTACT_TOLERANCE_METRES;
        let heads = builder
            .members
            .iter()
            .filter(|member| {
                matches!(
                    member.role,
                    crate::TimberMemberRole::Rail | crate::TimberMemberRole::Collar
                ) && (member.start.y - member.end.y).abs() < CONTACT_TOLERANCE_METRES
                    && (member.start.dot(normal) - tie.start.dot(normal)).abs()
                        < CONTACT_TOLERANCE_METRES
                    && (member.end.dot(normal) - tie.end.dot(normal)).abs()
                        < CONTACT_TOLERANCE_METRES
                    && member.start.y - member.section_metres.x * 0.5 - base
                        >= CLEAR_HEIGHT_METRES + MINIMUM_SILL_HEIGHT_METRES
            })
            .cloned()
            .collect::<Vec<_>>();
        for head in heads {
            let half_span = tie.start.distance(tie.end) * 0.5;
            let steps = (half_span / CANDIDATE_STEP_METRES) as usize;
            for side in [-1.0, 1.0] {
                for station in 0..steps {
                    let shift = BAY_WIDTH_METRES * 0.5
                        + FRAME_DEPTH_METRES
                        + station as f32 * CANDIDATE_STEP_METRES;
                    let frame = crate::WallLocalFrame {
                        origin: origin + tangent * side * shift,
                        tangent,
                        outward,
                        inside_room: None,
                        outside_room: None,
                    };
                    let candidate = Self {
                        frame,
                        base,
                        tie: tie.clone(),
                        height: head.start.y + head.section_metres.x * 0.5 + SPANDREL_HEIGHT_METRES
                            - base,
                        head: head.clone(),
                    };
                    if candidate.fits(builder, face) {
                        return Some(candidate);
                    }
                }
            }
        }
        None
    }

    fn fits(&self, builder: &TimberFrameBuilder<'_>, face: &RoofEnclosureFace) -> bool {
        // Include the complete new frame, which projects beyond the plaster face
        // and bears on the tie upper face. Only the typed tie/head contacts may overlap.
        let tie_plane = Vec2::new(self.tie.start.x, self.tie.start.z).dot(self.frame.outward);
        let frame_shift = tie_plane - self.frame.origin.dot(self.frame.outward);
        let depth_min = (-BAY_DEPTH_METRES * 0.5).min(frame_shift - JAMB_WIDTH_METRES * 0.5);
        let depth_max = (BAY_DEPTH_METRES * 0.5).max(frame_shift + JAMB_WIDTH_METRES * 0.5);
        let half = self.frame.tangent.abs() * BAY_WIDTH_METRES * 0.5
            + self.frame.outward.abs() * (depth_max - depth_min) * 0.5;
        let origin = self.frame.origin + self.frame.outward * (depth_min + depth_max) * 0.5;
        let bounds = (
            Vec3::new(
                origin.x - half.x,
                self.base + CONTACT_TOLERANCE_METRES,
                origin.y - half.y,
            ),
            Vec3::new(
                origin.x + half.x,
                self.base + self.height,
                origin.y + half.y,
            ),
        );
        let head_start = Vec2::new(self.head.start.x, self.head.start.z).dot(self.frame.tangent);
        let head_end = Vec2::new(self.head.end.x, self.head.end.z).dot(self.frame.tangent);
        let centre = origin.dot(self.frame.tangent);
        if centre - BAY_WIDTH_METRES * 0.5 < head_start.min(head_end)
            || centre + BAY_WIDTH_METRES * 0.5 > head_start.max(head_end)
        {
            return false;
        }
        if builder
            .members
            .iter()
            .filter(|member| member.id != self.tie.id && member.id != self.head.id)
            .any(|member| {
                builder
                    .geometry
                    .solids
                    .iter()
                    .find(|solid| solid.id == member.solid)
                    .is_some_and(|solid| {
                        crate::solid_overlap::overlaps_bounds(
                            solid,
                            bounds,
                            CONTACT_TOLERANCE_METRES,
                        )
                    })
            })
        {
            return false;
        }
        let tangent = face.tangent();
        let project = |p: Vec2, y| Vec2::new(Vec3::new(p.x, y, p.y).dot(tangent), y);
        let left = origin - self.frame.tangent * BAY_WIDTH_METRES * 0.5;
        let right = origin + self.frame.tangent * BAY_WIDTH_METRES * 0.5;
        let bay = closed_polygon([
            project(left, self.base),
            project(right, self.base),
            project(right, self.base + self.height),
            project(left, self.base + self.height),
        ]);
        MultiPolygon(vec![bay])
            .difference(&face.residual(&[]))
            .unsigned_area()
            < 0.0001
    }
}

pub(super) fn resolve(
    program: &BuildingProgram,
    builder: &mut TimberFrameBuilder<'_>,
    roofs: &mut [RoofAssembly],
    walls: &mut Vec<crate::WallAssembly>,
    openings: &mut Vec<crate::OpeningAssembly>,
    bays: &mut Vec<crate::TimberFrameBay>,
) {
    if !matches!(
        program.archetype,
        BuildingArchetype::TownHouse | BuildingArchetype::FachwerkMerchantHouse
    ) {
        return;
    }
    for roof in roofs
        .iter_mut()
        .filter(|roof| roof.parent.is_none() && roof.kind == RoofKind::Gable)
    {
        for (index, face) in roof.enclosure_faces.iter_mut().enumerate() {
            if face.material != RoofMaterial::TimberInfill || face.polygon.len() < 3 {
                continue;
            }
            let Some(candidate) = Candidate::find(builder, face) else {
                continue;
            };
            let slot = roof.id.0 * IDS_PER_GABLE + index as u64;
            let wall_id = crate::WallAssemblyId(GABLE_WALL_ID_BASE + slot);
            let opening_id = crate::OpeningAssemblyId(wall_id.0);
            let owner = GeometryOwnerId(GABLE_OWNER_ID_BASE + slot as u32);
            let wall_node = StructuralNodeId((u64::from(owner.0) << 16) | 1);
            let frame = candidate.frame;
            builder.geometry.structural_nodes.push(StructuralNode {
                id: wall_node,
                owner,
                kind: StructuralNodeKind::RoofWallPlate,
                position: Vec3::new(frame.origin.x, candidate.base, frame.origin.y),
                supported_by: vec![candidate.tie.start_node, candidate.tie.end_node],
                grounded: false,
            });
            let (wall, opening) = roof_wall_opening::RectangularRoofWindow {
                wall_id,
                opening_id,
                owner,
                source: crate::WallSourceId::RoofGable {
                    roof: roof.id,
                    enclosure: face.id,
                },
                origin: frame.origin,
                tangent: frame.tangent,
                outward: frame.outward,
                base: candidate.base,
                width: BAY_WIDTH_METRES,
                height: candidate.height,
                thickness: BAY_DEPTH_METRES,
                opening_width: CLEAR_WIDTH_METRES,
                clear_height: CLEAR_HEIGHT_METRES,
                sill_height: candidate.head.start.y
                    - candidate.head.section_metres.x * 0.5
                    - CLEAR_HEIGHT_METRES
                    - candidate.base,
                head_height: candidate.head.section_metres.x,
                head_member: Some(candidate.head.clone()),
                wall_node,
                storey_level: program.storeys.len() as u16,
                ornamental_frame: false,
            }
            .build(builder.geometry);
            // The fixed glass covers the complete clear section. It stays behind
            // the jambs and remains a bounded solid in every representation.
            for id in &opening.closure_solids {
                let solid = builder
                    .geometry
                    .solids
                    .iter_mut()
                    .find(|solid| solid.id == *id)
                    .unwrap();
                solid.size = Vec3::new(
                    frame.tangent.x.abs() * CLEAR_WIDTH_METRES
                        + frame.outward.x.abs() * crate::FIXED_GABLE_GLAZING_DEPTH_METRES,
                    CLEAR_HEIGHT_METRES,
                    frame.tangent.y.abs() * CLEAR_WIDTH_METRES
                        + frame.outward.y.abs() * crate::FIXED_GABLE_GLAZING_DEPTH_METRES,
                );
            }
            measured_bearings(builder.geometry, &opening);
            let member_ids = frame_members(builder, &candidate, &opening);
            let bay_id =
                crate::TimberFrameBayId(bays.iter().map(|bay| bay.id.0).max().unwrap_or(0) + 1);
            bays.push(crate::TimberFrameBay {
                id: bay_id,
                wall: Some(wall_id),
                opening: Some(opening_id),
                member_ids,
                infill_solids: wall.host_solids.clone(),
            });
            face.inset_walls.push(wall_id);
            walls.push(wall);
            openings.push(opening);
        }
    }
}

fn measured_bearings(geometry: &mut ResolvedGeometry, opening: &crate::OpeningAssembly) {
    let head = geometry
        .solids
        .iter()
        .find(|s| s.id == opening.head_solid)
        .unwrap()
        .cuboid_bounds();
    for (id, solid) in opening
        .head_bearing_interfaces
        .into_iter()
        .zip(opening.jamb_solids)
        .chain([(opening.wall_above_interface, opening.spandrel_solid)])
    {
        let contact = geometry
            .solids
            .iter()
            .find(|s| s.id == solid)
            .unwrap()
            .cuboid_bounds();
        let interface = geometry
            .support_interfaces
            .iter_mut()
            .find(|i| i.id == id)
            .unwrap();
        interface.bounds = ResolvedBounds {
            min: head.min.max(contact.min),
            max: head.max.min(contact.max),
        };
    }
}

fn frame_members(
    builder: &mut TimberFrameBuilder<'_>,
    candidate: &Candidate,
    opening: &crate::OpeningAssembly,
) -> Vec<crate::TimberMemberId> {
    // Reserve a separate ID range so later original floor and roof members
    // keep their identities when optional apertures are inserted.
    let counters = (
        builder.next_member,
        builder.next_node,
        builder.next_joint,
        builder.next_interface,
    );
    let slot = opening.id.0 - GABLE_WALL_ID_BASE;
    builder.next_member = GABLE_MEMBER_ID_BASE + slot * IDS_PER_GABLE;
    builder.next_node = GABLE_NODE_ID_BASE + slot * IDS_PER_GABLE;
    builder.next_joint = GABLE_MEMBER_ID_BASE + slot * IDS_PER_GABLE;
    builder.next_interface = GABLE_INTERFACE_ID_BASE + slot * IDS_PER_GABLE;
    let frame = candidate.frame;
    let tie_plane = Vec2::new(candidate.tie.start.x, candidate.tie.start.z).dot(frame.outward);
    let origin = frame.origin + frame.outward * (tie_plane - frame.origin.dot(frame.outward));
    let point = |x: f32, y: f32| {
        let p = origin + frame.tangent * x;
        Vec3::new(p.x, y, p.y)
    };
    let half = (CLEAR_WIDTH_METRES + JAMB_WIDTH_METRES) * 0.5;
    let mut members = Vec::new();
    for x in [-half, half] {
        members.push(builder.member(
            crate::TimberMemberRole::IntermediatePost,
            point(x, candidate.base),
            point(x, candidate.head.start.y),
            Vec2::splat(JAMB_WIDTH_METRES),
            crate::TimberFramePhase::RoofConstruction,
        ));
    }
    let height = opening.sill_elevation_metres - candidate.base;
    let y = candidate.base + height * 0.5;
    members.push(builder.member(
        crate::TimberMemberRole::Rail,
        point(-half, y),
        point(half, y),
        Vec2::new(height, FRAME_DEPTH_METRES),
        crate::TimberFramePhase::RoofConstruction,
    ));
    members.push(candidate.head.id);
    (
        builder.next_member,
        builder.next_node,
        builder.next_joint,
        builder.next_interface,
    ) = counters;
    members
}

#[cfg(test)]
mod tests;
