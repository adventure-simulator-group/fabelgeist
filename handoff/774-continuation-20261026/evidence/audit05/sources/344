use bevy::prelude::*;
use fabelgeist_geometry::SignedLength;

#[derive(Clone, Copy)]
pub(crate) enum TargetingRange {
    Bounded(SignedLength),
    MovementPrevalidated,
}

/// Native scene X/Y/Z metres at the final cursor-ranking numerical kernel.
pub(crate) struct TargetingCandidate<T> {
    pub(crate) candidate: T,
    pub(crate) position: Vec3,
    pub(crate) stable_key: u64,
}

/// Selects the in-range candidate closest to the camera's forward axis.
/// Distance only breaks equal cursor alignment, matching ground-item auto aim.
pub(crate) fn auto_aim_candidate<T: Copy>(
    camera_origin: Vec3,
    camera_forward: Vec3,
    actor_origin: Vec3,
    range: TargetingRange,
    candidates: impl IntoIterator<Item = TargetingCandidate<T>>,
) -> Option<T> {
    let camera_forward = camera_forward.try_normalize()?;
    // Scene-vector alignment, squared distances and ordering stay in this
    // complete native numerical kernel. Max-zero normalization is unchanged.
    let maximum_distance_squared = match range {
        TargetingRange::Bounded(distance) => Some(distance.metres().max(0.0).powi(2)),
        TargetingRange::MovementPrevalidated => None,
    };
    candidates
        .into_iter()
        .filter_map(|TargetingCandidate { candidate, position, stable_key }| {
            let actor_distance_squared = position.distance_squared(actor_origin);
            if maximum_distance_squared.is_some_and(|maximum| actor_distance_squared > maximum) {
                return None;
            }
            let alignment = (position - camera_origin)
                .try_normalize()
                .map_or(-1.0, |direction| direction.dot(camera_forward));
            let angular_error = 1.0 - alignment.clamp(-1.0, 1.0);
            Some((candidate, angular_error, actor_distance_squared, stable_key))
        })
        .min_by(|left, right| {
            left.1
                .total_cmp(&right.1)
                .then(left.2.total_cmp(&right.2))
                .then(left.3.cmp(&right.3))
        })
        .map(|(candidate, _, _, _)| candidate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_aim_prefers_cursor_alignment_over_character_distance() {
        assert_eq!(
            auto_aim_candidate(
                Vec3::Y,
                Vec3::NEG_Z,
                Vec3::ZERO,
                TargetingRange::Bounded(SignedLength::from_metres(2.0).unwrap()),
                [
                    TargetingCandidate { candidate: "nearby side", position: Vec3::new(0.5, 0.0, 0.0), stable_key: 1 },
                    TargetingCandidate { candidate: "pointed", position: Vec3::new(0.0, 0.0, -1.8), stable_key: 2 },
                ],
            ),
            Some("pointed")
        );
    }

    #[test]
    fn auto_aim_has_no_cursor_cone() {
        assert_eq!(
            auto_aim_candidate(
                Vec3::Y,
                Vec3::NEG_Z,
                Vec3::ZERO,
                TargetingRange::Bounded(SignedLength::from_metres(2.0).unwrap()),
                [TargetingCandidate { candidate: "behind", position: Vec3::new(0.0, 0.0, 1.5), stable_key: 1 }],
            ),
            Some("behind")
        );
    }

    #[test]
    fn auto_aim_excludes_candidates_outside_range() {
        assert_eq!(
            auto_aim_candidate(
                Vec3::Y,
                Vec3::NEG_Z,
                Vec3::ZERO,
                TargetingRange::Bounded(SignedLength::from_metres(2.0).unwrap()),
                [TargetingCandidate { candidate: "far", position: Vec3::new(0.0, 0.0, -2.01), stable_key: 1 }],
            ),
            None
        );
    }
}
