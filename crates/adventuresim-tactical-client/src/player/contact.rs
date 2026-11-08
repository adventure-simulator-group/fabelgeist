//! Prepare one checked predicted ray result before publishing combat events.

use super::*;

#[derive(EntityEvent)]
pub struct HitPerformed {
    pub entity: Entity,
    pub direction: Dir3,
    pub origin: Position<Scene>,
    pub length: SignedLength,
}

pub(super) struct PredictedAttackContact {
    pub(super) ray: HitPerformed,
    pub(super) result: PredictedContact,
}

pub(super) enum PredictedContact {
    Hit { body: Entity, body_part: BodyPart },
    Miss,
}

impl PredictedAttackContact {
    pub(super) fn observe(
        attacker: Entity,
        origin: Position<Scene>,
        reach: SignedLength,
        state: &AttackState,
        spatial: &SpatialQuery,
        colliders: &Query<(&GlobalTransform, &ColliderOf, &LimbHitbox)>,
        scene_items: &Query<Entity, With<TacticalSceneItem>>,
    ) -> MeleeGeometryResult<Self> {
        let Some(target) = state.target else {
            return Ok(Self {
                ray: HitPerformed {
                    entity: attacker,
                    direction: state.aim_direction,
                    origin,
                    length: reach,
                },
                result: PredictedContact::Miss,
            });
        };
        // Native scene-vector and Avian ray calculation ends at a checked
        // measured length. Keep the original direction and distance ordering.
        let intended_point = state.target_position.map(Position::metres)
            .unwrap_or(origin.metres() + *state.aim_direction * reach.metres());
        let delta = intended_point - origin.metres();
        let direction = Dir3::new(delta).unwrap_or(state.aim_direction);
        let excluded: Vec<_> = scene_items.iter().chain([attacker]).collect();
        let obstruction_filter = SpatialQueryFilter::from_excluded_entities(excluded);
        let obstruction = spatial.cast_ray(
            origin.metres(),
            direction,
            delta.length().min(reach.metres()),
            true,
            &obstruction_filter,
        );
        let unobstructed = obstruction.is_some_and(|hit| {
            hit.entity == target.body
                || colliders.get(hit.entity).is_ok_and(|(_, collider, _)| collider.body == target.body)
        });
        let distance = if unobstructed {
            obstruction.map_or(delta.length(), |hit| hit.distance)
        } else {
            obstruction.map_or(delta.length().min(reach.metres()), |hit| hit.distance)
        };
        Ok(Self {
            ray: HitPerformed {
                entity: attacker,
                direction,
                origin,
                length: SignedLength::from_metres(distance)?,
            },
            result: if unobstructed {
                PredictedContact::Hit { body: target.body, body_part: target.body_part }
            } else {
                PredictedContact::Miss
            },
        })
    }
}
