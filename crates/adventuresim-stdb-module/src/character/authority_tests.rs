//! Fault injection available only to the isolated authority acceptance runner.
//! Every entry point requires the compiled development bootstrap capability.

use super::*;
use crate::tactical::tactical_server_request_authority;

#[derive(Clone, Copy, SpacetimeType)]
pub enum NameAuthorityFault {
    MalformedJson,
    MissingIdentity,
    InvalidRendering,
    InvalidFamily,
    MismatchedFamily,
    InvalidSurname,
}

impl NameAuthorityFault {
    fn invalid_semantic_identity(self) -> Result<PersonalNameIdentity, String> {
        use adventuresim_world_schema::person_names::{
            GivenNameResolution, NameFamilyId, NameFormId, NameFormSelectionSeed,
        };
        let mut identity = PersonalNameIdentity {
            given: GivenNameResolution::Resolved {
                family_id: NameFamilyId::new("johannes"),
                native_form_id: NameFormId::new("johannes_hans_de"),
            },
            native_culture: Culture::German,
            form_selector: NameFormSelectionSeed::new(0),
            surname_id: None,
        };
        match self {
            Self::InvalidFamily | Self::MismatchedFamily => {
                let GivenNameResolution::Resolved { family_id, .. } = &mut identity.given else {
                    unreachable!()
                };
                *family_id = NameFamilyId::new(if matches!(self, Self::InvalidFamily) {
                    "missing-family"
                } else {
                    "heinrich"
                });
            }
            Self::InvalidSurname => {
                identity = authored_name_identity("Authored Name".into());
                identity.surname_id = Some(SurnameId::new("missing-surname"));
            }
            _ => return Err("Fault requires invalid semantic metadata".into()),
        }
        Ok(identity)
    }
}

/// Check the persisted projection against the semantic renderer after a write.
#[reducer]
pub fn authority_test_name_projection(
    ctx: &ReducerContext,
    bootstrap_token: String,
    id: u64,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    let identity = ctx
        .db
        .character_name_identity()
        .character_id()
        .find(id)
        .ok_or("Test character identity missing")?;
    let identity = NameIdentityJson::parse(identity.identity_json.as_str())
        .map_err(|error| error.to_string())?;
    let display = render_personal_name(
        &identity,
        identity.native_culture,
        NameRegister::Everyday,
        character_name_sex(ctx, id.into()).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let character = ctx
        .db
        .character()
        .id()
        .find(id)
        .ok_or("Test character missing")?;
    if character.name != display.as_str() {
        return Err("Persisted name disagrees with semantic identity".into());
    }
    Ok(())
}

/// The caller expects failure and checks that the damaged row rolls back.
#[reducer]
pub fn authority_test_name_failure(
    ctx: &ReducerContext,
    bootstrap_token: String,
    id: u64,
    fault: NameAuthorityFault,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    let mut row = ctx
        .db
        .character_name_identity()
        .character_id()
        .find(id)
        .ok_or("Test character has no identity")?;
    match fault {
        NameAuthorityFault::MalformedJson => {
            row.identity_json = NameIdentityJson { value: "{".into() };
            ctx.db.character_name_identity().character_id().update(row);
        }
        NameAuthorityFault::MissingIdentity => {
            ctx.db.character_name_identity().character_id().delete(id);
        }
        NameAuthorityFault::InvalidRendering => {
            row.identity_json =
                NameIdentityJson::from_identity(&authored_name_identity("\n".into()))
                    .map_err(|error| error.to_string())?;
            ctx.db.character_name_identity().character_id().update(row);
        }
        NameAuthorityFault::InvalidFamily
        | NameAuthorityFault::MismatchedFamily
        | NameAuthorityFault::InvalidSurname => {
            row.identity_json =
                NameIdentityJson::from_identity(&fault.invalid_semantic_identity()?)
                    .map_err(|error| error.to_string())?;
            ctx.db.character_name_identity().character_id().update(row);
        }
    }
    assign_authored_character_name(ctx, id.into(), "Should roll back".into())
        .map_err(|error| error.to_string())
}

#[reducer]
pub fn authority_test_invalid_name_assignment(
    ctx: &ReducerContext,
    bootstrap_token: String,
    id: u64,
    fault: NameAuthorityFault,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    assign_character_name_identity(ctx, id.into(), fault.invalid_semantic_identity()?)
        .map_err(|error| error.to_string())
}

#[reducer]
pub fn authority_test_absent_surname(
    ctx: &ReducerContext,
    bootstrap_token: String,
    id: u64,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    assign_character_name_identity(ctx, id.into(), authored_name_identity("No Surname".into()))
        .map_err(|error| error.to_string())?;
    if character_hereditary_surname(ctx, id.into())
        .map_err(|error| error.to_string())?
        .is_some()
    {
        return Err("Valid authored identity should have no hereditary surname".into());
    }
    Ok(())
}

#[reducer]
pub fn authority_test_claim_target(
    ctx: &ReducerContext,
    bootstrap_token: String,
    request_key: String,
    id: u64,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    let mut claim = ctx
        .db
        .starting_character_claim()
        .request_key()
        .find(request_key)
        .ok_or("Test claim missing")?;
    claim.character_id = id;
    ctx.db
        .starting_character_claim()
        .request_key()
        .update(claim);
    Ok(())
}

#[reducer]
pub fn authority_test_stale_assignment(
    ctx: &ReducerContext,
    bootstrap_token: String,
    id: u64,
    server: Identity,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    if server == Identity::ZERO
        || ctx
            .db
            .tactical_server_authority()
            .identity()
            .find(server)
            .is_some()
    {
        return Err("Test stale server must be nonzero and absent".into());
    }
    let mut row = ctx
        .db
        .character()
        .id()
        .find(id)
        .ok_or("Test character missing")?;
    row.server = server;
    ctx.db.character().id().update(row);
    Ok(())
}

/// Clone a valid capture to exercise roster validation and enrollment conflicts
/// without changing public mission-authoring rules.
#[reducer]
pub fn authority_test_request_roster(
    ctx: &ReducerContext,
    bootstrap_token: String,
    source_mission: String,
    mission_id: String,
    member_ids: Vec<u64>,
) -> Result<(), String> {
    use crate::strategic::mission_authority;
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    adventuresim_core::mission::MissionId::new(mission_id.clone())
        .map_err(|error| error.to_string())?;
    let mut request = ctx
        .db
        .tactical_server_request_authority()
        .mission_id()
        .find(&source_mission)
        .ok_or("Test source request missing")?;
    let mut mission = ctx
        .db
        .mission_authority()
        .id()
        .find(&source_mission)
        .ok_or("Test source mission missing")?;
    request.mission_id = mission_id.clone();
    request.authorized_party_member_ids = member_ids;
    // Enrollment-conflict fixtures share participant authority, but each server
    // must have its own opponents. These checks exercise party enrollment only.
    request.enemy_character_ids.clear();
    request.required_enemy_kills = 0;
    mission.id = mission_id;
    ctx.db.mission_authority().insert(mission);
    ctx.db.tactical_server_request_authority().insert(request);
    Ok(())
}

/// Assert the live join changes while historical generation evidence stays put.
#[reducer]
pub fn authority_test_resident_projection(
    ctx: &ReducerContext,
    bootstrap_token: String,
    id: u64,
) -> Result<(), String> {
    use crate::settlement_population::{
        resolve_settlement_resident, settlement_resident_seed_explanation,
    };
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    let before = ctx
        .db
        .settlement_resident_seed_explanation()
        .character_id()
        .find(id)
        .ok_or("Test resident explanation missing")?;
    let mut character = ctx
        .db
        .character()
        .id()
        .find(id)
        .ok_or("Test resident missing")?;
    character.age_years = 60;
    ctx.db.character().id().update(character);
    let mut personality = ctx
        .db
        .character_personality()
        .character_id()
        .find(id)
        .ok_or("Test resident personality missing")?;
    personality.sex = Sex::Female;
    personality.presentation = crate::personality::Presentation::Woman;
    ctx.db
        .character_personality()
        .character_id()
        .update(personality);
    assign_authored_character_name(ctx, id.into(), "Current Resident".into())
        .map_err(|error| error.to_string())?;
    let current = resolve_settlement_resident(ctx, id).ok_or("Test resident join missing")?;
    let after = ctx
        .db
        .settlement_resident_seed_explanation()
        .character_id()
        .find(id)
        .ok_or("Test resident explanation disappeared")?;
    if current.name != "Current Resident"
        || current.presentation != crate::personality::Presentation::Woman
        || current.sex != Sex::Female
        || current.age_band != adventuresim_core::settlement_population::AgeBand::Elder
        || current.profile.projection_id != id
        || before.relations_json != after.relations_json
    {
        return Err("Resident current projection or historical provenance disagrees".into());
    }
    Ok(())
}
