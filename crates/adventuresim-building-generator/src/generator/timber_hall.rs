//! Hall framing shares clear door approaches and high knee-braced bays.
use super::*;

const KNEE_BRACE_REACH_METRES: f32 = 0.60;
const KNEE_BRACE_PLATE_CLEARANCE_METRES: f32 = 0.01;
const KNEE_BRACE_SECTION_SCALE: f32 = 0.82;
pub(super) const POST_SECTION_SCALE: f32 = 1.15;
const BAY_TARGET_LENGTH_METRES: f32 = 3.0;
const POST_STATION_SEARCH_STEP_METRES: f32 = 0.05;
const MINIMUM_POST_SPACING_METRES: f32 = 0.8;
const GROUND_DOOR_SILL_TOLERANCE_METRES: f32 = 0.15;
const KNEE_BRACE_MAXIMUM_BAY_SHARE: f32 = 0.4;

pub(super) fn post_stations(
    centre: Vec2,
    tangent: Vec2,
    cross: Vec2,
    row_offset: f32,
    length: f32,
    section: Vec2,
    openings: &[crate::OpeningAssembly],
) -> Vec<f32> {
    let count = (length / BAY_TARGET_LENGTH_METRES).ceil() as usize;
    let nominal = (0..=count)
        .map(|i| -length * 0.5 + length * i as f32 / count as f32)
        .collect::<Vec<_>>();
    let mut stations = Vec::new();
    for (index, &preferred) in nominal.iter().enumerate() {
        let min = stations.last().map_or(-length * 0.5, |previous| {
            previous + MINIMUM_POST_SPACING_METRES
        });
        let max = nominal
            .get(index + 1)
            .map_or(length * 0.5, |next| next - MINIMUM_POST_SPACING_METRES);
        let clear = |along: f32| {
            [-1.0, 1.0].into_iter().all(|side| {
                let point = centre + tangent * along + cross * row_offset * side;
                openings
                    .iter()
                    .filter(|door| {
                        matches!(
                            door.use_kind,
                            crate::OpeningUse::Door | crate::OpeningUse::Gate
                        ) && door.sill_elevation_metres < GROUND_DOOR_SILL_TOLERANCE_METRES
                    })
                    .all(|door| {
                        let delta = point - door.frame.origin;
                        let half_width = door.profile.interior_width_metres() * 0.5
                            + door.closure.swing_clearance_metres * 0.5
                            + section.max_element() * 0.5;
                        let approach =
                            door.closure.swing_clearance_metres + section.max_element() * 0.5;
                        delta.dot(door.frame.tangent).abs() > half_width
                            || delta.dot(door.frame.outward).abs() > approach
                    })
            })
        };
        let steps = ((max - min) / POST_STATION_SEARCH_STEP_METRES).ceil() as usize;
        let selected = (0..=steps)
            .map(|i| (min + i as f32 * POST_STATION_SEARCH_STEP_METRES).min(max))
            .chain([preferred])
            .filter(|&v| v >= min && v <= max && clear(v))
            .min_by(|a, b| (a - preferred).abs().total_cmp(&(b - preferred).abs()))
            .unwrap_or(preferred);
        stations.push(selected);
    }
    stations
}

pub(super) fn aisle_head_braces(
    builder: &mut TimberFrameBuilder<'_>,
    a: Vec2,
    b: Vec2,
    height: f32,
    section: Vec2,
) -> [crate::TimberMemberId; 2] {
    let reach = KNEE_BRACE_REACH_METRES.min(a.distance(b) * KNEE_BRACE_MAXIMUM_BAY_SHARE);
    [(a, b), (b, a)].map(|(post, opposite)| {
        let direction = (opposite - post).normalize_or_zero();
        let brace_section = section * KNEE_BRACE_SECTION_SCALE;
        let reach = clear_plate_reach(
            builder.geometry,
            post,
            direction,
            height,
            reach,
            brace_section,
        );
        let tie_contact = post + direction * reach;
        builder.member(
            crate::TimberMemberRole::HeadBrace,
            Vec3::new(post.x, height - reach, post.y),
            Vec3::new(tie_contact.x, height, tie_contact.y),
            brace_section,
            crate::TimberFramePhase::PrimaryConstruction,
        )
    })
}

/// Keep the complete diagonal section above partition plates crossed by its bay.
fn clear_plate_reach(
    geometry: &ResolvedGeometry,
    post: Vec2,
    direction: Vec2,
    height: f32,
    desired: f32,
    section: Vec2,
) -> f32 {
    let half = section.length() * 0.5;
    let end = post + direction * desired;
    let min = post.min(end) - Vec2::splat(half);
    let max = post.max(end) + Vec2::splat(half);
    geometry
        .solids
        .iter()
        .filter(|solid| solid.role == SolidRole::FramePlate)
        .map(yaw_bounds)
        .filter(|bounds| {
            bounds.min.x < max.x
                && bounds.max.x > min.x
                && bounds.min.z < max.y
                && bounds.max.z > min.y
                && bounds.max.y < height
        })
        .map(|bounds| height - bounds.max.y - half - KNEE_BRACE_PLATE_CLEARANCE_METRES)
        .fold(desired, f32::min)
}
