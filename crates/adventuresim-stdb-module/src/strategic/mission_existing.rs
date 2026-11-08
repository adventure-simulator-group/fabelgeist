fn validate_existing_mission(
    existing: MissionAuthority,
    party_id: &str,
    observer_character_id: u64,
    case_site: &CaseSiteAuthority,
    scene_key: &str,
) -> Result<MissionAuthority, String> {
    if existing.party_id == party_id
        && existing.observer_character_id == observer_character_id
        && existing.case_site_id.as_ref() == Some(&case_site.id)
        && existing.scene_key == scene_key
        && existing.status == MissionAttemptStatus::Bound
    {
        Ok(existing)
    } else if existing.status != MissionAttemptStatus::Bound {
        Err("Mission ID is already terminal and cannot be reused".into())
    } else {
        Err("Mission ID is already bound to different authority".into())
    }
}

fn require_visible_mission_case(
    ctx: &ReducerContext,
    observer_character_id: u64,
    case_site: &CaseSiteAuthority,
) -> Result<CaseAuthority, String> {
    if exact_case_site_for_observer(ctx, observer_character_id, case_site.id.as_str()).is_none() {
        return Err("Mission observer does not know or have a visited exact case site".into());
    }
    ctx.db
        .case_authority()
        .id()
        .find(&case_site.case_id)
        .ok_or_else(|| "Case authority no longer exists".to_string())
}
