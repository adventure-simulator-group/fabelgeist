//! Character origin selection from the current loaded settlements.
use super::*;

pub(super) fn choose_start_settlement(
    ctx: &ReducerContext,
    options: &CharacterCreationOptions<'_>,
    starting: Option<&StartingCharacterSpec>,
) -> Result<Settlement, String> {
    let settlements: Vec<Settlement> = ctx.db.settlement().iter().collect();
    if settlements.is_empty() {
        return Err("Cannot create a character before at least one settlement is loaded".into());
    }
    let mut settlements = settlements;
    settlements.sort_by(|left, right| left.id.cmp(&right.id));
    let selector = starting.map_or_else(
        || {
            if options.mode.is_npc() {
                options.stable_seed
            } else {
                ctx.random::<u64>()
            }
        },
        |spec| spec.settlement_selector,
    );
    let selected = if let Some(origin_settlement_id) = options.origin_settlement_id {
        settlements
            .iter()
            .find(|settlement| settlement.id == origin_settlement_id)
            .ok_or_else(|| format!("Unknown origin settlement {origin_settlement_id}"))?
    } else if let Some(starting_organization) = starting.and_then(|spec| spec.organization.as_ref())
    {
        let organization =
            adventuresim_core::organization::organization(&starting_organization.organization_id)
                .ok_or("Starting organization is not in the catalog")?;
        let eligible = settlements
            .iter()
            .filter(|settlement| {
                organization.has_chapter(&settlement.id)
                    && organization.recognition.includes(&settlement.id)
            })
            .collect::<Vec<_>>();
        if eligible.is_empty() {
            // Small development worlds do not load the researched Viabundus
            // settlements referenced by the organization catalog. Keep the
            // professional package intact and place the character
            // deterministically in the loaded world; a complete world still
            // prefers a recognized chapter settlement below.
            log::warn!(
                "No loaded settlement hosts starting organization {}; using a loaded settlement",
                organization.id
            );
            &settlements[NPC_SETTLEMENT_SELECTION_DOMAIN
                .rng(selector, &[])
                .index(settlements.len())]
        } else {
            eligible[NPC_SETTLEMENT_SELECTION_DOMAIN
                .rng(selector, &[])
                .index(eligible.len())]
        }
    } else {
        &settlements[NPC_SETTLEMENT_SELECTION_DOMAIN
            .rng(selector, &[])
            .index(settlements.len())]
    };

    Ok(selected.clone())
}
