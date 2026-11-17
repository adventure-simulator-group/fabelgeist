use adventuresim_world_schema::calendar::StrategicMinute;
use sha2::{Digest, Sha256};
use spacetimedb::{
    Identity, ReducerContext, SpacetimeType, Table, ViewContext, reducer, table, view,
};
use std::collections::HashSet;

pub use consequence::{
    TacticalCharacterConsequence, TacticalConsequenceReceipt, TacticalEquipmentContact,
    TacticalEquipmentContactRole, TacticalHitInjury, TacticalMissionResolution,
    TacticalReceiptBodyPart,
};
use entry::leave_tactical_scene_for_server;
pub use entry::{enter_tactical_scene, leave_tactical_scene};
pub use hosting::{tactical_server, tactical_server_request};
pub use inventory::{ConnectedEquipmentOccupancy, ConnectedInventory, ConnectedPlayerItem};
pub use profile::{ConnectedPlayer, TacticalActorRole, connected_players};
pub use provisioning::record_tactical_scene_provisioning_failure;
pub use registration::create_tactical_server_for_request;
pub use session::{TacticalMissionSession, TacticalProvisioningStatus, TacticalSession};
pub use settlement::{request_settlement_scene, retire_settlement_scene};

pub use request::{TacticalBusinessOperator, TacticalSettlementSnapshot};
pub(crate) use request::{tactical_party_roster, tactical_settlement_snapshot};

use crate::repair::{ItemCondition, item_condition__view};
use crate::{
    CatalogItemKind, Character, CharacterAttributes, CharacterLimbs, CharacterSkills,
    CharacterStats, Item,
    character::{character, character_equipped_item, equipment_occupancy},
    character__view, character_attributes__view, character_equipped_item__view,
    character_limbs__view, character_skills__view, character_stats__view,
    condition::{character_condition__view, character_strategic_condition__view},
    equipment_occupancy__view, inventory_item__view,
    investigation::case_site_authority,
    item::{inventory_item, item, item__view},
    party_authority,
    strategic::{
        autoresolve_report, complete_bound_mission_success, ensure_bound_mission_authority,
        fail_bound_mission_attempt, hostile_group_authority, mission_authority,
        outcome_source_authority, strategic_gateway_authority__view,
    },
};

mod consequence;
pub(crate) mod dialogue;
mod entry;
mod hosting;
mod inventory;
pub(crate) mod local_time;
mod profile;
mod provisioning;
mod registration;
mod request;
mod residents;
mod session;
mod settlement;

/// Captured scene preparation input and retryable provisioning status. Runtime
/// positions and combat state remain on the transient server.
#[derive(Clone, Debug)]
#[table(accessor = tactical_server_request_authority)]
pub struct TacticalServerRequest {
    #[primary_key]
    #[unique]
    pub instance_id: String,
    #[index(btree)]
    pub gateway_bucket: u8,
    pub scene_key: String,
    pub requested_by: u64,
    pub longitude_e7: i32,
    pub latitude_e7: i32,
    pub absolute_minute: StrategicMinute,
    pub lunar_phase_minute: StrategicMinute,
    pub session: TacticalSession,
    pub provisioning: TacticalProvisioningStatus,
}

/// One operational server for a scene slot, shared by its admitted characters.
#[derive(Clone, Debug)]
#[table(accessor = tactical_server_authority)]
pub struct TacticalServer {
    #[primary_key]
    pub identity: Identity,
    pub hosting_connection: spacetimedb::ConnectionId,
    #[index(btree)]
    pub gateway_bucket: u8,
    #[index(btree)]
    #[unique]
    pub instance_id: String,
    pub scene_key: String,
    pub addr: String,
    pub cert_digest: String,
    pub session: TacticalSession,
}

#[derive(Clone, Debug)]
#[table(accessor = tactical_server_claim)]
pub struct TacticalServerClaim {
    #[primary_key]
    pub instance_id: String,
    pub claim_hash: Vec<u8>,
}

#[reducer]
pub fn authorize_tactical_server_claim(
    ctx: &ReducerContext,
    instance_id: String,
    claim_hash: Vec<u8>,
) -> Result<(), String> {
    crate::strategic::require_strategic_gateway(ctx)?;
    if claim_hash.len() != 32 {
        return Err("Tactical claim hash must be SHA-256".into());
    }
    if ctx
        .db
        .tactical_server_request_authority()
        .instance_id()
        .find(&instance_id)
        .is_none()
    {
        return Err("Tactical server request not found".into());
    }
    if ctx
        .db
        .tactical_server_claim()
        .instance_id()
        .find(&instance_id)
        .is_some()
    {
        return Err("Tactical server request is already claimed".into());
    }
    ctx.db.tactical_server_claim().insert(TacticalServerClaim {
        instance_id,
        claim_hash,
    });
    Ok(())
}

/// Release an unconsumed claim after the trusted dispatcher fails to start
/// its child process. Consumed claims and active servers have no row to revoke.
#[reducer]
pub fn revoke_tactical_server_claim(
    ctx: &ReducerContext,
    instance_id: String,
) -> Result<(), String> {
    crate::strategic::require_strategic_gateway(ctx)?;
    instance_id
        .parse::<adventuresim_core::tactical_session::TacticalInstanceId>()
        .map_err(|error| error.to_string())?;
    if ctx
        .db
        .tactical_server_request_authority()
        .instance_id()
        .find(&instance_id)
        .is_none()
    {
        return Err("Tactical server request is no longer pending".into());
    }
    ctx.db
        .tactical_server_claim()
        .instance_id()
        .delete(&instance_id);
    Ok(())
}

/// Retire an interrupted standalone development session before its persistent
/// profile is reused. The bootstrap capability is checked by the only caller;
/// keeping the cleanup here lets it share the authoritative mission teardown
/// primitives without exposing a new reducer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MissionCaseIdentity<'a> {
    Standalone { mission_id: &'a str },
    Other,
}

impl<'a> MissionCaseIdentity<'a> {
    fn parse(value: &'a str) -> Self {
        let mut components = value.splitn(3, ':');
        match (components.next(), components.next(), components.next()) {
            (Some("case"), Some("standalone"), Some(mission_id)) if !mission_id.is_empty() => {
                Self::Standalone { mission_id }
            }
            _ => Self::Other,
        }
    }
}

pub(crate) fn retire_interrupted_standalone_server_for_character(
    ctx: &ReducerContext,
    character_id: u64,
) -> Result<(), String> {
    let servers: Vec<_> =
        ctx.db
            .tactical_server_authority()
            .iter()
            .filter(|server| {
                server.session.mission().is_some_and(|mission| {
                    mission.authorized_party_member_ids.contains(&character_id)
                })
            })
            .collect();
    for server in servers {
        let Some(tactical) = server.session.mission() else {
            continue;
        };
        let mission = ctx
            .db
            .mission_authority()
            .id()
            .find(&tactical.mission_id)
            .ok_or("Interrupted tactical server has no mission authority")?;
        match MissionCaseIdentity::parse(&mission.case_id) {
            MissionCaseIdentity::Standalone { mission_id }
                if mission_id == tactical.mission_id.as_str() => {}
            MissionCaseIdentity::Standalone { .. } | MissionCaseIdentity::Other => {
                return Err("Refusing to retire a non-standalone tactical server".into());
            }
        }
        fail_bound_mission_attempt(ctx, &tactical.mission_id)?;
        let connected: Vec<_> = ctx
            .db
            .character()
            .server()
            .filter(server.identity)
            .collect();
        for character in connected {
            leave_tactical_scene_for_server(ctx, character, server.identity)?;
        }
        ctx.db
            .tactical_server_authority()
            .identity()
            .delete(server.identity);
        log::info!(
            "Retired interrupted standalone tactical server for mission '{}'",
            tactical.mission_id
        );
    }
    Ok(())
}

/// Request a new [`TacticalServer`] with generated *mission_id* from
/// the *scene_key* and the current timestamp.
#[reducer]
pub fn request_tactical_server_for_scene(
    ctx: &ReducerContext,
    character_id: u64,
    scene_key: String,
) -> Result<(), String> {
    let mission_id = format!(
        "mission:{scene_key}-{}",
        ctx.timestamp.to_micros_since_unix_epoch()
    );
    request_tactical_server(ctx, character_id, mission_id, scene_key)
}

/// Request a new [`TacticalServer`], unless there is already
/// a server with the same *mission_id*.
#[reducer]
pub fn request_tactical_server(
    ctx: &ReducerContext,
    character_id: u64,
    mission_id: String,
    scene_key: String,
) -> Result<(), String> {
    crate::strategic::require_strategic_character_authority(ctx, character_id)?;
    let mission_identity =
        adventuresim_core::mission::MissionId::new(mission_id.clone()).map_err(str::to_string)?;
    let character = crate::character::require_living_character(ctx, character_id)?;
    let party_id = character.party_id.ok_or("Character has no party")?;
    let party = ctx
        .db
        .party_authority()
        .id()
        .find(&party_id)
        .ok_or("Party not found")?;
    if party.leader_id != character_id {
        return Err("Only the party leader can request a tactical server".into());
    }
    let case_site = party
        .current_case_site_id
        .as_ref()
        .and_then(|site_id| {
            ctx.db
                .case_site_authority()
                .id_key()
                .find(site_id.to_string())
        })
        .ok_or("Party must be at a case site")?;
    if party.current_case_site_id.as_deref() != Some(case_site.id.as_str()) {
        return Err("Party must be at its active quest location".into());
    }
    if case_site.scene_key != scene_key {
        return Err("Tactical scene does not match the occupied case site".into());
    }
    if !case_site.coordinates_are_geographic
        || !(-1_800_000_000..=1_800_000_000).contains(&case_site.longitude_e7)
        || !(-900_000_000..=900_000_000).contains(&case_site.latitude_e7)
    {
        return Err("Tactical entry requires an exact geographic case-site position".into());
    }
    let (absolute_minute, lunar_phase_minute) =
        crate::strategic::party_wilderness_environment_minutes(&party)
            .ok_or("Case-site tactical entry requires a party wilderness clock")?;
    match crate::investigation::case_site_provenance_reducer(ctx, &case_site) {
        Some(None) => {}
        Some(Some(_)) => {
            return Err(
                "Generated quest finales support strategic autoresolve, not tactical entry".into(),
            );
        }
        None => return Err("Case-site combat provenance is invalid or ambiguous".into()),
    }
    let mission = ensure_bound_mission_authority(
        ctx,
        &mission_id,
        &party_id,
        character_id,
        &case_site,
        &scene_key,
    )?;
    if mission.status != adventuresim_core::strategic_state::vocabulary::MissionAttemptStatus::Bound
    {
        return Err("Tactical request requires a newly bound mission attempt".into());
    }
    let hostile_group_id = mission
        .hostile_group_id
        .clone()
        .ok_or("Case-site mission has no hostile group")?;
    let group = ctx
        .db
        .hostile_group_authority()
        .id()
        .find(&hostile_group_id)
        .ok_or("Hostile group not found")?;
    if group.disposition != crate::strategic::HostileGroupDisposition::Active {
        return Err("Hostile group is already resolved".into());
    }
    let battle_id = format!("battle:{mission_id}");
    let outcome_source_id = format!("outcome:{mission_id}");
    if ctx
        .db
        .autoresolve_report()
        .battle_id()
        .find(&battle_id)
        .is_some()
        || ctx
            .db
            .outcome_source_authority()
            .id()
            .find(&outcome_source_id)
            .is_some()
    {
        return Err("Mission attempt already has a strategic resolution".into());
    }
    if let Some(server) = ctx.db.tactical_server_authority().instance_id().find(
        adventuresim_core::tactical_session::TacticalInstanceId::for_mission(&mission_identity)
            .into_string(),
    ) {
        return Err(format!(
            "Server for mission '{mission_id}' already exist: {}",
            server.identity
        ));
    }
    if request::hostile_group_has_tactical_scene(ctx, &hostile_group_id) {
        return Err("Party quest already has a pending or active tactical server".into());
    }

    let enemy_roster = mission
        .enemy_roster(ctx)
        .map_err(|error| error.to_string())?;
    log::info!("Tactical server for '{mission_id}' requested");
    let authorized_party_member_ids = tactical_party_roster(ctx, &party_id)
        .map_err(|error| error.to_string())?
        .into_member_ids();
    let session = TacticalSession::Mission(TacticalMissionSession {
        mission_id,
        party_id,
        authorized_party_member_ids,
        settlement: tactical_settlement_snapshot(ctx, &case_site)?,
        required_enemy_kills: enemy_roster.enemy_count().get(),
        enemy_difficulty: mission.enemy_difficulty,
        enemy_combat_scale_bps: mission.enemy_combat_scale_bps,
        countermeasure_multiplier_bps: u32::from(adventuresim_world_schema::BASIS_POINTS_PER_WHOLE),
        normalized_combat_power: mission.normalized_combat_power,
        enemy_character_ids: enemy_roster.into_enemy_ids(),
        party_has_surprise: !mission.contacted_before_combat,
    });
    ctx.db
        .tactical_server_request_authority()
        .insert(TacticalServerRequest {
            provisioning: crate::tactical::TacticalProvisioningStatus::Pending,
            instance_id: session.instance_id()?.into_string(),
            gateway_bucket: 0,
            scene_key,
            requested_by: character_id,
            longitude_e7: case_site.longitude_e7,
            latitude_e7: case_site.latitude_e7,
            absolute_minute,
            lunar_phase_minute,
            session,
        });

    Ok(())
}

/// End a [`TacticallServer`] associated with the caller.
#[reducer]
pub fn end_tactical_server(
    ctx: &ReducerContext,
    resolution: TacticalMissionResolution,
    receipt: TacticalConsequenceReceipt,
) -> Result<(), String> {
    let Some(server) = ctx
        .db
        .tactical_server_authority()
        .identity()
        .find(ctx.sender())
    else {
        return Err(format!(
            "Can't end tactical server: sender's server with identity {} not found",
            ctx.sender()
        ));
    };

    end_tactical_server_by_instance(ctx, server, resolution, receipt)
}

fn receipt_limb(body_part: TacticalReceiptBodyPart) -> adventuresim_core::physiology::BodyRegion {
    match body_part {
        TacticalReceiptBodyPart::LeftArm => adventuresim_core::physiology::BodyRegion::LeftArm,
        TacticalReceiptBodyPart::RightArm => adventuresim_core::physiology::BodyRegion::RightArm,
        TacticalReceiptBodyPart::LeftLeg => adventuresim_core::physiology::BodyRegion::LeftLeg,
        TacticalReceiptBodyPart::RightLeg => adventuresim_core::physiology::BodyRegion::RightLeg,
        TacticalReceiptBodyPart::Chest => adventuresim_core::physiology::BodyRegion::Chest,
        TacticalReceiptBodyPart::Stomach => adventuresim_core::physiology::BodyRegion::Abdomen,
        TacticalReceiptBodyPart::Head => adventuresim_core::physiology::BodyRegion::Head,
    }
}

fn validate_tactical_receipt(
    ctx: &ReducerContext,
    server: &TacticalServer,
    receipt: &TacticalConsequenceReceipt,
) -> Result<(), String> {
    let mission = server
        .session
        .mission()
        .ok_or("Combat receipts require a mission session")?;
    validate_tactical_receipt_bounds(receipt)?;
    let characters: HashSet<_> = receipt.party.iter().map(|row| row.character_id).collect();
    for consequence in &receipt.party {
        let character = ctx
            .db
            .character()
            .id()
            .find(consequence.character_id)
            .ok_or("Tactical receipt character not found")?;
        let assigned_elsewhere_active = character.has_tactical_server_assignment()
            && character.server != server.identity
            && ctx
                .db
                .tactical_server_authority()
                .identity()
                .find(character.server)
                .is_some();
        if !receipt_character_is_authorized(
            character.temporary,
            mission
                .authorized_party_member_ids
                .contains(&consequence.character_id),
            character.party_id.as_deref() == Some(mission.party_id.as_str()),
            assigned_elsewhere_active,
        ) {
            return Err("Tactical receipt character is not an authorized Party participant".into());
        }
    }
    for contact in &receipt.equipment_contacts {
        if !characters.contains(&contact.character_id) {
            return Err("Tactical equipment contact is not bounded to a receipt member".into());
        }
        let item = ctx
            .db
            .inventory_item()
            .id()
            .find(contact.inventory_item_id)
            .ok_or("Tactical equipment contact item not found")?;
        if item.character_id != contact.character_id || item.quantity != 1 {
            return Err("Tactical equipment contact has invalid custody".into());
        }
        let equipped = ctx
            .db
            .character_equipped_item()
            .inventory_item_id()
            .find(contact.inventory_item_id)
            .ok_or("Tactical equipment contact item is not equipped")?;
        if equipped.character_id != contact.character_id {
            return Err("Tactical equipment contact has invalid equipped custody".into());
        }
        let occupancy: Vec<_> = ctx
            .db
            .equipment_occupancy()
            .inventory_item_id()
            .filter(contact.inventory_item_id)
            .collect();
        let held = occupancy
            .iter()
            .any(|row| row.channel == adventuresim_core::item_catalog::EquipmentChannel::Held);
        let worn = occupancy.iter().any(|row| {
            row.anchor_kind == crate::character::EquipmentAnchorKind::CharacterLocation
                && row.channel != adventuresim_core::item_catalog::EquipmentChannel::Held
        });
        let definition = ctx
            .db
            .item()
            .id()
            .find(&item.item_id)
            .ok_or("Tactical equipment contact item definition not found")?;
        let valid_role = match contact.role {
            TacticalEquipmentContactRole::AttackerWeapon => {
                held && definition.kind == CatalogItemKind::Weapon
            }
            TacticalEquipmentContactRole::DefenderEquipment => {
                (held
                    && matches!(
                        definition.kind,
                        CatalogItemKind::Weapon | CatalogItemKind::Shield
                    ))
                    || (worn && definition.kind == CatalogItemKind::Armor)
            }
        };
        if !valid_role {
            return Err(
                "Tactical equipment contact role does not match equipped combat gear".into(),
            );
        }
    }
    Ok(())
}

fn receipt_character_is_authorized(
    temporary: bool,
    in_authorized_roster: bool,
    in_mission_party: bool,
    assigned_elsewhere_active: bool,
) -> bool {
    !temporary && in_authorized_roster && in_mission_party && !assigned_elsewhere_active
}

fn validate_tactical_receipt_bounds(receipt: &TacticalConsequenceReceipt) -> Result<(), String> {
    use adventuresim_core::mission::*;
    if receipt.party.len() > MAX_TACTICAL_RECEIPT_PARTICIPANTS
        || receipt.equipment_contacts.len() > MAX_TACTICAL_EQUIPMENT_CONTACTS
    {
        return Err("Tactical receipt exceeds record limits".into());
    }
    let mut characters = HashSet::new();
    for consequence in &receipt.party {
        if !characters.insert(consequence.character_id) {
            return Err("Tactical receipt contains a duplicate character".into());
        }
        if consequence.injuries.len() > MAX_TACTICAL_INJURIES_PER_PARTICIPANT
            || !consequence.blood_loss_fraction.is_finite()
            || !(0.0..=1.0).contains(&consequence.blood_loss_fraction)
            || consequence.ammunition_used > MAX_TACTICAL_AMMUNITION_USED
            || consequence.injuries.iter().any(|injury| {
                !injury.cut_damage.is_finite()
                    || !injury.blunt_damage.is_finite()
                    || !injury.max_single_hit_blunt_damage.is_finite()
                    || !(0.0..=MAX_TACTICAL_DAMAGE_PER_HIT).contains(&injury.cut_damage)
                    || !(0.0..=MAX_TACTICAL_DAMAGE_PER_HIT).contains(&injury.blunt_damage)
                    || !(0.0..=injury.blunt_damage).contains(&injury.max_single_hit_blunt_damage)
                    || injury.cut_damage + injury.blunt_damage > MAX_TACTICAL_DAMAGE_PER_HIT
            })
        {
            return Err("Tactical receipt contains out-of-range consequences".into());
        }
    }
    let mut contacted_items = HashSet::new();
    for contact in &receipt.equipment_contacts {
        if !contacted_items.insert(contact.inventory_item_id) {
            return Err("Tactical receipt contains duplicate equipment contacts".into());
        }
        if !contact.contact_stress.is_finite()
            || !(0.0..=MAX_TACTICAL_CONTACT_STRESS).contains(&contact.contact_stress)
        {
            return Err("Tactical equipment contact is not bounded to a receipt member".into());
        }
    }
    Ok(())
}

fn apply_tactical_receipt(
    ctx: &ReducerContext,
    receipt: &TacticalConsequenceReceipt,
) -> Result<(), String> {
    for consequence in &receipt.party {
        for injury in &consequence.injuries {
            crate::surgery::commit_aggregated_hit_injury(
                ctx,
                consequence.character_id,
                receipt_limb(injury.body_part),
                injury.cut_damage,
                injury.blunt_damage,
                injury.max_single_hit_blunt_damage,
                None,
            )?;
        }
        crate::condition::apply_blood_loss(
            ctx,
            consequence.character_id,
            consequence.blood_loss_fraction,
        )?;
        crate::strategic::consume_autoresolve_ammunition(
            ctx,
            consequence.character_id,
            consequence.ammunition_used,
        );
        crate::capability::refresh_character_capability(ctx, consequence.character_id)?;
        crate::filth::deposit_now(
            ctx,
            consequence.character_id,
            adventuresim_core::filth::FilthSubstance::Dirt,
            None,
            1,
        )?;
    }
    for contact in &receipt.equipment_contacts {
        crate::repair::apply_impact(ctx, contact.inventory_item_id, contact.contact_stress);
    }
    Ok(())
}

/// End a [`TacticallServer`].
fn end_tactical_server_by_instance(
    ctx: &ReducerContext,
    server: TacticalServer,
    resolution: TacticalMissionResolution,
    receipt: TacticalConsequenceReceipt,
) -> Result<(), String> {
    if ctx
        .db
        .tactical_server_authority()
        .identity()
        .find(server.identity)
        .is_none()
    {
        return Err(format!(
            "Can't end tactical server: server with identity {} not found",
            server.identity
        ));
    }

    let tactical = server
        .session
        .mission()
        .ok_or("Only combat mission sessions can submit an outcome")?;
    validate_tactical_receipt(ctx, &server, &receipt)?;
    let connected: Vec<_> = ctx
        .db
        .character()
        .server()
        .filter(server.identity)
        .collect();
    if resolution.succeeded() {
        let mission = ctx
            .db
            .mission_authority()
            .id()
            .find(&tactical.mission_id)
            .ok_or("Mission authority no longer exists")?;
        if mission.party_id != tactical.party_id {
            return Err("Mission authority changed before completion".into());
        }
        if connected
            .iter()
            .any(|character| character.party_id.as_deref() == Some(tactical.party_id.as_str()))
        {
            complete_bound_mission_success(ctx, &tactical.mission_id)?;
        } else {
            fail_bound_mission_attempt(ctx, &tactical.mission_id)?;
        }
    } else {
        fail_bound_mission_attempt(ctx, &tactical.mission_id)?;
    }

    apply_tactical_receipt(ctx, &receipt)?;
    if !resolution.succeeded() {
        for character in connected.iter().filter(|character| !character.temporary) {
            crate::condition::record_morale_event(
                ctx,
                character.id,
                adventuresim_core::morale::MoraleEventKind::Defeat,
                -5.0,
                Some(tactical.mission_id.clone()),
            )?;
        }
    }
    // Apply persistent character progression. Loot is handled by the battle result.
    for character in connected {
        leave_tactical_scene_for_server(ctx, character, server.identity)?;
    }

    ctx.db
        .tactical_server_authority()
        .identity()
        .delete(server.identity);

    log::info!(
        "Tactical server for mission '{}' ended: resolution={resolution:?}",
        tactical.mission_id,
    );
    Ok(())
}

#[cfg(test)]
mod authority_tests {
    use super::*;

    fn consequence(character_id: u64) -> TacticalCharacterConsequence {
        TacticalCharacterConsequence {
            character_id,
            injuries: Vec::new(),
            blood_loss_fraction: 0.0,
            ammunition_used: 0,
        }
    }

    #[test]
    fn standalone_case_identity_requires_the_exact_structured_tag() {
        assert_eq!(
            MissionCaseIdentity::parse("case:standalone:mission:animation-walk"),
            MissionCaseIdentity::Standalone {
                mission_id: "mission:animation-walk"
            }
        );
        for malformed in [
            "case:standalone:",
            "case:other:mission:animation-walk",
            "prefix:case:standalone:mission:animation-walk",
            "case-standalone:mission:animation-walk",
        ] {
            assert_eq!(
                MissionCaseIdentity::parse(malformed),
                MissionCaseIdentity::Other
            );
        }
    }

    #[test]
    fn empty_tactical_receipt_is_bounded() {
        assert!(validate_tactical_receipt_bounds(&TacticalConsequenceReceipt::default()).is_ok());
    }

    #[test]
    fn tactical_receipt_rejects_duplicates_and_nonfinite_values() {
        let duplicate = TacticalConsequenceReceipt {
            party: vec![consequence(1), consequence(1)],
            equipment_contacts: Vec::new(),
        };
        assert!(validate_tactical_receipt_bounds(&duplicate).is_err());

        let mut invalid = consequence(1);
        invalid.injuries.push(TacticalHitInjury {
            body_part: TacticalReceiptBodyPart::Chest,
            cut_damage: f32::NAN,
            blunt_damage: 0.0,
            max_single_hit_blunt_damage: 0.0,
        });
        assert!(
            validate_tactical_receipt_bounds(&TacticalConsequenceReceipt {
                party: vec![invalid],
                equipment_contacts: Vec::new(),
            })
            .is_err()
        );

        let mut excessive_combined_damage = consequence(2);
        excessive_combined_damage.injuries.push(TacticalHitInjury {
            body_part: TacticalReceiptBodyPart::Chest,
            cut_damage: 0.6,
            blunt_damage: 0.6,
            max_single_hit_blunt_damage: 0.6,
        });
        assert!(
            validate_tactical_receipt_bounds(&TacticalConsequenceReceipt {
                party: vec![excessive_combined_damage],
                equipment_contacts: Vec::new(),
            })
            .is_err()
        );
    }

    #[test]
    fn disconnected_roster_member_remains_authorized_but_foreign_assignment_does_not() {
        assert!(receipt_character_is_authorized(false, true, true, false));
        assert!(!receipt_character_is_authorized(false, false, true, false));
        assert!(!receipt_character_is_authorized(false, true, false, false));
        assert!(!receipt_character_is_authorized(false, true, true, true));
        assert!(!receipt_character_is_authorized(true, true, true, false));
    }

    #[test]
    fn tactical_receipt_validation_keeps_authority_and_custody_checks() {
        let source = crate::production_source(include_str!("tactical.rs"));
        assert!(source.contains("character.temporary"));
        assert!(source.contains("character.server != server.identity"));
        assert!(source.contains("character.party_id.as_deref()"));
        assert!(source.contains("item.character_id != contact.character_id"));
        assert!(source.contains("item.quantity != 1"));
    }

    #[test]
    fn tactical_schema_has_no_quest_keyed_mission_authority() {
        let source = crate::production_source(include_str!("tactical.rs"));
        for schema in ["TacticalServerRequest", "TacticalServer"] {
            let body = source
                .split(&format!("pub struct {schema}"))
                .nth(1)
                .and_then(|tail| tail.split('}').next())
                .expect("schema body");
            assert!(!body.contains("quest_id"));
            assert!(!body.contains("hostile_group_id"));
            assert!(!body.contains("case_site_id"));
            assert!(!body.contains("expected_resolution"));
            assert!(!body.contains("capture_subject"));
            assert!(!body.contains("outcome_entropy"));
        }
    }

    #[test]
    fn tactical_result_variants_cannot_choose_the_strategic_outcome() {
        use super::TacticalMissionResolution as T;
        assert!(!T::Failed.succeeded());
        assert!(
            !T::CaptureTargetKilled.succeeded(),
            "typed contradictory terminal evidence must fail without sampling"
        );
        for report in [T::Defeated, T::DrivenOff, T::Captured] {
            assert!(report.succeeded());
        }
    }

    #[test]
    fn gateway_entry_points_require_character_authority() {
        let source = crate::production_source(include_str!("tactical.rs"));
        let request = source
            .split("pub fn request_tactical_server(")
            .nth(1)
            .and_then(|tail| tail.split("#[reducer]").next())
            .expect("request reducer");
        assert!(request.contains("require_strategic_character_authority(ctx, character_id)?"));
    }

    #[test]
    fn tactical_entry_is_manual_only_and_scene_wrapper_cannot_bypass_it() {
        let source = crate::production_source(include_str!("tactical.rs"));
        let wrapper = source
            .split("pub fn request_tactical_server_for_scene")
            .nth(1)
            .and_then(|tail| tail.split("/// Request a new").next())
            .expect("scene request wrapper");
        assert!(
            wrapper.contains("request_tactical_server(ctx, character_id, mission_id, scene_key)")
        );

        let request = source
            .split("pub fn request_tactical_server(")
            .nth(1)
            .and_then(|tail| tail.split("#[reducer]").next())
            .expect("request reducer");
        let provenance = request
            .find("case_site_provenance_reducer")
            .expect("manual-only provenance guard");
        let binding = request
            .find("ensure_bound_mission_authority")
            .expect("mission binding");
        assert!(provenance < binding);
        assert!(request.contains("Some(None) => {}"));
        assert!(request.contains("Some(Some(_))"));
        assert!(request.contains("strategic autoresolve, not tactical entry"));
    }

    #[test]
    fn claim_is_gateway_authorized_hashed_and_consumed_once() {
        let source = format!(
            "{}\n{}",
            crate::production_source(include_str!("tactical.rs")),
            include_str!("tactical/registration.rs")
        );
        assert!(source.contains("require_strategic_gateway(ctx)?"));
        assert!(source.contains("Sha256::digest(claim.as_bytes())"));
        assert!(source.contains(".tactical_server_claim()"));
        assert!(source.contains(".delete(&instance_id)"));
        assert!(!source.contains("pub claim:"));
    }

    #[test]
    fn strategic_request_binds_captured_party_members_only_for_mission_sessions() {
        let source = crate::production_source(include_str!("tactical.rs"));
        for schema in ["TacticalServerRequest", "TacticalServer"] {
            let body = source
                .split(&format!("pub struct {schema}"))
                .nth(1)
                .and_then(|tail| tail.split('}').next())
                .expect("schema body");
            assert!(!body.contains("expected_party_members"));
            assert!(body.contains("pub session: TacticalSession"));
        }
        let session = include_str!("tactical/session.rs");
        assert!(session.contains("pub authorized_party_member_ids: Vec<u64>"));
        assert!(session.contains("Settlement(TacticalSettlementSnapshot)"));
        let create = include_str!("tactical/registration.rs");
        assert!(create.contains("request.session.validate()?"));
    }

    #[test]
    fn tactical_completion_is_an_opaque_signal_to_strategic_authority() {
        let source = crate::production_source(include_str!("tactical.rs"));
        let completion = source
            .split("fn end_tactical_server_by_instance")
            .nth(1)
            .and_then(|tail| tail.split("#[cfg(test)]").next())
            .expect("tactical completion body");
        assert!(completion.contains("complete_bound_mission_success"));
        assert!(!completion.contains("commit_hostile_battle_resolution"));
        assert!(!completion.contains("HostileResolutionKind"));
        assert!(!completion.contains("capture_subject"));
    }
}
