//! Public rest availability preserves the shared settlement economy policy.

use super::error::RestServiceAdmissionError;
use adventuresim_core::settlement_economy::action_service_available;
use adventuresim_world_schema::{SettlementActionService, SettlementEconomyProfile};

pub(super) fn require_settlement_rest_service(
    profile: &SettlementEconomyProfile,
    service: SettlementActionService,
) -> Result<(), RestServiceAdmissionError> {
    if action_service_available(profile, service) {
        Ok(())
    } else {
        Err(RestServiceAdmissionError::UnavailableService { service })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn settlement_rest_rejects_unavailable_inn_and_temple_services() {
        use adventuresim_world_schema::{SettlementActionService, SettlementService};

        let mut profile = adventuresim_world_schema::SettlementEconomyProfile::stage_placeholder();
        assert!(require_settlement_rest_service(&profile, SettlementActionService::Inn).is_ok());
        assert_eq!(
            require_settlement_rest_service(&profile, SettlementActionService::Temple),
            Err(RestServiceAdmissionError::UnavailableService {
                service: SettlementActionService::Temple
            })
        );
        profile.services.clear();
        assert_eq!(
            require_settlement_rest_service(&profile, SettlementActionService::Inn),
            Err(RestServiceAdmissionError::UnavailableService {
                service: SettlementActionService::Inn
            })
        );
        profile.services.push(SettlementService::Temple);
        assert!(require_settlement_rest_service(&profile, SettlementActionService::Temple).is_ok());
    }
}
