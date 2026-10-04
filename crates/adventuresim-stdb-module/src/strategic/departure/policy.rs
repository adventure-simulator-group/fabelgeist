//! Departure admission owns location, stop, incident and readiness policy.

use super::DepartureRevalidationError;
use adventuresim_core::identity::{CharacterId, IdentityError, SettlementId};
use adventuresim_core::strategic_place::CaseSiteId;

/// Retain both stored location fields when checking a synchronized snapshot.
/// An absent or ambiguous location never qualifies for a withdrawal bypass.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DepartureLocation {
    settlement: Option<SettlementId>,
    case_site: Option<CaseSiteId>,
}

impl DepartureLocation {
    pub(crate) fn new(settlement: Option<SettlementId>, case_site: Option<CaseSiteId>) -> Self {
        Self {
            settlement,
            case_site,
        }
    }

    pub(crate) fn from_storage(
        settlement: Option<String>,
        case_site: Option<CaseSiteId>,
    ) -> Result<Self, IdentityError> {
        Ok(Self::new(
            settlement.map(SettlementId::try_new).transpose()?,
            case_site,
        ))
    }

    pub(crate) fn settlement(&self) -> Option<&SettlementId> {
        self.settlement.as_ref()
    }

    pub(crate) fn case_site(&self) -> Option<&CaseSiteId> {
        self.case_site.as_ref()
    }

    pub(crate) fn admit_members(
        &self,
        mut locations: impl Iterator<Item = Result<Option<Self>, DepartureRevalidationError>>,
    ) -> Result<(), DepartureRevalidationError> {
        let first = locations
            .next()
            .ok_or(DepartureRevalidationError::MemberLocationChanged)??;
        for location in std::iter::once(Ok(first)).chain(locations) {
            if location?.as_ref() != Some(self) {
                return Err(DepartureRevalidationError::MemberLocationChanged);
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DepartureReadinessRule {
    RequireReady,
    CaseSiteWithdrawal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DepartureReadinessRequirement {
    ReadyParty,
    Withdrawal,
}

impl DepartureReadinessRule {
    pub(crate) fn requirement(self, location: &DepartureLocation) -> DepartureReadinessRequirement {
        match (self, &location.settlement, &location.case_site) {
            (Self::CaseSiteWithdrawal, None, Some(_)) => DepartureReadinessRequirement::Withdrawal,
            _ => DepartureReadinessRequirement::ReadyParty,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DepartureStop {
    AtLocation,
    Camped,
}

pub(crate) struct PartyDepartureSnapshot {
    leader: CharacterId,
    location: DepartureLocation,
    stop: DepartureStop,
}

impl PartyDepartureSnapshot {
    pub(crate) fn new(
        leader: CharacterId,
        location: DepartureLocation,
        stop: DepartureStop,
    ) -> Self {
        Self {
            leader,
            location,
            stop,
        }
    }

    pub(crate) fn admit(
        &self,
        expected_leader: CharacterId,
        expected_location: &DepartureLocation,
        incidents: &PendingDepartureIncidents,
    ) -> Result<(), DepartureRevalidationError> {
        if self.leader != expected_leader
            || self.location != *expected_location
            || self.stop == DepartureStop::Camped
        {
            return Err(DepartureRevalidationError::Interrupted);
        }
        incidents.admit(expected_location.case_site())
    }
}

/// Preserve duplicate incidents: even two at the same site prevent departure.
pub(crate) struct PendingDepartureIncidents(Vec<CaseSiteId>);

impl PendingDepartureIncidents {
    pub(crate) fn new(sites: impl Iterator<Item = CaseSiteId>) -> Self {
        Self(sites.collect())
    }

    fn admit(&self, expected_site: Option<&CaseSiteId>) -> Result<(), DepartureRevalidationError> {
        match self.0.as_slice() {
            [] => Ok(()),
            [site] if expected_site == Some(site) => Ok(()),
            _ => Err(DepartureRevalidationError::Interrupted),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn case_location(site: CaseSiteId) -> DepartureLocation {
        DepartureLocation::new(None, Some(site))
    }

    #[test]
    fn departure_requires_unchanged_party_members_and_incident_snapshot() {
        let leader = CharacterId::from(u64::MAX);
        let site = CaseSiteId::try_new("site:a").unwrap();
        let location = case_location(site.clone());
        let party =
            PartyDepartureSnapshot::new(leader, location.clone(), DepartureStop::AtLocation);
        let clear = PendingDepartureIncidents::new(std::iter::empty());
        let blocked =
            PendingDepartureIncidents::new([CaseSiteId::try_new("site:b").unwrap()].into_iter());
        assert!(matches!(
            party.admit(leader, &location, &blocked),
            Err(DepartureRevalidationError::Interrupted)
        ));
        assert!(matches!(
            party.admit(CharacterId::from(0), &location, &clear),
            Err(DepartureRevalidationError::Interrupted)
        ));
        assert!(matches!(
            location.admit_members([Ok(None)].into_iter()),
            Err(DepartureRevalidationError::MemberLocationChanged)
        ));
        party.admit(leader, &location, &clear).unwrap();
        location
            .admit_members([Ok(Some(location.clone()))].into_iter())
            .unwrap();
        let camp = PartyDepartureSnapshot::new(leader, location.clone(), DepartureStop::Camped);
        assert!(matches!(
            camp.admit(leader, &location, &clear),
            Err(DepartureRevalidationError::Interrupted)
        ));
        let changed = case_location(CaseSiteId::try_new("site:b").unwrap());
        assert!(matches!(
            party.admit(leader, &changed, &clear),
            Err(DepartureRevalidationError::Interrupted)
        ));
        assert!(matches!(
            location.admit_members([Ok(Some(changed))].into_iter()),
            Err(DepartureRevalidationError::MemberLocationChanged)
        ));
        assert!(matches!(
            location.admit_members(std::iter::empty()),
            Err(DepartureRevalidationError::MemberLocationChanged)
        ));
    }

    #[test]
    fn only_case_site_withdrawal_may_bypass_departure_readiness() {
        let site = CaseSiteId::try_new("site:a").unwrap();
        let settlement = SettlementId::try_new("settlement:a").unwrap();
        let withdrawal = DepartureReadinessRule::CaseSiteWithdrawal;
        assert_eq!(
            withdrawal.requirement(&case_location(site.clone())),
            DepartureReadinessRequirement::Withdrawal
        );
        for location in [
            DepartureLocation::new(Some(settlement.clone()), None),
            DepartureLocation::new(None, None),
            DepartureLocation::new(Some(settlement), Some(site.clone())),
        ] {
            assert_eq!(
                withdrawal.requirement(&location),
                DepartureReadinessRequirement::ReadyParty
            );
        }
        assert_eq!(
            DepartureReadinessRule::RequireReady.requirement(&case_location(site)),
            DepartureReadinessRequirement::ReadyParty
        );
    }

    #[test]
    fn only_the_exact_departing_incident_site_may_be_avoided() {
        let site = CaseSiteId::try_new("site:a").unwrap();
        PendingDepartureIncidents::new(std::iter::empty())
            .admit(None)
            .unwrap();
        PendingDepartureIncidents::new([site.clone()].into_iter())
            .admit(Some(&site))
            .unwrap();
        for (expected, pending) in [
            (Some(&site), vec![CaseSiteId::try_new("site:b").unwrap()]),
            (None, vec![site.clone()]),
            (
                Some(&site),
                vec![site.clone(), CaseSiteId::try_new("site:b").unwrap()],
            ),
            (Some(&site), vec![site.clone(), site.clone()]),
        ] {
            assert!(matches!(
                PendingDepartureIncidents::new(pending.into_iter()).admit(expected),
                Err(DepartureRevalidationError::Interrupted)
            ));
        }
    }

    #[test]
    fn member_checks_stop_at_the_first_refusal() {
        let location = case_location(CaseSiteId::try_new("site:a").unwrap());
        let mut later = std::iter::once_with(|| panic!("later SDK reads must remain lazy"));
        let result = location.admit_members(std::iter::once(Ok(None)).chain(&mut later));
        assert!(matches!(
            result,
            Err(DepartureRevalidationError::MemberLocationChanged)
        ));
        assert_eq!(later.size_hint(), (1, Some(1)));
    }
    #[test]
    fn settlement_travel_requests_the_bypass_only_for_case_site_origins() {
        let source = include_str!("../travel_reducers.rs");
        let travel = source
            .split("fn travel_to_settlement_impl")
            .nth(1)
            .and_then(|tail| tail.split("pub fn set_party_camp_fatigue_percent").next())
            .expect("settlement travel implementation");
        let normalized = travel.split_whitespace().collect::<String>();
        assert!(normalized.contains(&"(origin_kind == \"case_site\").then(|| CaseSiteId::try_new(origin_id.clone())) .transpose().map_err(TravelError::CaseSiteIdentity)?".split_whitespace().collect::<String>()));
        assert!(normalized.contains(&"if origin_kind == \"case_site\" { DepartureReadinessRule::CaseSiteWithdrawal } else { DepartureReadinessRule::RequireReady }".split_whitespace().collect::<String>()));
        assert!(travel.contains("require_party_ready(ctx, &party.id)?"));
    }
}
