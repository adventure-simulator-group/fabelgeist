//! Contradictory flattened strategic lifecycle fields.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateParseError {
    PartyIdentity(crate::identity::IdentityError),
    ContractLifecycle,
    IncompleteMissionBinding,
    MissionResolution,
    CommitmentLifecycle,
    CourtshipRoute,
    CourtshipLifecycle,
    MarriageLifecycle,
    PregnancyOutcome,
}

impl std::fmt::Display for StateParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Self::PartyIdentity(error) = self {
            return error.fmt(f);
        }
        f.write_str(match self {
            Self::PartyIdentity(_) => unreachable!("identity errors are handled above"),
            Self::ContractLifecycle => "contract status and lifecycle fields disagree",
            Self::IncompleteMissionBinding => "mission binding must contain both identifiers",
            Self::MissionResolution => "mission status and resolution fields disagree",
            Self::CommitmentLifecycle => "commitment status and terminal fields disagree",
            Self::CourtshipRoute => "courtship kind and route fields disagree",
            Self::CourtshipLifecycle => "courtship status and terminal fields disagree",
            Self::MarriageLifecycle => "marriage status and terminal minute disagree",
            Self::PregnancyOutcome => "pregnancy status and outcome fields disagree",
        })
    }
}
impl std::error::Error for StateParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::PartyIdentity(error) => Some(error),
            _ => None,
        }
    }
}
