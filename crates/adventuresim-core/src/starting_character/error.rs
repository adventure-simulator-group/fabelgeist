//! Starting roster request and professional-life admission failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StartingCharacterError {
    GeneratorVersion,
    CandidateSeed,
    CandidateSlot,
    ProfessionalRequirements,
    ConflictingReligions,
    YoungProfession,
    MissingOrganizationRole,
    NoEligibleOrganization,
    MissingOrganization,
}
impl std::fmt::Display for StartingCharacterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::GeneratorVersion => "unsupported candidate generator version",
            Self::CandidateSeed => "candidate seed must be 32 lowercase hexadecimal characters",
            Self::CandidateSlot => "candidate slot is out of range",
            Self::ProfessionalRequirements => {
                "simulated professional life does not meet its starting requirements"
            }
            Self::ConflictingReligions => "starting organization role has conflicting religions",
            Self::YoungProfession => "young characters cannot have a profession",
            Self::MissingOrganizationRole => "starting organization role is missing",
            Self::NoEligibleOrganization => "profession has no eligible starting organization",
            Self::MissingOrganization => "starting organization is not in the catalog",
        })
    }
}
impl std::error::Error for StartingCharacterError {}
