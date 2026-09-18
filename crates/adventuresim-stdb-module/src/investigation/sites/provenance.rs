//! Validation of site provenance and observer-facing aliases.
use super::*;

pub(crate) fn case_site_provenance_view(
    ctx: &ViewContext,
    site: &CaseSiteAuthority,
) -> Option<Option<(String, String)>> {
    let case = ctx.db.case_authority().id().find(&site.case_id)?;
    let mut authorities = Vec::new();
    for alias in [&case.id, &case.generated_case_id] {
        if alias.is_empty()
            || authorities
                .iter()
                .any(|authority: &crate::strategic::QuestGenerationAuthority| {
                    authority.case_id == alias.as_str()
                })
        {
            continue;
        }
        if let Some(authority) = ctx.db.quest_generation_authority().case_id().find(alias) {
            authorities.push(authority);
        }
        authorities.extend(
            ctx.db
                .quest_generation_authority()
                .public_case_id()
                .filter(alias),
        );
    }
    validated_case_site_aliases(&case, authorities)
}

pub(crate) fn case_site_provenance_reducer(
    ctx: &ReducerContext,
    site: &CaseSiteAuthority,
) -> Option<Option<(String, String)>> {
    let case = ctx.db.case_authority().id().find(&site.case_id)?;
    let mut authorities = Vec::new();
    for alias in [&case.id, &case.generated_case_id] {
        if alias.is_empty()
            || authorities
                .iter()
                .any(|authority: &crate::strategic::QuestGenerationAuthority| {
                    authority.case_id == alias.as_str()
                })
        {
            continue;
        }
        if let Some(authority) = ctx.db.quest_generation_authority().case_id().find(alias) {
            authorities.push(authority);
        }
        authorities.extend(
            ctx.db
                .quest_generation_authority()
                .public_case_id()
                .filter(alias),
        );
    }
    validated_case_site_aliases(&case, authorities)
}

pub(super) fn validated_case_site_aliases(
    case: &crate::strategic::CaseAuthority,
    authorities: impl IntoIterator<Item = crate::strategic::QuestGenerationAuthority>,
) -> Option<Option<(String, String)>> {
    let mut authorities: Vec<_> = authorities
        .into_iter()
        .filter(|authority| {
            authority.case_id == case.id
                || authority.public_case_id == case.id
                || (!case.generated_case_id.is_empty()
                    && (authority.case_id == case.generated_case_id
                        || authority.public_case_id == case.generated_case_id))
        })
        .collect();
    authorities.sort_by(|left, right| left.case_id.cmp(&right.case_id));
    authorities.dedup_by(|left, right| left.case_id == right.case_id);
    match case.provenance_kind {
        InvestigationProvenanceKind::Manual
            if case.generated_case_id.is_empty() && authorities.is_empty() =>
        {
            Some(None)
        }
        InvestigationProvenanceKind::Generated
            if case.generated_case_id == case.id && authorities.len() == 1 =>
        {
            let validated = validate_quest_generation_authority(&authorities[0]).ok()?;
            (validated.manifest.canonical_case_id == case.id).then_some(Some((
                validated.manifest.canonical_case_id,
                validated.manifest.public_case_id,
            )))
        }
        _ => None,
    }
}
