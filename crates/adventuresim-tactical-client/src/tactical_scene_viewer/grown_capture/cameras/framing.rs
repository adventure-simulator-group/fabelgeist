//! Original-source envelopes and a bounded supported-eye projection solve.

use super::*;
use adventuresim_environment::render::{
    ForestLodProjection, ProjectedRadiusPixels, source_bounds_radius,
};
use bevy::camera::primitives::Aabb;
use fabelgeist_math::units::Ratio;

const FRAME_MARGIN: Ratio = Ratio::from_ratio(1.15);
const BRACKET_STEPS: usize = 12;
const REFINE_STEPS: usize = 20;

/// Native world-metre geometry boundary; never uses authored candidate bounds.
pub(super) struct SourceEnvelopes {
    roots: Vec<RootEnvelope>,
    low: Vec3,
    high: Vec3,
}

struct RootEnvelope {
    ordinal: layout::RootOrdinal,
    bounds: Aabb,
    radius: Metres,
}

#[derive(Serialize)]
pub(super) struct RootProjection {
    root: layout::RootOrdinal,
    radius_pixels: ProjectedRadiusPixels,
    from_full: Level,
    from_middle: Level,
    from_coarse: Level,
}

impl SourceEnvelopes {
    pub(super) fn new(bounds: TreeBounds, placements: &layout::RootPlacements) -> Result<Self> {
        let source_low = Vec3::from_array(bounds.low.map(Metres::metres));
        let source_high = Vec3::from_array(bounds.high.map(Metres::metres));
        ensure!(
            source_low.is_finite()
                && source_high.is_finite()
                && source_low.cmple(source_high).all(),
            "invalid source camera bounds"
        );
        let mut low = Vec3::splat(f32::INFINITY);
        let mut high = Vec3::splat(f32::NEG_INFINITY);
        let mut roots = Vec::new();
        for placement in placements.iter() {
            ensure!(
                placement.frame.rotation == Quat::IDENTITY && placement.frame.scale == Vec3::ONE,
                "camera witness requires translated source roots"
            );
            let root_low = source_low + placement.frame.translation;
            let root_high = source_high + placement.frame.translation;
            low = low.min(root_low);
            high = high.max(root_high);
            roots.push(RootEnvelope {
                ordinal: placement.ordinal,
                bounds: Aabb::from_min_max(root_low, root_high),
                radius: source_bounds_radius(root_low, root_high),
            });
        }
        ensure!(!roots.is_empty(), "source camera roots missing");
        Ok(Self { roots, low, high })
    }

    pub(super) fn close(&self, support: &support::Reader<'_>) -> Result<PlateCamera> {
        self.at(
            crown_distance(self.low, self.high, CROWN_EYE_DIRECTION.normalize())?,
            support,
        )
    }

    pub(super) fn solve(
        &self,
        target: ProjectedRadiusPixels,
        projection: ForestLodProjection,
        support: &support::Reader<'_>,
    ) -> Result<PlateCamera> {
        let mut near = crown_distance(self.low, self.high, CROWN_EYE_DIRECTION.normalize())?;
        let close = self.at(near, support)?;
        ensure!(
            self.largest(close.position, projection)? >= target,
            "source-fit camera already below target proxy"
        );
        let mut far = near;
        let mut bracketed = false;
        for _ in 0..BRACKET_STEPS {
            far = Metres::from_metres(far.metres() * 2.0);
            if self.largest(self.at(far, support)?.position, projection)? <= target {
                bracketed = true;
                break;
            }
        }
        ensure!(
            bracketed,
            "supported source ray cannot bracket requested camera proxy"
        );
        for _ in 0..REFINE_STEPS {
            let distance = Metres::from_metres((near.metres() + far.metres()) * 0.5);
            if self.largest(self.at(distance, support)?.position, projection)? > target {
                near = distance;
            } else {
                far = distance;
            }
        }
        self.at(far, support)
    }

    pub(super) fn report(
        &self,
        eye: Vec3,
        projection: ForestLodProjection,
        detail: &adventuresim_environment::render::ForestDetail,
    ) -> Result<Vec<RootProjection>> {
        self.roots
            .iter()
            .map(|root| {
                let pixels = projection
                    .projected_radius(eye, &root.bounds, root.radius)
                    .context("invalid per-root camera projection")?;
                Ok(RootProjection {
                    root: root.ordinal,
                    radius_pixels: pixels,
                    from_full: Level::for_projected_radius(pixels, Level::Full, detail),
                    from_middle: Level::for_projected_radius(pixels, Level::Middle, detail),
                    from_coarse: Level::for_projected_radius(pixels, Level::Coarse, detail),
                })
            })
            .collect()
    }

    fn at(&self, distance: Metres, support: &support::Reader<'_>) -> Result<PlateCamera> {
        let centre = (self.low + self.high) * 0.5;
        Ok(PlateCamera {
            position: support
                .clamp_eye(centre + CROWN_EYE_DIRECTION.normalize() * distance.metres())?,
            target: centre,
            up: Vec3::Y,
        })
    }

    fn largest(&self, eye: Vec3, projection: ForestLodProjection) -> Result<ProjectedRadiusPixels> {
        let mut largest = ProjectedRadiusPixels::from_pixels(0.0).context("zero pixel radius")?;
        for root in &self.roots {
            let pixels = projection
                .projected_radius(eye, &root.bounds, root.radius)
                .context("invalid source camera projection")?;
            if pixels > largest {
                largest = pixels;
            }
        }
        Ok(largest)
    }
}

// World-space original-source envelope metres enter a perspective-fit boundary.
// Every original source-box corner must fit both image axes with the margin.
fn crown_distance(low: Vec3, high: Vec3, direction: Vec3) -> Result<Metres> {
    let centre = (low + high) * 0.5;
    let right = Vec3::Y.cross(direction).normalize();
    let up = direction.cross(right);
    let vertical_scale = FRAME_MARGIN.ratio() / (FOV_DEGREES.to_radians() * 0.5).tan();
    let horizontal_scale =
        vertical_scale / super::super::super::capture_resolution::CAPTURE_ASPECT_RATIO;
    let mut distance = 0_f32;
    for x in [low.x, high.x] {
        for y in [low.y, high.y] {
            for z in [low.z, high.z] {
                let offset = Vec3::new(x, y, z) - centre;
                let transverse = (offset.dot(up).abs() * vertical_scale)
                    .max(offset.dot(right).abs() * horizontal_scale);
                distance = distance.max(offset.dot(direction) + transverse);
            }
        }
    }
    ensure!(
        distance.is_finite() && distance > 0.0,
        "invalid grown source camera envelope"
    );
    Ok(Metres::from_metres(distance))
}
