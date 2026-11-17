//! Reconnect projection retains authority and orders, rebuilding route decisions.

use super::*;

use bevy::ecs::query::QueryData;

#[derive(QueryData)]
pub(crate) struct ProjectedCore {
    pub(super) name: &'static Name,
    pub(super) player: &'static Player,
    pub(super) character_id: &'static CharacterId,
    pub(super) bestiary_categories: &'static BestiaryCategories,
    pub(super) skills: &'static Skills,
    pub(super) limbs: &'static Limbs,
    pub(super) attributes: &'static TacticalAttributes,
    pub(super) stats: &'static Stats,
    pub(super) combat_state: &'static TacticalCombatState,
    pub(super) equipment_action_state: &'static EquipmentActionState,
    pub(super) combat_side: &'static TacticalCombatSide,
    pub(super) control: &'static adventuresim_tactical_core::control::CharacterControl,
    pub(super) order: &'static adventuresim_tactical_core::control::CharacterOrder,
    pub(super) waypoints: &'static adventuresim_tactical_core::control::WaypointQueue,
    pub(super) policy: &'static adventuresim_tactical_core::control::BehaviorPolicy,
}
pub(super) type ProjectedPlayerCoreQuery<'w, 's> = Query<'w, 's, ProjectedCore>;

#[derive(QueryData)]
pub(crate) struct ProjectedMotion {
    pub(super) transform: &'static Transform,
    pub(super) look: &'static CharacterLook,
    pub(super) movement_intent: &'static AuthoritativeMovementIntent,
    pub(super) input_tick: &'static AuthoritativeInputTick,
    pub(super) motion_snapshot: &'static CharacterMotionSnapshot,
    pub(super) quickstep_push: &'static QuickstepPush,
    pub(super) melee_lunge: Option<&'static MeleeLungeMovement>,
    pub(super) posture_intent: &'static AuthoritativePostureIntent,
    pub(super) pace: &'static MovementPace,
    pub(super) mass: &'static Mass,
    pub(super) velocity: &'static LinearVelocity,
    pub(super) skeleton: &'static SkeletonState,
    pub(super) melee_authority: &'static MeleeAttackAuthority,
    pub(super) pending_melee_contact: Option<&'static PendingMeleeContact>,
    pub(super) ranged_authority: &'static RangedAttackAuthority,
    pub(super) ranged_contact: Option<&'static crate::combat::PendingRangedContact>,
}
pub(super) type ProjectedPlayerMotionQuery<'w, 's> = Query<'w, 's, ProjectedMotion>;

#[derive(Clone)]
pub(crate) struct ProjectedPlayerSnapshot {
    pub(super) name: Name,
    pub(super) player: Player,
    pub(super) character_id: CharacterId,
    pub(super) bestiary_categories: BestiaryCategories,
    pub(super) skills: Skills,
    pub(super) limbs: Limbs,
    pub(super) attributes: TacticalAttributes,
    pub(super) stats: Stats,
    pub(super) combat_state: TacticalCombatState,
    pub(super) equipment_action_state: EquipmentActionState,
    pub(super) combat_side: TacticalCombatSide,
    pub(super) control: adventuresim_tactical_core::control::CharacterControl,
    pub(super) order: adventuresim_tactical_core::control::CharacterOrder,
    pub(super) waypoints: adventuresim_tactical_core::control::WaypointQueue,
    pub(super) policy: adventuresim_tactical_core::control::BehaviorPolicy,
    pub(super) transform: Transform,
    pub(super) look: CharacterLook,
    pub(super) movement_intent: AuthoritativeMovementIntent,
    pub(super) input_tick: AuthoritativeInputTick,
    pub(super) motion_snapshot: CharacterMotionSnapshot,
    pub(super) quickstep_push: QuickstepPush,
    pub(super) melee_lunge: Option<MeleeLungeMovement>,
    pub(super) posture_intent: AuthoritativePostureIntent,
    pub(super) pace: MovementPace,
    pub(super) mass: Mass,
    pub(super) velocity: LinearVelocity,
    pub(super) skeleton: SkeletonState,
    pub(super) melee_authority: MeleeAttackAuthority,
    pub(super) pending_melee_contact: Option<PendingMeleeContact>,
    pub(super) ranged_authority: RangedAttackAuthority,
    pub(super) ranged_contact: Option<crate::combat::PendingRangedContact>,
    pub(super) collider: Collider,
    pub(super) collision_margin: CollisionMargin,
    pub(super) controller: CharacterController,
    pub(super) accumulated_input: AccumulatedInput,
}

impl DisconnectedProjection {
    pub(crate) fn combat_state_mut(&mut self) -> Option<&mut TacticalCombatState> {
        match self {
            Self::Loading(_) => None,
            Self::Projected(snapshot) => Some(&mut snapshot.combat_state),
        }
    }

    pub(super) fn projected(&self) -> bool {
        matches!(self, Self::Projected(_))
    }

    pub(super) fn insert(self, commands: &mut Commands, target: Entity) {
        match self {
            Self::Loading(loading) => {
                commands.entity(target).insert(loading);
            }
            Self::Projected(snapshot) => {
                let snapshot = *snapshot;
                commands.entity(target).insert((
                    snapshot.name,
                    snapshot.player,
                    snapshot.character_id,
                    snapshot.bestiary_categories,
                    snapshot.skills,
                    snapshot.limbs,
                    snapshot.attributes,
                    snapshot.stats,
                    snapshot.combat_state,
                    snapshot.equipment_action_state,
                    snapshot.combat_side,
                    snapshot.control,
                    snapshot.order,
                    snapshot.waypoints,
                    snapshot.policy,
                ));
                commands.entity(target).insert((
                    snapshot.transform,
                    snapshot.look,
                    snapshot.movement_intent,
                    snapshot.input_tick,
                    snapshot.motion_snapshot,
                    snapshot.quickstep_push,
                    snapshot.posture_intent,
                    snapshot.pace,
                    snapshot.mass,
                    snapshot.velocity,
                    snapshot.skeleton,
                    snapshot.melee_authority,
                    snapshot.ranged_authority,
                ));
                if snapshot.control.mode
                    == adventuresim_tactical_core::control::ControlMode::Indirect
                {
                    commands.queue(move |world: &mut World| {
                        let packages = crate::control::indirect_packages(
                            world.resource::<TacticalCombatConfig>(),
                        );
                        world.entity_mut(target).insert(packages);
                    });
                }
                if let Some(pending) = snapshot.ranged_contact {
                    commands.entity(target).insert(pending);
                }
                if let Some(pending) = snapshot.pending_melee_contact {
                    commands.entity(target).insert(pending);
                }
                if let Some(melee_lunge) = snapshot.melee_lunge {
                    commands.entity(target).insert(melee_lunge);
                }
                commands.entity(target).insert((
                    snapshot.collider,
                    snapshot.collision_margin,
                    snapshot.controller,
                    snapshot.accumulated_input,
                ));
            }
        }
    }
}

pub(super) const RECONNECT_GRACE_SECS: f32 = 30.0;

impl ProjectedPlayerSnapshot {
    pub(super) fn new(
        core: ProjectedCoreItem<'_, '_>,
        motion: ProjectedMotionItem<'_, '_>,
        physics: (
            &Collider,
            &CollisionMargin,
            &CharacterController,
            &AccumulatedInput,
        ),
    ) -> Self {
        Self {
            name: core.name.clone(),
            player: core.player.clone(),
            character_id: *core.character_id,
            bestiary_categories: core.bestiary_categories.clone(),
            skills: core.skills.clone(),
            limbs: core.limbs.clone(),
            attributes: core.attributes.clone(),
            stats: core.stats.clone(),
            combat_state: core.combat_state.clone(),
            equipment_action_state: *core.equipment_action_state,
            combat_side: *core.combat_side,
            control: *core.control,
            order: *core.order,
            waypoints: core.waypoints.clone(),
            policy: *core.policy,
            transform: *motion.transform,
            look: motion.look.clone(),
            movement_intent: *motion.movement_intent,
            input_tick: *motion.input_tick,
            motion_snapshot: *motion.motion_snapshot,
            quickstep_push: *motion.quickstep_push,
            melee_lunge: motion.melee_lunge.copied(),
            posture_intent: *motion.posture_intent,
            pace: *motion.pace,
            mass: *motion.mass,
            velocity: *motion.velocity,
            skeleton: motion.skeleton.clone(),
            melee_authority: motion.melee_authority.clone(),
            pending_melee_contact: motion.pending_melee_contact.copied(),
            ranged_authority: motion.ranged_authority.clone(),
            ranged_contact: motion.ranged_contact.copied(),
            collider: physics.0.clone(),
            collision_margin: *physics.1,
            controller: physics.2.clone(),
            accumulated_input: physics.3.clone(),
        }
    }
}
