//! Fixed camera contracts derived only from full source geometry, never candidate bounds.

use super::*;
use adventuresim_tactical_core::prelude::SceneTerrain;
use fabelgeist_math::units::{Metres, Ratio};

const CAMERA_ROTATION_EPSILON: f32 = 1.0e-5;
const ROOT_EYE: Metres = Metres::from_metres(1.6);

const FOV_DEGREES: f32 = 60.0;
const FRAME_MARGIN: Ratio = Ratio::from_ratio(1.15);
const GAMEPLAY_AXIAL_OFFSET: Ratio = Ratio::from_ratio(0.75);
const GAMEPLAY_EYE: Metres = Metres::from_metres(super::super::STANDING_EYE_HEIGHT_METRES);
const FOREGROUND_BPS: u16 = 500;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub(in super::super) struct PlateCamera {
    pub plate: Plate,
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(in super::super) enum Plate {
    Root,
    Crown,
    Gameplay,
}

#[derive(Serialize)]
pub(super) struct Cameras {
    root: PlateCamera,
    additional_root_contacts: Option<[PlateCamera; 2]>,
    crown: PlateCamera,
    gameplay: PlateCamera,
}

impl Cameras {
    pub(super) fn from_source(
        source: &TreeSourceGeometry,
        roots: &[Vec3],
        terrain: &SceneTerrain,
    ) -> Result<Self> {
        let root = *roots.first().context("source camera requires a root")?;
        let source_low = Vec3::from_array(source.minimum.map(Metres::metres));
        let source_high = Vec3::from_array(source.maximum.map(Metres::metres));
        let mut low = source_low;
        let mut high = source_high;
        for position in roots {
            let offset = *position - root;
            low = low.min(source_low + offset);
            high = high.max(source_high + offset);
        }
        let centre = (low + high) * 0.5 + root;
        let radius = low.distance(high) * 0.5;
        let distance = radius / (FOV_DEGREES.to_radians() * 0.5).sin() * FRAME_MARGIN.ratio();
        ensure!(
            distance.is_finite() && distance > 0.0,
            "invalid source camera envelope"
        );
        Ok(Self {
            root: root_contact(terrain, root)?,
            additional_root_contacts: match roots {
                [_] => None,
                [_, left, right] => Some([
                    root_contact(terrain, *left)?,
                    root_contact(terrain, *right)?,
                ]),
                _ => anyhow::bail!("unsupported source contact layout"),
            },
            crown: PlateCamera {
                plate: Plate::Crown,
                position: crown_eye(
                    terrain,
                    centre + Vec3::new(1.0, 0.18, 1.0).normalize() * distance,
                )?,
                target: centre,
                up: Vec3::Y,
            },
            gameplay: PlateCamera {
                plate: Plate::Gameplay,
                position: ground_eye(
                    terrain,
                    root + Vec3::new(
                        distance * GAMEPLAY_AXIAL_OFFSET.ratio(),
                        0.0,
                        distance * GAMEPLAY_AXIAL_OFFSET.ratio(),
                    ),
                    GAMEPLAY_EYE,
                )?,
                target: centre,
                up: Vec3::Y,
            },
        })
    }

    pub(super) fn views(&self, representation: TreeCacheRepresentation) -> Vec<CaptureViewSpec> {
        let mut views = vec![
            view("warmup", "Pipeline warmup", self.gameplay, representation).warmup(),
            view(
                "cache-root",
                "Source tree root contact",
                self.root,
                representation,
            )
            .settled_readback_pair(),
        ];
        if let Some([left, right]) = self.additional_root_contacts {
            views.extend([
                view(
                    "cache-root-left",
                    "Left tree root contact",
                    left,
                    representation,
                )
                .settled_readback_pair(),
                view(
                    "cache-root-right",
                    "Right tree root contact",
                    right,
                    representation,
                )
                .settled_readback_pair(),
            ]);
        }
        views.extend([
            view(
                "cache-crown",
                "Source tree full crown",
                self.crown,
                representation,
            )
            .settled_readback_pair(),
            view(
                "cache-gameplay",
                "Source tree at eye height",
                self.gameplay,
                representation,
            )
            .settled_readback_pair(),
        ]);
        views
    }
}

fn root_contact(terrain: &SceneTerrain, root: Vec3) -> Result<PlateCamera> {
    Ok(PlateCamera {
        plate: Plate::Root,
        position: ground_eye(terrain, root + Vec3::new(3.8, 0.0, 4.2), ROOT_EYE)?,
        target: root + Vec3::Y * 0.65,
        up: Vec3::Y,
    })
}

fn view(
    slug: &'static str,
    label: &'static str,
    camera: PlateCamera,
    representation: TreeCacheRepresentation,
) -> CaptureViewSpec {
    let mut view = CaptureViewSpec::new(
        slug,
        label,
        CapturePose::Cache(camera),
        FOV_DEGREES,
        FOREGROUND_BPS,
    )
    .render_lod(representation.lod())
    .hide_obstacles()
    .suppress_grass()
    .suppress_understory();
    if let Some(leaf) = representation.leaf() {
        view = view.leaf_lod(leaf);
    }
    view
}

#[derive(Resource, Serialize)]
pub(in super::super) struct PerspectiveDepth {
    near: Metres,
    far: Metres,
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
        .max(ground_eye(terrain, position, GAMEPLAY_EYE)?.y);
    Ok(position)
}
