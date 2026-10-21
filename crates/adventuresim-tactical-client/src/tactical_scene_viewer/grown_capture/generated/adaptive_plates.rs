//! Supported approach/retreat checkpoints using actual source pixel thresholds.

use super::super::cameras::SupportedCameras;
use super::*;
use adventuresim_environment::render::{
    ForestLodProjection, ProjectedRadiusPixels, source_bounds_radius,
};
use bevy::camera::primitives::Aabb;
use fabelgeist_math::units::{Metres, Ratio};

const BRACKET_STEPS: usize = 16;
const REFINE_STEPS: usize = 24;
// Stay below the coarse reset boundary without leaving retained terrain support.
// Every actual root is still checked from both finer prior levels below.
const WARMUP_TARGET: Ratio = Ratio::from_ratio(0.60);
const STEPS: [Step; 11] = [
    Step {
        slug: "grown-adaptive-coarse-approach",
        label: "Approach inside coarse keep band",
        boundary: Boundary::Middle,
        fraction: Ratio::from_ratio(0.95),
    },
    Step {
        slug: "grown-adaptive-enter-middle",
        label: "First middle selection",
        boundary: Boundary::Middle,
        fraction: Ratio::from_ratio(1.05),
    },
    Step {
        slug: "grown-adaptive-middle-keep",
        label: "Middle hysteresis hold",
        boundary: Boundary::Middle,
        fraction: Ratio::from_ratio(0.90),
    },
    Step {
        slug: "grown-adaptive-leave-middle",
        label: "Retreat to coarse",
        boundary: Boundary::Middle,
        fraction: Ratio::from_ratio(0.80),
    },
    Step {
        slug: "grown-adaptive-middle-again",
        label: "Return to middle",
        boundary: Boundary::Middle,
        fraction: Ratio::from_ratio(1.10),
    },
    Step {
        slug: "grown-adaptive-enter-full",
        label: "First full selection",
        boundary: Boundary::Full,
        fraction: Ratio::from_ratio(1.05),
    },
    Step {
        slug: "grown-adaptive-full-keep",
        label: "Full hysteresis hold",
        boundary: Boundary::Full,
        fraction: Ratio::from_ratio(0.90),
    },
    Step {
        slug: "grown-adaptive-leave-full",
        label: "Retreat to middle",
        boundary: Boundary::Full,
        fraction: Ratio::from_ratio(0.80),
    },
    Step {
        slug: "grown-adaptive-middle-keep-full",
        label: "Middle below full entry",
        boundary: Boundary::Full,
        fraction: Ratio::from_ratio(0.95),
    },
    Step {
        slug: "grown-adaptive-full-again",
        label: "Return to full",
        boundary: Boundary::Full,
        fraction: Ratio::from_ratio(1.10),
    },
    Step {
        slug: "grown-adaptive-coarse-return",
        label: "Final coarse return",
        boundary: Boundary::Middle,
        fraction: Ratio::from_ratio(0.75),
    },
];

struct Step {
    slug: &'static str,
    label: &'static str,
    boundary: Boundary,
    fraction: Ratio,
}

struct Focus {
    root: Entity,
    bounds: Aabb,
    radius: Metres,
    origin: Vec3,
}

enum Boundary {
    Full,
    Middle,
}

impl Focus {
    fn from_root(held: &census::HeldScene, root: &census::Root) -> Result<Self> {
        let library = held.assets.catalogue().bank_assets().bank().library();
        let model = library
            .models
            .get(root.attachment.source.variant.index())
            .context("adaptive camera source bounds")?;
        let origin = root.attachment.frame.translation
            - Vec3::Y * (adventuresim_tactical_core::prelude::TREE_TRUNK_HEIGHT_METRES * 0.5);
        let low = Vec3::from_array(model.bounds.low.map(Metres::metres));
        let high = Vec3::from_array(model.bounds.high.map(Metres::metres));
        Ok(Self {
            root: root.attachment.root,
            bounds: Aabb::from_min_max(low + origin, high + origin),
            radius: source_bounds_radius(low, high),
            origin,
        })
    }

    fn pixels(
        &self,
        camera: PlateCamera,
        projection: ForestLodProjection,
    ) -> Result<ProjectedRadiusPixels> {
        projection
            .projected_radius(camera.position, &self.bounds, self.radius)
            .context("adaptive camera projection")
    }

    fn solve(
        &self,
        target: ProjectedRadiusPixels,
        projection: ForestLodProjection,
        at: &impl Fn(Metres) -> Result<PlateCamera>,
    ) -> Result<PlateCamera> {
        let low = Vec3::from(self.bounds.min());
        let high = Vec3::from(self.bounds.max());
        let mut near =
            super::super::cameras::crown_distance(low, high, plates::EYE_DIRECTION.normalize())?;
        ensure!(
            self.pixels(at(near)?, projection)? >= target,
            "focused source-fit camera cannot reach threshold bracket"
        );
        let mut far = near;
        let mut bracketed = false;
        for _ in 0..BRACKET_STEPS {
            far = Metres::from_metres(far.metres() * 2.0);
            if self.pixels(at(far)?, projection)? <= target {
                bracketed = true;
                break;
            }
        }
        ensure!(bracketed, "adaptive supported camera cannot bracket target");
        for _ in 0..REFINE_STEPS {
            let middle = Metres::from_metres((near.metres() + far.metres()) * 0.5);
            if self.pixels(at(middle)?, projection)? > target {
                near = middle;
            } else {
                far = middle;
            }
        }
        at(far)
    }
}

pub(super) fn slugs() -> Vec<&'static str> {
    std::iter::once("warmup")
        .chain(STEPS.iter().map(|step| step.slug))
        .collect()
}

pub(super) fn resolve(
    world: &mut World,
    held: &census::HeldScene,
) -> Result<SupportedCameras<Vec<CaptureViewSpec>>> {
    let focus = held
        .roots
        .iter()
        .map(|root| Focus::from_root(held, root))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .max_by(|left, right| {
            left.radius
                .metres()
                .total_cmp(&right.radius.metres())
                .then_with(|| {
                    left.origin
                        .dot(plates::EYE_DIRECTION)
                        .total_cmp(&right.origin.dot(plates::EYE_DIRECTION))
                })
        })
        .context("adaptive camera requires an actual root")?;
    let (camera, projection) = world.query_filtered::<(&Camera,&Projection),With<crate::presentation::TacticalGameplayCamera>>().single(world)?;
    let Projection::Perspective(perspective) = projection else {
        anyhow::bail!("adaptive camera requires perspective");
    };
    let mut perspective = perspective.clone();
    perspective.fov = cameras::FOV_DEGREES.to_radians();
    let projection = ForestLodProjection::from_perspective(
        &perspective,
        camera
            .physical_viewport_size()
            .context("adaptive camera physical viewport")?,
    )
    .context("adaptive camera pixel projection")?;
    let detail = held
        .assets
        .catalogue()
        .bank_assets()
        .bank()
        .library()
        .detail;
    let metadata = serde_json::json!({"root_entity_bits":focus.root.to_bits(),"source_radius_metres":focus.radius.metres(),"bounds":{"low":Vec3::from(focus.bounds.min()),"high":Vec3::from(focus.bounds.max())},"full_pixels":detail.full_pixels,"middle_pixels":detail.middle_pixels,"sequence":"prefix preserves approach and retreat history; checkpoint images are settled, GPU transition proof is separate"});
    let resolved = cameras::with_support(
        world,
        |support| {
            let centre = Vec3::from(focus.bounds.center);
            let at = |distance: Metres| -> Result<PlateCamera> {
                Ok(PlateCamera {
                    position: support.clamp_eye(
                        centre + plates::EYE_DIRECTION.normalize() * distance.metres(),
                    )?,
                    target: centre,
                    up: Vec3::Y,
                })
            };
            let target =
                ProjectedRadiusPixels::from_pixels(detail.middle_pixels * WARMUP_TARGET.ratio())
                    .context("adaptive warmup pixel target")?;
            let warmup = focus.solve(target, projection, &at)?;
            for root in &held.roots {
                let pixels = Focus::from_root(held, root)?.pixels(warmup, projection)?;
                ensure!(
                    Level::for_projected_radius(pixels, Level::Full, &detail) == Level::Coarse
                        && Level::for_projected_radius(pixels, Level::Middle, &detail)
                            == Level::Coarse,
                    "adaptive warmup does not reset every root to coarse"
                );
            }
            let mut views =
                vec![view("warmup", "Adaptive coarse pipeline warmup", warmup).warmup()];
            for step in &STEPS {
                let threshold = match step.boundary {
                    Boundary::Full => detail.full_pixels,
                    Boundary::Middle => detail.middle_pixels,
                };
                let target = ProjectedRadiusPixels::from_pixels(threshold * step.fraction.ratio())
                    .context("adaptive checkpoint target")?;
                let camera = focus.solve(target, projection, &at)?;
                views.push(view(step.slug, step.label, camera).settled_readback_pair());
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
    )?;
    world.resource_mut::<Review>().metadata["adaptive_focus"] = metadata;
    Ok(resolved)
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
