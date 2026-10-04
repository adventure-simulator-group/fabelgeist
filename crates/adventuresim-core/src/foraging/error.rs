//! Foraging domain admission failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ForageError {
    Duration,
    TerrainMixture,
    SourceCount,
    DuplicateSource,
    UnknownSource,
    UnavailableSource,
    LicenseSubject,
    LicenseResource,
    LicenseBinding,
}
impl std::fmt::Display for ForageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Duration => "Foraging duration must use whole hours from one to 24 hours",
            Self::TerrainMixture => "Foraging terrain mixture is not normalized",
            Self::SourceCount => "Choose between one and five forage sources",
            Self::DuplicateSource => "Forage sources must be unique",
            Self::UnknownSource => "Unknown forage source",
            Self::UnavailableSource => "A forage source is unavailable in this vicinity",
            Self::LicenseSubject => "foraging license subject must be a character",
            Self::LicenseResource => "foraging license resource must be a source",
            Self::LicenseBinding => "foraging license decision does not bind its question",
        })
    }
}
impl std::error::Error for ForageError {}
