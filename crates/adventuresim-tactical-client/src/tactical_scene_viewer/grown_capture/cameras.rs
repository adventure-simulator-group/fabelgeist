//! Camera from the union of original source envelopes at actual grounded roots.

use super::*;
use adventuresim_tactical_core::prelude::SceneTerrain;
use fabelgeist_math::units::{Metres, Ratio};

const CAMERA_ROTATION_EPSILON: f32 = 1.0e-5;
const FOV_DEGREES: f32 = 60.0;
const FRAME_MARGIN: Ratio = Ratio::from_ratio(1.15);
const CROWN_EYE_DIRECTION: Vec3 = Vec3::new(1.0, 0.9, 1.0);
const SUPPORT_EYE: Metres = Metres::from_metres(super::super::STANDING_EYE_HEIGHT_METRES);
const FOREGROUND_BPS: u16 = 500;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub(in super::super) struct PlateCamera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
}

#[derive(Serialize)]
pub(super) struct Cameras {
    shared: PlateCamera,
}

#[derive(Resource, Serialize)]
pub(in super::super) struct PerspectiveDepth {
    near: Metres,
    far: Metres,
}

impl Cameras {
    /// Native geometry boundary: source-local metre bounds plus world root frames.
    pub(super) fn from_source(
        bounds: TreeBounds,
        placements: &layout::RootPlacements,
        terrain: &SceneTerrain,
    ) -> Result<Self> {
        let source_low = Vec3::from_array(bounds.low.map(Metres::metres));
        let source_high = Vec3::from_array(bounds.high.map(Metres::metres));
        let mut low = Vec3::splat(f32::INFINITY);
        let mut high = Vec3::splat(f32::NEG_INFINITY);
        for placement in placements.iter() {
            low = low.min(source_low + placement.frame.translation);
            high = high.max(source_high + placement.frame.translation);
        }
        let centre = (low + high) * 0.5;
        let direction = CROWN_EYE_DIRECTION.normalize();
        let distance = crown_distance(low, high, direction)?;
        Ok(Self {
            shared: PlateCamera {
                position: crown_eye(terrain, centre + direction * distance.metres())?,
                target: centre,
                up: Vec3::Y,
            },
        })
    }

    pub(super) fn views(&self) -> Vec<CaptureViewSpec> {
        vec![
            view("warmup", "Grown shared pipeline warmup", self.shared).warmup(),
            view("grown-shared", "Three shared grown trees", self.shared).settled_readback_pair(),
        ]
    }
}

fn view(slug: &'static str, label: &'static str, camera: PlateCamera) -> CaptureViewSpec {
    CaptureViewSpec::new(
        slug,
        label,
        CapturePose::Grown(camera),
        FOV_DEGREES,
        FOREGROUND_BPS,
    )
    .hide_obstacles()
    .suppress_grass()
    .suppress_understory()
}

// World-space original-source envelope metres enter a perspective-fit boundary.
// Every original source-box corner must fit both image axes with the margin.
fn crown_distance(low: Vec3, high: Vec3, direction: Vec3) -> Result<Metres> {
    let centre = (low + high) * 0.5;
    let right = Vec3::Y.cross(direction).normalize();
    let up = direction.cross(right);
    let vertical_scale = FRAME_MARGIN.ratio() / (FOV_DEGREES.to_radians() * 0.5).tan();
    let horizontal_scale = vertical_scale / super::super::capture_resolution::CAPTURE_ASPECT_RATIO;
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

impl PerspectiveDepth {
    pub(super) fn from_projection(projection: &Projection) -> Result<Self> {
        let Projection::Perspective(projection) = projection else {
            anyhow::bail!("review requires perspective depth");
        };
        ensure!(
            projection.near.is_finite()
                && projection.far.is_finite()
                && projection.near >= 0.0
                && projection.far > projection.near,
            "invalid production projection depth"
        );
        Ok(Self {
            near: Metres::from_metres(projection.near),
            far: Metres::from_metres(projection.far),
        })
    }
}

impl PlateCamera {
    pub(super) fn matches(
        self,
        camera: &bevy::prelude::Camera,
        transform: &GlobalTransform,
        projection: &Projection,
        depth: &PerspectiveDepth,
    ) -> bool {
        let Projection::Perspective(projection) = projection else {
            return false;
        };
        let expected = Transform::from_translation(self.position)
            .looking_at(self.target, self.up)
            .rotation;
        let rotation = transform.rotation();
        camera.is_active
            && transform.translation() == self.position
            && (rotation.abs_diff_eq(expected, CAMERA_ROTATION_EPSILON)
                || rotation.abs_diff_eq(-expected, CAMERA_ROTATION_EPSILON))
            && projection.near == depth.near.metres()
            && projection.far == depth.far.metres()
            && projection.fov == FOV_DEGREES.to_radians()
            && projection.aspect_ratio == super::super::capture_resolution::CAPTURE_ASPECT_RATIO
    }

    pub(super) fn actual(
        self,
        transform: &GlobalTransform,
        projection: &Projection,
    ) -> Result<serde_json::Value> {
        let Projection::Perspective(projection) = projection else {
            anyhow::bail!("review requires perspective projection");
        };
        Ok(
            serde_json::json!({"expected": self, "position_metres": transform.translation(), "rotation": transform.rotation(),
            "vertical_fov_radians": projection.fov, "aspect": projection.aspect_ratio, "near_metres": projection.near, "far_metres": projection.far}),
        )
    }
}

fn ground_eye(terrain: &SceneTerrain, mut position: Vec3, eye: Metres) -> Result<Vec3> {
    let ground = terrain
        .height_at(position.xz())
        .context("source-derived camera lacks production terrain support")?;
    ensure!(ground.is_finite(), "nonfinite camera ground");
    position.y = ground + eye.metres();
    Ok(position)
}

fn crown_eye(terrain: &SceneTerrain, mut position: Vec3) -> Result<Vec3> {
    position.y = position
        .y
        .max(ground_eye(terrain, position, SUPPORT_EYE)?.y);
    Ok(position)
}
