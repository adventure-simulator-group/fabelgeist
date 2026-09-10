//! Camera composition queries authoritative solid envelopes; rendering remains unchanged.
use adventuresim_building_generator::{CollisionCuboid, ResolvedItemId};
use bevy::prelude::*;

const CAMERA_CLEARANCE_METRES: f32 = 0.12;
use super::super::ROOM_FOV_DEGREES;
const REVIEW_ASPECT: f32 = crate::tactical_scene_viewer::VIEW_WIDTH as f32
    / crate::tactical_scene_viewer::VIEW_HEIGHT as f32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Owner {
    Architecture(ResolvedItemId),
    Furniture(usize),
}

pub(super) struct Blocker {
    pub owner: Owner,
    pub solid: CollisionCuboid,
}

pub(super) struct Subject {
    pub owner: Owner,
    pub points: Vec<Vec3>,
    pub importance: f32,
    pub signature: bool,
}

struct PreparedBlocker {
    owner: Owner,
    centre: Vec3,
    inverse_rotation: Quat,
    half: Vec3,
    world_half: Vec3,
    world_padding: Vec3,
}

impl From<&Blocker> for PreparedBlocker {
    fn from(blocker: &Blocker) -> Self {
        let solid = &blocker.solid;
        let rotation = Quat::from_rotation_y(solid.yaw_radians)
            * Quat::from_rotation_x(solid.crossfall_radians)
            * Quat::from_rotation_z(solid.longfall_radians);
        let [x, y, z] = [Vec3::X, Vec3::Y, Vec3::Z].map(|axis| (rotation * axis).abs());
        let half = solid.size * 0.5;
        Self {
            owner: blocker.owner,
            centre: solid.centre,
            inverse_rotation: rotation.inverse(),
            half,
            world_half: x * half.x + y * half.y + z * half.z,
            world_padding: x + y + z,
        }
    }
}

impl PreparedBlocker {
    /// Broad world bounds reject separate segments before an exact oriented slab test.
    fn segment_hit(&self, eye: Vec3, target: Vec3, margin: f32) -> Option<f32> {
        let world_half = self.world_half + self.world_padding * margin;
        if eye.min(target).cmpgt(self.centre + world_half).any()
            || eye.max(target).cmplt(self.centre - world_half).any()
        {
            return None;
        }
        let start = self.inverse_rotation * (eye - self.centre);
        let delta = self.inverse_rotation * (target - eye);
        let half = self.half + Vec3::splat(margin);
        let (mut enter, mut leave) = (0.0_f32, 1.0_f32);
        for axis in 0..3 {
            if delta[axis].abs() < f32::EPSILON {
                if start[axis].abs() > half[axis] {
                    return None;
                }
            } else {
                let a = (-half[axis] - start[axis]) / delta[axis];
                let b = (half[axis] - start[axis]) / delta[axis];
                enter = enter.max(a.min(b));
                leave = leave.min(a.max(b));
                if enter > leave {
                    return None;
                }
            }
        }
        Some(enter)
    }
}

fn clear_line(eye: Vec3, target: Vec3, owner: Owner, blockers: &[PreparedBlocker]) -> bool {
    blockers
        .iter()
        .all(|blocker| blocker.owner == owner || blocker.segment_hit(eye, target, 0.0).is_none())
}

fn in_frame(point: Vec3, eye: Vec3, forward: Vec3) -> bool {
    let right = forward.cross(Vec3::Y).normalize_or_zero();
    let up = right.cross(forward);
    let relative = point - eye;
    let depth = relative.dot(forward);
    let half_height = depth * (ROOM_FOV_DEGREES.to_radians() * 0.5).tan() * 0.88;
    depth > 0.4
        && relative.dot(right).abs() < half_height * REVIEW_ASPECT
        && relative.dot(up).abs() < half_height
}

/// Choose a clear view of the requested signature furniture from proven route points.
pub(super) fn choose(
    eyes: &[Vec3],
    subjects: &[Subject],
    blockers: &[Blocker],
) -> Option<(Vec3, Vec3)> {
    let blockers: Vec<_> = blockers.iter().map(PreparedBlocker::from).collect();
    let needs_signature = subjects.iter().any(|subject| subject.signature);
    let mut best: Option<(f32, Vec3, Vec3)> = None;
    for &eye in eyes {
        if blockers.iter().any(|blocker| {
            blocker
                .segment_hit(eye, eye, CAMERA_CLEARANCE_METRES)
                .is_some()
        }) {
            continue;
        }
        let visible: Vec<Vec<_>> = subjects
            .iter()
            .map(|subject| {
                subject
                    .points
                    .iter()
                    .copied()
                    .filter(|point| clear_line(eye, *point, subject.owner, &blockers))
                    .collect()
            })
            .collect();
        for &target in visible.iter().flatten() {
            let offset = target - eye;
            if offset.length() < 1.0 || offset.xz().length() < 0.8 {
                continue;
            }
            let forward = offset.normalize();
            let mut signature_visible = false;
            let mut score = 0.0;
            for (subject, points) in subjects.iter().zip(&visible) {
                let framed = points
                    .iter()
                    .filter(|point| in_frame(**point, eye, forward))
                    .count();
                let fraction = framed as f32 / subject.points.len() as f32;
                signature_visible |= subject.signature && fraction >= 0.4;
                // Saturate the distance reward: retain a room ensemble instead of
                // selecting an extreme close-up of one large foreground object.
                let distance = eye.distance(subject.points[0]);
                let legibility = (6.0 / distance.max(3.0)).min(1.0);
                score += subject.importance * fraction * legibility;
            }
            if needs_signature && !signature_visible {
                continue;
            }
            if best.is_none_or(|(previous, _, _)| score > previous) {
                best = Some((score, eye, target));
            }
        }
    }
    best.map(|(_, eye, target)| (eye, target))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wall() -> CollisionCuboid {
        CollisionCuboid {
            source: ResolvedItemId(1),
            centre: Vec3::ZERO,
            size: Vec3::new(0.2, 3.0, 2.0),
            yaw_radians: 0.0,
            crossfall_radians: 0.0,
            longfall_radians: 0.0,
        }
    }

    fn segment_hit(eye: Vec3, target: Vec3, solid: &CollisionCuboid, margin: f32) -> Option<f32> {
        PreparedBlocker::from(&Blocker {
            owner: Owner::Architecture(solid.source),
            solid: *solid,
        })
        .segment_hit(eye, target, margin)
    }

    #[test]
    fn broad_bounds_preserve_hits_on_rotated_sloping_and_padded_shapes() {
        let mut solid = wall();
        solid.centre = Vec3::new(12.0, 4.0, -7.0);
        solid.yaw_radians = 0.8;
        solid.crossfall_radians = 0.3;
        solid.longfall_radians = -0.2;
        let rotation = Quat::from_rotation_y(solid.yaw_radians)
            * Quat::from_rotation_x(solid.crossfall_radians)
            * Quat::from_rotation_z(solid.longfall_radians);
        let eye = solid.centre + rotation * Vec3::new(-2.0, 0.0, 0.0);
        let target = solid.centre + rotation * Vec3::new(2.0, 0.0, 0.0);
        let hit = segment_hit(eye, target, &solid, 0.0).unwrap();
        assert!((hit - 0.475).abs() < 0.0001);
        let padded_corner = solid.centre + rotation * (solid.size * 0.5 + Vec3::splat(0.05));
        assert!(segment_hit(padded_corner, padded_corner, &solid, 0.0).is_none());
        assert!(
            segment_hit(
                padded_corner,
                padded_corner,
                &solid,
                CAMERA_CLEARANCE_METRES
            )
            .is_some()
        );
    }

    #[test]
    fn sightlines_respect_partition_rotation_and_clear_doorway_space() {
        let mut partition = wall();
        assert!(
            segment_hit(
                Vec3::new(-2.0, 1.0, 0.0),
                Vec3::new(2.0, 1.0, 0.0),
                &partition,
                0.0
            )
            .is_some()
        );
        assert!(
            segment_hit(
                Vec3::new(-2.0, 1.0, 2.0),
                Vec3::new(2.0, 1.0, 2.0),
                &partition,
                0.0
            )
            .is_none()
        );
        partition.yaw_radians = std::f32::consts::FRAC_PI_2;
        assert!(
            segment_hit(
                Vec3::new(0.0, 1.0, -2.0),
                Vec3::new(0.0, 1.0, 2.0),
                &partition,
                0.0
            )
            .is_some()
        );
    }

    #[test]
    fn signature_view_rejects_farthest_route_point_behind_partition() {
        let blocked = Vec3::new(-4.0, 1.55, 0.0);
        let clear = Vec3::new(3.0, 1.55, 3.0);
        let subject = Subject {
            owner: Owner::Furniture(0),
            points: vec![Vec3::new(3.0, 0.8, 0.0)],
            importance: 4.0,
            signature: true,
        };
        let blocker = Blocker {
            owner: Owner::Architecture(ResolvedItemId(1)),
            solid: wall(),
        };
        let (eye, _) = choose(&[blocked, clear], &[subject], &[blocker]).unwrap();
        assert_eq!(eye, clear);
    }
}
