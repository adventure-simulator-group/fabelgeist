//! A knee brace can close a frame through exact joints on continuous members.
use crate::TimberFrameMember;
use bevy::math::Vec3;

const JOINT_ALIGNMENT_METRES: f32 = 0.004;
const MINIMUM_BRACED_DOUBLE_AREA: f32 = 0.08;

pub(super) fn closes_triangle(brace: &TimberFrameMember, members: &[&TimberFrameMember]) -> bool {
    members
        .iter()
        .filter(|first| first.id != brace.id && point_on_member(brace.start, first))
        .any(|first| {
            members
                .iter()
                .filter(|second| {
                    second.id != brace.id
                        && second.id != first.id
                        && point_on_member(brace.end, second)
                })
                .any(|second| {
                    [(first.start_node, first.start), (first.end_node, first.end)]
                        .into_iter()
                        .any(|(node, point)| {
                            (node == second.start_node || node == second.end_node)
                                && (brace.end - brace.start)
                                    .cross(point - brace.start)
                                    .length()
                                    > MINIMUM_BRACED_DOUBLE_AREA
                        })
                })
        })
}
fn point_on_member(point: Vec3, member: &TimberFrameMember) -> bool {
    let axis = member.end - member.start;
    let t = ((point - member.start).dot(axis) / axis.length_squared()).clamp(0.0, 1.0);
    point.distance(member.start + axis * t) <= JOINT_ALIGNMENT_METRES
}

#[test]
fn high_knee_braces_require_exact_contact_with_the_post_and_tie() {
    let plan = crate::generate(&crate::BuildingProgram::fixture(
        crate::BuildingArchetype::HallHouse,
        42,
    ))
    .unwrap();
    let frame = plan.timber_frame.as_ref().unwrap();
    let members = frame.members.iter().collect::<Vec<_>>();
    let brace = frame
        .members
        .iter()
        .find(|m| {
            m.role == crate::TimberMemberRole::HeadBrace
                && m.start.y > 1.8
                && closes_triangle(m, &members)
        })
        .unwrap();
    let mut detached = brace.clone();
    detached.start += Vec3::new(0.17, 0.13, 0.11);
    assert!(!closes_triangle(&detached, &members));
}
