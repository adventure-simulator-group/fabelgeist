//! Address projection required by generated native resource code.

use std::num::TryFromIntError;

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
#[derive(Debug, thiserror::Error)]
pub enum TextureAddressError {
    #[error("texture extent {extent:?} overflows the widened population calculation")]
    VolumeOverflow { extent: wgpu::Extent3d },
    #[error("texture extent {extent:?} exceeds its {addressing:?} native projection")]
    Projection {
        extent: wgpu::Extent3d,
        addressing: TextureAddressing,
        #[source]
        source: TryFromIntError,
    },
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
