//! Cantilever framing and its measured floor bearing interfaces.
use super::*;
const FLOOR_THICKNESS_METRES: f32 = 0.16;
const BACKSPAN_METRES: f32 = 0.95;
pub(super) struct JettyFrame<'a> {
    pub projection: f32,
    pub storey_height: f32,
    pub base: f32,
    pub section: Vec2,
    pub tangent: Vec2,
    pub outward: Vec2,
    pub facade_walls: &'a [&'a crate::WallAssembly],
    pub line_length: f32,
    pub storey_id: u64,
}

impl JettyFrame<'_> {
    pub(super) fn build(
        &self,
        builder: &mut TimberFrameBuilder<'_>,
        storey_member_ids: &mut Vec<crate::TimberMemberId>,
    ) -> crate::TimberJettyAssembly {
        let Self {
            projection,
            base,
            section,
            tangent,
            outward,
            line_length,
            ..
        } = *self;
        let backspan = BACKSPAN_METRES;
        let beam_elevation = base - FLOOR_THICKNESS_METRES - section.x * 0.5;
        let mut jetty_beams = Vec::new();
        let mut knaggen = Vec::new();
        let mut corner_supports = Vec::new();
        for (index, wall) in self.facade_walls.iter().enumerate() {
            let plane = wall.frame.origin
                + wall.frame.outward * (wall.thickness_metres * 0.5 - section.y * 0.5);
            for sign in [-1.0_f32, 1.0] {
                let boundary = plane + tangent * sign * wall.length_metres * 0.5;
                let outer = Vec3::new(boundary.x, beam_elevation, boundary.y);
                let inner_plan = boundary - outward * (projection + backspan);
                let inner = Vec3::new(inner_plan.x, beam_elevation, inner_plan.y);
                let beam = builder.member(
                    crate::TimberMemberRole::JettyBeam,
                    inner,
                    outer,
                    section,
                    crate::TimberFramePhase::UpperStoreyAddition,
                );
                jetty_beams.push(beam);
                storey_member_ids.push(builder.member(
                    crate::TimberMemberRole::PrimaryPost,
                    outer,
                    Vec3::new(outer.x, base, outer.z),
                    section,
                    crate::TimberFramePhase::UpperStoreyAddition,
                ));
                let lower_plan = boundary - outward * projection;
                let lower = Vec3::new(lower_plan.x, base - self.storey_height * 0.28, lower_plan.y);
                let knagge = builder.member(
                    crate::TimberMemberRole::Knagge,
                    lower,
                    outer,
                    section * 0.9,
                    crate::TimberFramePhase::UpperStoreyAddition,
                );
                knaggen.push(knagge);
                if index == 0 || index + 1 == self.facade_walls.len() {
                    corner_supports.push(knagge);
                }
            }
        }

        self.bearings(builder, &mut jetty_beams, storey_member_ids);
        knaggen.sort_unstable();
        knaggen.dedup();
        corner_supports.sort_unstable();
        corner_supports.dedup();
        storey_member_ids.extend(jetty_beams.iter().copied());
        storey_member_ids.extend(knaggen.iter().copied());
        let (floor_solid, floor_bearing_interfaces, outer_plane) = self.deck(builder, &jetty_beams);
        let half_length = line_length * 0.5;
        let left_outer = outer_plane - tangent * half_length;
        let right_outer = outer_plane + tangent * half_length;
        let structural_depth = projection + backspan;
        let left_inner = left_outer - outward * structural_depth;
        let right_inner = right_outer - outward * structural_depth;
        crate::TimberJettyAssembly {
            projection_metres: projection,
            backspan_metres: backspan,
            jetty_beams,
            knaggen,
            corner_supports,
            floor_solid,
            floor_bearing_interfaces,
            support_polygon: vec![left_inner, right_inner, right_outer, left_outer],
        }
    }
    fn bearings(
        &self,
        builder: &mut TimberFrameBuilder<'_>,
        jetty_beams: &mut Vec<crate::TimberMemberId>,
        storey_member_ids: &mut Vec<crate::TimberMemberId>,
    ) {
        let Self {
            section, tangent, ..
        } = *self;
        jetty_beams.sort_unstable();
        jetty_beams.dedup();
        let mut outer_bearings = jetty_beams
            .iter()
            .filter_map(|id| builder.members.iter().find(|member| member.id == *id))
            .map(|member| member.end)
            .collect::<Vec<_>>();
        outer_bearings.sort_by(|a, b| {
            Vec2::new(a.x, a.z)
                .dot(tangent)
                .total_cmp(&Vec2::new(b.x, b.z).dot(tangent))
        });
        if let (Some(first), Some(last)) = (outer_bearings.first(), outer_bearings.last()) {
            storey_member_ids.push(builder.member(
                crate::TimberMemberRole::Sill,
                *first,
                *last,
                section,
                crate::TimberFramePhase::UpperStoreyAddition,
            ));
        }
        let mut inner_bearings = jetty_beams
            .iter()
            .filter_map(|id| builder.members.iter().find(|member| member.id == *id))
            .map(|member| member.start)
            .collect::<Vec<_>>();
        inner_bearings.sort_by(|left, right| {
            Vec2::new(left.x, left.z)
                .dot(tangent)
                .total_cmp(&Vec2::new(right.x, right.z).dot(tangent))
        });
        if let (Some(first), Some(last)) = (
            inner_bearings.first().copied(),
            inner_bearings.last().copied(),
        ) && first.distance(last) > 0.10
        {
            let inner_girder = builder.member(
                crate::TimberMemberRole::Girder,
                first,
                last,
                section * 1.12,
                crate::TimberFramePhase::UpperStoreyAddition,
            );
            storey_member_ids.push(inner_girder);
        }
    }
    fn deck(
        &self,
        builder: &mut TimberFrameBuilder<'_>,
        jetty_beams: &[crate::TimberMemberId],
    ) -> (ResolvedItemId, Vec<ResolvedItemId>, Vec2) {
        let Self {
            projection,
            base,
            section,
            tangent,
            outward,
            line_length,
            storey_id: next_storey,
            ..
        } = *self;
        let owner = builder.owner;
        let floor_thickness = FLOOR_THICKNESS_METRES;
        let outer_plane = self
            .facade_walls
            .iter()
            .map(|wall| {
                wall.frame.origin
                    + wall.frame.outward * (wall.thickness_metres * 0.5 - section.y * 0.5)
            })
            .sum::<Vec2>()
            / self.facade_walls.len() as f32;
        // Only the projecting strip is a separate jetty plate. The
        // backspan remains part of the main storey floor assembled
        // below, avoiding duplicate overlapping floor authority.
        let floor_depth = projection;
        let floor_centre_plan = outer_plane - outward * floor_depth * 0.5;
        let floor_solid =
            ResolvedItemId((1_u64 << 60) | (u64::from(owner.0) << 32) | 0x0f00_0000 | next_storey);
        let floor_support_nodes = jetty_beams
            .iter()
            .filter_map(|id| builder.members.iter().find(|member| member.id == *id))
            .flat_map(|member| [member.start_node, member.end_node])
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        builder.geometry.solids.push(ResolvedSolid {
            id: floor_solid,
            owner,
            centre: Vec3::new(
                floor_centre_plan.x,
                base - floor_thickness * 0.5,
                floor_centre_plan.y,
            ),
            size: Vec3::new(line_length, floor_thickness, floor_depth),
            yaw_radians: (-tangent.y).atan2(tangent.x),
            crossfall_radians: 0.0,
            longfall_radians: 0.0,
            role: SolidRole::FrameFloor,
            shape: crate::ResolvedSolidShape::Cuboid,
            supported_by: floor_support_nodes,
        });
        let mut floor_bearing_interfaces = Vec::new();
        for member in jetty_beams
            .iter()
            .filter_map(|id| builder.members.iter().find(|member| member.id == *id))
        {
            let inward = (member.start - member.end).normalize_or_zero();
            let contact = member.end + inward * (projection * 0.5) + Vec3::Y * section.x * 0.5;
            let interface = ResolvedItemId(
                (4_u64 << 60) | (u64::from(owner.0) << 32) | 0x300_000 | builder.next_interface,
            );
            builder.next_interface += 1;
            builder.geometry.support_interfaces.push(SupportInterface {
                id: interface,
                owner,
                node: member.end_node,
                bounds: ResolvedBounds {
                    min: contact - Vec3::new(0.07, 0.025, 0.07),
                    max: contact + Vec3::new(0.07, 0.025, 0.07),
                },
            });
            floor_bearing_interfaces.push(interface);
        }

        (floor_solid, floor_bearing_interfaces, outer_plane)
    }
}
