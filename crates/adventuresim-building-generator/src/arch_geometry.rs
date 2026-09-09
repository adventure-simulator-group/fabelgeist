//! Shared resolved arch sections for visible masonry and static collision.
use bevy::math::Vec3;

use crate::{ResolvedSolid, ResolvedSolidShape, SolidRole, WallAssembly};

const ARCH_SEGMENTS: usize = 24;

/// One convex section, ordered along its local front face.
pub(crate) struct ArchStrip {
    pub front: [Vec3; 4],
    pub depth: Vec3,
}

pub(crate) struct ArchGeometry {
    centre: Vec3,
    tangent: Vec3,
    outward: Vec3,
    width: f32,
    height: f32,
    depth: f32,
    clear_span: f32,
    spring: f32,
    rise: f32,
    radius: Option<f32>,
    panel: bool,
}

impl ArchGeometry {
    pub(crate) fn from_solid(solid: &ResolvedSolid, wall: Option<&WallAssembly>) -> Option<Self> {
        let (clear_span, spring, rise, radius) = match solid.shape {
            ResolvedSolidShape::PointedArchRing {
                clear_span_metres,
                spring_height_metres,
                apex_height_metres,
                arc_radius_metres,
                ..
            } => (
                clear_span_metres,
                spring_height_metres,
                apex_height_metres - spring_height_metres,
                Some(arc_radius_metres),
            ),
            ResolvedSolidShape::SegmentalArchRing {
                clear_span_metres,
                spring_height_metres,
                rise_metres,
                ..
            } => (clear_span_metres, spring_height_metres, rise_metres, None),
            _ => return None,
        };
        let tangent = wall
            .map_or_else(
                || {
                    if solid.size.z > solid.size.x {
                        Vec3::Z
                    } else {
                        Vec3::X
                    }
                },
                |wall| Vec3::new(wall.frame.tangent.x, 0.0, wall.frame.tangent.y),
            )
            .normalize();
        let outward = tangent.cross(Vec3::Y);
        Some(Self {
            centre: solid.centre,
            tangent,
            outward,
            width: solid.size.dot(tangent.abs()),
            height: solid.size.y,
            depth: solid.size.dot(outward.abs()),
            clear_span,
            spring,
            rise,
            radius,
            panel: matches!(
                solid.role,
                SolidRole::OpeningClosure | SolidRole::LeadedGlazing
            ),
        })
    }

    fn crown(&self, x: f32) -> f32 {
        let half = self.clear_span * 0.5;
        if x.abs() >= half {
            return 0.0;
        }
        if let Some(radius) = self.radius {
            let offset = radius - half;
            (radius * radius - (x.abs() + offset).powi(2))
                .max(0.0)
                .sqrt()
        } else {
            let radius = self.clear_span * self.clear_span / (8.0 * self.rise) + self.rise * 0.5;
            ((radius * radius - x * x).max(0.0).sqrt() + self.rise - radius).max(0.0)
        }
    }

    pub(crate) fn strips(&self) -> Vec<ArchStrip> {
        let half = self.width * 0.5;
        let mouth = (self.clear_span * 0.5).min(half);
        let mut coordinates = vec![-half];
        coordinates.extend(
            (0..=ARCH_SEGMENTS)
                .map(|index| -mouth + 2.0 * mouth * index as f32 / ARCH_SEGMENTS as f32),
        );
        coordinates.push(half);
        coordinates.dedup_by(|a, b| (*a - *b).abs() < f32::EPSILON);
        coordinates
            .windows(2)
            .map(|pair| {
                let [x0, x1] = [pair[0], pair[1]];
                let bottom = -self.height * 0.5;
                let top = self.height * 0.5;
                let crown = |x| {
                    (bottom + self.crown(x) + if self.panel { self.spring } else { 0.0 }).min(top)
                };
                let (lower0, lower1, upper0, upper1) = if self.panel {
                    (bottom, bottom, crown(x0), crown(x1))
                } else {
                    (crown(x0), crown(x1), top, top)
                };
                let point = |x, y| {
                    self.centre + self.tangent * x + Vec3::Y * y + self.outward * self.depth * 0.5
                };
                ArchStrip {
                    front: [
                        point(x0, lower0),
                        point(x1, lower1),
                        point(x1, upper1),
                        point(x0, upper0),
                    ],
                    depth: -self.outward * self.depth,
                }
            })
            .collect()
    }
}
