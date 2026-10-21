//! One deterministic placement and range policy for both tree representations.
use super::{
    SceneEnvironment, SceneTerrain, Transform, Vec2, Vec3, VisibilityRange, VistaLod, bps,
    stable_text_seed, streams, tree_root_height,
};
use fabelgeist_determinism::Seed;
use fabelgeist_math::units::{Metres, Ratio};

/// Borrowed, already admitted terrain inputs for one vista ring.
pub(super) struct VistaTreeBand<'a> {
    pub lod: &'a VistaLod,
    pub coarser_lod: Option<&'a VistaLod>,
    pub inner_half_extent: Vec2,
    pub scene_digest: &'a str,
    pub environment: &'a SceneEnvironment,
    pub terrain: &'a SceneTerrain,
}

pub(super) struct VistaTreePlacement {
    pub origin: VistaGroundOrigin,
    pub seed: Seed,
    pub scale: Ratio,
}

#[derive(bevy::prelude::Component)]
#[component(immutable)]
pub(super) struct VistaTreeCandidate(pub(super) Seed);

/// Ground origin in tactical-local Bevy axes, measured in metres.
/// This is distinct from a playable collider-centred root.
pub(super) struct VistaGroundOrigin(Vec3);

impl VistaGroundOrigin {
    /// Native spatial-kernel boundary for source selection and ECS transforms.
    pub fn position(&self) -> Vec3 {
        self.0
    }
    pub fn frame(&self, scale: Ratio) -> Transform {
        Transform::from_translation(self.0).with_scale(Vec3::splat(scale.ratio()))
    }
}

pub(super) fn collect(band: &VistaTreeBand<'_>) -> Vec<VistaTreePlacement> {
    let lod = band.lod;
    let width = usize::from(lod.width);
    let depth = usize::from(lod.depth);
    let center = Vec2::new((width - 1) as f32, (depth - 1) as f32) * 0.5;
    let scene_seed = stable_text_seed(band.scene_digest);
    let spacing = Metres::from_metres(lod.spacing_metres);
    let mut placements = Vec::new();
    for z in 0..depth - 1 {
        for x in 0..width - 1 {
            let sample = lod.environment[z * width + x];
            let canopy = bps(sample.canopy_bps)
                * (1.0 - bps(sample.water_bps))
                * (1.0 - bps(sample.cultivation_bps) * 0.85);
            // A sample represents a regional stand. Preserve the exact existing
            // physical-area count, cap, stream order and exclusion collar.
            let candidate_count = vista_tree_candidate_count(
                Ratio::from_ratio(canopy),
                spacing,
                streams::tree_count_seed(scene_seed, x, z),
            )
            .min(if lod.spacing_metres <= 250.0 { 24 } else { 3 });
            if candidate_count == 0 {
                continue;
            }
            let cell_min = (Vec2::new(x as f32, z as f32) - center) * lod.spacing_metres;
            for candidate in 0..candidate_count {
                let seed = streams::tree_seed(scene_seed, x, z, candidate);
                let local = cell_min + streams::tree_jitter(seed) * lod.spacing_metres;
                if local.x.abs() <= band.inner_half_extent.x + 7.0
                    && local.y.abs() <= band.inner_half_extent.y + 7.0
                {
                    continue;
                }
                let world = local
                    + Vec2::new(
                        lod.origin_east_metres as f32,
                        lod.origin_north_metres as f32,
                    );
                let Some(height) = tree_root_height(band.terrain, lod, band.coarser_lod, world)
                else {
                    continue;
                };
                let variation =
                    Ratio::from_ratio(streams::TREE_SCALE.rng(seed, &[]).inclusive_unit_f32());
                placements.push(VistaTreePlacement {
                    origin: VistaGroundOrigin(Vec3::new(local.x, height, local.y)),
                    seed,
                    scale: vista_tree_scale(spacing, variation),
                });
            }
        }
    }
    placements
}

// Numerical range/scale kernels: unwrap units only for arithmetic and Bevy's
// native VisibilityRange representation. Keep the existing policy unchanged.
pub(super) const VISTA_TREE_MAX_ANGULAR_HEIGHT_RADIANS: f32 = 16.0_f32.to_radians();

pub(super) fn vista_tree_scale(spacing: Metres, variation: Ratio) -> Ratio {
    let spacing_metres = spacing.metres();
    let variation = variation.ratio();
    // Candidate count already represents regional stand density. Each card
    // must remain one plausible tree; scaling a single trunk into a 32-metre
    // stand creates the near-ring columns seen from the playable scene.
    let coarse_scale = if spacing_metres <= 250.0 { 1.0 } else { 1.25 };
    Ratio::from_ratio((0.85 + variation.clamp(0.0, 1.0) * 0.4) * coarse_scale)
}

pub(super) fn vista_tree_visibility(
    spacing: Metres,
    unscaled_height: Metres,
    scale: Ratio,
) -> VisibilityRange {
    let spacing_metres = spacing.metres();
    let scaled_height = unscaled_height.metres().max(0.0) * scale.ratio().max(0.0);
    // Keep regional stand cards out of the near field. The first visible
    // sample is at most 16 degrees; the fade completes farther away. Using
    // this distance as the end of the fade admitted partially visible 17-19
    // degree cards, contradicting the background-size contract.
    let first_visible_distance =
        scaled_height / (2.0 * (VISTA_TREE_MAX_ANGULAR_HEIGHT_RADIANS * 0.5).tan());
    VisibilityRange {
        start_margin: first_visible_distance..(first_visible_distance * 1.12),
        end_margin: if spacing_metres <= 250.0 {
            1_600.0..1_900.0
        } else {
            4_600.0..5_200.0
        },
        use_aabb: false,
    }
}

pub(super) fn vista_tree_candidate_count(canopy: Ratio, spacing: Metres, seed: Seed) -> usize {
    let canopy = canopy.ratio();
    let spacing_metres = spacing.metres();
    let expected = canopy.clamp(0.0, 1.0) * spacing_metres * spacing_metres / 3_200.0;
    expected.floor() as usize
        + usize::from(
            streams::TREE_COUNT_FRACTION
                .rng(seed, &[])
                .inclusive_unit_f32()
                < expected.fract(),
        )
}
