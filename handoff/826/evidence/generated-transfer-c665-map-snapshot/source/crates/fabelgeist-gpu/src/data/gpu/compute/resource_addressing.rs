//! Address projection required by generated native resource code.

use std::num::TryFromIntError;
use std::{error::Error, fmt};

/// One generated fetch and its controlling texture-address requirement.
#[derive(Clone, Debug)]
pub struct ResourceFetch {
    code: String,
    addressing: TextureAddressing,
}

/// Native projection generated for a controlling texture's coordinates.
#[derive(Clone, Copy, Debug)]
pub enum TextureAddressing {
    Coordinates,
    LinearIndex,
    LinearPopulation,
}

/// The actual scalar projection cannot represent the texture extent.
#[derive(Debug)]
pub enum TextureAddressError {
    VolumeOverflow { extent: wgpu::Extent3d },
    Projection {
        extent: wgpu::Extent3d,
        addressing: TextureAddressing,
        source: TryFromIntError,
    },
}

impl fmt::Display for TextureAddressError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::VolumeOverflow { extent } => write!(
                formatter,
                "texture extent {extent:?} overflows the widened population calculation"
            ),
            Self::Projection { extent, addressing, .. } => write!(
                formatter,
                "texture extent {extent:?} exceeds its {addressing:?} native projection"
            ),
        }
    }
}

impl Error for TextureAddressError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Projection { source, .. } => Some(source),
            Self::VolumeOverflow { .. } => None,
        }
    }
}

impl ResourceFetch {
    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn addressing(&self) -> TextureAddressing {
        self.addressing
    }

    pub(super) fn new(code: String, addressing: TextureAddressing) -> Self {
        Self { code, addressing }
    }
}

impl TextureAddressing {
    /// Combine requirements from native code emitted into the same program.
    pub fn combine(self, other: Self) -> Self {
        match (self, other) {
            (Self::LinearPopulation, _) | (_, Self::LinearPopulation) => Self::LinearPopulation,
            (Self::LinearIndex, _) | (_, Self::LinearIndex) => Self::LinearIndex,
            _ => Self::Coordinates,
        }
    }

    /// SDK extents count texels along each resource axis. This numerical
    /// boundary proves the generated unsigned flattening before native use;
    /// independent-coordinate programs need no such scalar projection.
    pub fn admit_extent(self, extent: wgpu::Extent3d) -> Result<(), TextureAddressError> {
        if matches!(self, Self::Coordinates) {
            return Ok(());
        }
        let population = u64::from(extent.width)
            .checked_mul(u64::from(extent.height))
            .and_then(|plane| plane.checked_mul(u64::from(extent.depth_or_array_layers)))
            .ok_or(TextureAddressError::VolumeOverflow { extent })?;
        let projected = match self {
            Self::LinearIndex => population.saturating_sub(1),
            Self::LinearPopulation => population,
            Self::Coordinates => return Ok(()),
        };
        u32::try_from(projected).map_err(|source| TextureAddressError::Projection {
            extent,
            addressing: self,
            source,
        })?;
        Ok(())
    }
}
