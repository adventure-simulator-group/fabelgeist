//! Independent skull, jaw and neck sections for a close helmet carrier.
use serde::{Deserialize, Serialize};

use super::{CloseHelmetDesign, HelmetDesign};
use crate::{DesignError, GenerateError, PartFrame, PartMesh};

const SUBMENTAL_NECK_FRACTION: f32 = 0.35;

#[path = "helmets_close_sections.rs"]
mod sections;

/// Outer sectional bounds in metres in the head frame, including the chosen
/// padding and plate reserve. Anatomy sets these bounds; style sets projection,
/// ridge, lip and openings. No output vertex is projected onto body topology.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CloseHelmetProfile {
    pub skull_half_width: f32,
    pub temple_half_width: f32,
    pub skull_front: f32,
    pub skull_back: f32,
    pub jaw_half_width: f32,
    pub jaw_front: f32,
    pub submental_front: f32,
    pub neck_half_width: f32,
    pub throat_front: f32,
    pub nape_back: f32,
    pub nape_waist: f32,
    neck_enclosure: sections::NeckEnclosure,
}

impl CloseHelmetProfile {
    /// Preserve the measured section shape while seating its intermediate
    /// rows outside the local support. Only the necessary width and posterior
    /// reserves are added; the cranial bowl and terminal section stay separate.
    pub fn enclose_neck(
        &mut self,
        design: &CloseHelmetDesign,
        frame: &PartFrame,
        positions: &[[f32; 3]],
        faces: &[[u32; 3]],
        margin: crate::Millimeters,
    ) -> Result<(), GenerateError> {
        frame.validate()?;
        self.validate()?;
        self.neck_enclosure =
            sections::NeckEnclosure::fit(self, design, frame, positions, faces, margin)?;
        self.validate()?;
        Ok(())
    }
    /// Fit neck stations to triangle/plane intersections. Moving stations use
    /// the actual skin and plate surfaces, independent of vertex-band density.
    pub fn from_surface(
        design: &CloseHelmetDesign,
        frame: &PartFrame,
        head_samples: &[[f32; 3]],
        positions: &[[f32; 3]],
        faces: &[[u32; 3]],
        section_margin: crate::Millimeters,
    ) -> Result<Self, GenerateError> {
        let points = local_samples(frame, head_samples);
        let head = HeadProfile::measure(design, frame, &points)?;
        let neck = sections::measure(design, frame, positions, faces, section_margin)?;
        let profile = head.with_neck(neck);
        profile.validate()?;
        Ok(profile)
    }

    /// Fit point-cloud previews using broad anatomical vertex bands. Production
    /// surfaces use `from_surface` so neck fits are independent of vertex rows.
    pub fn from_samples(
        design: &CloseHelmetDesign,
        frame: &PartFrame,
        samples: &[[f32; 3]],
    ) -> Result<Self, GenerateError> {
        let points = local_samples(frame, samples);
        let head = HeadProfile::measure(design, frame, &points)?;
        let neck = sections::from_samples(design, frame, &points)?;
        let profile = head.with_neck(neck);
        profile.validate()?;
        Ok(profile)
    }

    /// Nominal sections for standalone recipe previews. Production fitting can
    /// supply independently measured sections through `generate_close_helmet`.
    pub(super) fn authored(radii: [f32; 3], design: &CloseHelmetDesign) -> Self {
        let reserve = design.face_clearance.metres() - design.fit.clearance.metres();
        Self {
            neck_enclosure: Default::default(),
            skull_half_width: radii[0],
            temple_half_width: radii[0],
            skull_front: radii[2] * 0.84,
            skull_back: -radii[2],
            jaw_half_width: radii[0] * 0.73 + reserve,
            jaw_front: radii[2] * 0.84 + reserve,
            submental_front: radii[2] * 0.52 + reserve,
            neck_half_width: radii[0] * 0.67 + reserve,
            throat_front: radii[2] * 0.43 + reserve,
            nape_back: -radii[2] * 0.80 - reserve,
            nape_waist: -radii[2] * 0.76 - reserve,
        }
    }

    /// Check sectional ordering and finite positive enclosure dimensions.
    pub fn validate(self) -> Result<(), DesignError> {
        let widths = [
            self.skull_half_width,
            self.temple_half_width,
            self.jaw_half_width,
            self.neck_half_width,
        ];
        let depths = [
            self.skull_front,
            self.skull_back,
            self.jaw_front,
            self.submental_front,
            self.throat_front,
            self.nape_back,
            self.nape_waist,
        ];
        if !self.neck_enclosure.is_valid()
            || widths.iter().any(|v| !v.is_finite() || *v <= 0.0)
            || depths.iter().any(|v| !v.is_finite())
            || self.skull_front <= self.skull_back
            || self.throat_front <= self.nape_back
            || self.jaw_front <= self.throat_front
        {
            return Err(DesignError::ParametricParameters);
        }
        Ok(())
    }

    pub(super) fn skull_center(self) -> f32 {
        (self.skull_front + self.skull_back) * 0.5
    }

    pub(super) fn skull_depth(self) -> f32 {
        (self.skull_front - self.skull_back) * 0.5
    }

    pub(super) fn section(
        self,
        design: &CloseHelmetDesign,
        angle: f32,
        jaw_blend: f32,
        neck_fraction: f32,
        back_blend: f32,
    ) -> [f32; 2] {
        let neck_blend = neck_fraction * neck_fraction * (3.0 - 2.0 * neck_fraction);
        let upper_width = self.temple_half_width;
        let jaw_width = self
            .jaw_half_width
            .max(upper_width * design.jaw_width.unit());
        let neck_width = self
            .neck_half_width
            .max(upper_width * design.neck_width.unit());
        let width = lerp(
            lerp(upper_width, jaw_width, jaw_blend),
            neck_width,
            neck_blend,
        );
        let reserve = self.neck_enclosure.at(neck_fraction);
        let width = width + reserve.half_width * jaw_blend;
        let front = if neck_fraction <= SUBMENTAL_NECK_FRACTION {
            lerp(
                lerp(self.skull_front, self.jaw_front, jaw_blend),
                self.submental_front,
                neck_fraction / SUBMENTAL_NECK_FRACTION,
            )
        } else {
            lerp(
                self.submental_front,
                self.throat_front,
                (neck_fraction - SUBMENTAL_NECK_FRACTION) / (1.0 - SUBMENTAL_NECK_FRACTION),
            )
        };
        let back = lerp(
            lerp(self.skull_back, self.nape_waist, back_blend),
            self.nape_back,
            neck_blend,
        ) - reserve.back_depth * jaw_blend;
        let center = (front + back) * 0.5;
        [
            width * angle.sin(),
            center + (front - back) * 0.5 * angle.cos(),
        ]
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

struct HeadProfile {
    skull_half_width: f32,
    temple_half_width: f32,
    skull_front: f32,
    skull_back: f32,
    jaw_half_width: f32,
    jaw_front: f32,
}

impl HeadProfile {
    fn measure(
        design: &CloseHelmetDesign,
        frame: &PartFrame,
        points: &[[f32; 3]],
    ) -> Result<Self, GenerateError> {
        HelmetDesign::CloseHelmet(*design).validate()?;
        frame.validate()?;
        const JAW_WIDTH_UPPER_M: f32 = 0.020;
        const JAW_FACE_HEIGHT_M: f32 = 0.018;
        let chin = -frame.half_extents[1];
        let skull = Bounds::measure(
            points,
            frame.half_extents[1] * super::shapes::BROW_HEIGHT,
            f32::INFINITY,
        )?;
        let head_width = Bounds::measure(points, chin, f32::INFINITY)?;
        let jaw = Bounds::measure(points, chin, chin + JAW_WIDTH_UPPER_M)?;
        let face = Bounds::measure(points, chin, chin + JAW_FACE_HEIGHT_M)?;
        let gap = design.fit.clearance.metres() + design.fit.wall_thickness.metres();
        let face_gap = design.face_clearance.metres() + design.fit.wall_thickness.metres();
        Ok(Self {
            skull_half_width: skull.half_width() + gap,
            temple_half_width: (skull.half_width() + gap).max(
                head_width.half_width()
                    + design.fit.wall_thickness.metres()
                    + design.temple_clearance.metres(),
            ),
            skull_front: skull.high[2] + gap,
            skull_back: skull.low[2] - gap,
            jaw_half_width: jaw.half_width() + face_gap,
            jaw_front: face.high[2] + face_gap,
        })
    }

    fn with_neck(self, neck: sections::NeckProfile) -> CloseHelmetProfile {
        CloseHelmetProfile {
            neck_enclosure: Default::default(),
            skull_half_width: self.skull_half_width,
            temple_half_width: self.temple_half_width,
            skull_front: self.skull_front,
            skull_back: self.skull_back,
            jaw_half_width: self.jaw_half_width,
            jaw_front: self.jaw_front,
            submental_front: neck.submental_front,
            neck_half_width: neck.half_width,
            throat_front: neck.throat_front,
            nape_back: neck.nape_back,
            nape_waist: neck.nape_waist,
        }
    }
}

fn local_samples(frame: &PartFrame, samples: &[[f32; 3]]) -> Vec<[f32; 3]> {
    samples
        .iter()
        .map(|point| {
            frame
                .axes
                .map(|axis| (0..3).map(|i| axis[i] * (point[i] - frame.origin[i])).sum())
        })
        .collect()
}

struct Bounds {
    low: [f32; 3],
    high: [f32; 3],
}

impl Bounds {
    fn measure(points: &[[f32; 3]], bottom: f32, top: f32) -> Result<Self, DesignError> {
        const MINIMUM_SECTION_SAMPLES: usize = 4;
        let mut result = Self {
            low: [f32::INFINITY; 3],
            high: [f32::NEG_INFINITY; 3],
        };
        let mut count = 0;
        for p in points.iter().filter(|p| p[1] >= bottom && p[1] <= top) {
            if p.iter().any(|v| !v.is_finite()) {
                return Err(DesignError::ParametricParameters);
            }
            count += 1;
            for (axis, v) in p.iter().enumerate() {
                result.low[axis] = result.low[axis].min(*v);
                result.high[axis] = result.high[axis].max(*v);
            }
        }
        if count < MINIMUM_SECTION_SAMPLES {
            return Err(DesignError::ParametricParameters);
        }
        Ok(result)
    }
    fn half_width(&self) -> f32 {
        self.low[0].abs().max(self.high[0].abs())
    }
}

pub fn generate_close_helmet(
    design: &CloseHelmetDesign,
    frame: &PartFrame,
    profile: &CloseHelmetProfile,
) -> Result<PartMesh, GenerateError> {
    HelmetDesign::CloseHelmet(*design).validate()?;
    frame.validate()?;
    profile.validate()?;
    let gap = design.fit.clearance.metres() + design.fit.wall_thickness.metres();
    let crown = frame.half_extents[1] * design.fit.crown_height.unit() + gap;
    Ok(
        super::close::generate_fitted(crown, frame.half_extents[1], design, profile)?
            .transformed(frame),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn anatomy() -> (PartFrame, Vec<[f32; 3]>) {
        let frame = PartFrame {
            origin: [0.0; 3],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            half_extents: [0.085, 0.115, 0.105],
        };
        let mut points = Vec::new();
        for (y, width, front, back) in [
            (0.03, 0.08, 0.09, -0.10),
            (0.08, 0.06, 0.06, -0.08),
            (-0.02, 0.09, 0.13, -0.10),
            (-0.09, 0.06, 0.095, -0.08),
            (-0.105, 0.06, 0.095, -0.08),
            (-0.114, 0.055, 0.07, -0.075),
            (-0.125, 0.05, 0.045, -0.071),
            (-0.138, 0.05, 0.04, -0.07),
        ] {
            points.extend([
                [-width, y, 0.0],
                [width, y, 0.0],
                [0.0, y, front],
                [0.0, y, back],
            ]);
        }
        (frame, points)
    }

    #[test]
    fn nose_projection_does_not_inflate_the_skull_and_neck_changes_keep_topology() {
        let d = CloseHelmetDesign::default();
        let (frame, mut points) = anatomy();
        let a = CloseHelmetProfile::from_samples(&d, &frame, &points).unwrap();
        points.push([0.0, -0.02, 0.30]);
        let b = CloseHelmetProfile::from_samples(&d, &frame, &points).unwrap();
        assert_eq!(a.skull_front, b.skull_front);
        assert_eq!(a.skull_back, b.skull_back);
        assert_eq!(a.jaw_front, b.jaw_front);
        for p in points.iter_mut().filter(|p| p[1] < -0.11) {
            p[0] *= 1.6;
        }
        let c = CloseHelmetProfile::from_samples(&d, &frame, &points).unwrap();
        assert!(c.neck_half_width > b.neck_half_width);
        let before = generate_close_helmet(&d, &frame, &b).unwrap();
        let after = generate_close_helmet(&d, &frame, &c).unwrap();
        assert_eq!(before.indices, after.indices);
        assert_eq!(before.components, after.components);
        assert!(
            before.positions != after.positions,
            "wider anatomy must expand the neck enclosure"
        );
    }

    #[test]
    fn unsupported_anatomy_and_plate_gauge_fail_explicitly() {
        let mut d = CloseHelmetDesign::default();
        let (mut frame, points) = anatomy();
        assert!(CloseHelmetProfile::from_samples(&d, &frame, &[]).is_err());
        d.fit.wall_thickness = crate::Millimeters(8);
        assert!(HelmetDesign::CloseHelmet(d).validate().is_err());
        d = CloseHelmetDesign::default();
        frame.axes[0] = [0.0; 3];
        assert!(CloseHelmetProfile::from_samples(&d, &frame, &points).is_err());
    }
}
