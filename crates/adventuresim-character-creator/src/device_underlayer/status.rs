//! The underlayer kernels' status word and its domain failures.

use std::fmt;

/// Why an underlayer fit failed; discriminants are the device protocol bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum FitFailure {
    /// Too many vertices hash to one weld bucket to search.
    CrowdedWeld = 1,
    /// More coincident vertices than one physical vertex may gather.
    LargeGroup = 2,
    /// The prism checks of one compression pass did not settle.
    Unsettled = 4,
    /// An offset ray spans more grid cells than a unit direction can.
    LongRay = 8,
    /// A cut vertex has no skin influences.
    NoInfluence = 16,
}

impl FitFailure {
    pub(super) const ALL: [Self; 5] = [
        Self::CrowdedWeld,
        Self::LargeGroup,
        Self::Unsettled,
        Self::LongRay,
        Self::NoInfluence,
    ];
}

impl fmt::Display for FitFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::CrowdedWeld => "too many coincident body vertices share a weld bucket",
            Self::LargeGroup => "too many coincident body vertices form one physical vertex",
            Self::Unsettled => "the underlayer compression sweep did not settle",
            Self::LongRay => "an underlayer offset direction is not a unit vector",
            Self::NoInfluence => "cut vertex has no skin influences",
        })
    }
}

impl std::error::Error for FitFailure {}

/// A device status word containing only the underlayer protocol's known bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FitStatus(u32);

impl TryFrom<u32> for FitStatus {
    type Error = UnknownFitStatus;

    /// Decode the raw GPU readback before entering the domain error path.
    fn try_from(bits: u32) -> Result<Self, UnknownFitStatus> {
        let mut known = 0;
        for failure in FitFailure::ALL {
            known |= failure as u32;
        }
        let unknown = bits & !known;
        if unknown != 0 {
            return Err(UnknownFitStatus { bits: unknown });
        }
        Ok(Self(bits))
    }
}

impl FitStatus {
    /// Report the first failure in protocol priority order.
    pub fn into_result(self) -> Result<(), FitFailure> {
        for failure in FitFailure::ALL {
            if self.0 & failure as u32 != 0 {
                return Err(failure);
            }
        }
        Ok(())
    }
}

/// Readback contained a flag outside the underlayer protocol.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnknownFitStatus {
    bits: u32,
}

impl fmt::Display for UnknownFitStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "unknown underlayer status bits: {:#x}",
            self.bits
        )
    }
}

impl std::error::Error for UnknownFitStatus {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn successful_and_combined_failures_keep_protocol_priority() {
        assert_eq!(FitStatus::try_from(0).unwrap().into_result(), Ok(()));
        let status =
            FitStatus::try_from(FitFailure::CrowdedWeld as u32 | FitFailure::Unsettled as u32)
                .unwrap();
        assert_eq!(status.into_result(), Err(FitFailure::CrowdedWeld));
        assert_eq!(
            FitStatus::try_from(FitFailure::NoInfluence as u32)
                .unwrap()
                .into_result(),
            Err(FitFailure::NoInfluence)
        );
    }

    #[test]
    fn reserved_flags_are_rejected_even_alongside_known_failures() {
        let reserved = 1 << 31;
        let error = FitStatus::try_from(reserved | FitFailure::LargeGroup as u32).unwrap_err();
        assert_eq!(error, UnknownFitStatus { bits: reserved });
    }

    #[test]
    fn context_preserves_a_classifiable_fit_failure() {
        let failure = FitStatus::try_from(FitFailure::Unsettled as u32)
            .unwrap()
            .into_result()
            .unwrap_err();
        let error = anyhow::Error::new(failure).context("fitting underlayer");
        assert_eq!(
            error.downcast_ref::<FitFailure>(),
            Some(&FitFailure::Unsettled)
        );
        assert_eq!(
            error.root_cause().to_string(),
            "the underlayer compression sweep did not settle"
        );
    }
}
