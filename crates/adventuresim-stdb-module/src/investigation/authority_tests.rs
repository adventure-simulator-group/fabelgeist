//! Guarded storage vocabulary, generated binding, and prerequisite acceptance.
use super::*;

fn stored_methods(ctx: &ReducerContext) -> Result<Vec<InvestigationActionCapability>, String> {
    let owner = 732940;
    crate::character::create_named_character_with_id(
        ctx,
        owner,
        "Investigation key fixture".into(),
    )?;
    let terrains = [
        Terrain::Road,
        Terrain::Settlement,
        Terrain::Plains,
        Terrain::Forest,
        Terrain::Hills,
        Terrain::Marsh,
        Terrain::Ruins,
        Terrain::Underground,
    ];
    let mut rows = Vec::new();
    for (index, kind) in [
        InvestigationActionKind::InspectSite,
        InvestigationActionKind::SearchArea,
        InvestigationActionKind::FollowTracks,
        InvestigationActionKind::ReacquireTracks,
        InvestigationActionKind::LocateContact,
        InvestigationActionKind::Watch,
        InvestigationActionKind::Patrol,
        InvestigationActionKind::LayAmbush,
        InvestigationActionKind::ApproachLead,
    ]
    .into_iter()
    .enumerate()
    {
        let terrain = terrains[index % terrains.len()];
        let id = format!("authority-investigation:{index}");
        issue_investigation_action_capability(
            ctx,
            id.clone(),
            owner,
            "authority-investigation-case".into(),
            InvestigationProvenanceKind::Manual,
            String::new(),
            kind,
            action::InvestigationTargetKind::Route,
            "authority-route".into(),
            terrain,
            fabelgeist_determinism::Seed::from_u64(7),
            0,
            "Fixture route".into(),
            "Fixture prerequisite".into(),
            "Fixture result".into(),
            InvestigationActionConsequence::None,
            String::new(),
            format!("authority-investigation:{}", (index + 1) % 9),
        )?;
        let row = ctx
            .db
            .investigation_action_capability()
            .id()
            .find(id)
            .ok_or("Capability missing")?;
        if row
            .method
            .parse::<InvestigationActionKind>()
            .map_err(|error| error.to_string())?
            != kind
            || row
                .target_terrain
                .parse::<Terrain>()
                .map_err(|error| error.to_string())?
                != terrain
        {
            return Err("Stored capability vocabulary changed the issued method or terrain".into());
        }
        rows.push(row);
    }
    Ok(rows)
}

fn check_tracking(
    ctx: &ReducerContext,
    prototype: &InvestigationActionCapability,
) -> Result<(), String> {
    let mut root = prototype.clone();
    root.id = "authority-track-root".into();
    root.method = InvestigationActionKind::InspectSite.stable_id().into();
    root.target_kind = action::InvestigationTargetKind::Area;
    root.alternate_route_action_id = "authority-track-follow".into();
    let mut follow = prototype.clone();
    follow.id = "authority-track-follow".into();
    follow.method = InvestigationActionKind::FollowTracks.stable_id().into();
    follow.required_action_id = root.id.clone();
    follow.alternate_route_action_id = root.id.clone();
    ctx.db
        .investigation_action_capability()
        .insert(root.clone());
    ctx.db
        .investigation_action_capability()
        .insert(follow.clone());
    validate_action_route_graph_structure(&[root.clone(), follow.clone()])?;
    let kind = follow
        .method
        .parse::<InvestigationActionKind>()
        .map_err(|error| error.to_string())?;
    let coherent = |succeeded, missing| {
        tracking_capability_chain_is_coherent(
            &follow,
            kind,
            |id| {
                if missing {
                    None
                } else {
                    ctx.db
                        .investigation_action_capability()
                        .id()
                        .find(id.to_owned())
                }
            },
            |_| succeeded,
        )
    };
    if !coherent(true, false) || coherent(false, false) || coherent(true, true) {
        return Err("Parsed prerequisite admitted a missing or unsuccessful predecessor".into());
    }
    root.method = "unknown".into();
    ctx.db
        .investigation_action_capability()
        .id()
        .update(root.clone());
    if coherent(true, false) || validate_action_route_graph_structure(&[root, follow]).is_ok() {
        return Err("Malformed predecessor method passed route admission".into());
    }
    Ok(())
}

fn check_generated_binding(
    ctx: &ReducerContext,
    prototype: &InvestigationActionCapability,
) -> Result<(), String> {
    use adventuresim_core::{
        local_problem::Scope,
        quest_generation::{
            GeneratedActionOutput, GenerationContext, TemplateFamily, generate, observer_scoped_id,
            test_witnesses,
        },
    };
    let context = GenerationContext {
        seed: fabelgeist_determinism::Seed::from_u64(7),
        observer_entropy_hi: 11,
        observer_entropy_lo: 13,
        settlement_id: "lubeck".into(),
        settlement_name: "Lubeck".into(),
        scope: Scope::Settlement {
            settlement_id: "lubeck".into(),
        },
        ordinal: 0,
        now_minute: StrategicMinute::new(50_000),
        incident_weather: adventuresim_core::weather::Precipitation::Clear,
        requested_family: Some(TemplateFamily::RecurringDepredation),
        witness_candidates: test_witnesses(),
    };
    let manifest = generate(&context).map_err(|error| format!("{error:?}"))?;
    let generated = manifest
        .actions
        .iter()
        .find(|action| {
            action
                .outputs
                .iter()
                .any(|output| matches!(output, GeneratedActionOutput::PatternCondition { .. }))
        })
        .ok_or("Pattern action missing")?;
    let remap = |id: &adventuresim_core::quest_generation::ActionId| {
        observer_scoped_id(
            &context,
            "capability",
            &format!("{}:{}", prototype.owner_character_id, id.0),
        )
    };
    let mut capability = prototype.clone();
    capability.id = remap(&generated.id);
    capability.case_id = manifest.public_case_id.clone();
    capability.provenance_kind = InvestigationProvenanceKind::Generated;
    capability.generated_case_id = manifest.canonical_case_id.clone();
    capability.method = generated.kind.stable_id().into();
    capability.target_kind = generated.target_kind;
    capability.target_id = generated.target_id.clone();
    capability.target_terrain = generated_action_terrain(&manifest, generated)
        .stable_id()
        .into();
    capability.required_action_id = generated
        .prerequisite
        .as_ref()
        .map_or_else(String::new, &remap);
    capability.alternate_route_action_id = remap(&generated.alternate);
    capability.safe_summary = generated.safe_summary.clone();
    (
        capability.known_prerequisites,
        capability.safe_result_on_success,
    ) = generated_capability_safe_text(&manifest, generated);
    ctx.db
        .investigation_action_capability()
        .insert(capability.clone());
    let persisted = ctx
        .db
        .investigation_action_capability()
        .id()
        .find(&capability.id)
        .ok_or("Generated capability missing")?;
    let manifest_json = serde_json::to_string(&manifest).map_err(|error| error.to_string())?;
    let context_json = serde_json::to_string(&context).map_err(|error| error.to_string())?;
    let outputs_json =
        serde_json::to_string(&generated.outputs).map_err(|error| error.to_string())?;
    let authority = Some((manifest_json.as_str(), context_json.as_str()));
    if !matches!(
        generated_pattern_authority(&persisted, authority, Some(&outputs_json)),
        GeneratedPatternAuthority::Pattern { .. }
    ) {
        return Err("Stored generated vocabulary disagrees with its manifest".into());
    }
    for malformed_method in ["unknown", InvestigationActionKind::FollowTracks.stable_id()] {
        capability.method = malformed_method.into();
        if generated_pattern_authority(&capability, authority, Some(&outputs_json))
            != GeneratedPatternAuthority::Invalid
        {
            return Err("Changed generated method passed snapshot binding".into());
        }
    }
    capability.method = persisted.method;
    capability.target_terrain = "unknown".into();
    if generated_pattern_authority(&capability, authority, Some(&outputs_json))
        != GeneratedPatternAuthority::Invalid
    {
        return Err("Changed generated terrain passed snapshot binding".into());
    }
    Ok(())
}

#[reducer]
pub fn authority_test_investigation_keys(
    ctx: &ReducerContext,
    bootstrap_token: String,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    let rows = stored_methods(ctx)?;
    check_tracking(ctx, &rows[0])?;
    check_generated_binding(ctx, &rows[0])
}

#[reducer]
pub fn authority_test_invalid_investigation_key(
    ctx: &ReducerContext,
    bootstrap_token: String,
    method: String,
    terrain: String,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    let mut row = ctx
        .db
        .investigation_action_capability()
        .id()
        .find("authority-investigation:0".to_owned())
        .ok_or("Fixture missing")?;
    row.id = "authority-investigation:invalid".into();
    row.method = method;
    row.target_terrain = terrain;
    ctx.db.investigation_action_capability().insert(row.clone());
    row.method
        .parse::<InvestigationActionKind>()
        .map_err(|error| error.to_string())?;
    row.target_terrain
        .parse::<Terrain>()
        .map_err(|error| error.to_string())?;
    Err("Malformed fixture unexpectedly parsed".into())
}
