//! Cardinality of successful surface resolutions, including repeated passes.
/// A resolution counts once when it applies a positional correction. The same
/// pair may count again in a later pass; this is not a unique-pair cardinality.
///
/// ```compile_fail
/// use fabelgeist_shell::surface_contact::SurfaceContactCount;
/// use fabelgeist_shell::ParticleCount;
/// fn resolutions(_: SurfaceContactCount) {}
/// resolutions(ParticleCount::from(1));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct SurfaceContactCount(usize);
impl SurfaceContactCount {
    pub const NONE: Self = Self(0);
    pub(super) const RESOLVED: Self = Self(1);
}
impl std::ops::AddAssign for SurfaceContactCount {
    fn add_assign(&mut self, resolutions: Self) {
        self.0 += resolutions.0;
    }
}
impl std::fmt::Display for SurfaceContactCount {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}
