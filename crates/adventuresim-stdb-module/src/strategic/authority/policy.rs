//! Registered gateway authority and the alternate disposable-character admission.

use super::Identity;
use crate::simulation::SimulationCharacterAuthorityError;
use adventuresim_core::identity::CharacterId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GatewayAdmissionError {
    Unregistered,
    DifferentSender { owner: Identity, sender: Identity },
}
impl std::fmt::Display for GatewayAdmissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Unregistered => "Strategic gateway is not registered",
            Self::DifferentSender { .. } => {
                "This reducer may only be called by the strategic gateway"
            }
        })
    }
}
impl std::error::Error for GatewayAdmissionError {}

/// A stored gateway owner admits exactly that identity, including native zero.
/// Authentication is checked by registration, not by this stored-owner check.
pub(crate) struct RegisteredStrategicGateway {
    owner: Identity,
}
impl RegisteredStrategicGateway {
    pub(crate) fn new(owner: Identity) -> Self {
        Self { owner }
    }
    pub(crate) fn require_sender(self, sender: Identity) -> Result<(), GatewayAdmissionError> {
        if self.owner == sender {
            Ok(())
        } else {
            Err(GatewayAdmissionError::DifferentSender {
                owner: self.owner,
                sender,
            })
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum StrategicCharacterAuthorityError {
    Denied {
        character: CharacterId,
        gateway: GatewayAdmissionError,
        simulation: Box<SimulationCharacterAuthorityError>,
    },
}
impl std::fmt::Display for StrategicCharacterAuthorityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Character-mutating strategic reducers may only be called by the strategic gateway or the owner of the target disposable simulation character")
    }
}
impl std::error::Error for StrategicCharacterAuthorityError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Denied { simulation, .. } => Some(simulation.as_ref()),
        }
    }
}

/// Gateway admission takes precedence and avoids reading simulation authority.
/// If both authorities refuse, retain both refusals and the exact target.
pub(crate) struct CharacterAuthorityAdmission {
    character: CharacterId,
    gateway: Result<(), GatewayAdmissionError>,
}
impl CharacterAuthorityAdmission {
    pub(crate) fn new(character: CharacterId, gateway: Result<(), GatewayAdmissionError>) -> Self {
        Self { character, gateway }
    }
    pub(crate) fn require_with(
        self,
        simulation: impl FnOnce(CharacterId) -> Result<(), SimulationCharacterAuthorityError>,
    ) -> Result<(), StrategicCharacterAuthorityError> {
        match self.gateway {
            Ok(()) => Ok(()),
            Err(gateway) => simulation(self.character).map_err(|simulation| {
                StrategicCharacterAuthorityError::Denied {
                    character: self.character,
                    gateway,
                    simulation: Box::new(simulation),
                }
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn gateway_checks_only_the_recorded_owner_and_retains_both_identities() {
        let owner = Identity::from_byte_array([1; 32]);
        let sender = Identity::from_byte_array([2; 32]);
        RegisteredStrategicGateway::new(owner)
            .require_sender(owner)
            .unwrap();
        let error = RegisteredStrategicGateway::new(owner)
            .require_sender(sender)
            .unwrap_err();
        assert_eq!(
            error,
            GatewayAdmissionError::DifferentSender { owner, sender }
        );
        assert_eq!(
            error.to_string(),
            "This reducer may only be called by the strategic gateway"
        );
        assert_eq!(
            GatewayAdmissionError::Unregistered.to_string(),
            "Strategic gateway is not registered"
        );
        // The registration reducer owns authentication; stored equality is unchanged.
        RegisteredStrategicGateway::new(Identity::ZERO)
            .require_sender(Identity::ZERO)
            .unwrap();
    }

    #[test]
    fn a_gateway_grant_never_reads_disposable_authority() {
        CharacterAuthorityAdmission::new(CharacterId::from(u64::MAX), Ok(()))
            .require_with(|_| panic!("a gateway grant must not read the simulation"))
            .unwrap();
    }

    #[test]
    fn either_gateway_denial_can_fall_back_to_the_owned_character() {
        let character = CharacterId::from(0);
        for gateway in [
            GatewayAdmissionError::Unregistered,
            GatewayAdmissionError::DifferentSender {
                owner: Identity::from_byte_array([1; 32]),
                sender: Identity::from_byte_array([2; 32]),
            },
        ] {
            CharacterAuthorityAdmission::new(character, Err(gateway))
                .require_with(|actual| {
                    assert_eq!(actual, character);
                    Ok(())
                })
                .unwrap();
        }
    }

    #[test]
    fn denied_alternatives_keep_the_full_target_and_simulation_cause() {
        let character = CharacterId::from(u64::MAX);
        let gateway = GatewayAdmissionError::DifferentSender {
            owner: Identity::from_byte_array([1; 32]),
            sender: Identity::from_byte_array([2; 32]),
        };
        let simulation = SimulationCharacterAuthorityError::CharacterOutsideRun { character };
        let error = CharacterAuthorityAdmission::new(character, Err(gateway))
            .require_with(|_| Err(simulation))
            .unwrap_err();
        assert_eq!(
            error,
            StrategicCharacterAuthorityError::Denied {
                character,
                gateway,
                simulation: Box::new(simulation)
            }
        );
        assert_eq!(
            error
                .source()
                .unwrap()
                .downcast_ref::<SimulationCharacterAuthorityError>(),
            Some(&simulation)
        );
        assert_eq!(
            error.to_string(),
            "Character-mutating strategic reducers may only be called by the strategic gateway or the owner of the target disposable simulation character"
        );
    }

    #[test]
    fn gateway_and_character_readers_keep_missing_and_alternate_read_order() {
        let source = include_str!("../authority.rs");
        let missing = source
            .find(".ok_or(GatewayAdmissionError::Unregistered)")
            .unwrap();
        let sender = source.find("require_sender(ctx.sender())").unwrap();
        let gateway = source
            .find("let gateway = require_strategic_gateway(ctx)")
            .unwrap();
        let simulation = source
            .find("crate::simulation::require_simulation_character_authority")
            .unwrap();
        assert!(missing < sender && sender < gateway && gateway < simulation);
        assert!(!source.contains("is_ok()"));
        assert!(!source.contains("error.to_string()"));
    }

    #[test]
    fn route_and_travel_retain_gateway_refusal_until_presentation() {
        use crate::strategic::{route_error::RouteAdmissionError, travel_error::TravelError};
        let gateway = GatewayAdmissionError::Unregistered;
        let travel = TravelError::from(RouteAdmissionError::from(gateway));
        let route = travel
            .source()
            .unwrap()
            .downcast_ref::<RouteAdmissionError>()
            .unwrap();
        assert_eq!(
            route
                .source()
                .unwrap()
                .downcast_ref::<GatewayAdmissionError>(),
            Some(&gateway)
        );
        assert_eq!(travel.to_string(), "Strategic gateway is not registered");
    }
}
