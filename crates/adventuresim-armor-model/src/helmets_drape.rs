//! Smooth longitudinal and transverse cloth profiles in the head frame.

use serde::{Deserialize, Serialize};

use super::{CoifDesign, HelmetDesign};
use crate::{DesignError, GenerateError, PartFrame, PartMesh};

pub const COIF_DRAPE_SECTIONS: usize = 5;
pub(super) const FLAP_EDGE: usize = super::geometry::AROUND / 12;

fn flap_half_angle() -> f32 {
    FLAP_EDGE as f32 / super::geometry::AROUND as f32 * std::f32::consts::TAU
}

/// The lower neck boundary follows the neck/shoulder junction. Heights and
/// depths are physical metres in the head frame, not head-size multipliers.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CoifNeckDrape {
    pub front_height: f32,
    pub side_height: f32,
    pub back_height: f32,
    pub half_width: f32,
    pub center_depth: f32,
    pub front_depth: f32,
    pub back_depth: f32,
}

impl CoifNeckDrape {
    pub(super) fn point(self, angle: f32) -> [f32; 3] {
        let cosine = angle.cos();
        let (end_height, end_depth) = if cosine >= 0.0 {
            (self.front_height, self.front_depth)
        } else {
            (self.back_height, self.back_depth)
        };
        [
            self.half_width * angle.sin(),
            self.side_height + (end_height - self.side_height) * cosine.powi(2),
            self.center_depth + (end_depth - self.center_depth) * cosine.abs().sqrt(),
        ]
    }
}

/// One horizontal body-envelope section, in physical metres in the head frame.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CoifDrapeSection {
    pub height: f32,
    pub center_depth: f32,
    pub edge_depth: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CoifFlapDrape {
    /// Ordered from the lowest hem toward the neck attachment.
    pub sections: [CoifDrapeSection; COIF_DRAPE_SECTIONS],
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CoifDrapeProfile {
    pub neck: CoifNeckDrape,
    pub front: CoifFlapDrape,
    pub back: CoifFlapDrape,
}

impl CoifDrapeProfile {
    /// A head-proportioned drape for isolated design previews. Worn equipment
    /// replaces these section depths with its wearer's chest and back envelope.
    pub fn from_head(design: &CoifDesign, frame: &PartFrame) -> Self {
        let h = frame.half_extents[1];
        let r = frame.half_extents[2]
            + design.fit.clearance.metres()
            + design.fit.wall_thickness.metres();
        let width = frame.half_extents[0]
            + design.fit.clearance.metres()
            + design.fit.wall_thickness.metres();
        Self::authored(design, h, [width, 0.0, r])
    }

    pub(super) fn authored(design: &CoifDesign, h: f32, radii: [f32; 3]) -> Self {
        let front_height = -h * (1.0 + 0.65 * design.neck_coverage.unit());
        Self::with_neck(
            design,
            CoifNeckDrape {
                front_height,
                side_height: front_height + h * 0.34,
                back_height: front_height + h * 0.22,
                half_width: radii[0] * 1.50,
                center_depth: -radii[2] * 0.36,
                front_depth: radii[2] * 0.38,
                back_depth: -radii[2] * 1.32,
            },
        )
    }

    pub fn with_neck(design: &CoifDesign, neck: CoifNeckDrape) -> Self {
        let flap = |front: bool| {
            let (top, bottom, length) = if front {
                (
                    neck.point(flap_half_angle())[1],
                    neck.front_height,
                    design.front_flap_length.metres(),
                )
            } else {
                (
                    neck.point(std::f32::consts::PI - flap_half_angle())[1],
                    neck.back_height,
                    design.back_flap_length.metres(),
                )
            };
            CoifFlapDrape {
                sections: std::array::from_fn(|i| {
                    let t = i as f32 / (COIF_DRAPE_SECTIONS - 1) as f32;
                    let z = if front {
                        neck.front_depth + length * (1.0 - t) * 0.25
                    } else {
                        neck.back_depth - length * (1.0 - t) * 0.25
                    };
                    CoifDrapeSection {
                        height: bottom - length + (top - bottom + length) * t,
                        center_depth: z,
                        edge_depth: z,
                    }
                }),
            }
        };
        Self {
            neck,
            front: flap(true),
            back: flap(false),
        }
    }

    pub fn flap_half_width(&self, design: &CoifDesign) -> f32 {
        self.neck.half_width * flap_half_angle().sin() * design.flap_width.unit()
    }

    pub fn validate(&self) -> Result<(), GenerateError> {
        let n = self.neck;
        if ![
            n.front_height,
            n.side_height,
            n.back_height,
            n.half_width,
            n.center_depth,
            n.front_depth,
            n.back_depth,
        ]
        .iter()
        .all(|v| v.is_finite())
            || n.half_width <= 0.0
            || n.front_depth <= n.center_depth
            || n.back_depth >= n.center_depth
        {
            return Err(GenerateError::Design(DesignError::ParametricParameters));
        }
        for flap in [&self.front, &self.back] {
            if flap.sections.iter().any(|s| {
                !s.height.is_finite() || !s.center_depth.is_finite() || !s.edge_depth.is_finite()
            }) || flap.sections.windows(2).any(|s| s[0].height >= s[1].height)
            {
                return Err(GenerateError::Design(DesignError::ParametricParameters));
            }
        }
        Ok(())
    }
}

impl CoifFlapDrape {
    pub(super) fn depth(&self, height: f32, across: f32) -> f32 {
        let center = self.interpolate(height, |s| s.center_depth);
        let edge = self.interpolate(height, |s| s.edge_depth);
        center + (edge - center) * across.powi(2)
    }

    fn interpolate(&self, height: f32, depth: impl Fn(CoifDrapeSection) -> f32) -> f32 {
        let last = COIF_DRAPE_SECTIONS - 1;
        if height <= self.sections[0].height {
            return depth(self.sections[0]);
        }
        if height >= self.sections[last].height {
            return depth(self.sections[last]);
        }
        let i = (0..last)
            .find(|&i| height <= self.sections[i + 1].height)
            .unwrap();
        let a = self.sections[i];
        let b = self.sections[i + 1];
        let span = b.height - a.height;
        let t = (height - a.height) / span;
        let slope = |j: usize| {
            let low = j.saturating_sub(1);
            let high = (j + 1).min(last);
            (depth(self.sections[high]) - depth(self.sections[low]))
                / (self.sections[high].height - self.sections[low].height)
        };
        // Cubic Hermite interpolation gives a continuous tangent across the
        // anatomical sections; unlike axial extrusion it follows both profiles.
        (2.0 * t.powi(3) - 3.0 * t.powi(2) + 1.0) * depth(a)
            + (t.powi(3) - 2.0 * t.powi(2) + t) * span * slope(i)
            + (-2.0 * t.powi(3) + 3.0 * t.powi(2)) * depth(b)
            + (t.powi(3) - t.powi(2)) * span * slope(i + 1)
    }
}

pub fn generate_coif_with_drape(
    design: &CoifDesign,
    frame: &PartFrame,
    drape: &CoifDrapeProfile,
) -> Result<PartMesh, GenerateError> {
    HelmetDesign::MailCoif(*design).validate()?;
    frame.validate()?;
    drape.validate()?;
    let gap = design.fit.clearance.metres() + design.fit.wall_thickness.metres();
    let head = frame.half_extents;
    let radii = [
        head[0] + gap,
        head[1] * design.fit.crown_height.unit() + gap,
        head[2] + gap,
    ];
    Ok(super::coif::generate_fitted(
        radii,
        head[1] * super::shapes::BROW_HEIGHT,
        head[1],
        design,
        drape,
    )?
    .transformed(frame))
}
