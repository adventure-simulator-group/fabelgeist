fn skill_bps(skill: Skill, hours: f32, attributes: &crate::CharacterAttributes) -> u16 {
    (skill.capped_training_rank(hours, attributes) * 2_000.0)
        .round()
        .clamp(
            0.0,
            f32::from(adventuresim_world_schema::BASIS_POINTS_PER_WHOLE),
        ) as u16
}

fn investigation_terrain_skill(terrain: action::Terrain) -> Skill {
    match terrain {
        action::Terrain::Forest => Skill::TerrainForest,
        action::Terrain::Hills | action::Terrain::Underground => Skill::TerrainHills,
        action::Terrain::Settlement | action::Terrain::Ruins => Skill::TerrainUrban,
        action::Terrain::Marsh => Skill::TerrainWetlands,
        action::Terrain::Plains | action::Terrain::Road => Skill::TerrainPlains,
    }
}

#[cfg(test)]
mod terrain_skill_tests {
    use super::*;

    #[test]
    fn marsh_investigation_uses_wetlands() {
        assert_eq!(
            investigation_terrain_skill(action::Terrain::Marsh),
            Skill::TerrainWetlands
        );
        assert_eq!(
            investigation_terrain_skill(action::Terrain::Road),
            Skill::TerrainPlains
        );
    }
}

fn party_action_skills(
    ctx: &ReducerContext,
    party_id: &str,
    actor_id: adventuresim_core::identity::CharacterId,
    terrain: action::Terrain,
) -> Result<action::SkillContribution, String> {
    let actor = ctx
        .db
        .character_skills()
        .character_id()
        .find(u64::from(actor_id))
        .ok_or("Character skills not found")?;
    let actor_attributes = ctx
        .db
        .character_attributes()
        .character_id()
        .find(u64::from(actor_id))
        .ok_or("Character attributes not found")?;
    let terrain_skill = investigation_terrain_skill(terrain);
    let terrain_bps = skill_bps(
        terrain_skill,
        actor.effective_skill_hours(terrain_skill),
        &actor_attributes,
    );
    let mut assistance = 0u16;
    for member_id in living_party_member_ids(ctx, party_id) {
        if member_id == actor_id {
            continue;
        }
        if let Some(skills) = ctx
            .db
            .character_skills()
            .character_id()
            .find(u64::from(member_id))
        {
            let Some(attributes) = ctx
                .db
                .character_attributes()
                .character_id()
                .find(u64::from(member_id))
            else {
                continue;
            };
            let contribution = skill_bps(
                terrain_skill,
                skills.effective_skill_hours(terrain_skill),
                &attributes,
            ) / 4;
            assistance = assistance.saturating_add(contribution).min(2_000);
        }
    }
    Ok(action::SkillContribution {
        terrain_bps,
        awareness_bps: skill_bps(Skill::Insight, actor.insight_hours, &actor_attributes),
        stealth_bps: skill_bps(Skill::Stealth, actor.stealth_hours, &actor_attributes),
        assistance_bps: assistance,
        // No authoritative locality-familiarity source exists yet.
        familiarity_bps: 0,
    })
}

fn actor_action_terrain(ctx: &ReducerContext, actor: &crate::Character) -> action::Terrain {
    if actor.current_settlement_id.is_some() {
        return action::Terrain::Settlement;
    }
    character_case_site_id(ctx, (actor.id).into())
        .and_then(|id| ctx.db.case_site_authority().id_key().find(&id))
        .and_then(|site| site.scene_key.parse::<Terrain>().ok())
        .unwrap_or(action::Terrain::Road)
}

fn actor_action_weather(
    ctx: &ReducerContext,
    actor: &crate::Character,
    started_at: StrategicMinute,
) -> action::WeatherAuthority {
    let coordinates = actor
        .current_settlement_id
        .as_ref()
        .and_then(|id| ctx.db.settlement().id().find(id))
        .and_then(|settlement| {
            adventuresim_world_schema::coordinates::Wgs84CoordinateMicrodegrees::from_longitude_latitude_degrees(
                settlement.coord_x,
                settlement.coord_y,
            )
            .map(|coordinate| (coordinate.latitude().get(), coordinate.longitude().get()))
        })
        .or_else(|| {
            character_case_site_id(ctx, (actor.id).into())
                .and_then(|id| ctx.db.case_site_authority().id_key().find(&id))
                .and_then(|site| {
                    if site.coordinates_are_geographic {
                        adventuresim_world_schema::coordinates::Wgs84CoordinateE7::new(
                            site.latitude_e7,
                            site.longitude_e7,
                        )
                        .map(adventuresim_world_schema::coordinates::Wgs84CoordinateMicrodegrees::from_e7)
                        .map(|coordinate| {
                            (coordinate.latitude().get(), coordinate.longitude().get())
                        })
                    } else {
                        use adventuresim_world_schema::coordinates::UnboundedCoordinateE7;
                        Some((
                            UnboundedCoordinateE7::from_raw(site.latitude_e7)
                                .millionths_of_coordinate_unit(),
                            UnboundedCoordinateE7::from_raw(site.longitude_e7)
                                .millionths_of_coordinate_unit(),
                        ))
                    }
                })
        })
        .unwrap_or((0, 0));
    let weather = adventuresim_core::weather::weather_at(
        adventuresim_core::weather::WORLD_WEATHER_SEED,
        started_at,
        coordinates.0,
        coordinates.1,
        0,
    );
    match weather.precipitation {
        adventuresim_core::weather::Precipitation::Clear => action::WeatherAuthority::Clear {
            snow_cover_bps: weather.snow_cover_bps,
        },
        adventuresim_core::weather::Precipitation::Rain => action::WeatherAuthority::Rain {
            intensity_bps: weather.intensity_bps,
            snow_cover_bps: weather.snow_cover_bps,
        },
        adventuresim_core::weather::Precipitation::Snow => action::WeatherAuthority::Snow {
            intensity_bps: weather.intensity_bps,
            snow_cover_bps: weather.snow_cover_bps,
        },
    }
}

fn persist_action_result_lead(
    ctx: &ReducerContext,
    capability: &InvestigationActionCapability,
    attempt_id: &str,
    resolution: &action::Resolution,
) -> Result<(), String> {
    let result_provenance::ActionResultProvenance {
        public_case_id,
        kind,
        generated_outputs,
    } = result_provenance::ActionResultProvenance::from_capability(ctx, capability)?;
    let typed_destination = generated_outputs.as_ref().and_then(|outputs| {
        outputs.iter().find_map(|output| match output {
            adventuresim_core::quest_generation::GeneratedActionOutput::Destination {
                stage,
                site_id,
            } => Some((*stage, site_id.as_ref())),
            _ => None,
        })
    });
    let exact_site_id = typed_destination.and_then(|(stage, site_id)| {
        (stage == DestinationKnowledgeStage::ExactBelieved)
            .then_some(site_id)
            .flatten()
    });
    let exact = resolution.success
        && if generated_outputs.is_some() {
            exact_site_id.is_some()
        } else {
            capability.target_kind == action::InvestigationTargetKind::Site
                && (kind == action::InvestigationActionKind::InspectSite
                    || resolution.resulting_uncertainty_bps <= 1_500)
        };
    let site = if exact {
        let site_id = exact_site_id.map_or_else(
            || capability.target_id.clone(),
            |site_id| site_id.as_str().to_owned(),
        );
        ctx.db.case_site_authority().id_key().find(&site_id)
    } else {
        None
    };
    let lead_id = generated_observer_id(ctx, &capability.case_id, "lead", attempt_id)
        .unwrap_or_else(|| inv::compound_id(&["lead", "action", attempt_id]));
    if ctx.db.investigation_lead().id().find(&lead_id).is_some() {
        return Ok(());
    }
    let typed_stage = typed_destination.map(|(stage, _)| stage);
    let exact_location_label = site
        .as_ref()
        .map(|site| site.name.clone())
        .unwrap_or_default();
    let (stage, exact_location_id, latitude_e7, longitude_e7) = if let Some(site) = site {
        (
            DestinationKnowledgeStage::ExactBelieved,
            site.id.into_string(),
            site.latitude_e7,
            site.longitude_e7,
        )
    } else if resolution.success {
        (
            typed_stage.unwrap_or(DestinationKnowledgeStage::ApproximateArea),
            String::new(),
            0,
            0,
        )
    } else {
        (DestinationKnowledgeStage::Unknown, String::new(), 0, 0)
    };
    ctx.db.investigation_lead().insert(InvestigationLead {
        id: lead_id,
        owner_character_id: capability.owner_character_id,
        case_id: public_case_id,
        proposition_id: String::new(),
        summary: if resolution.success {
            capability.safe_result_on_success.clone()
        } else {
            "The attempt found nothing conclusive; the lead remains open through another approach."
                .into()
        },
        source_label: "your party's investigation".into(),
        confidence_bps: if resolution.success { 8_000 } else { 3_000 },
        destination_stage: stage,
        directions: if exact {
            String::new()
        } else {
            capability.safe_summary.clone()
        },
        exact_location_id,
        latitude_e7,
        longitude_e7,
        witness_name: String::new(),
        witness_description: String::new(),
        witness_occupation_or_relationship: String::new(),
        expected_location: String::new(),
        current_learned_location: exact_location_label,
        contradiction_group: format!("action-location:{}", capability.case_id),
        corrected_by: String::new(),
        recorded_at: official_minute(ctx),
    });
    if resolution.success
        && let Some(outputs) = generated_outputs
    {
        for evidence_id in outputs.iter().filter_map(|output| match output {
            adventuresim_core::quest_generation::GeneratedActionOutput::Evidence {
                evidence_id,
            } => Some(evidence_id.as_str()),
            _ => None,
        }) {
            let source_id = crate::outbreak::source_material_knowledge_provenance(
                ctx,
                &capability.generated_case_id,
                &capability.target_id,
            )?
            .unwrap_or_else(|| attempt_id.to_owned());
            record_evidence_knowledge(
                ctx,
                capability.owner_character_id,
                &capability.case_id,
                evidence_id,
                &source_id,
            )?;
        }
    }
    Ok(())
}

fn validate_generated_pattern_condition(
    ctx: &ReducerContext,
    capability: &InvestigationActionCapability,
    kind: action::InvestigationActionKind,
    started_at: StrategicMinute,
) -> Result<(), String> {
    let output = ctx
        .db
        .investigation_generated_action_output()
        .capability_id()
        .find(&capability.id);
    let authority = generated_authority_reducer(ctx, capability)
        .map_err(|()| "Generated action authority is ambiguous")?;
    let (evidence_id, condition) = match generated_pattern_authority(
        capability,
        authority
            .as_ref()
            .map(|(manifest, context)| (manifest.as_str(), context.as_str())),
        output.as_ref().map(|output| output.outputs_json.as_str()),
    ) {
        GeneratedPatternAuthority::Manual | GeneratedPatternAuthority::GeneratedWithoutPattern => {
            return Ok(());
        }
        GeneratedPatternAuthority::Pattern {
            evidence_id,
            condition,
        } => (evidence_id, condition),
        GeneratedPatternAuthority::Invalid => {
            return Err("Generated action output authority is invalid".into());
        }
    };
    let observer_case_id = reducer_action_public_case_id(ctx, capability)
        .ok_or("Generated action has no observer-safe case binding")?;
    if !observer_pattern_route_has_live_corroborated_clue(
        capability.owner_character_id,
        &observer_case_id,
        &evidence_id,
        started_at,
        ctx.db
            .investigation_evidence_knowledge()
            .owner_character_id()
            .filter(capability.owner_character_id),
    ) {
        return Err("The selected pattern has not been corroborated yet".into());
    }
    use adventuresim_core::quest_generation::GeneratedPatternCondition as C;
    match &condition {
        C::NightWindow
            if started_at.minute_of_day() >= 360 && started_at.minute_of_day() < 1_200 =>
        {
            Err(adventuresim_core::reducer_error::coded_reducer_error(
                adventuresim_core::reducer_error::ReducerErrorCode::InvestigationNightWindow,
                "The learned pattern requires acting during the nighttime window",
            ))
        }
        C::RoadRoute if capability.target_kind != action::InvestigationTargetKind::Route => {
            Err("The learned roadside pattern is not bound to route geography".into())
        }
        C::VictimProfile {
            cohort_id,
            demographic,
            age_band,
            sex,
            profession,
        } => {
            if kind != action::InvestigationActionKind::Patrol
                || capability.target_kind != action::InvestigationTargetKind::Cohort
                || capability.target_id != *cohort_id
            {
                return Err("The learned victim profile targets another cohort".into());
            }
            let target = ctx
                .db
                .investigation_pattern_target_authority()
                .cohort_id()
                .find(cohort_id)
                .ok_or_else(|| {
                    adventuresim_core::reducer_error::coded_reducer_error(
                        adventuresim_core::reducer_error::ReducerErrorCode::VictimCohortStateChanged,
                        "Victim cohort authority no longer exists",
                    )
                })?;
            let expected_demographic = demographic.as_str();
            if target.case_id != capability.case_id
                || target.demographic != expected_demographic
                || target.age_band != *age_band
                || target.sex != *sex
                || target.profession != *profession
            {
                return Err(adventuresim_core::reducer_error::coded_reducer_error(
                    adventuresim_core::reducer_error::ReducerErrorCode::VictimCohortStateChanged,
                    "Victim cohort profile no longer matches its authority",
                ));
            }
            let npc = crate::settlement_population::resolve_settlement_resident(
                ctx,
                target.resident_character_id,
            )
            .ok_or_else(|| {
                adventuresim_core::reducer_error::coded_reducer_error(
                    adventuresim_core::reducer_error::ReducerErrorCode::VictimCohortStateChanged,
                    "Victim cohort NPC no longer exists",
                )
            })?;
            let presence = ctx
                .db
                .settlement_resident_presence()
                .character_id()
                .find(target.resident_character_id)
                .ok_or_else(|| {
                    adventuresim_core::reducer_error::coded_reducer_error(
                        adventuresim_core::reducer_error::ReducerErrorCode::VictimCohortStateChanged,
                        "Victim cohort target is unavailable",
                    )
                })?;
            let expected = adventuresim_core::quest_generation::GeneratedPatternTarget {
                cohort_id: target.cohort_id.clone(),
                resident_character_id: target.resident_character_id,
                demographic: *demographic,
                age_band: target.age_band.clone(),
                sex: target.sex,
                profession: target.profession.clone(),
                expected_settlement_id: adventuresim_core::identity::SettlementId::try_new(
                    target.expected_settlement_id.clone(),
                )
                .map_err(|error| error.to_string())?,
                expected_location: target.expected_location.clone(),
                expected_location_label: String::new(),
                presence_version: target.presence_version,
            };
            let current = if target.sex.is_none() {
                crate::strategic::developer_npc_witness_candidate(&npc, &presence)
                    .ok_or_else(|| {
                        adventuresim_core::reducer_error::coded_reducer_error(
                            adventuresim_core::reducer_error::ReducerErrorCode::VictimCohortStateChanged,
                            "Victim cohort NPC no longer has a visible demographic",
                        )
                    })?
            } else {
                adventuresim_core::quest_generation::WitnessCandidate {
                    resident_character_id: npc.character_id,
                    display_name: npc.name.clone(),
                    demographic: crate::strategic::generated_npc_demographic(&npc),
                    age_band: npc.age_band.stable_id().to_owned(),
                    sex: Some(npc.sex),
                    profession: npc.profession.clone(),
                    visible_description: String::new(),
                    expected_location: presence.location_id.clone(),
                    expected_location_label: presence.location_id.clone(),
                    presence_version: crate::strategic::generated_npc_presence_version(
                        &npc, &presence,
                    ),
                    allowed_circumstances: Default::default(),
                }
            };
            if !adventuresim_core::quest_generation::pattern_target_matches(
                &expected,
                &current,
                &presence.settlement_id,
            ) || !crate::settlement_population::npc_is_present(ctx, &presence, started_at)
            {
                return Err(adventuresim_core::reducer_error::coded_reducer_error(
                    adventuresim_core::reducer_error::ReducerErrorCode::VictimCohortStateChanged,
                    "Victim cohort target moved, changed, or is unavailable",
                ));
            }
            Ok(())
        }
        C::BroadSurvey
            if kind != action::InvestigationActionKind::SearchArea
                || capability.target_kind != action::InvestigationTargetKind::Area =>
        {
            Err("An irregular pattern requires a broad area search".into())
        }
        _ => Ok(()),
    }
}

fn case_objective_contains_custody_target(
    ctx: &ReducerContext,
    case_id: &str,
    object_kind: CustodyObjectKind,
    object_id: &str,
) -> Result<bool, String> {
    let case = ctx
        .db
        .case_authority()
        .id()
        .find(case_id.to_string())
        .ok_or("Investigation case no longer exists")?;
    let expression: adventuresim_core::case::ObjectiveExpression =
        serde_json::from_str(&case.objective_expression_json)
            .map_err(|_| "Case objective authority is invalid")?;
    use adventuresim_core::case::ObjectiveRequirement as R;
    Ok(expression
        .alternatives
        .iter()
        .flat_map(|path| &path.objectives)
        .any(|objective| match (&objective.requirement, object_kind) {
            (R::Retrieve { asset_id }, CustodyObjectKind::Asset) => asset_id.as_str() == object_id,
            (R::Rescue { subject_id }, CustodyObjectKind::Subject) => {
                subject_id.as_str() == object_id
            }
            _ => false,
        }))
}

fn validate_pickup_custody(
    ctx: &ReducerContext,
    capability: &InvestigationActionCapability,
    party_id: &str,
    object_kind: CustodyObjectKind,
    object_id: &str,
    expected_next_version: u32,
) -> Result<u32, String> {
    if !case_objective_contains_custody_target(ctx, &capability.case_id, object_kind, object_id)? {
        return Err("Capability target is not an unresolved objective of this case".into());
    }
    let current = ctx
        .db
        .case_custody()
        .object_id()
        .find(object_id.to_string())
        .ok_or("Capability target has no custody authority")?;
    if current.case_id != capability.case_id
        || current.object_kind != object_kind
        || current.holder_kind != CustodyHolderKind::Site
        || capability.target_kind != action::InvestigationTargetKind::Site
        || current.holder_id != capability.target_id
    {
        return Err("Capability target is not legally present at this investigation site".into());
    }
    let party = ctx
        .db
        .party_authority()
        .id()
        .find(party_id.to_string())
        .ok_or("Party not found")?;
    if party.current_case_site_id.as_deref() != Some(current.holder_id.as_str()) {
        return Err("Party is not at the custody site".into());
    }
    let next = current.version.saturating_add(1);
    if expected_next_version != next {
        return Err("Capability custody version is stale and must be reissued".into());
    }
    Ok(next)
}

fn reissue_stale_custody_capability(
    ctx: &ReducerContext,
    capability: &mut InvestigationActionCapability,
    party_id: &str,
) -> Result<bool, String> {
    let consequence: InvestigationActionConsequence =
        serde_json::from_str(&capability.consequence_json)
            .map_err(|_| "Investigation action consequence authority is invalid")?;
    let (object_kind, object_id, expected) = match &consequence {
        InvestigationActionConsequence::RetrieveAsset { asset_id, version } => {
            (CustodyObjectKind::Asset, asset_id.as_str(), *version)
        }
        InvestigationActionConsequence::RescueSubject {
            subject_id,
            version,
        } => (CustodyObjectKind::Subject, subject_id.as_str(), *version),
        _ => return Ok(false),
    };
    let current = ctx
        .db
        .case_custody()
        .object_id()
        .find(object_id.to_string())
        .ok_or("Capability target has no custody authority")?;
    let next = current.version.saturating_add(1);
    if expected == next {
        return Ok(false);
    }
    // A changed version is recoverable only while every semantic binding is
    // still identical. Holder/site/case changes are authority failures.
    validate_pickup_custody(ctx, capability, party_id, object_kind, object_id, next)?;
    let refreshed = match consequence {
        InvestigationActionConsequence::RetrieveAsset { asset_id, .. } => {
            InvestigationActionConsequence::RetrieveAsset {
                asset_id,
                version: next,
            }
        }
        InvestigationActionConsequence::RescueSubject { subject_id, .. } => {
            InvestigationActionConsequence::RescueSubject {
                subject_id,
                version: next,
            }
        }
        _ => unreachable!(),
    };
    capability.consequence_json = serde_json::to_string(&refreshed)
        .map_err(|_| "Refreshed investigation consequence is invalid")?;
    capability.version = capability.version.saturating_add(1);
    capability.seed = ctx.random::<u64>();
    ctx.db
        .investigation_action_capability()
        .id()
        .update(capability.clone());
    ctx.db
        .investigation_action_outcome()
        .insert(InvestigationActionOutcome {
        id: generated_observer_id(
            ctx,
            &capability.case_id,
            "outcome",
            &format!("reissue:{}:{}", capability.id, capability.version),
        )
        .unwrap_or_else(|| {
            inv::compound_id(&[
                "outcome",
                "reissue",
                &capability.id,
                &capability.version.to_string(),
            ])
        }),
        owner_character_id: capability.owner_character_id,
        case_id: capability.case_id.clone(),
        capability_id: capability.id.clone(),
        attempt_id: String::new(),
        safe_wording:
            "The situation changed before you acted; the lead was refreshed without spending time."
                .into(),
        recorded_at: character_strategic_minute(ctx, capability.owner_character_id),
        official_recorded_at: official_minute(ctx),
    });
    Ok(true)
}

fn commit_action_consequence(
    ctx: &ReducerContext,
    capability: &InvestigationActionCapability,
    party_id: &str,
    attempt_id: &str,
) -> Result<(), String> {
    let consequence: InvestigationActionConsequence =
        serde_json::from_str(&capability.consequence_json)
            .map_err(|_| "Investigation action consequence authority is invalid")?;
    match consequence {
        InvestigationActionConsequence::None => Ok(()),
        InvestigationActionConsequence::RetrieveAsset { asset_id, version } => {
            let version = validate_pickup_custody(
                ctx,
                capability,
                party_id,
                CustodyObjectKind::Asset,
                &asset_id,
                version,
            )?;
            crate::strategic::record_asset_retrieved(
                ctx,
                attempt_id,
                &capability.case_id,
                party_id,
                &asset_id,
                version,
            )
            .map(|_| ())
        }
        InvestigationActionConsequence::RescueSubject {
            subject_id,
            version,
        } => {
            let version = validate_pickup_custody(
                ctx,
                capability,
                party_id,
                CustodyObjectKind::Subject,
                &subject_id,
                version,
            )?;
            crate::strategic::record_subject_rescued_or_released(
                ctx,
                attempt_id,
                &capability.case_id,
                party_id,
                &subject_id,
                version,
                false,
            )
            .map(|_| ())
        }
    }
}

fn commit_generated_remediation(
    ctx: &ReducerContext,
    capability: &InvestigationActionCapability,
    party_id: &str,
    attempt_id: &str,
) -> Result<(), String> {
    let Some(output) = ctx
        .db
        .investigation_generated_action_output()
        .capability_id()
        .find(&capability.id)
    else {
        return Ok(());
    };
    let outputs = serde_json::from_str::<
        Vec<adventuresim_core::quest_generation::GeneratedActionOutput>,
    >(&output.outputs_json)
    .map_err(|_| "Generated action output authority is invalid")?;
    let remediation_ids = outputs
        .iter()
        .filter_map(|output| match output {
            adventuresim_core::quest_generation::GeneratedActionOutput::Remediation {
                remediation_id,
            } => Some(remediation_id.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    if remediation_ids.is_empty() {
        return Ok(());
    }
    if remediation_ids.len() != 1
        || capability.provenance_kind != InvestigationProvenanceKind::Generated
        || capability.target_kind != action::InvestigationTargetKind::Site
        || capability.generated_case_id.is_empty()
    {
        return Err("Generated remediation capability is incoherent".into());
    }
    let at_minute = crate::time::refresh_clock(ctx)?;
    crate::outbreak::commit_source_remediation(
        ctx,
        &capability.generated_case_id,
        party_id,
        attempt_id,
        remediation_ids[0],
        &capability.target_id,
        at_minute,
    )?;
    crate::strategic::ingest_case_outcome_fact(
        ctx,
        &format!("outcome:{attempt_id}:remediation"),
        &capability.generated_case_id,
        party_id,
        adventuresim_core::case::OutcomeFactKind::SourceRemediated {
            remediation_id: remediation_ids[0].into(),
        },
    )
}

fn generated_progress_kind(kind: action::InvestigationActionKind) -> bool {
    use action::InvestigationActionKind as K;
    match kind {
        K::InspectSite
        | K::SearchArea
        | K::FollowTracks
        | K::ReacquireTracks
        | K::LocateContact
        | K::Watch
        | K::Patrol
        | K::LayAmbush
        | K::ApproachLead => true,
    }
}

fn capability_uses_bounded_progress(
    provenance_kind: InvestigationProvenanceKind,
    kind: action::InvestigationActionKind,
) -> bool {
    provenance_kind == InvestigationProvenanceKind::Generated && generated_progress_kind(kind)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AttemptHistoryKind {
    CompletedFailure,
    Break,
}

fn failed_attempt_history_kind(private_resolution_json: &str) -> AttemptHistoryKind {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct InterruptedReceipt {
        status: String,
        requested_minutes: u64,
        completion_effects_applied: bool,
    }

    let Ok(value) = serde_json::from_str::<serde_json::Value>(private_resolution_json) else {
        return AttemptHistoryKind::Break;
    };
    if value.get("status").is_some() {
        let Ok(receipt) = serde_json::from_value::<InterruptedReceipt>(value) else {
            return AttemptHistoryKind::Break;
        };
        let _is_terminal_interruption = receipt.status == "interrupted"
            && !receipt.completion_effects_applied
            && receipt.requested_minutes > 0;
        return AttemptHistoryKind::Break;
    }
    if let Ok(resolution) = serde_json::from_value::<action::Resolution>(value.clone())
        && !resolution.success
        && value.as_object().is_some_and(|object| {
            object.len() == 7
                && [
                    "result",
                    "success",
                    "cost",
                    "resulting_uncertainty_bps",
                    "risk_bps",
                    "risk_triggered",
                    "effective_skill_bps",
                ]
                .into_iter()
                .all(|key| object.contains_key(key))
        })
    {
        return AttemptHistoryKind::CompletedFailure;
    }
    if let Some(resolution) = value
        .get("resolution")
        .cloned()
        .and_then(|value| serde_json::from_value::<action::Resolution>(value).ok())
        && !resolution.success
        && value.as_object().is_some_and(|object| {
            object.len() == 5
                && [
                    "resolution",
                    "attempt_number",
                    "persistent_progress_bps",
                    "success_threshold_bps",
                    "guaranteed_by_attempt",
                ]
                .into_iter()
                .all(|key| object.contains_key(key))
        })
    {
        return AttemptHistoryKind::CompletedFailure;
    }
    AttemptHistoryKind::Break
}

fn contiguous_failed_attempts(
    capability_id: &str,
    owner_character_id: u64,
    method: &str,
    current_version: u32,
    attempts: impl IntoIterator<Item = InvestigationActionAttempt>,
) -> u32 {
    let attempts = attempts
        .into_iter()
        .filter(|attempt| {
            attempt.capability_id == capability_id
                && attempt.owner_character_id == owner_character_id
                && attempt.method == method
                && attempt.expected_version < current_version
        })
        .map(|attempt| {
            let kind = if attempt.success {
                AttemptHistoryKind::Break
            } else {
                failed_attempt_history_kind(&attempt.private_resolution_json)
            };
            (attempt.expected_version, kind)
        })
        .collect::<BTreeMap<_, _>>();
    let mut cursor = current_version;
    let mut failures = 0;
    while cursor > 0 {
        cursor -= 1;
        match attempts.get(&cursor) {
            Some(AttemptHistoryKind::CompletedFailure) => failures += 1,
            Some(AttemptHistoryKind::Break) | None => break,
        }
    }
    failures
}

fn bounded_failure_wording(
    progress: action::BoundedProgressResolution,
    alternate_available: bool,
) -> String {
    let threshold_whole = progress.success_threshold_bps / 100;
    let threshold_fraction = progress.success_threshold_bps % 100;
    let progress_whole = progress.persistent_progress_bps / 100;
    let progress_fraction = progress.persistent_progress_bps % 100;
    let alternate = if alternate_available {
        " Another currently supported route is also available."
    } else {
        " No alternate route is currently supported by the leads in your journal."
    };
    format!(
        "No conclusive result. Persistent fieldwork advanced this exact route to attempt {} of {}; accumulated progress added {progress_whole}.{progress_fraction:02}% to this attempt's bounded success threshold of {threshold_whole}.{threshold_fraction:02}%, uncertainty fell to {}.{:02}%, and contiguous work guarantees success by attempt {}.{alternate}",
        progress.attempt_number,
        progress.guaranteed_by_attempt,
        progress.resolution.resulting_uncertainty_bps / 100,
        progress.resolution.resulting_uncertainty_bps % 100,
        progress.guaranteed_by_attempt,
    )
}

fn private_action_resolution_json(
    resolution: action::Resolution,
    bounded_progress: Option<action::BoundedProgressResolution>,
) -> Result<String, String> {
    if let Some(progress) = bounded_progress {
        serde_json::to_string(&serde_json::json!({
            "resolution": progress.resolution,
            "attempt_number": progress.attempt_number,
            "persistent_progress_bps": progress.persistent_progress_bps,
            "success_threshold_bps": progress.success_threshold_bps,
            "guaranteed_by_attempt": progress.guaranteed_by_attempt,
        }))
    } else {
        // Preserve the historical bare Resolution audit shape for manual and
        // otherwise unbounded actions.
        serde_json::to_string(&resolution)
    }
    .map_err(|_| "Investigation resolution could not be recorded".into())
}

fn capability_progress_depends_on_exact_lead(
    capability: &InvestigationActionCapability,
    lead: &InvestigationLead,
    generated_case_aliases: Option<(&str, &str)>,
) -> bool {
    let case_matches = match (capability.provenance_kind, generated_case_aliases) {
        (InvestigationProvenanceKind::Manual, None) => capability.case_id == lead.case_id,
        (InvestigationProvenanceKind::Generated, Some((canonical, public))) => {
            capability.generated_case_id == canonical
                && (capability.case_id == canonical || capability.case_id == public)
                && (lead.case_id == canonical || lead.case_id == public)
        }
        _ => false,
    };
    capability.provenance_kind == InvestigationProvenanceKind::Generated
        && capability.active
        && capability.owner_character_id == lead.owner_character_id
        && case_matches
        && capability.target_kind == action::InvestigationTargetKind::Site
        && capability.target_id == lead.exact_location_id
        && lead.destination_stage.is_exact()
        && capability
            .method
            .parse::<InvestigationActionKind>()
            .is_ok_and(generated_progress_kind)
}

fn dependent_capability_ids_for_exact_lead(
    ctx: &ReducerContext,
    lead: &InvestigationLead,
) -> BTreeSet<String> {
    if lead.exact_location_id.is_empty() {
        return BTreeSet::new();
    }
    ctx.db
        .investigation_action_capability()
        .owner_character_id()
        .filter(lead.owner_character_id)
        .filter(|capability| {
            let aliases = generated_authority_reducer(ctx, capability)
                .ok()
                .flatten()
                .and_then(|(manifest, _)| {
                    serde_json::from_str::<adventuresim_core::quest_generation::GeneratedCase>(
                        &manifest,
                    )
                    .ok()
                    .map(|generated| (generated.canonical_case_id, generated.public_case_id))
                });
            capability_progress_depends_on_exact_lead(
                capability,
                lead,
                aliases
                    .as_ref()
                    .map(|(canonical, public)| (canonical.as_str(), public.as_str())),
            )
        })
        .map(|capability| capability.id)
        .collect()
}

fn unique_capability_ids(capability_ids: impl IntoIterator<Item = String>) -> BTreeSet<String> {
    capability_ids.into_iter().collect()
}

fn correction_requires_progress_reset(has_live_replacement_support: bool) -> bool {
    !has_live_replacement_support
}

fn reset_capability_progress_if_unsupported(
    capability: &mut InvestigationActionCapability,
    has_live_replacement_support: bool,
    replacement_seed: impl FnOnce() -> u64,
) -> bool {
    if !correction_requires_progress_reset(has_live_replacement_support) {
        return false;
    }
    capability.version = capability.version.saturating_add(1);
    capability.seed = replacement_seed();
    true
}

fn reset_unsupported_capability_progress(
    ctx: &ReducerContext,
    capability_ids: impl IntoIterator<Item = String>,
) -> Result<(), String> {
    for capability_id in unique_capability_ids(capability_ids) {
        let Some(mut capability) = ctx
            .db
            .investigation_action_capability()
            .id()
            .find(&capability_id)
        else {
            continue;
        };
        let has_live_replacement_support =
            exact_action_case_site_for_observer(ctx, &capability).is_some();
        if !reset_capability_progress_if_unsupported(
            &mut capability,
            has_live_replacement_support,
            || ctx.random::<u64>(),
        ) {
            continue;
        }
        ctx.db
            .investigation_action_capability()
            .id()
            .update(capability);
    }
    Ok(())
}

#[expect(
    clippy::too_many_arguments,
    reason = "planning keeps the independently validated authority inputs visible"
)]
fn site_bound_investigation_plan(
    ctx: &ReducerContext,
    actor_id: adventuresim_core::identity::CharacterId,
    party_leader_id: u64,
    rights: &adventuresim_core::rights::PrivateRightsDecision<action::InvestigationRightsEvidence>,
    rights_question: &action::InvestigationRightsQuestion,
    capability: &InvestigationActionCapability,
    attempt_id: &str,
    started_at: StrategicMinute,
    members: &[adventuresim_core::identity::CharacterId],
    resolution: action::Resolution,
    resolution_input: action::ResolutionInput,
) -> Result<Option<action::InvestigationPlanningOutcome>, String> {
    use adventuresim_core::{
        physical_object::CustodyCharacterId,
        strategic_action::{
            ActionCoordinates, ActionDefinitionId, ActionRequestId, ActionTarget,
            AuthoritativeSnapshot, AuthorityBinding, PlanProvenance, RequestedDuration,
            ScheduledInterruption, SnapshotDigest, SnapshotRevision, TimeBoundaries,
        },
    };
    use sha2::Digest as _;

    // This representative vertical deliberately covers exact site actions.
    // Area and route actions retain their existing domain path until they have
    // an equally canonical strategic-place representation.
    if capability.target_kind != action::InvestigationTargetKind::Site
        || resolution_input.kind != action::InvestigationActionKind::InspectSite
    {
        return Ok(None);
    }
    let Some(ExactActionCaseSite { site, .. }) =
        exact_action_case_site_for_observer(ctx, capability)
    else {
        return Ok(None);
    };
    let place = site.id.to_place();
    let actor =
        CustodyCharacterId::try_from(adventuresim_core::identity::CharacterId::from(actor_id))
            .map_err(|_| "Investigation actor identity is malformed")?;
    let coordinates = ActionCoordinates::try_new(
        actor,
        ActionTarget::Place(place.clone()),
        place.clone(),
        None,
        Vec::new(),
    )
    .map_err(|_| "Investigation action coordinates are inconsistent")?;
    let requested = u64::from(resolution.cost.minutes);
    let safe = members.iter().try_fold(requested, |safe, member_id| {
        crate::time::preview_travel_time(ctx, (*member_id).into(), requested)
            .map(|value| safe.min(value))
    })?;

    let mut hasher = sha2::Sha256::new();
    let mut frame = |bytes: &[u8]| {
        hasher.update((bytes.len() as u64).to_le_bytes());
        hasher.update(bytes);
    };
    frame(b"investigation-plan-snapshot-v2");
    frame(place.to_string().as_bytes());
    frame(capability.id.as_bytes());
    frame(capability.provenance_kind.as_str().as_bytes());
    frame(capability.generated_case_id.as_bytes());
    frame(capability.case_id.as_bytes());
    frame(capability.method.as_bytes());
    frame(capability.target_kind.as_str().as_bytes());
    frame(capability.target_id.as_bytes());
    frame(capability.target_terrain.as_bytes());
    frame(&capability.version.to_le_bytes());
    frame(&capability.seed.to_le_bytes());
    frame(&capability.evidence_age_origin_minute.get().to_le_bytes());
    frame(&capability.uncertainty_bps.to_le_bytes());
    frame(capability.required_action_id.as_bytes());
    frame(capability.alternate_route_action_id.as_bytes());
    frame(&[u8::from(capability.active)]);
    frame(&u64::from(actor_id).to_le_bytes());
    frame(&party_leader_id.to_le_bytes());
    frame(&started_at.get().to_le_bytes());
    frame(&requested.to_le_bytes());
    frame(&safe.to_le_bytes());
    frame(
        &serde_json::to_vec(&resolution_input)
            .map_err(|_| "Investigation planning input could not be bound")?,
    );
    frame(&rights.provenance().evidence_revision.0.to_le_bytes());
    frame(&rights.provenance().question_digest);
    frame(
        &serde_json::to_vec(&resolution)
            .map_err(|_| "Investigation resolution could not be bound")?,
    );
    frame(&[match rights.kind() {
        adventuresim_core::rights::RightsDecisionKind::Allowed => 1,
        adventuresim_core::rights::RightsDecisionKind::Denied => 0,
    }]);
    frame(
        &rights
            .evidence()
            .iter()
            .map(|evidence| match evidence {
                action::InvestigationRightsEvidence::PartyLeader => 1,
                action::InvestigationRightsEvidence::LeaderApproval => 2,
            })
            .collect::<Vec<_>>(),
    );
    for member_id in members {
        frame(&u64::from(*member_id).to_le_bytes());
    }
    let input_digest: [u8; 32] = hasher.finalize().into();
    let mut binding_hasher = sha2::Sha256::new();
    binding_hasher.update(b"investigation-plan-binding-v1\0");
    binding_hasher.update(input_digest);
    binding_hasher.update(attempt_id.as_bytes());
    let authority_binding: [u8; 32] = binding_hasher.finalize().into();
    if !matches!(
        rights_question.jurisdiction(),
        adventuresim_core::rights::RightsJurisdiction::Place(bound) if bound == &place
    ) || rights.provenance().question_digest
        != action::investigation_rights_question_digest(rights_question)
    {
        return Err("Investigation rights question is inconsistent".into());
    }
    let interruption = (safe < requested).then_some(ScheduledInterruption {
        at_minute: started_at.saturating_add_minutes(safe),
        cause: action::InvestigationPlanInterruption::ParticipantBoundary,
    });
    let member_ids = members
        .iter()
        .map(|id| {
            CustodyCharacterId::try_from(*id)
                .map_err(|_| "Investigation party identity is malformed".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Some(action::build_investigation_plan(
        action::InvestigationPlanAuthority {
            coordinates,
            provenance: PlanProvenance {
                request_id: ActionRequestId::try_new(attempt_id)
                    .map_err(|_| "Investigation request identity is malformed")?,
                action_id: ActionDefinitionId::try_new(format!(
                    "investigation:{}",
                    capability.method
                ))
                .map_err(|_| "Investigation definition identity is malformed")?,
                input_digest: SnapshotDigest(input_digest),
                authority_binding: AuthorityBinding(authority_binding),
            },
            snapshot: AuthoritativeSnapshot {
                revision: SnapshotRevision(u64::from(capability.version)),
                digest: SnapshotDigest(input_digest),
            },
            current_minute: started_at,
            duration: RequestedDuration::try_new(requested)
                .map_err(|_| "Investigation duration must be positive")?,
            boundaries: TimeBoundaries {
                terminal_minute: None,
                interruption,
            },
            rights: rights.clone(),
            capability_current: capability.active,
            live_prerequisites: true,
            generated_condition: true,
            member_ids,
            resolution,
        },
    )))
}

fn private_interrupted_action_resolution_json(requested_minutes: u64) -> Result<String, String> {
    serde_json::to_string(&serde_json::json!({
        "status": "interrupted",
        "requested_minutes": requested_minutes,
        "completion_effects_applied": false,
    }))
    .map_err(|_| "Interrupted investigation receipt could not be recorded".into())
}

#[reducer]
pub fn perform_investigation_action(
    ctx: &ReducerContext,
    actor_id: u64,
    action_id: String,
    method: String,
    expected_version: u32,
) -> Result<(), String> {
    perform_investigation_action_authorized(
        ctx,
        actor_id,
        action_id,
        method,
        expected_version,
        false,
    )
    .map_err(|error: InvestigationExecutionError| error.to_string())
}
