//! Frame identity and admission before publishing a temporal running mean.

use super::Globals;
use crate::gpu::error::{HairGpuError, Result};
use crate::lod::{Blend, HairShading, LodPolicyError, PixelSpan, Representation, ShadowMapSize};

/// Effective inputs of the last view, excluding its temporal jitter and pose.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct RenderedView {
    pub globals: Globals,
    pub representation: Representation,
    pub blend: Option<Blend>,
    pub shadow: Option<Representation>,
    pub min_pixels: PixelSpan,
    /// The observation used to select a band when checking a new policy.
    pub pixels: PixelSpan,
    pub shading: HairShading,
    pub shadow_resolution: Option<ShadowMapSize>,
}

/// A frame contains a non-finite numerical input.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("hair render input {part:?} contains a non-finite value")]
pub struct RenderInputError {
    pub part: RenderInputPart,
}

#[derive(Clone, Copy)]
pub(super) enum History {
    First,
    Second,
}

/// The effective frame input that could not be compared or uploaded safely.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderInputPart {
    Uniforms,
    HeadPixelHeight,
    MinimumStrandPixels,
    StrandWidth,
    BlendAmount,
}

impl Globals {
    pub(super) fn validate(self) -> Result<Self> {
        // This POD consists solely of f32 shader uniforms and explicit
        // padding. Validate the actual upload, including the moving pose,
        // before projecting its identity into a temporal cache key.
        let uniforms: &[f32] = bytemuck::cast_slice(std::slice::from_ref(&self));
        if !uniforms.iter().all(|value| value.is_finite()) {
            return Err(RenderInputError {
                part: RenderInputPart::Uniforms,
            }
            .into());
        }
        Ok(self)
    }
}

impl RenderedView {
    pub fn validate(self) -> Result<Self> {
        let part = if !self.pixels.pixels().is_finite() {
            Some(RenderInputPart::HeadPixelHeight)
        } else if !self.min_pixels.pixels().is_finite() {
            Some(RenderInputPart::MinimumStrandPixels)
        } else if !finite_width(self.representation)
            || self
                .shadow
                .is_some_and(|representation| !finite_width(representation))
            || self.blend.is_some_and(|blend| !finite_width(blend.to))
        {
            Some(RenderInputPart::StrandWidth)
        } else if self
            .blend
            .is_some_and(|blend| !blend.amount.ratio().is_finite())
        {
            Some(RenderInputPart::BlendAmount)
        } else {
            None
        };
        if let Some(part) = part {
            return Err(RenderInputError { part }.into());
        }
        Ok(self)
    }
}

impl History {
    pub fn next(self) -> Self {
        match self {
            Self::First => Self::Second,
            Self::Second => Self::First,
        }
    }

    /// Index the native pair of temporal targets and bind groups.
    pub fn index(self) -> usize {
        match self {
            Self::First => 0,
            Self::Second => 1,
        }
    }
}

impl From<LodPolicyError> for HairGpuError {
    fn from(cause: LodPolicyError) -> Self {
        Self::Backend(cause.into())
    }
}

impl From<RenderInputError> for HairGpuError {
    fn from(cause: RenderInputError) -> Self {
        Self::Backend(cause.into())
    }
}

fn finite_width(representation: Representation) -> bool {
    match representation {
        Representation::Strands { width_scale, .. } => width_scale.ratio().is_finite(),
        Representation::Cards { .. } | Representation::Empty => true,
    }
}
