//! Guarded acceptance of physical projectile identity across injury commitment.

use super::*;

#[reducer]
pub fn authority_test_projectile_identity(
    ctx: &ReducerContext,
    bootstrap_token: String,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    let character_id = 732063;
    crate::character::create_named_character_with_id(
        ctx,
        character_id,
        "Projectile Fixture".into(),
    )?;
    for (kind, limb) in [
        (ProjectileKind::Arrowhead, BodyRegion::LeftArm),
        (ProjectileKind::Ball, BodyRegion::RightArm),
    ] {
        commit_hit_injury(ctx, character_id, limb, 0.1, 0.1, Some(kind))?;
        let records = ctx
            .db
            .retained_projectile()
            .character_id()
            .filter(character_id)
            .filter(|row| row.limb == limb)
            .collect::<Vec<_>>();
        if records.len() != 1
            || records[0].kind != kind
            || records[0].source_damage != 0.2
            || !records[0].extraction_dc.is_finite()
            || records[0].extraction_dc <= 0.0
            || injury_for(ctx, (character_id).into(), limb).cut_damage != 0.1
            || injury_for(ctx, (character_id).into(), limb).bruise_damage != 0.1
        {
            return Err("Retained projectile lost committed hit identity or damage".into());
        }
    }
    commit_hit_injury(
        ctx,
        character_id,
        BodyRegion::Head,
        0.0,
        0.0,
        Some(ProjectileKind::Ball),
    )?;
    if ctx
        .db
        .retained_projectile()
        .character_id()
        .filter(character_id)
        .count()
        != 2
    {
        return Err("Zero damage retained a projectile".into());
    }
    Ok(())
}
