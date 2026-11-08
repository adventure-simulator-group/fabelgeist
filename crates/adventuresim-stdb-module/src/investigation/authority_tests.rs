//! Guarded storage vocabulary, generated binding, and prerequisite acceptance.
use super::*;

fn stored_methods(
    ctx: &ReducerContext,
) -> crate::StrategicOperationResult<Vec<InvestigationActionCapability>> {
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
    check_developer_authority(ctx, prototype, &context, &manifest)?;
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
    let validated = crate::strategic::ValidatedQuestGenerationAuthority {
        manifest: manifest.clone(),
        context: context.clone(),
    };
    let outputs_json =
        serde_json::to_string(&generated.outputs).map_err(|error| error.to_string())?;
    let authority = Some(&validated);
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

fn check_developer_authority(
    ctx: &ReducerContext,
    prototype: &InvestigationActionCapability,
    context: &adventuresim_core::quest_generation::GenerationContext,
    generated: &adventuresim_core::quest_generation::GeneratedCase,
) -> Result<(), String> {
    use adventuresim_core::developer_quest::{
        DeveloperGenerationContext, DeveloperQuestDefinition, compile,
    };
    let developer = DeveloperGenerationContext {
        base: context.clone(),
        definition: DeveloperQuestDefinition::from_generated(generated.clone()),
        allow_implausible: true,
    };
    let manifest = compile(&developer).map_err(|error| format!("{error:?}"))?;
    let context_snapshot_json =
        serde_json::to_string(&developer).map_err(|error| error.to_string())?;
    let authority = crate::strategic::QuestGenerationAuthority {
        case_id: manifest.canonical_case_id.clone(),
        public_case_id: manifest.public_case_id.clone(),
        settlement_id: context.settlement_id.clone(),
        settlement_name: context.settlement_name.clone(),
        seed: context.seed,
        catalog_revision: manifest.catalog_revision.clone(),
        context_commitment: crate::strategic::quest_generation_context_commitment(
            &context_snapshot_json,
        )?,
        context_snapshot_json,
        manifest_json: serde_json::to_string(&manifest).map_err(|error| error.to_string())?,
        factor_trace_json: serde_json::to_string(&manifest.factor_trace)
            .map_err(|error| error.to_string())?,
    };
    ctx.db.quest_generation_authority().insert(authority);
    let mut capability = prototype.clone();
    capability.case_id = manifest.public_case_id.clone();
    capability.generated_case_id = manifest.canonical_case_id.clone();
    capability.provenance_kind = InvestigationProvenanceKind::Generated;
    let admitted = generated_authority_reducer(ctx, &capability)
        .map_err(|()| "Developer authority was ambiguous or invalid")?
        .ok_or("Developer authority was not found")?;
    if admitted.context != *context || admitted.manifest != manifest {
        return Err("Developer authority lookup changed validated context or manifest".into());
    }
    let adventuresim_core::quest_generation::CanonicalCause::Hostile(threat) = manifest.cause
    else {
        return Err("Developer fixture must exercise hostile investigability".into());
    };
    let expected = adventuresim_core::bestiary::profile(threat)
        .investigation
        .investigability;
    if generated_investigability(ctx, &capability) != Some(expected) {
        return Err("Developer authority lost its investigability".into());
    }
    record_developer_view_fixture(ctx, prototype, context, &manifest)?;
    Ok(())
}

fn record_developer_view_fixture(
    ctx: &ReducerContext,
    prototype: &InvestigationActionCapability,
    context: &adventuresim_core::quest_generation::GenerationContext,
    manifest: &adventuresim_core::quest_generation::GeneratedCase,
) -> Result<(), String> {
    use adventuresim_core::quest_generation::{GeneratedActionOutput, observer_scoped_id};
    let generated = manifest
        .actions
        .iter()
        .find(|candidate| {
            let kind = candidate
                .kind
                .stable_id()
                .parse::<InvestigationActionKind>();
            let Ok(kind) = kind else { return false };
            let prerequisites = action::prerequisites(kind);
            candidate.active_initially
                && candidate.prerequisite.is_none()
                && !prerequisites.requires_approximate_destination
                && !prerequisites.requires_tracks
                && !(kind == InvestigationActionKind::InspectSite
                    && candidate.target_kind == action::InvestigationTargetKind::Site)
                && !candidate.outputs.iter().any(|output| {
                    matches!(
                        output,
                        GeneratedActionOutput::PatternCondition { .. }
                            | GeneratedActionOutput::Consequence { .. }
                    )
                })
        })
        .ok_or("Developer fixture has no directly projectable action")?;
    let remap = |id: &adventuresim_core::quest_generation::ActionId| {
        observer_scoped_id(
            context,
            "capability",
            &format!("{}:{}", prototype.owner_character_id, id.0),
        )
    };
    let mut capability = prototype.clone();
    capability.id = remap(&generated.id);
    capability.case_id = manifest.public_case_id.clone();
    capability.generated_case_id = manifest.canonical_case_id.clone();
    capability.provenance_kind = InvestigationProvenanceKind::Generated;
    capability.active = generated.active_initially;
    capability.method = generated.kind.stable_id().into();
    capability.target_kind = generated.target_kind;
    capability.target_id = generated.target_id.clone();
    capability.target_terrain = generated_action_terrain(manifest, generated)
        .stable_id()
        .into();
    capability.required_action_id = String::new();
    capability.alternate_route_action_id = remap(&generated.alternate);
    capability.safe_summary = generated.safe_summary.clone();
    (
        capability.known_prerequisites,
        capability.safe_result_on_success,
    ) = generated_capability_safe_text(manifest, generated);
    capability.consequence_json = serde_json::to_string(&InvestigationActionConsequence::None)
        .map_err(|error| error.to_string())?;
    ctx.db
        .investigation_generated_action_output()
        .insert(InvestigationGeneratedActionOutput {
            capability_id: capability.id.clone(),
            outputs_json: serde_json::to_string(&generated.outputs)
                .map_err(|error| error.to_string())?,
        });
    if action::prerequisites(
        generated
            .kind
            .stable_id()
            .parse::<InvestigationActionKind>()
            .map_err(|error| error.to_string())?,
    )
    .requires_contact_referral
    {
        ctx.db.investigation_lead().insert(InvestigationLead {
            id: "authority-developer-view-referral".into(),
            owner_character_id: capability.owner_character_id,
            case_id: manifest.public_case_id.clone(),
            proposition_id: String::new(),
            summary: "Fixture referral".into(),
            source_label: "Fixture".into(),
            confidence_bps: 0,
            destination_stage: adventuresim_core::investigation::DestinationKnowledgeStage::Textual,
            directions: String::new(),
            exact_location_id: String::new(),
            latitude_e7: 0,
            longitude_e7: 0,
            witness_name: "Fixture witness".into(),
            witness_description: String::new(),
            witness_occupation_or_relationship: String::new(),
            expected_location: String::new(),
            current_learned_location: String::new(),
            contradiction_group: String::new(),
            corrected_by: String::new(),
            recorded_at: StrategicMinute::ZERO,
        });
    }
    ctx.db.investigation_action_capability().insert(capability);
    Ok(())
}

#[reducer]
pub fn authority_test_investigation_keys(
    ctx: &ReducerContext,
    bootstrap_token: String,
) -> crate::StrategicOperationResult<()> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    let rows = stored_methods(ctx)?;
    check_tracking(ctx, &rows[0])?;
    check_generated_binding(ctx, &rows[0]).map_err(Into::into)
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
