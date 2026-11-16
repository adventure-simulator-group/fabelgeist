use std::time::Duration;

use super::defense::PendingBotReaction;
use super::trees::behaviour_in_world;
use super::*;
use crate::combat::{
    MeleeAttackAuthority, MeleeAttackIntent, PendingDefenderResponse, TacticalWounds,
};
use crate::player_projection::AuthoritativePostureIntent;

#[test]
fn fixture_stems_resolve_for_direct_server_commands() {
    assert!(
        resolve_scene_fixture("dense-woodland")
            .unwrap()
            .ends_with("assets/tactical-scenes/dense-woodland.json")
    );
    assert_eq!(
        load_enemy_fixture("animation-demo").unwrap().enemy_count(),
        5
    );
}

#[derive(Resource, Default)]
struct RecordedAttacks(Vec<(Entity, Entity)>);

#[derive(Resource, Default)]
struct RecordedRangedAttacks(Vec<(Entity, Entity)>);

fn record_attack(event: On<MeleeAttackIntent>, mut attacks: ResMut<RecordedAttacks>) {
    attacks.0.push((event.attacker, event.target));
}

fn record_ranged_attack(event: On<RangedAttackIntent>, mut attacks: ResMut<RecordedRangedAttacks>) {
    if let Some(target) = event.target {
        attacks.0.push((event.attacker, target));
    }
}

fn apply_deterministic_test_hit(
    event: On<MeleeAttackIntent>,
    mut combatants: Query<(&mut Limbs, &mut TacticalCombatState, &mut TacticalWounds)>,
) {
    let Ok([attacker, defender]) = combatants.get_many_mut([event.attacker, event.target]) else {
        return;
    };
    let (_, mut attacker_state, _) = attacker;
    let (mut defender_limbs, mut defender_state, mut defender_wounds) = defender;
    crate::combat::apply_transient_attack_result(
        &mut attacker_state,
        &mut defender_limbs,
        &mut defender_state,
        &mut defender_wounds.0,
        AttackResult::ToDefender {
            cut_damage: 160.0,
            blunt_damage: 0.0,
            balance_damage: 0.0,
            contact_force: 160.0,
            armor_impact: None,
        },
        BodyPart::Head,
    );
}

fn spawn_test_ai(world: &mut World, side: TacticalCombatSide, position: Vec3) -> Entity {
    let behaviour = behaviour_in_world(world, STANDARD_COMBAT_TREE).unwrap();
    // Temporary characters receive this production loadout from
    // `insert_new_character` when no authored starting package is given.
    const KATZBALGER_REACH: f32 = 0.8;
    let actor = world
        .spawn((
            Player::default(),
            Transform::from_translation(position)
                .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
            side,
            CharacterLook::default(),
            input::AccumulatedInput::default(),
            AuthoritativeMovementIntent::default(),
            behaviour,
            TacticalCombatState::default(),
            TacticalWounds::default(),
            SkeletonState::default(),
            CharacterDimensions::default(),
            Collider::cylinder(0.4, 1.9),
            MeleeAttackAuthority::default(),
        ))
        .id();
    let weapon = world.spawn(ItemOf(actor)).id();
    // Match production's insertion order: the equip hook must see both
    // the owning inventory relationship and the weapon classification.
    world.entity_mut(weapon).insert(WeaponItem {
        striking_material: adventuresim_core::item_catalog_schema::EquipmentMaterial::RoughSteel,
        skill_weights: [0.0; 9],

        prefers_stab: false,
        precision: 1.0,
        reach: KATZBALGER_REACH,
        grip_to_tip_m: KATZBALGER_REACH,
        moment_of_inertia_kg_m2: 0.0,

        melee: true,
        ranged: false,
    });
    world.entity_mut(weapon).insert(EquipSlot::HoldingRight);
    actor
}

fn spawn_test_ranged_ai(
    world: &mut World,
    side: TacticalCombatSide,
    position: Vec3,
) -> (Entity, Entity) {
    const TEST_WEAPON_REACH: f32 = 8.0;
    let behaviour = behaviour_in_world(world, STANDARD_COMBAT_TREE).unwrap();
    let actor = world
        .spawn((
            Player::default(),
            Transform::from_translation(position)
                .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
            side,
            CharacterLook::default(),
            input::AccumulatedInput::default(),
            AuthoritativeMovementIntent::default(),
            behaviour,
            TacticalCombatState::default(),
            TacticalWounds::default(),
            SkeletonState::default(),
            CharacterDimensions::default(),
            Collider::cylinder(0.4, 1.9),
            MeleeAttackAuthority::default(),
        ))
        .id();
    let weapon = world.spawn(ItemOf(actor)).id();
    world.entity_mut(weapon).insert(WeaponItem {
        striking_material: adventuresim_core::item_catalog_schema::EquipmentMaterial::RoughSteel,
        skill_weights: [0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0],

        prefers_stab: false,
        precision: 1.0,
        reach: TEST_WEAPON_REACH,
        grip_to_tip_m: 1.0,
        moment_of_inertia_kg_m2: 0.0,

        melee: true,
        ranged: true,
    });
    world.entity_mut(weapon).insert(EquipSlot::HoldingRight);
    let ammo = world
        .spawn((
            ItemOf(actor),
            ItemProperties {
                id: ARROW_ID.to_owned(),
                weight: 0.05,
            },
            TacticalItemQuantity::default(),
        ))
        .id();
    (actor, ammo)
}

fn spawn_test_target(world: &mut World, side: TacticalCombatSide, position: Vec3) -> Entity {
    world
        .spawn((
            Player::default(),
            Transform::from_translation(position)
                .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
            side,
            CharacterLook::default(),
            TacticalCombatState::default(),
            SkeletonState::default(),
            Collider::cylinder(0.4, 1.9),
        ))
        .id()
}

/// Runs the bot behaviour trees without movement physics or combat resolution, so
/// tests drive the bot's behaviour-tree decisions directly.
fn bot_app() -> App {
    let mut app = App::new();
    app.add_plugins((TaskPoolPlugin::default(), AssetPlugin::default()))
        .insert_resource(Time::<()>::default())
        .init_resource::<TacticalCombatConfig>()
        .insert_resource(CombatRandom::seeded(
            fabelgeist_determinism::Seed::from_u64(1),
        ))
        .add_plugins(BotPlugin);
    app
}

fn advance_frame(app: &mut App) {
    app.world_mut()
        .resource_mut::<Time<()>>()
        .advance_by(Duration::from_millis(100));
    app.update();
}

fn test_app() -> App {
    let mut app = bot_app();
    app.init_resource::<RecordedAttacks>()
        .init_resource::<RecordedRangedAttacks>()
        .add_observer(record_attack)
        .add_observer(record_ranged_attack)
        .add_observer(crate::combat::on_melee_attack_started)
        .add_systems(Update, crate::combat::resolve_pending_melee_contacts);
    app
}

fn spawn_bot(world: &mut World, tree: &str, side: TacticalCombatSide) -> Entity {
    let behaviour = behaviour_in_world(world, tree).unwrap();
    world
        .spawn((
            Player::default(),
            Transform::default(),
            side,
            CharacterLook::default(),
            AuthoritativeMovementIntent::default(),
            TacticalCombatState::default(),
            SkeletonState::default(),
            behaviour,
        ))
        .id()
}

#[test]
fn trees_grant_their_capabilities() {
    let mut app = bot_app();
    let world = app.world_mut();
    let passive = spawn_bot(world, "passive.bt.ron", TacticalCombatSide::Enemy);
    let blocker = spawn_bot(
        world,
        "always_block_without_facing.bt.ron",
        TacticalCombatSide::Enemy,
    );
    let standard = spawn_bot(world, "standard_combat.bt.ron", TacticalCombatSide::Enemy);
    let dodger = spawn_bot(world, "always_dodge.bt.ron", TacticalCombatSide::Enemy);

    app.update();

    let world = app.world();
    for bot in [passive, blocker, standard, dodger] {
        assert!(world.entity(bot).contains::<Behaviour>());
    }
    let raised = |bot| world.get::<SkeletonState>(bot).unwrap().weapon_guard();
    assert_eq!(raised(passive), WeaponGuardState::default());
    assert_eq!(raised(blocker), WeaponGuardState::Raised);
    assert_eq!(raised(standard), WeaponGuardState::Raised);
    assert_eq!(raised(dodger), WeaponGuardState::Raised);

    assert!(!world.entity(blocker).contains::<ReactiveDefenseAi>());
    assert!(!world.entity(blocker).contains::<DefenseChances>());
    assert!(
        world
            .get::<ReactiveDefenseAi>(standard)
            .unwrap()
            .requires_facing
    );
    assert_eq!(
        world.get::<DefenseChances>(standard),
        Some(&DefenseChances::default())
    );
    assert!(
        world
            .get::<ReactiveDefenseAi>(dodger)
            .unwrap()
            .requires_facing
    );
    assert_eq!(
        world.get::<DefenseChances>(dodger),
        Some(&DefenseChances { dodge_chance: 1.0 })
    );
}

#[test]
fn untargeted_windup_only_reacts_on_the_nearest_enemy() {
    let mut app = App::new();
    app.init_resource::<TacticalCombatConfig>()
        .insert_resource(CombatRandom::seeded(
            fabelgeist_determinism::Seed::from_u64(1),
        ));
    app.add_observer(on_attack_started);
    let attacker = app
        .world_mut()
        .spawn((
            CharacterLook::default(),
            Transform::default(),
            TacticalCombatSide::Party,
        ))
        .id();
    let passive = app
        .world_mut()
        .spawn((
            MissionEnemy,
            CharacterLook::default(),
            Transform::from_xyz(0.0, 0.0, 1.0),
            TacticalCombatSide::Enemy,
            TacticalCombatState::default(),
        ))
        .id();
    let blocker = app
        .world_mut()
        .spawn((
            MissionEnemy,
            CharacterLook::default(),
            Transform::from_xyz(0.0, 0.0, 2.0),
            TacticalCombatSide::Enemy,
            TacticalCombatState::default(),
            ReactiveDefenseAi {
                requires_facing: false,
            },
            DefenseChances { dodge_chance: 1.0 },
        ))
        .id();
    let start = FromClient {
        client_id: adventuresim_tactical_netcode::bevy_replicon::prelude::ClientId::Client(
            attacker,
        ),
        message: MeleeActionRequest {
            strike_family: StrikeFamily::Swing,
            hand: AttackHand::Main,
            target: None,
        },
    };

    app.world_mut().trigger(start);
    app.world_mut().flush();
    assert!(!app.world().entity(blocker).contains::<PendingBotReaction>());

    app.world_mut().despawn(passive);
    app.world_mut().trigger(start);
    app.world_mut().flush();
    assert!(app.world().entity(blocker).contains::<PendingBotReaction>());
}

#[test]
fn completed_bot_reaction_enters_the_shared_dodge_animation() {
    let mut app = App::new();
    app.insert_resource(Time::<()>::default())
        .init_resource::<TacticalCombatConfig>()
        .insert_resource(CombatRandom::seeded(
            fabelgeist_determinism::Seed::from_u64(1),
        ))
        .add_observer(on_attack_started)
        .add_observer(crate::combat::apply_defend_intent)
        .add_systems(Update, tick_bot_reactions);
    let attacker = app
        .world_mut()
        .spawn((
            CharacterLook::default(),
            Transform::default(),
            TacticalCombatSide::Party,
        ))
        .id();
    let blocker = app
        .world_mut()
        .spawn((
            MissionEnemy,
            CharacterLook::default(),
            Transform::from_xyz(0.0, 0.0, 1.0),
            TacticalCombatSide::Enemy,
            TacticalCombatState::default(),
            SkeletonState::default().with_weapon_guard(WeaponGuardState::Raised),
            AuthoritativePostureIntent::default(),
            QuickstepPush::default(),
            ReactiveDefenseAi {
                requires_facing: false,
            },
            DefenseChances { dodge_chance: 1.0 },
        ))
        .id();
    app.world_mut().trigger(FromClient {
        client_id: adventuresim_tactical_netcode::bevy_replicon::prelude::ClientId::Client(
            attacker,
        ),
        message: MeleeActionRequest {
            strike_family: StrikeFamily::Swing,
            hand: AttackHand::Main,
            target: None,
        },
    });
    app.world_mut().flush();
    app.world_mut()
        .resource_mut::<Time<()>>()
        .advance_by(Duration::from_secs(1));

    app.update();
    app.world_mut().flush();

    assert_eq!(
        app.world()
            .get::<SkeletonState>(blocker)
            .unwrap()
            .action_kind(),
        SkeletonAction::Dodge
    );
    assert!(
        app.world()
            .get::<PendingDefenderResponse>(blocker)
            .is_some()
    );
}

#[test]
fn dodger_faces_the_nearest_opponent() {
    let mut app = bot_app();
    let dodger = spawn_bot(
        app.world_mut(),
        "always_dodge.bt.ron",
        TacticalCombatSide::Enemy,
    );
    app.world_mut().spawn((
        Player::default(),
        Transform::from_xyz(2.0, 0.0, 0.0),
        TacticalCombatSide::Party,
        TacticalCombatState::default(),
        SkeletonState::default(),
    ));

    app.update();

    let look = app.world().get::<CharacterLook>(dodger).unwrap();
    assert!((look.yaw + std::f32::consts::FRAC_PI_2).abs() < 0.0001);
}

#[test]
fn passive_tree_starts_authored_get_up_without_skipping_upright() {
    let mut app = bot_app();
    let prone = spawn_bot(app.world_mut(), "passive.bt.ron", TacticalCombatSide::Enemy);
    app.world_mut()
        .entity_mut(prone)
        .insert(SkeletonState::default().with_body_state(BodyState::Prone));

    app.update();

    let skeleton = app.world().get::<SkeletonState>(prone).unwrap();
    assert_eq!(skeleton.body(), BodyState::Prone);
    assert_eq!(
        skeleton.posture_transition().unwrap().kind(),
        PostureTransitionKind::ProneToUpright
    );
}

#[test]
fn ranged_ai_fires_then_falls_back_to_melee_when_ammo_is_exhausted() {
    assert!(!ranged_weapon_needs_ammo_lookup(false, 1.0));
    assert!(!ranged_weapon_needs_ammo_lookup(true, 0.0));
    assert!(ranged_weapon_needs_ammo_lookup(true, 8.0));
    let mut app = test_app();
    let (actor, ammo) = spawn_test_ranged_ai(
        app.world_mut(),
        TacticalCombatSide::Enemy,
        Vec3::new(0.0, 0.0, 2.0),
    );
    let target = spawn_test_target(
        app.world_mut(),
        TacticalCombatSide::Party,
        Vec3::new(0.0, 0.0, -2.0),
    );

    for _ in 0..30 {
        app.world_mut()
            .resource_mut::<Time<()>>()
            .advance_by(Duration::from_millis(100));
        app.update();
        if !app.world().resource::<RecordedRangedAttacks>().0.is_empty() {
            break;
        }
    }
    assert_eq!(
        app.world().resource::<RecordedRangedAttacks>().0,
        vec![(actor, target)]
    );
    assert!(app.world().resource::<RecordedAttacks>().0.is_empty());

    // Production consumes this through `resolve_ranged_attack`; removing
    // it here isolates deterministic controller selection from physics.
    app.world_mut().despawn(ammo);
    // Put the target just outside the hybrid weapon's preferred measure,
    // while remaining inside its absolute melee reach. In the
    // production schedule movement would create this separation while the
    // bot retreats; this controller-only test intentionally has no motor.
    app.world_mut()
        .entity_mut(target)
        .get_mut::<Transform>()
        .unwrap()
        .translation = Vec3::new(0.0, 0.0, -5.0);
    for _ in 0..30 {
        app.world_mut()
            .resource_mut::<Time<()>>()
            .advance_by(Duration::from_millis(100));
        app.update();
        if !app.world().resource::<RecordedAttacks>().0.is_empty() {
            break;
        }
    }

    assert!(
        app.world()
            .resource::<RecordedAttacks>()
            .0
            .contains(&(actor, target)),
        "ammo exhaustion should return a melee-capable weapon to melee cadence"
    );
    assert_eq!(app.world().resource::<RecordedRangedAttacks>().0.len(), 1);
}

#[test]
fn opposing_ai_approach_face_stop_and_both_attack() {
    let mut app = test_app();
    let party = spawn_test_ai(
        app.world_mut(),
        TacticalCombatSide::Party,
        Vec3::new(0.0, 0.0, -3.0),
    );
    let enemy = spawn_test_ai(
        app.world_mut(),
        TacticalCombatSide::Enemy,
        Vec3::new(0.0, 0.0, 3.0),
    );

    // Simulate only the small, deterministic movement seam driven by the
    // existing controller input. No physics, wall clock, spawn randomness,
    // or global RNG is involved in this headless behavior test.
    for _ in 0..20 {
        app.world_mut()
            .resource_mut::<Time<()>>()
            .advance_by(Duration::from_millis(100));
        app.update();

        let snapshots: Vec<_> = [party, enemy]
            .into_iter()
            .map(|actor| {
                let entity = app.world().entity(actor);
                (
                    actor,
                    entity.get::<AuthoritativeMovementIntent>().unwrap().0,
                    entity.get::<CharacterLook>().unwrap().yaw,
                )
            })
            .collect();
        for (actor, movement, yaw) in snapshots {
            if movement.is_some() {
                let forward = Vec3::new(-yaw.sin(), 0.0, -yaw.cos());
                app.world_mut()
                    .entity_mut(actor)
                    .get_mut::<Transform>()
                    .unwrap()
                    .translation += forward * 0.5;
            }
        }
    }

    let party_transform = app.world().entity(party).get::<Transform>().unwrap();
    let enemy_transform = app.world().entity(enemy).get::<Transform>().unwrap();
    let separation = party_transform
        .translation
        .xz()
        .distance(enemy_transform.translation.xz());
    const KATZBALGER_REACH: f32 = 0.8;
    const TWO_CHARACTER_COLLIDER_RADII: f32 = 0.8;
    let combat_config = TacticalCombatConfig::default();
    let quickstep_distance = quickstep_target_displacement_metres(
        combat_config
            .movement
            .motor
            .reference_quickstep_leg_length_metres,
        &combat_config.movement.motor,
    );
    assert!(
        separation
            <= maximum_melee_lunge_range(
                CharacterDimensions::default().arm_reach_metres,
                KATZBALGER_REACH,
                quickstep_distance,
            ) + TWO_CHARACTER_COLLIDER_RADII,
        "AI stopped outside reachable lunge range: {separation}"
    );
    assert!(
        separation
            > melee_interaction_range(
                CharacterDimensions::default().arm_reach_metres,
                KATZBALGER_REACH,
            ) + TWO_CHARACTER_COLLIDER_RADII,
        "AI should commit its attack while the lunge still has a gap to close: {separation}"
    );
    assert!(
        separation >= TWO_CHARACTER_COLLIDER_RADII,
        "AI test movement collapsed production-sized character colliders: {separation}"
    );
    for (actor, target) in [(party, enemy), (enemy, party)] {
        let entity = app.world().entity(actor);
        let target_entity = app.world().entity(target);
        let yaw = entity.get::<CharacterLook>().unwrap().yaw;
        let forward = Vec2::new(-yaw.sin(), -yaw.cos());
        let toward_target = (target_entity.get::<Transform>().unwrap().translation.xz()
            - entity.get::<Transform>().unwrap().translation.xz())
        .normalize();
        let facing = forward.dot(toward_target);
        assert!(
            facing > 0.999,
            "AI {actor:?} yaw {yaw} faces {forward:?}, target direction {toward_target:?}, dot {facing}, separation {separation}"
        );
    }
    assert_eq!(
        app.world()
            .entity(party)
            .get::<AuthoritativeMovementIntent>()
            .unwrap()
            .0,
        None
    );
    assert_eq!(
        app.world()
            .entity(enemy)
            .get::<AuthoritativeMovementIntent>()
            .unwrap()
            .0,
        None
    );

    let attacks = &app.world().resource::<RecordedAttacks>().0;
    assert!(attacks.contains(&(party, enemy)));
    assert!(attacks.contains(&(enemy, party)));
}

#[test]
fn bot_holds_guard_instead_of_starting_into_an_already_committed_attack() {
    let mut app = test_app();
    let actor = spawn_test_ai(
        app.world_mut(),
        TacticalCombatSide::Party,
        Vec3::new(0.0, 0.0, 0.0),
    );
    let target = spawn_test_target(
        app.world_mut(),
        TacticalCombatSide::Enemy,
        Vec3::new(0.0, 0.0, 1.5),
    );
    app.world_mut()
        .entity_mut(target)
        .get_mut::<SkeletonState>()
        .unwrap()
        .begin_attack(
            AttackSpec::default(),
            adventuresim_tactical_core::animation::AnimationSampleTick::from(0u64),
            adventuresim_tactical_core::animation::AnimationSampleTick::from(1_000u64),
        )
        .unwrap();
    app.world_mut()
        .entity_mut(target)
        .get_mut::<SkeletonState>()
        .unwrap()
        // Just before the windup's half: plainly visible, still committed.
        .advance_action(adventuresim_tactical_core::animation::AnimationSampleTick::from(499u64))
        .unwrap();
    // Instinct this high reads a windup this visible every time, so the
    // seeded recognition roll cannot let the bot trade into the strike.
    let mut attributes = TacticalAttributes::default();
    attributes.0.instinct = 6.0;
    app.world_mut().entity_mut(actor).insert(attributes);

    for _ in 0..20 {
        advance_frame(&mut app);
    }
    assert!(app.world().resource::<RecordedAttacks>().0.is_empty());

    *app.world_mut()
        .entity_mut(target)
        .get_mut::<SkeletonState>()
        .unwrap() = SkeletonState::default();
    for _ in 0..20 {
        app.world_mut()
            .resource_mut::<Time<()>>()
            .advance_by(Duration::from_millis(100));
        app.update();
        if app
            .world()
            .resource::<RecordedAttacks>()
            .0
            .contains(&(actor, target))
        {
            break;
        }
    }
    assert!(
        app.world()
            .resource::<RecordedAttacks>()
            .0
            .contains(&(actor, target))
    );
}

#[test]
fn committed_defense_reenters_seeded_assessment_before_counterattack() {
    let mut app = test_app();
    let actor = spawn_test_ai(
        app.world_mut(),
        TacticalCombatSide::Party,
        Vec3::new(0.0, 0.0, 0.0),
    );
    let attacker = spawn_test_target(
        app.world_mut(),
        TacticalCombatSide::Enemy,
        Vec3::new(0.0, 0.0, 1.5),
    );

    app.world_mut()
        .trigger(crate::combat::MeleeAttackCommittedToDefense {
            defender: actor,
            incoming_attacker: attacker,
            canceled_attack_key: MeleeAttackKey::from(7),
            response: DefenderResponse::Parry {
                input_reflex: 0.8,
                precision: 0.6,
            },
            engagement: 0.4,
        });
    app.world_mut().flush();
    assert!(app.world().entity(actor).contains::<InitiativeLost>());

    // The next decision reassesses, which regains the initiative.
    for _ in 0..3 {
        advance_frame(&mut app);
    }
    assert!(!app.world().entity(actor).contains::<InitiativeLost>());
}

#[test]
fn nearest_target_ties_use_entity_identity() {
    let mut app = test_app();
    let actor = spawn_test_ai(app.world_mut(), TacticalCombatSide::Party, Vec3::ZERO);
    let first = spawn_test_ai(
        app.world_mut(),
        TacticalCombatSide::Enemy,
        Vec3::new(-2.0, 0.0, 0.0),
    );
    let second = spawn_test_ai(
        app.world_mut(),
        TacticalCombatSide::Enemy,
        Vec3::new(2.0, 0.0, 0.0),
    );
    app.update();

    let selected = app.world().get::<BotTarget>(actor).map(|target| target.0);
    let expected = if first.to_bits() < second.to_bits() {
        first
    } else {
        second
    };
    assert_eq!(selected, Some(expected));
}

#[test]
fn ai_duel_stops_incapacitated_combatants() {
    let mut app = bot_app();
    app.init_resource::<RecordedAttacks>()
        .add_observer(record_attack)
        .add_observer(apply_deterministic_test_hit)
        .add_observer(crate::combat::on_melee_attack_started)
        .add_systems(
            Update,
            (
                crate::combat::update_tactical_combat_state,
                crate::combat::resolve_pending_melee_contacts,
            )
                .chain(),
        );
    let party = spawn_test_ai(
        app.world_mut(),
        TacticalCombatSide::Party,
        Vec3::new(0.0, 0.0, -3.0),
    );
    let enemy = spawn_test_ai(
        app.world_mut(),
        TacticalCombatSide::Enemy,
        Vec3::new(0.0, 0.0, 3.0),
    );

    for _ in 0..30 {
        app.world_mut()
            .resource_mut::<Time<()>>()
            .advance_by(Duration::from_millis(100));
        app.update();
        let snapshots: Vec<_> = [party, enemy]
            .into_iter()
            .map(|actor| {
                let entity = app.world().entity(actor);
                (
                    actor,
                    entity.get::<AuthoritativeMovementIntent>().unwrap().0,
                    entity.get::<CharacterLook>().unwrap().yaw,
                )
            })
            .collect();
        for (actor, movement, yaw) in snapshots {
            if movement.is_some() {
                let forward = Vec3::new(-yaw.sin(), 0.0, -yaw.cos());
                app.world_mut()
                    .entity_mut(actor)
                    .get_mut::<Transform>()
                    .unwrap()
                    .translation += forward * 0.5;
            }
        }
        if [party, enemy].into_iter().any(|actor| {
            app.world()
                .entity(actor)
                .get::<TacticalCombatState>()
                .is_some_and(TacticalCombatState::is_incapacitated)
        }) {
            break;
        }
    }

    let incapacitated: Vec<_> = [party, enemy]
        .into_iter()
        .filter(|actor| {
            app.world()
                .entity(*actor)
                .get::<TacticalCombatState>()
                .unwrap()
                .is_incapacitated()
        })
        .collect();
    assert!(!incapacitated.is_empty());
    for actor in incapacitated {
        assert_eq!(
            app.world()
                .entity(actor)
                .get::<AuthoritativeMovementIntent>()
                .unwrap()
                .0,
            None
        );
    }
    assert!(!app.world().resource::<RecordedAttacks>().0.is_empty());

    let party_defeated = app
        .world()
        .entity(party)
        .get::<TacticalCombatState>()
        .unwrap()
        .is_incapacitated();
    let enemy_defeated = app
        .world()
        .entity(enemy)
        .get::<TacticalCombatState>()
        .unwrap()
        .is_incapacitated();
    let resolution = crate::mission::terminal_resolution(crate::mission::TerminalCombatSnapshot {
        required_enemies: 1,
        loaded_enemies: 1,
        incapacitated_enemies: u32::from(enemy_defeated),
        loaded_party: 1,
        incapacitated_party: u32::from(party_defeated),
        enrollment_sealed: true,
    });
    let expected = if party_defeated {
        Some(adventuresim_stdb_client::TacticalMissionResolution::Failed)
    } else {
        Some(adventuresim_stdb_client::TacticalMissionResolution::Defeated)
    };
    assert_eq!(resolution, expected);
}
