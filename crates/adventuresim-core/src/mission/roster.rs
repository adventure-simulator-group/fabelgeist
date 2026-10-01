//! Captured party membership is the sole tactical enrollment authority.

use std::{collections::BTreeSet, fmt, num::NonZeroU32};

use super::MAX_TACTICAL_RECEIPT_PARTICIPANTS;

/// A nonempty, bounded roster with unique durable character IDs.
///
/// Database and SDK adapters carry the IDs; scalar launch counts are derived
/// from this validated snapshot, never maintained as a second stored fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TacticalPartyRoster {
    members: Vec<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TacticalRosterError {
    Empty,
    TooLarge,
    DuplicateMember,
}

impl fmt::Display for TacticalRosterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Empty => "A tactical mission requires at least one living party member",
            Self::TooLarge => "Party exceeds the tactical receipt participant limit",
            Self::DuplicateMember => "Tactical participant authority contains duplicate members",
        })
    }
}

impl std::error::Error for TacticalRosterError {}

impl TryFrom<Vec<u64>> for TacticalPartyRoster {
    type Error = TacticalRosterError;

    fn try_from(members: Vec<u64>) -> Result<Self, Self::Error> {
        if members.is_empty() {
            return Err(TacticalRosterError::Empty);
        }
        if members.len() > MAX_TACTICAL_RECEIPT_PARTICIPANTS {
            return Err(TacticalRosterError::TooLarge);
        }
        if members.iter().collect::<BTreeSet<_>>().len() != members.len() {
            return Err(TacticalRosterError::DuplicateMember);
        }
        Ok(Self { members })
    }
}

impl TacticalPartyRoster {
    pub fn expected_members(&self) -> NonZeroU32 {
        NonZeroU32::new(u32::try_from(self.members.len()).expect("the roster limit fits u32"))
            .expect("a validated roster is nonempty")
    }

    pub fn into_member_ids(self) -> Vec<u64> {
        self.members
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enrollment_count_and_order_come_from_the_captured_members() {
        let roster = TacticalPartyRoster::try_from(vec![9, 3]).unwrap();
        assert_eq!(roster.expected_members().get(), 2);
        assert_eq!(roster.into_member_ids(), vec![9, 3]);
    }

    #[test]
    fn invalid_rosters_cannot_be_used_for_enrollment() {
        assert_eq!(
            TacticalPartyRoster::try_from(vec![]),
            Err(TacticalRosterError::Empty)
        );
        assert_eq!(
            TacticalPartyRoster::try_from(vec![7, 7]),
            Err(TacticalRosterError::DuplicateMember)
        );
        let limit = u64::try_from(MAX_TACTICAL_RECEIPT_PARTICIPANTS).unwrap();
        assert!(TacticalPartyRoster::try_from((0..limit).collect::<Vec<_>>()).is_ok());
        assert_eq!(
            TacticalPartyRoster::try_from((0..=limit).collect::<Vec<_>>()),
            Err(TacticalRosterError::TooLarge)
        );
    }
}
