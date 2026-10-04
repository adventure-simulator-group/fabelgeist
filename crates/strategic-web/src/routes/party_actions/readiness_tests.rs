use super::{PartyAction, PartyReadinessRequirement};
use crate::spacetimedb::{CaseSiteId, PartyView};

fn party(case_site_id: Option<CaseSiteId>) -> PartyView {
    PartyView {
        id: "party".into(),
        gateway_bucket: 0,
        name: "Party".into(),
        leader_id: 7,
        current_settlement_id: case_site_id.is_none().then(|| "ironforge".into()),
        current_case_site_id: case_site_id.clone(),
        active_contract_id: None,
        is_solo: true,
        camp_fatigue_percent: 50,
        walking_minutes_per_day: 480,
        travel_at_night: false,
        journey_start_minute_of_day: 0,
        wilderness_canonical_anchor_minute: case_site_id
            .is_some()
            .then_some(adventuresim_world_schema::calendar::StrategicMinute::ZERO),
        wilderness_elapsed_minutes: 0,
        camp_destination: None,
        camp_remaining_minutes: 0,
        physiology_target: 0.0,
        command_target: 0.0,
        religion_target: 0.0,
    }
}

#[test]
fn only_exact_case_site_settlement_withdrawal_bypasses_web_readiness() {
    let withdrawal = PartyAction::TravelToSettlement {
        settlement_id: "ironforge".into(),
    };
    let onsite_party = party(Some(CaseSiteId::try_new("site:old-graveyard").unwrap()));
    assert_eq!(
        withdrawal.readiness(
            Some(&CaseSiteId::try_new("site:old-graveyard").unwrap()),
            &onsite_party
        ),
        PartyReadinessRequirement::Exempt
    );
    assert_eq!(
        withdrawal.readiness(
            Some(&CaseSiteId::try_new("site:other").unwrap()),
            &onsite_party
        ),
        PartyReadinessRequirement::Required
    );
    assert_eq!(
        withdrawal.readiness(None, &onsite_party),
        PartyReadinessRequirement::Required
    );
    assert_eq!(
        withdrawal.readiness(None, &party(None)),
        PartyReadinessRequirement::Required
    );

    let travel = PartyAction::TravelToCaseSite {
        case_site_id: "site:old-graveyard".into(),
    };
    assert_eq!(
        travel.readiness(onsite_party.current_case_site_id.as_ref(), &onsite_party),
        PartyReadinessRequirement::Required
    );
    let cancel = PartyAction::CancelMission {
        mission_id: "mission-3".into(),
    };
    assert_eq!(
        cancel.readiness(None, &onsite_party),
        PartyReadinessRequirement::Exempt
    );
    let investigate = PartyAction::PerformInvestigation {
        action_id: "action:inspect".into(),
        method: "inspect_site".into(),
        expected_version: 1,
    };
    assert_eq!(
        investigate.readiness(
            Some(&CaseSiteId::try_new("site:old-graveyard").unwrap()),
            &onsite_party
        ),
        PartyReadinessRequirement::Required
    );
}
