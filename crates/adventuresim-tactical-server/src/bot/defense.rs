use super::*;

/// Per-bot chance (out of 1.0) that a reflex defense (see
/// `try_start_reaction`) resolves to a dodge. Inserted by a reactive
/// defense behavior package with balance-tuned defaults; BRP tests
/// mutate it to force deterministic dodge outcomes (see
/// `DefenseChances` in `scripts/adventuresim_brp_lib.py`, regenerated via
/// `just generate-brp-types`).
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq)]
#[reflect(Component)]
pub struct DefenseChances {
    pub dodge_chance: f64,
}

impl Default for DefenseChances {
    fn default() -> Self {
        Self { dodge_chance: 0.2 }
    }
}

#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Eq)]
#[reflect(Component)]
pub struct ReactiveDefenseAi {
    pub requires_facing: bool,
}

impl Default for ReactiveDefenseAi {
    fn default() -> Self {
        Self {
            requires_facing: true,
        }
    }
}

#[derive(Component)]
pub(crate) struct PendingBotReaction {
    timer: Timer,
    choice: DefendRequest,
}

struct BotReactionAttempt<'a> {
    defender: Entity,
    attacker_look: &'a CharacterLook,
    defender_look: &'a CharacterLook,
    defense: ReactiveDefenseAi,
    chances: DefenseChances,
    windup_seconds: AnimationElapsed,
}

/// Every source reaches defense only after an accepted, perceived attack.
pub(super) fn on_perceived_threat(
    event: On<crate::control::perception::PerceivedThreat>,
    mut cmd: Commands,
    q_character: Query<&CharacterLook>,
    q_ai: Query<(
        &CharacterLook,
        &TacticalCombatState,
        &ReactiveDefenseAi,
        Option<&DefenseChances>,
    )>,
    combat_config: Res<TacticalCombatConfig>,
    mut random: ResMut<CombatRandom>,
) {
    let Ok(attacker_look) = q_character.get(event.attacker) else {
        return;
    };
    let Ok((defender_look, state, defense, chances)) = q_ai.get(event.defender) else {
        return;
    };
    if state.is_incapacitated() {
        return;
    }
    if let Err(error) = try_start_reaction(
        &mut cmd,
        BotReactionAttempt {
            defender: event.defender,
            attacker_look,
            defender_look,
            defense: *defense,
            chances: chances.copied().unwrap_or_default(),
            windup_seconds: event.windup,
        },
        &combat_config.ai.ordinary.defense,
        &mut random,
    ) {
        warn!(%error, "cannot schedule perceived defense");
    }
}

fn try_start_reaction(
    cmd: &mut Commands,
    attempt: BotReactionAttempt<'_>,
    config: &AiDefenseConfig,
    random: &mut CombatRandom,
) -> ElapsedResult<()> {
    let (a2, a1) = attempt.attacker_look.yaw.sin_cos();
    let (d2, d1) = attempt.defender_look.yaw.sin_cos();
    if attempt.defense.requires_facing
        && flanking_from_dir((a1, a2), (d1, d2)) > config.frontal_flanking_max
    {
        return Ok(());
    }
    let Some(choice) = roll_defend_choice(attempt.chances, random) else {
        return Ok(());
    };
    let delay = if attempt.chances.dodge_chance >= 1.0 {
        // The authored test dodger is deliberately anticipatory. Leave enough
        // of even a fast fist windup for the quickstep's launch phase.
        (attempt.windup_seconds.seconds() - 0.16).clamp(0.02, 0.12)
    } else {
        random.range_f32(
            config.reaction_delay_min_seconds.seconds(),
            config.reaction_delay_max_seconds.seconds(),
        )
    };
    let delay = AnimationElapsed::from_seconds(delay)?;
    let timer = Timer::new(delay.native_duration()?, TimerMode::Once);
    cmd.entity(attempt.defender)
        .insert(PendingBotReaction { timer, choice });
    Ok(())
}

pub(super) fn roll_defend_choice(
    chances: DefenseChances,
    random: &mut CombatRandom,
) -> Option<DefendRequest> {
    let roll = random.unit_f64();
    if roll < chances.dodge_chance {
        Some(DefendRequest::Dodge {
            direction: if random.coin_flip() {
                Vec2::X
            } else {
                Vec2::NEG_X
            },
        })
    } else {
        None
    }
}

pub(super) fn tick_bot_reactions(
    mut cmd: Commands,
    time: Res<Time<()>>,
    mut q_reacting: Query<(Entity, &mut PendingBotReaction, &TacticalCombatState)>,
) {
    for (bot, mut reaction, state) in &mut q_reacting {
        if state.is_incapacitated() {
            cmd.entity(bot).remove::<PendingBotReaction>();
            continue;
        }
        reaction.timer.tick(time.delta());
        if !reaction.timer.is_finished() {
            continue;
        }

        let choice = reaction.choice;
        cmd.entity(bot).remove::<PendingBotReaction>();
        cmd.trigger(DefendIntent {
            defender: bot,
            choice,
        });
    }
}
