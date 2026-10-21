//! Finite source-only plates shared by every immutable level and encoding.

use super::*;
use adventuresim_environment::render::{ForestDetail, ForestLodProjection, ProjectedRadiusPixels};
use fabelgeist_math::units::Ratio;

const OUTSIDE_TRANSITION: Ratio = Ratio::from_ratio(1.05);
const INSIDE_TRANSITION: Ratio = Ratio::from_ratio(0.80);

#[derive(Serialize)]
pub(in super::super) struct Cameras {
    plates: Vec<Plate>,
    physical_viewport: UVec2,
    vertical_fov_degrees: f32,
}

#[derive(Serialize)]
struct Plate {
    kind: PlateKind,
    camera: PlateCamera,
    target_radius_pixels: Option<ProjectedRadiusPixels>,
    actual_roots: Vec<framing::RootProjection>,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum PlateKind {
    Close,
    FullSide,
    MiddleFullSide,
    MiddleInterior,
    MiddleCoarseSide,
    CoarseSide,
}

impl PlateKind {
    const ALL: [Self; 6] = [
        Self::Close,
        Self::FullSide,
        Self::MiddleFullSide,
        Self::MiddleInterior,
        Self::MiddleCoarseSide,
        Self::CoarseSide,
    ];

    fn slug(self) -> &'static str {
        match self {
            Self::Close => "grown-shared",
            Self::FullSide => "grown-full-side",
            Self::MiddleFullSide => "grown-middle-full-side",
            Self::MiddleInterior => "grown-middle-interior",
            Self::MiddleCoarseSide => "grown-middle-coarse-side",
            Self::CoarseSide => "grown-coarse-side",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Close => "Three shared grown trees, close diagnostic",
            Self::FullSide => "Full side of grown full transition",
            Self::MiddleFullSide => "Middle side of grown full transition",
            Self::MiddleInterior => "Grown middle band interior",
            Self::MiddleCoarseSide => "Middle side of grown coarse transition",
            Self::CoarseSide => "Coarse side of grown coarse transition",
        }
    }

    fn target(self, detail: &ForestDetail) -> Result<Option<ProjectedRadiusPixels>> {
        let pixels = match self {
            Self::Close => return Ok(None),
            Self::FullSide => detail.full_pixels * OUTSIDE_TRANSITION.ratio(),
            Self::MiddleFullSide => detail.full_pixels * INSIDE_TRANSITION.ratio(),
            Self::MiddleInterior => (detail.full_pixels * detail.middle_pixels).sqrt(),
            Self::MiddleCoarseSide => detail.middle_pixels * OUTSIDE_TRANSITION.ratio(),
            Self::CoarseSide => detail.middle_pixels * INSIDE_TRANSITION.ratio(),
        };
        let target =
            ProjectedRadiusPixels::from_pixels(pixels).context("invalid grown camera target")?;
        let expected = match self {
            Self::FullSide => Level::Full,
            Self::CoarseSide => Level::Coarse,
            _ => Level::Middle,
        };
        ensure!(
            Level::for_projected_radius(target, Level::Coarse, detail) == expected,
            "source thresholds cannot admit named grown camera band"
        );
        Ok(Some(target))
    }
}

impl Cameras {
    pub(in super::super) fn slugs() -> Vec<&'static str> {
        std::iter::once("warmup")
            .chain(PlateKind::ALL.map(PlateKind::slug))
            .collect()
    }

    pub(super) fn from_source(
        bounds: TreeBounds,
        placements: &layout::RootPlacements,
        detail: &ForestDetail,
        support: &support::Reader<'_>,
    ) -> Result<Self> {
        ensure!(
            detail.full_pixels.is_finite()
                && detail.middle_pixels.is_finite()
                && detail.full_pixels > detail.middle_pixels
                && detail.middle_pixels > 0.0,
            "invalid grown source detail thresholds"
        );
        let physical_viewport = super::super::super::capture_resolution::physical_pixels(PROFILE);
        let lens = PerspectiveProjection {
            fov: FOV_DEGREES.to_radians(),
            aspect_ratio: super::super::super::capture_resolution::CAPTURE_ASPECT_RATIO,
            ..default()
        };
        let projection = ForestLodProjection::from_perspective(&lens, physical_viewport)
            .context("invalid declared grown plate projection")?;
        let envelopes = framing::SourceEnvelopes::new(bounds, placements)?;
        let mut plates = Vec::new();
        for kind in PlateKind::ALL {
            let target = kind.target(detail)?;
            let camera = match target {
                Some(target) => envelopes.solve(target, projection, support)?,
                None => envelopes.close(support)?,
            };
            plates.push(Plate {
                kind,
                camera,
                target_radius_pixels: target,
                actual_roots: envelopes.report(camera.position, projection, detail)?,
            });
        }
        Ok(Self {
            plates,
            physical_viewport,
            vertical_fov_degrees: FOV_DEGREES,
        })
    }

    pub(in super::super) fn views(&self) -> Vec<CaptureViewSpec> {
        let mut views = Vec::new();
        for plate in &self.plates {
            let mut spec =
                view(plate.kind.slug(), plate.kind.label(), plate.camera).settled_readback_pair();
            if !matches!(plate.kind, PlateKind::Close) {
                spec = spec.vista();
            }
            if views.is_empty() {
                views.push(view("warmup", "Grown shared pipeline warmup", plate.camera).warmup());
            }
            views.push(spec);
        }
        views
    }

    pub(super) fn eyes(&self) -> impl Iterator<Item = Vec3> + '_ {
        self.plates.iter().map(|plate| plate.camera.position)
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
