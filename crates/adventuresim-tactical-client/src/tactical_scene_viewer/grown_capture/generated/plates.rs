//! Source-envelope framing at actual generated root placements.
use super::super::cameras::SupportedCameras;
use super::*;

pub(super) const EYE_DIRECTION: Vec3 = Vec3::new(1.0, 0.9, 1.0);
const PLATES: [PlateSpec; 5] = [
    PlateSpec {
        slug: "grown-generated-context",
        label: "Actual grown tactical forest",
    },
    PlateSpec {
        slug: "grown-generated-first-source",
        label: "First actual botanical source at a generated root",
    },
    PlateSpec {
        slug: "grown-generated-second-source",
        label: "Second actual botanical source at a generated root",
    },
    PlateSpec {
        slug: "grown-generated-ground",
        label: "Grown forest from the existing supported ground camera",
    },
    PlateSpec {
        slug: "grown-generated-root",
        label: "Original focused tree root and surrounding soil",
    },
];

struct PlateSpec {
    slug: &'static str,
    label: &'static str,
}

struct SceneCameras {
    ground: PlateCamera,
    root: PlateCamera,
}

struct Envelope {
    low: Vec3,
    high: Vec3,
}

impl SceneCameras {
    fn admit(world: &World, held: &census::HeldScene) -> Result<Self> {
        let state = world.resource::<SceneCaptureState>();
        let focused = state
            .tree_focus_entity
            .context("generated root camera needs original focus")?;
        let original = held
            .roots
            .iter()
            .find(|root| root.attachment.root == focused)
            .context("root camera focus is not an original generated tree")?;
        ensure!(
            state.tree_focus == Some(original.attachment.frame.translation),
            "root camera focus differs from original physical frame"
        );
        let camera = |pose| {
            let (transform, target) = super::super::super::camera_for_view(pose, state);
            PlateCamera {
                position: transform.translation,
                target,
                up: Vec3::Y,
            }
        };
        Ok(Self {
            ground: camera(CapturePose::Ground),
            root: camera(CapturePose::Root),
        })
    }
}

pub(super) fn slugs() -> Vec<&'static str> {
    std::iter::once("warmup")
        .chain(PLATES.map(|plate| plate.slug))
        .collect()
}

pub(super) fn resolve(
    world: &mut World,
    held: &census::HeldScene,
) -> Result<SupportedCameras<Vec<CaptureViewSpec>>> {
    let scene = SceneCameras::admit(world, held)?;
    cameras::with_support(
        world,
        |support| {
            let mut low = Vec3::splat(f32::INFINITY);
            let mut high = Vec3::splat(f32::NEG_INFINITY);
            for root in &held.roots {
                let envelope = envelope(held, root)?;
                low = low.min(envelope.low);
                high = high.max(envelope.high);
            }
            let direction = EYE_DIRECTION.normalize();
            let context = frame(low, high, direction, |eye| support.clamp_eye(eye))?;
            let mut cameras = vec![context];
            for source in held.assets.catalogue().sources() {
                let root = held
                    .roots
                    .iter()
                    .filter(|root| root.attachment.source == *source)
                    .max_by(|a, b| {
                        a.attachment
                            .frame
                            .translation
                            .dot(direction)
                            .total_cmp(&b.attachment.frame.translation.dot(direction))
                    })
                    .context("generated source has no actual root")?;
                let bounds = envelope(held, root)?;
                cameras.push(frame(bounds.low, bounds.high, direction, |eye| {
                    support.clamp_eye(eye)
                })?);
            }
            for camera in [scene.ground, scene.root] {
                cameras.push(PlateCamera {
                    position: support.clamp_eye(camera.position)?,
                    ..camera
                });
            }
            ensure!(
                cameras.len() == PLATES.len(),
                "generated camera/source cardinality"
            );
            let mut views =
                vec![view("warmup", "Generated forest pipeline warmup", context).warmup()];
            for (PlateSpec { slug, label }, camera) in PLATES.into_iter().zip(cameras) {
                views.push(view(slug, label, camera).settled_readback_pair());
            }
            Ok(views)
        },
        |views| {
            views
                .iter()
                .filter_map(|view| match view.pose {
                    CapturePose::Grown(camera) => Some(camera.position),
                    _ => None,
                })
                .collect()
        },
    )
}

fn envelope(held: &census::HeldScene, root: &census::Root) -> Result<Envelope> {
    let model = held
        .assets
        .catalogue()
        .bank_assets()
        .bank()
        .library()
        .models
        .get(root.attachment.source.variant.index())
        .context("source envelope model")?;
    let origin = root.attachment.frame.translation
        - Vec3::Y * (adventuresim_tactical_core::prelude::TREE_TRUNK_HEIGHT_METRES * 0.5);
    Ok(Envelope {
        low: Vec3::from_array(model.bounds.low.map(|value| value.metres())) + origin,
        high: Vec3::from_array(model.bounds.high.map(|value| value.metres())) + origin,
    })
}

// Native source bounds and camera coordinates enter in world metres.
fn frame(
    low: Vec3,
    high: Vec3,
    direction: Vec3,
    clamp: impl FnOnce(Vec3) -> Result<Vec3>,
) -> Result<PlateCamera> {
    let target = (low + high) * 0.5;
    let distance = cameras::crown_distance(low, high, direction)?;
    Ok(PlateCamera {
        position: clamp(target + direction * distance.metres())?,
        target,
        up: Vec3::Y,
    })
}

fn view(slug: &'static str, label: &'static str, camera: PlateCamera) -> CaptureViewSpec {
    CaptureViewSpec::new(
        slug,
        label,
        CapturePose::Grown(camera),
        cameras::FOV_DEGREES,
        cameras::FOREGROUND_BPS,
    )
    .vista()
}
