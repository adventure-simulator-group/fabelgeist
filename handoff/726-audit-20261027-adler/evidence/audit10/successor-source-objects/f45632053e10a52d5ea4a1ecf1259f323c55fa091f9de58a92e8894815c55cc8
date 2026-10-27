use super::*;

pub(super) struct RangedContact<'a> {
    pub(super) shot: &'a authority::AuthorizedRangedShot,
    pub(super) target_transform: &'a Transform,
    pub(super) target: Entity,
    pub(super) body_part: BodyPart,
    pub(super) result: AttackResult,
    pub(super) defender_response: DefenderResponse,
    pub(super) flanking: f32,
    pub(super) attacker_weapon_slot: EquipSlot,
    pub(super) defender_blocking_slot: Option<EquipSlot>,
}

impl RangedContact<'_> {
    pub(super) fn publish(
        self,
        commands: &mut Commands,
        viewer: &TacticalPlayerViewer,
        attacker_transform: &Transform,
        attacker_view: &TacticalPlayerView<'_, '_, '_>,
        defender_view: &TacticalPlayerView<'_, '_, '_>,
        config: &TacticalCombatConfig,
    ) {
        let Self {
            target,
            body_part,
            result,
            defender_response,
            flanking,
            attacker_weapon_slot,
            defender_blocking_slot,
            ..
        } = self;
        let (impact_recipient, impact_velocity_change, impact_point, impact_normal) =
            authoritative_impact(
                result,
                self.shot.attacker(),
                attacker_transform.translation,
                attacker_view.body_weight() + attacker_view.inventory_weight(),
                target,
                self.target_transform,
                defender_view.body_weight() + defender_view.inventory_weight(),
                body_part,
                config,
            );
        let impact_effects = authoritative_impact_effects(
            &viewer.inventory,
            self.shot.attacker(),
            AttackHand::Main,
            result,
        );
        publish(
            commands,
            target,
            attacker_weapon_slot,
            defender_blocking_slot,
            SuccessfulAttackResponse {
                attacker: self.shot.attacker(),
                hit: vec![target],
                body_part,
                result,
                flanking,
                defender_response,
                impact_recipient,
                impact_velocity_change,
                impact_point,
                impact_normal,
                impact_effects,
            },
        );
    }
}

/// Commit the same resolved ranged contact to gameplay and client feedback.
pub(super) fn publish(
    commands: &mut Commands,
    target: Entity,
    attacker_weapon_slot: EquipSlot,
    defender_blocking_slot: Option<EquipSlot>,
    response: SuccessfulAttackResponse,
) {
    commands.trigger(ApplyMeleeAttackResult {
        attacker: response.attacker,
        target,
        body_part: response.body_part,
        anatomical_subregion: anatomical_subregion(response.body_part, 0.5),
        surface_coordinate: 0.5,
        result: response.result,
        defender_response: response.defender_response,
        defense_success_probability: None,
        defense_alignment_sample: None,
        defense_engagement: None,
        attacker_weapon_slot,
        defender_blocking_slot,
        attacker_weapon_contact: false,
        impact_recipient: response.impact_recipient,
        impact_velocity_change: response.impact_velocity_change,
        contact_at_time: MeleeContactAtTime::intended(0.0),
    });
    commands.server_trigger(ToClients {
        targets: SendTargets::All,
        message: response,
    });
}
