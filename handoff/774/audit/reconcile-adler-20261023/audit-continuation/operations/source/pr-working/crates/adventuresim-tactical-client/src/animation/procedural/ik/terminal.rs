//! Terminal stop contacts own their correction only after contact preparation.

use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct TerminalReach {
    pub(super) base_translation: Option<Vec3>,
    pub(super) shift: SignedLength,
    pub(super) target_shift: Option<SignedLength>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(super) enum TerminalContacts {
    #[default]
    Inactive,
    Seeded {
        shift: SignedLength,
    },
    Prepared(TerminalReach),
}

impl TerminalContacts {
    pub(super) fn is_prepared(self) -> bool {
        matches!(self, Self::Prepared(_))
    }

    /// Ordinary idle has no terminal correction; its neutral offset is zero.
    pub(super) fn shift(self) -> SignedLength {
        match self {
            Self::Inactive => SignedLength::ZERO,
            Self::Seeded { shift } => shift,
            Self::Prepared(reach) => reach.shift,
        }
    }

    pub(super) fn prepared_mut(&mut self) -> Option<&mut TerminalReach> {
        match self {
            Self::Prepared(reach) => Some(reach),
            Self::Inactive | Self::Seeded { .. } => None,
        }
    }

    /// A failed terrain lookup retains the visible seed, not prepared ownership.
    pub(super) fn seed(&mut self, shift: SignedLength) {
        if !self.is_prepared() {
            *self = Self::Seeded { shift };
        }
    }

    pub(super) fn prepare(&mut self) {
        if !self.is_prepared() {
            *self = Self::Prepared(TerminalReach {
                base_translation: None,
                shift: self.shift(),
                target_shift: None,
            });
        }
    }
}
