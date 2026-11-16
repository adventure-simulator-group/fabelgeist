//! Accepted ranged attacks complete on the server across control changes.

use super::*;

#[derive(Component, Clone, Copy)]
pub(crate) struct PendingRangedContact {
    pub(crate) target: Option<Entity>,
    pub(crate) resolve_at: CombatInstant,
}

pub(crate) fn resolve_pending_ranged_contacts(
    mut commands: Commands,
    time: Res<Time>,
    pending: Query<(Entity, &PendingRangedContact)>,
    config: Res<TacticalCombatConfig>,
) {
    let now = CombatInstant::from_elapsed(&time);
    let Some(precision) = ReportedPrecision::new(config.targeting.reported_hit_precision) else {
        return;
    };
    for (attacker, contact) in &pending {
        if now < contact.resolve_at {
            continue;
        }
        commands.entity(attacker).remove::<PendingRangedContact>();
        commands.trigger(RangedAttackIntent {
            attacker,
            target: contact.target,
            body_part: config.ai.ordinary.offense.target_body_part,
            reported_precision: precision,
        });
    }
}
