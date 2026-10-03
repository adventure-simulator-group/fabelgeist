//! Minimal generated resident fixture for isolated authority acceptance checks.
use super::*;

#[spacetimedb::reducer]
pub fn authority_test_seed_resident(
    ctx: &ReducerContext,
    bootstrap_token: String,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    insert_resident_draft(
        ctx,
        "riverdale",
        ResidentDraft {
            character_id: 732004,
            seed: "authority-resident".into(),
            location: "residences".into(),
            service: None,
            profession: "householder".into(),
            role: "resident".into(),
            is_default: true,
            business_id: None,
            sex: Sex::Male,
            presentation: Presentation::Man,
            exact_age: Some(30),
            inherited_surname: None,
        },
    )
}

#[spacetimedb::reducer]
pub fn authority_test_seed_errantry_issuer(
    ctx: &ReducerContext,
    bootstrap_token: String,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    let order = adventuresim_core::organization::organization(
        crate::strategic::ERRANTRY_ISSUER_ORGANIZATION_ID,
    )
    .ok_or("Order definition missing")?;
    let chapter = order.chapters.first().ok_or("Order chapter missing")?;
    let settlement = ctx
        .db
        .settlement()
        .id()
        .find(&chapter.settlement_id)
        .ok_or("Chapter settlement missing")?;
    let id = adventuresim_core::organization::organization_representative_id(
        &chapter.settlement_id,
        &order.id,
    );
    let location = adventuresim_core::organization::chapter_effective_location_id(
        order,
        chapter,
        &settlement.economy,
    );
    insert_resident_draft(
        ctx,
        &chapter.settlement_id,
        ResidentDraft {
            character_id: id,
            seed: "authority-order-issuer".into(),
            location: location.into(),
            service: None,
            profession: "knight".into(),
            role: "representative".into(),
            is_default: true,
            business_id: None,
            sex: Sex::Male,
            presentation: Presentation::Man,
            exact_age: Some(30),
            inherited_surname: None,
        },
    )?;
    let mut profile = ctx
        .db
        .settlement_resident_profile()
        .character_id()
        .find(id)
        .ok_or("Chapter representative missing")?;
    profile.service_id.clear();
    profile.organization_id = order.id.clone();
    profile.conversation_id = "organization-representative".into();
    ctx.db
        .settlement_resident_profile()
        .character_id()
        .update(profile);
    Ok(())
}

#[spacetimedb::reducer]
pub fn authority_test_materialize_property_household(
    ctx: &ReducerContext,
    bootstrap_token: String,
    settlement_id: String,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    crate::strategic::require_strategic_gateway(ctx)?;
    let (drafts, groups) = settlement_resident_drafts(ctx, &settlement_id)?;
    let family = groups
        .into_iter()
        .find(|group| group.len() > 1)
        .ok_or("Generated family missing")?;
    for draft in drafts
        .into_iter()
        .filter(|draft| family.contains(&draft.character_id()))
    {
        insert_resident_draft(ctx, &settlement_id, draft)?;
    }
    crate::relationship::ensure_seeded_family_households(ctx, &settlement_id, &[family])?;
    crate::residence::bind_generated_households(ctx, &settlement_id)?;
    crate::residence::bind_generated_households(ctx, &settlement_id)?;
    Ok(())
}
