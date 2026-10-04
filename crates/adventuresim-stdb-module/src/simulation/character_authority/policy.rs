//! Disposable run ownership admits only its recorded simulation characters.

use super::Identity;
use adventuresim_core::identity::CharacterId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SimulationCharacterAuthorityError {
    Unclaimed {
        character: CharacterId,
    },
    DifferentOwner {
        character: CharacterId,
        owner: Identity,
        sender: Identity,
    },
    CharacterOutsideRun {
        character: CharacterId,
    },
}
impl std::fmt::Display for SimulationCharacterAuthorityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Only an owned disposable simulation may simulate NPC interaction")
    }
}
impl std::error::Error for SimulationCharacterAuthorityError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SimulationCharacterMembership {
    Member,
    OutsideRun,
}

pub(crate) struct DisposableSimulationOwner {
    owner: Identity,
}
impl DisposableSimulationOwner {
    pub(crate) fn new(owner: Identity) -> Self {
        Self { owner }
    }
    pub(crate) fn require_character(
        self,
        character: CharacterId,
        sender: Identity,
        membership: impl FnOnce(CharacterId) -> SimulationCharacterMembership,
    ) -> Result<(), SimulationCharacterAuthorityError> {
        if self.owner != sender {
            return Err(SimulationCharacterAuthorityError::DifferentOwner {
                character,
                owner: self.owner,
                sender,
            });
        }
        match membership(character) {
            SimulationCharacterMembership::Member => Ok(()),
            SimulationCharacterMembership::OutsideRun => {
                Err(SimulationCharacterAuthorityError::CharacterOutsideRun { character })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn another_owner_refuses_before_reading_character_membership() {
        let owner = Identity::from_byte_array([1; 32]);
        let sender = Identity::from_byte_array([2; 32]);
        let character = CharacterId::from(u64::MAX);
        let error = DisposableSimulationOwner::new(owner)
            .require_character(character, sender, |_| {
                panic!("another owner cannot read simulation membership")
            })
            .unwrap_err();
        assert_eq!(
            error,
            SimulationCharacterAuthorityError::DifferentOwner {
                character,
                owner,
                sender
            }
        );
        assert_eq!(
            error.to_string(),
            "Only an owned disposable simulation may simulate NPC interaction"
        );
    }

    #[test]
    fn owned_run_still_requires_the_exact_disposable_character() {
        let owner = Identity::from_byte_array([1; 32]);
        for character in [CharacterId::from(0), CharacterId::from(u64::MAX)] {
            DisposableSimulationOwner::new(owner)
                .require_character(character, owner, |actual| {
                    assert_eq!(actual, character);
                    SimulationCharacterMembership::Member
                })
                .unwrap();
            let error = DisposableSimulationOwner::new(owner)
                .require_character(character, owner, |_| {
                    SimulationCharacterMembership::OutsideRun
                })
                .unwrap_err();
            assert_eq!(
                error,
                SimulationCharacterAuthorityError::CharacterOutsideRun { character }
            );
        }
    }

    #[test]
    fn native_run_reader_keeps_claim_and_owner_before_membership() {
        let source = include_str!("../character_authority.rs");
        let source = source.split_whitespace().collect::<String>();
        let run = source.find(".simulation_run()").unwrap();
        let missing = source
            .find(".ok_or(SimulationCharacterAuthorityError::Unclaimed")
            .unwrap();
        let owner = source
            .find("DisposableSimulationOwner::new(run.owner)")
            .unwrap();
        let sender = source.find("ctx.sender()").unwrap();
        let member = source.find(".simulation_character()").unwrap();
        assert!(run < missing && missing < owner && owner < sender && sender < member);
        assert!(!source.contains(".run_id"));
        assert!(!source.contains(".character()"));
    }
}
