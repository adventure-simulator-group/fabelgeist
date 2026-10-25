//! Preflight all scaled capsule dimensions before any physics entities are queued.

use super::{RagdollBodyPart, definition::RagdollDefinition, entry::BodySeed};
use adventuresim_building_generator::spatial_geometry::{
    GeometryResult, PositiveLength, SignedLength,
};
use adventuresim_tactical_core::animation::NonnegativeLength;
use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct CapsuleShape {
    pub(super) radius: PositiveLength,
    /// Bind-space cylinder length between the hemispherical caps.
    pub(super) length: PositiveLength,
}

pub(super) struct ScaledCapsule {
    pub(super) part: RagdollBodyPart,
    pub(super) segment: SignedLength,
    pub(super) collider_local: Transform,
}

pub(super) struct RagdollGeometry {
    pub(super) capsules: Vec<Option<ScaledCapsule>>,
    pub(super) margin: NonnegativeLength,
}

impl RagdollGeometry {
    pub(super) fn prepare(
        definition: &RagdollDefinition,
        seeds: &[Option<BodySeed>],
        scale: f32,
        margin: NonnegativeLength,
    ) -> GeometryResult<Self> {
        let capsules = definition
            .bodies
            .iter()
            .zip(seeds)
            .map(|(body, seed)| {
                if seed.is_none() {
                    return Ok(None);
                }
                // Avian forwards radius to Parry's positive-radius shape contract.
                // Signed segment length instead swaps its two endpoints; preserve it.
                let radius = PositiveLength::from_metres(body.shape.radius.metres() * scale)?;
                let segment = SignedLength::from_metres(body.shape.length.metres() * scale)?;
                let half_length = SignedLength::from_metres(segment.metres() * 0.5)?;
                let collider_local =
                    Transform::from_translation(body.collider_local.translation * scale)
                        .with_rotation(body.collider_local.rotation);
                Ok(Some(ScaledCapsule {
                    part: RagdollBodyPart {
                        radius,
                        half_length,
                        center_local: collider_local.translation,
                        axis_local: body.collider_local.rotation * Vec3::Y,
                        kinematic: body.kinematic,
                    },
                    segment,
                    collider_local,
                }))
            })
            .collect::<GeometryResult<Vec<_>>>()?;
        Ok(Self { capsules, margin })
    }
}
