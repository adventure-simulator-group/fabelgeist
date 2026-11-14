//! Shared low-poly hook solids, with paired eyes or a separate ring target.
use crate::cloth_closure_error::{ClothClosureError, Result};
use crate::cloth_fastener_mesh::{
    ClosureFrame, ClosureMaterial, CordEnds, CordPath, FastenerFrame, FastenerMesh,
};
use crate::cloth_opening::ClosureSide;
use fabelgeist_armor::{ArmorComponentMaterial, ClothFastener};
use fabelgeist_math::Vec3;
use std::f32::consts::{PI, TAU};
pub(crate) const HOOK_TAIL_SETBACK_SHARE: f32 = 0.6;
const EYE_CENTER_REACH_SHARE: f32 = 0.35;
const EYE_RADIUS_REACH_SHARE: f32 = 0.5;
/// Nominal planar sewing extents from the eye template's origin, in metres.
pub(crate) struct EyeSewingFootprint {
    pub forward: f32,
    pub back: f32,
}
/// Native hook template dimensions in metres.
#[derive(Clone, Copy)]
pub(crate) struct HookDimensions {
    pub reach: f32,
    pub width: f32,
    pub gauge: f32,
}
impl FastenerMesh {
    pub(crate) fn eye_sewing_footprint(design: &ClothFastener) -> Result<EyeSewingFootprint> {
        let ClothFastener::HookEye { length, wire, .. } = design else {
            return Err(ClothClosureError::EyeRequiresHook);
        };
        let radius = length.metres() * EYE_RADIUS_REACH_SHARE;
        let center = length.metres() * EYE_CENTER_REACH_SHARE;
        let gauge = wire.metres() * 0.5;
        Ok(EyeSewingFootprint {
            forward: center + radius + gauge,
            back: radius - center + gauge,
        })
    }

    pub(crate) fn hook_eye(
        &mut self,
        frame: &ClosureFrame,
        dimensions: HookDimensions,
        material: &ClosureMaterial,
    ) -> Result<()> {
        let reach = dimensions.reach;
        let width = dimensions.width;
        let gauge = dimensions.gauge;
        if frame.gap.partial_cmp(&reach) != Some(std::cmp::Ordering::Less) {
            return Err(ClothClosureError::InsufficientHookReach {
                gap: fabelgeist_math::units::Metres::from_metres(frame.gap),
                reach: fabelgeist_math::units::Metres::from_metres(reach),
            });
        }
        let eye = (0..ClothFastener::HOOK_CURVE_SEGMENTS)
            .map(|i| {
                let t = TAU * i as f32 / ClothFastener::HOOK_CURVE_SEGMENTS as f32;
                Vec3::new(
                    frame.gap
                        + reach * EYE_CENTER_REACH_SHARE
                        + reach * EYE_RADIUS_REACH_SHARE * t.cos(),
                    width * 0.5 * t.sin(),
                    gauge,
                )
            })
            .collect::<Vec<_>>();
        self.cord(
            frame,
            &CordPath {
                points: eye,
                normal: Vec3::forward(),
                ends: CordEnds::Joined,
            },
            gauge * 0.5,
            ClosureSide::Right,
            material,
        );
        self.hook(frame, dimensions, material)?;
        Ok(())
    }

    fn hook(
        &mut self,
        frame: &ClosureFrame,
        dimensions: HookDimensions,
        material: &ClosureMaterial,
    ) -> Result<()> {
        let HookDimensions {
            reach,
            width,
            gauge,
        } = dimensions;
        if frame.gap.partial_cmp(&reach) != Some(std::cmp::Ordering::Less) {
            return Err(ClothClosureError::InsufficientHookReach {
                gap: fabelgeist_math::units::Metres::from_metres(frame.gap),
                reach: fabelgeist_math::units::Metres::from_metres(reach),
            });
        }
        let mut hook = vec![Vec3::new(-reach * HOOK_TAIL_SETBACK_SHARE, 0., gauge)];
        hook.extend((0..ClothFastener::HOOK_CURVE_SEGMENTS).map(|i| {
            let t = PI + PI * 1.5 * i as f32 / (ClothFastener::HOOK_CURVE_SEGMENTS - 1) as f32;
            Vec3::new(
                frame.gap + width * 0.2 + width * 0.4 * t.cos(),
                0.,
                gauge + width * 0.4 * (1. + t.sin()),
            )
        }));
        self.cord(
            frame,
            &CordPath {
                points: hook,
                normal: Vec3::up(),
                ends: CordEnds::Capped,
            },
            gauge * 0.5,
            ClosureSide::Left,
            material,
        );
        Ok(())
    }
    pub(crate) fn single_hook(design: &ClothFastener, frame: FastenerFrame) -> Result<Self> {
        let ClothFastener::HookEye {
            length,
            width,
            wire,
            color,
            roughness,
        } = design
        else {
            return Err(ClothClosureError::ReturnRequiresHook);
        };
        let frame = ClosureFrame::new(frame)?;
        let material = ClosureMaterial {
            surface: ArmorComponentMaterial {
                base_color: color.material().base_color,
                metallic: 1.,
                roughness: roughness.unit(),
            },
            fabric: None,
        };
        let mut mesh = Self::empty();
        mesh.hook(
            &frame,
            HookDimensions {
                reach: length.metres(),
                width: width.metres(),
                gauge: wire.metres(),
            },
            &material,
        )?;
        Ok(mesh)
    }
}
