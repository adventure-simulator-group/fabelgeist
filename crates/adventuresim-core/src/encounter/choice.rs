//! Choices accepted at the strategic encounter boundary.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncounterChoice {
    Sneak,
    Detour,
    Attack,
    Run,
    Surrender,
}

impl EncounterChoice {
    pub const fn stable_id(self) -> &'static str {
        match self {
            Self::Sneak => "sneak",
            Self::Detour => "detour",
            Self::Attack => "attack",
            Self::Run => "run",
            Self::Surrender => "surrender",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EncounterChoiceParseError;

impl std::fmt::Display for EncounterChoiceParseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Unknown encounter choice")
    }
}

impl std::error::Error for EncounterChoiceParseError {}

impl std::str::FromStr for EncounterChoice {
    type Err = EncounterChoiceParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        [
            Self::Sneak,
            Self::Detour,
            Self::Attack,
            Self::Run,
            Self::Surrender,
        ]
        .into_iter()
        .find(|choice| choice.stable_id() == value)
        .ok_or(EncounterChoiceParseError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transport_tokens_round_trip_and_reject_unknown_actions() {
        for choice in [
            EncounterChoice::Sneak,
            EncounterChoice::Detour,
            EncounterChoice::Attack,
            EncounterChoice::Run,
            EncounterChoice::Surrender,
        ] {
            assert_eq!(choice.stable_id().parse(), Ok(choice));
        }
        assert_eq!(
            "withdraw".parse::<EncounterChoice>(),
            Err(EncounterChoiceParseError)
        );
    }
}
