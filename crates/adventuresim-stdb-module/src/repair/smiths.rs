//! Stable settlement craft skills and smith initialization.
use super::*;

pub(super) fn stable_skill(settlement_id: &str, stream: fabelgeist_determinism::StreamId) -> u8 {
    3 + fabelgeist_determinism::Seed::derive(settlement_id.as_bytes(), stream, &[])
        .rng()
        .index(3) as u8
}

pub(crate) fn ensure_settlement_smith(
    ctx: &ReducerContext,
    settlement_id: &str,
) -> SettlementSmith {
    if let Some(row) = ctx
        .db
        .settlement_smith()
        .settlement_id()
        .find(settlement_id.to_owned())
    {
        return row;
    }
    ctx.db.settlement_smith().insert(SettlementSmith {
        settlement_id: settlement_id.to_owned(),
        weaponsmith_skill: stable_skill(
            settlement_id,
            fabelgeist_determinism::StreamId::new("settlement.weaponsmith-skill"),
        ),
        armourer_skill: stable_skill(
            settlement_id,
            fabelgeist_determinism::StreamId::new("settlement.armourer-skill"),
        ),
        tailor_skill: stable_skill(
            settlement_id,
            fabelgeist_determinism::StreamId::new("settlement.tailor-skill"),
        ),
    })
}

pub(super) fn service_skill(
    ctx: &ReducerContext,
    settlement_id: &str,
    kind: PersistedItemKind,
) -> Result<u8, String> {
    use adventuresim_world_schema::SettlementService as S;
    let specialist = match kind {
        PersistedItemKind::Weapon | PersistedItemKind::Shield => S::Weaponsmith,
        PersistedItemKind::Armor => S::Armorer,
        PersistedItemKind::Clothing => S::Tailor,
        _ => return Err("This service does not repair that item kind".into()),
    };
    if crate::strategic::require_settlement_service(ctx, settlement_id, specialist).is_err() {
        crate::strategic::require_settlement_service(ctx, settlement_id, S::GeneralBlacksmith)?;
    }
    let service = ensure_settlement_smith(ctx, settlement_id);
    match kind {
        PersistedItemKind::Weapon | PersistedItemKind::Shield => Ok(service.weaponsmith_skill),
        PersistedItemKind::Armor => Ok(service.armourer_skill),
        PersistedItemKind::Clothing => Ok(service.tailor_skill),
        _ => Err("This service does not repair that item kind".into()),
    }
}
