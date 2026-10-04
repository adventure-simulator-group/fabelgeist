//! Guarded persistence checks of treatment request lifetime and replay.

use super::*;

fn injured_actor(ctx: &ReducerContext, id: u64) -> Result<(), String> {
    crate::character::create_named_character_with_id(ctx, id, "Treatment Fixture".into())?;
    let mut actor = ctx.db.character().id().find(id).ok_or("Actor missing")?;
    actor.current_settlement_id = Some("riverdale".into());
    ctx.db.character().id().update(actor);
    crate::item::upsert_surgery_items(ctx);
    crate::item::add_inventory_item_checked(ctx, id, "bandage", 1)?;
    let mut injury = blank_injury(id, BodyRegion::LeftArm);
    injury.cut_damage = 0.1;
    store_injury(ctx, injury);
    Ok(())
}

#[reducer]
pub fn authority_test_treatment_receipts(
    ctx: &ReducerContext,
    bootstrap_token: String,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    let actor = 732031;
    injured_actor(ctx, actor)?;
    let treat = |id, soap| {
        treat_limb(
            ctx,
            actor,
            actor,
            BodyRegion::LeftArm.slug().into(),
            SurgeryProcedure::Bandage,
            None,
            soap,
            id,
            None,
            None,
        )
    };
    let supplies_before = item_quantity(ctx, actor, "bandage");
    treat("completed".into(), false)?;
    let injury = injury_for(ctx, actor, BodyRegion::LeftArm);
    let minute = ctx
        .db
        .character_time()
        .character_id()
        .find(actor)
        .ok_or("Clock missing")?
        .minutes;
    let supplies_after = item_quantity(ctx, actor, "bandage");
    if supplies_after + 1 != supplies_before
        || !injury.bandaged
        || ctx
            .db
            .treatment_action_receipt()
            .id()
            .find(format!("treatment:{actor}:completed"))
            .is_none()
    {
        return Err("Completed treatment lacks injury effect or receipt".into());
    }
    treat("completed".into(), false)?;
    if injury_for(ctx, actor, BodyRegion::LeftArm).cut_damage != injury.cut_damage
        || ctx
            .db
            .character_time()
            .character_id()
            .find(actor)
            .ok_or("Clock missing")?
            .minutes
            != minute
        || item_quantity(ctx, actor, "bandage") != supplies_after
    {
        return Err("Exact treatment replay reapplied effects".into());
    }
    if treat("completed".into(), true).err().as_deref() != Some("Conflicting treatment retry") {
        return Err("Treatment receipt accepted conflicting parameters".into());
    }
    let interrupted = 732032;
    injured_actor(ctx, interrupted)?;
    let mut condition = ctx
        .db
        .character_condition()
        .character_id()
        .find(interrupted)
        .ok_or("Condition missing")?;
    condition.current_blood_ml = condition.maximum_blood_ml * 0.09;
    ctx.db
        .character_condition()
        .character_id()
        .update(condition);
    let supplies = item_quantity(ctx, interrupted, "bandage");
    treat_limb(
        ctx,
        interrupted,
        interrupted,
        BodyRegion::LeftArm.slug().into(),
        SurgeryProcedure::Bandage,
        None,
        false,
        "interrupted".into(),
        None,
        None,
    )?;
    if ctx
        .db
        .treatment_action_receipt()
        .id()
        .find(format!("treatment:{interrupted}:interrupted"))
        .is_some()
        || injury_for(ctx, interrupted, BodyRegion::LeftArm).bandaged
        || item_quantity(ctx, interrupted, "bandage") != supplies
    {
        return Err("Interrupted treatment committed a receipt or consumed supplies".into());
    }
    Ok(())
}
