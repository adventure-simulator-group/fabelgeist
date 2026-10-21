//! Ecosystem simulation policies, independent of UI checkboxes.
/// Whether plants disperse seeds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Seeding {
    Disabled,
    Enabled,
}
/// Whether plants die rather than remaining preserved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mortality {
    Disabled,
    Enabled,
}
/// Whether the scene shadow grid contributes to exposure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlobalShadowing {
    Disabled,
    Enabled,
}

impl Seeding {
    pub const fn from_enabled(enabled: bool) -> Self {
        if enabled {
            Self::Enabled
        } else {
            Self::Disabled
        }
    }
    pub const fn enabled(self) -> bool {
        matches!(self, Self::Enabled)
    }
}

impl Mortality {
    pub const fn from_enabled(enabled: bool) -> Self {
        if enabled {
            Self::Enabled
        } else {
            Self::Disabled
        }
    }
    pub const fn enabled(self) -> bool {
        matches!(self, Self::Enabled)
    }
}

impl GlobalShadowing {
    pub const fn from_enabled(enabled: bool) -> Self {
        if enabled {
            Self::Enabled
        } else {
            Self::Disabled
        }
    }
    pub const fn enabled(self) -> bool {
        matches!(self, Self::Enabled)
    }
}
