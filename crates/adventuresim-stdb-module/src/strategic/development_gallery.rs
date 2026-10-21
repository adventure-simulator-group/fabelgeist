//! Curated development-gallery recipes, index dispatch and postconditions.

use super::*;

/// Register every fixture from the one strategic bootstrap and materialize
/// feature states against distinct primary characters.
/// The five puzzle scenarios materialized by the development gallery, in the
/// order they occupy the flat gallery-item index.
const GALLERY_PUZZLE_KINDS: [PuzzleKind; 5] = [
    PuzzleKind::OrderedSigils,
    PuzzleKind::TruthfulWitnesses,
    PuzzleKind::RuneTransformation,
    PuzzleKind::LogicGrid,
    PuzzleKind::ResourceAllocation,
];
/// Flat gallery-item indices 0..GALLERY_PUZZLE_BASE are the fixed scenarios
/// (static block, autopsy, outbreak, threat); the puzzles follow, then the
/// road encounters. Splitting the gallery across these indices lets each
/// SpacetimeDB reducer call stay under the per-reducer compute budget.
const GALLERY_PUZZLE_BASE: usize = 4;
const GALLERY_ROAD_BASE: usize = GALLERY_PUZZLE_BASE + GALLERY_PUZZLE_KINDS.len();

/// Materialize a single flat gallery item. Returns `Ok(false)` once `index` is
/// past the end of the gallery, so callers can drive materialization
/// incrementally (one bounded transaction per item) without knowing the total
/// road-encounter count. `index == 0` seeds the static scenarios in one call
/// (they are cheap registrations); every other index does exactly one scenario.
pub(crate) fn materialize_gallery_item(
    ctx: &ReducerContext,
    index: usize,
) -> StrategicCombatResult<bool> {
    match index {
        0 => materialize_static_scenarios(ctx)?,
        1 => materialize_autopsy_scenario(ctx)?,
        2 => materialize_outbreak_scenario(ctx)?,
        3 => materialize_recurring_threat_scenario(ctx)?,
        i if i < GALLERY_ROAD_BASE => {
            let offset = i - GALLERY_PUZZLE_BASE;
            let kind = GALLERY_PUZZLE_KINDS[offset];
            let slug = format!("puzzle-{}", kind.slug());
            let character_id = 9_999_999_999_999_950 - offset as u64;
            ensure_scenario_character(
                ctx,
                character_id,
                &format!("{} Puzzle Tester", kind.slug()),
            )?;
            let materialized = materialize_order_errantry(
                ctx,
                character_id,
                None,
                ErrantryLaunch::DirectDemoCamp(kind),
            )?;
            register_development_scenario(
                ctx,
                &slug,
                "Puzzles",
                &format!("{} puzzle", kind.slug().replace('-', " ")),
                "Solve this puzzle from its ordinary journey-camp entry state.",
                character_id,
                "/locations/camp",
            )?;
            register_development_subject(ctx, &slug, "case", &materialized.case_id)?;
        }
        i => {
            let ordinal = i - GALLERY_ROAD_BASE;
            let definitions = adventuresim_core::road_encounter_catalog::definitions();
            let Some(definition) = definitions.get(ordinal) else {
                return Ok(false);
            };
            let scenario_slug = format!("road-{}", definition.id);
            let character_id = 9_999_999_999_990_000_u64.saturating_sub(ordinal as u64);
            ensure_scenario_character(ctx, character_id, &format!("Road Tester {}", ordinal + 1))?;
            let occurrence_id =
                materialize_development_road_encounter(ctx, character_id, &definition.id)?;
            let label = definition.cast.first().map_or_else(
                || definition.id.replace(['-', '_'], " "),
                |speaker| format!("Encounter with {}", speaker.name),
            );
            register_development_scenario(
                ctx,
                &scenario_slug,
                "Road encounters",
                &label,
                "Play this compiled encounter through its ordinary journey-camp presentation.",
                character_id,
                "/locations/camp",
            )?;
            register_development_subject(ctx, &scenario_slug, "road_encounter", &occurrence_id)?;
        }
    }
    Ok(true)
}

/// Postcondition validation makes a partial gallery abort transactionally. Run
/// once after every gallery item has been materialized.
pub(crate) fn validate_gallery_postcondition(ctx: &ReducerContext) -> Result<(), String> {
    for scenario in ctx.db.development_scenario().iter() {
        if ctx
            .db
            .character()
            .id()
            .find(scenario.primary_character_id)
            .is_none()
        {
            return Err(format!(
                "Scenario {} has no primary character",
                scenario.slug
            ));
        }
    }
    Ok(())
}

pub(crate) fn materialize_development_scenario_gallery(
    ctx: &ReducerContext,
) -> StrategicCombatResult<()> {
    let mut index = 0;
    while materialize_gallery_item(ctx, index)? {
        index += 1;
    }
    validate_gallery_postcondition(ctx)?;
    Ok(())
}

fn materialize_static_scenarios(ctx: &ReducerContext) -> StrategicCombatResult<()> {
    let static_scenarios = [
        (
            "health-disease-party",
            "Health",
            "Disease diagnosis party",
            "Diagnose a party containing every authored disease stage.",
            9_999_999_999_999_998,
            "/characters",
        ),
        (
            "health-wounded-party",
            "Health",
            "Wounds and surgery",
            "Treat wounds, retained projectiles, splints, and damaged equipment.",
            9_999_999_999_999_999,
            "/characters",
        ),
        (
            "knowledge-religion",
            "Knowledge",
            "Religion knowledge",
            "Inspect the complete bounded religion skill presentation.",
            9_999_999_999_999_988,
            "/characters",
        ),
        (
            "knowledge-bestiary",
            "Knowledge",
            "Bestiary knowledge",
            "Inspect broad bestiary category knowledge and evidence.",
            9_999_999_999_999_987,
            "/characters",
        ),
        (
            "knowledge-herbalism",
            "Knowledge",
            "Herbalism and foraging",
            "Exercise every bounded herbalism method, public grade, and the terrain-backed foraging flow.",
            9_999_999_999_999_986,
            "/characters",
        ),
        (
            "social-affinity",
            "Social",
            "Affinity and courtship",
            "Exercise visible affinity, belief, morale, and social actions.",
            9_999_999_999_999_977,
            "/characters",
        ),
        (
            "social-prayer",
            "Social",
            "Zealous prayer",
            "Exercise conviction-sensitive prayer interactions.",
            9_999_999_999_999_975,
            "/characters",
        ),
    ];
    for (slug, category, label, description, character_id, route) in static_scenarios {
        register_development_scenario(
            ctx,
            slug,
            category,
            label,
            description,
            character_id,
            route,
        )?;
    }
    Ok(())
}

fn materialize_autopsy_scenario(ctx: &ReducerContext) -> StrategicCombatResult<()> {
    const AUTOPSY_ID: u64 = 9_999_999_999_999_960;
    ensure_scenario_character(ctx, AUTOPSY_ID, "Anatomist Demo")?;
    crate::corpse::seed_autopsy_demo(ctx, AUTOPSY_ID)?;
    register_development_scenario(
        ctx,
        "health-autopsy",
        "Health",
        "Autopsy",
        "Examine a deterministic corpse through the ordinary settlement UI.",
        AUTOPSY_ID,
        "/characters",
    )?;
    Ok(())
}

fn materialize_outbreak_scenario(ctx: &ReducerContext) -> StrategicCombatResult<()> {
    const OUTBREAK_ID: u64 = 9_999_999_999_999_959;
    const OUTBREAK_SETTLEMENT: &str = "dev-scenario-outbreak";
    ensure_scenario_settlement(ctx, OUTBREAK_SETTLEMENT, "Outbreak Scenario Hamlet")?;
    ensure_scenario_character_at(
        ctx,
        OUTBREAK_ID,
        "Outbreak Investigator",
        OUTBREAK_SETTLEMENT,
    )?;
    let outbreak_problem_id = seed_outbreak_demo(ctx, OUTBREAK_ID)?;
    crate::local_problem::discover_development_problem(
        ctx,
        OUTBREAK_ID,
        &outbreak_problem_id,
        "quest-outbreak",
    )?;
    register_development_scenario(
        ctx,
        "quest-outbreak",
        "Quests",
        "Discovered outbreak",
        "Continue a deterministic outbreak from its ordinary rumor-derived journal entry, referral, and investigation actions.",
        OUTBREAK_ID,
        "/quests",
    )?;
    register_development_subject(
        ctx,
        "quest-outbreak",
        "generated_problem",
        &outbreak_problem_id,
    )?;
    Ok(())
}

fn materialize_recurring_threat_scenario(ctx: &ReducerContext) -> StrategicCombatResult<()> {
    const THREAT_ID: u64 = 9_999_999_999_999_958;
    const THREAT_SETTLEMENT: &str = "dev-scenario-recurring-threat";
    ensure_scenario_settlement(ctx, THREAT_SETTLEMENT, "Threat Scenario Hamlet")?;
    ensure_scenario_character_at(ctx, THREAT_ID, "Threat Investigator", THREAT_SETTLEMENT)?;
    if let Some(mut attributes) = ctx.db.character_attributes().character_id().find(THREAT_ID) {
        attributes.instinct = 5.0;
        ctx.db
            .character_attributes()
            .character_id()
            .update(attributes);
    }
    if let Some(mut skills) = ctx.db.character_skills().character_id().find(THREAT_ID) {
        skills.charm_hours = skills.charm_hours.max(10_000.0);
        skills.command_hours = skills.command_hours.max(10_000.0);
        skills.oral_languages.east_central = 10_000.0;
        skills.oral_languages.west_central = 10_000.0;
        skills.oral_languages.low = 10_000.0;
        skills.oral_languages.yiddish = 10_000.0;
        skills.oral_languages.latin = 10_000.0;
        skills.oral_languages.romani = 10_000.0;
        skills.oral_languages.elven = 10_000.0;
        skills.oral_languages.dwarfish = 10_000.0;
        ctx.db.character_skills().character_id().update(skills);
    }
    ensure_recurring_threat_provisions(ctx, THREAT_ID)?;
    debug_assert!(
        adventuresim_core::strategic_action::assess_negotiated_withdrawal(5.0, 1.0, 0.0, 50,)
            .accepted
    );
    debug_assert!(
        adventuresim_core::strategic_action::assess_hostile_surrender(5.0, 1.0, 0.0, 50, 6_000)
            .accepts_demand
    );
    let threat_problem_id = materialize_preferred_generated_fixture(
        ctx,
        THREAT_ID,
        adventuresim_core::quest_generation::TemplateFamily::RecurringDepredation,
        fabelgeist_determinism::Seed::from_u64(0x5448_5245_4154_0001),
    )?;
    ensure_recurring_threat_offer_awareness(ctx, &threat_problem_id)?;
    crate::local_problem::discover_development_problem(
        ctx,
        THREAT_ID,
        &threat_problem_id,
        "quest-recurring-threat",
    )?;
    register_development_scenario(
        ctx,
        "quest-recurring-threat",
        "Quests",
        "Combat, withdrawal, or surrender",
        "Follow a deterministic sapient hostile threat to its case site, then resolve it through ordinary combat, negotiated withdrawal, or whole-group pre-combat surrender.",
        THREAT_ID,
        "/quests",
    )?;
    register_development_subject(
        ctx,
        "quest-recurring-threat",
        "generated_problem",
        &threat_problem_id,
    )?;
    Ok(())
}
