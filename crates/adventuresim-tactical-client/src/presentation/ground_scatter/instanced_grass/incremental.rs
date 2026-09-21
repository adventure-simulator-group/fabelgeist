//! Bounded deterministic construction for a playable scene's instanced sward.

use adventuresim_tactical_core::prelude::{SceneGround, SceneTerrain};
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on};

use super::{
    CoverageMask, GrassCommunityProfile, GrassMeshLod, GrassSpecies, GrassWorld, InstanceData,
    ScenePlacement, TierSpeciesBatches, TuftPigment, TuftPlacement,
    configured_tuft_footprint_metres, scatter_one_cell, spawn_tuft_batches,
};
use crate::presentation::config::GrassConfig;

/// Upper bound on lattice cells and copied near-edge instances in one update.
/// A cell can contain 144 dense near tufts, so one cell keeps the presentation
/// work below a frame-sized unit while input and replication stay responsive.
const GRASS_GENERATION_WORK_UNITS_PER_UPDATE: usize = 1;

/// Owns all temporary data for one scene until its complete batch set is ready.
pub(super) struct GrassGeneration {
    scene: Entity,
    terrain: SceneTerrain,
    ground: SceneGround,
    mask: Option<CoverageMask>,
    mask_task: Option<Task<CoverageMask>>,
    profile: GrassCommunityProfile,
    base_seed: u64,
    pigment: TuftPigment,
    grass: GrassConfig,
    batches: TierSpeciesBatches,
    stage: Option<GenerationStage>,
    started: web_time::Instant,
}

impl GrassGeneration {
    #[expect(
        clippy::too_many_arguments,
        reason = "a generation snapshot owns the scene inputs needed after the ECS query yields"
    )]
    pub(super) fn new(
        scene: Entity,
        terrain: SceneTerrain,
        ground: SceneGround,
        profile: GrassCommunityProfile,
        base_seed: u64,
        mask_seed: u64,
        pigment: TuftPigment,
        grass: GrassConfig,
    ) -> Self {
        let mask_ground = ground.clone();
        let mask_task = AsyncComputeTaskPool::get().spawn(async move {
            let mask = CoverageMask::new(&mask_ground, mask_seed);
            mask
        });
        Self {
            scene,
            terrain,
            ground,
            mask: None,
            mask_task: Some(mask_task),
            profile,
            base_seed,
            pigment,
            grass,
            batches: TierSpeciesBatches::default(),
            stage: None,
            started: web_time::Instant::now(),
        }
    }

    pub(super) const fn scene(&self) -> Entity {
        self.scene
    }

    /// Advances at most one bounded slice, returning true once every tier is
    /// complete and ready for normal production spawning.
    pub(super) fn advance(&mut self) -> bool {
        if self.mask.is_none() {
            let Some(task) = self.mask_task.as_ref() else {
                return false;
            };
            if !task.is_finished() {
                return false;
            }
            let task = self
                .mask_task
                .take()
                .expect("finished task remains present");
            self.mask = Some(block_on(task));
            let placement = ScenePlacement {
                terrain: &self.terrain,
                ground: &self.ground,
                mask: self.mask.as_ref().expect("completed task supplies a mask"),
                profile: self.profile,
                base_seed: self.base_seed,
            };
            self.stage = Some(GenerationStage::Near(ScatterCursor::new(
                &placement,
                self.base_seed,
                GrassMeshLod::Near,
                self.grass.placement.playable_patch_spacing_m,
            )));
        }
        let Self {
            terrain,
            ground,
            mask,
            profile,
            base_seed,
            grass,
            batches,
            stage,
            ..
        } = self;
        let stage = stage
            .as_mut()
            .expect("mask completion initializes the stage");
        let placement = ScenePlacement {
            terrain,
            ground,
            mask: mask.as_ref().expect("mask completed before placement"),
            profile: *profile,
            base_seed: *base_seed,
        };
        let mut remaining = GRASS_GENERATION_WORK_UNITS_PER_UPDATE;
        loop {
            match stage {
                GenerationStage::Near(cursor) => {
                    remaining -= cursor.advance(
                        &mut batches[GrassMeshLod::Near.tier_index()],
                        &placement,
                        grass,
                        remaining,
                    );
                    if !cursor.is_complete() {
                        return false;
                    }
                    *stage = GenerationStage::Far(ScatterCursor::new(
                        &placement,
                        *base_seed,
                        GrassMeshLod::Far,
                        grass.placement.playable_patch_spacing_m,
                    ));
                }
                GenerationStage::Far(cursor) => {
                    remaining -= cursor.advance(
                        &mut batches[GrassMeshLod::Far.tier_index()],
                        &placement,
                        grass,
                        remaining,
                    );
                    if !cursor.is_complete() {
                        return false;
                    }
                    *stage = GenerationStage::NearEdge(NearEdgeCopyCursor::default());
                }
                GenerationStage::NearEdge(cursor) => {
                    remaining -= cursor.advance(batches, remaining);
                    if !cursor.is_complete() {
                        return false;
                    }
                    *stage = GenerationStage::Vista(ScatterCursor::new(
                        &placement,
                        *base_seed ^ 0x7669_7374_615f_6c6f,
                        GrassMeshLod::Vista,
                        grass.placement.vista_patch_spacing_m,
                    ));
                }
                GenerationStage::Vista(cursor) => {
                    cursor.advance(
                        &mut batches[GrassMeshLod::Vista.tier_index()],
                        &placement,
                        grass,
                        remaining,
                    );
                    return cursor.is_complete();
                }
            }
            if remaining == 0 {
                return false;
            }
        }
    }

    pub(super) fn spawn(mut self, world: GrassWorld<'_, '_, '_>) {
        spawn_tuft_batches(
            world,
            &mut self.batches,
            "Instanced grass",
            ChildOf(self.scene),
            self.base_seed,
            self.pigment,
            &self.grass,
        );
        tracing::info!(
            elapsed_ms = self.started.elapsed().as_millis(),
            "Generated instanced tactical grass"
        );
    }
}

enum GenerationStage {
    Near(ScatterCursor),
    Far(ScatterCursor),
    NearEdge(NearEdgeCopyCursor),
    Vista(ScatterCursor),
}

/// Resumable traversal of the same inclusive x-then-z lattice as the legacy
/// synchronous walk. Each step consumes cells in exactly that order.
struct ScatterCursor {
    minimum: IVec2,
    maximum: IVec2,
    next: IVec2,
    base_seed: u64,
    lod: GrassMeshLod,
    cell_spacing: f32,
}

impl ScatterCursor {
    fn new(
        placement: &impl TuftPlacement,
        base_seed: u64,
        lod: GrassMeshLod,
        cell_spacing: f32,
    ) -> Self {
        let (minimum, maximum) = placement.lattice_bounds(cell_spacing);
        Self {
            minimum,
            maximum,
            next: minimum,
            base_seed,
            lod,
            cell_spacing,
        }
    }

    fn is_complete(&self) -> bool {
        self.next.y > self.maximum.y
    }

    fn advance(
        &mut self,
        batches: &mut [Vec<InstanceData>; GrassSpecies::ALL.len()],
        placement: &impl TuftPlacement,
        grass: &GrassConfig,
        work_units: usize,
    ) -> usize {
        let mut consumed = 0;
        while consumed < work_units && !self.is_complete() {
            let cell = self.next;
            self.next.x += 1;
            if self.next.x > self.maximum.x {
                self.next.x = self.minimum.x;
                self.next.y += 1;
            }
            scatter_one_cell(
                batches,
                placement,
                self.base_seed,
                self.lod,
                self.cell_spacing,
                configured_tuft_footprint_metres(self.lod, grass),
                cell,
                grass,
            );
            consumed += 1;
        }
        consumed
    }
}

/// Incrementally duplicates the exact near placements for the near-edge mesh.
#[derive(Default)]
struct NearEdgeCopyCursor {
    species_index: usize,
    next_instance: usize,
}

impl NearEdgeCopyCursor {
    fn is_complete(&self) -> bool {
        self.species_index == GrassSpecies::ALL.len()
    }

    fn advance(&mut self, batches: &mut TierSpeciesBatches, work_units: usize) -> usize {
        let mut copied = 0;
        while copied < work_units && !self.is_complete() {
            let near = GrassMeshLod::Near.tier_index();
            let edge = GrassMeshLod::NearEdge.tier_index();
            let (before_edge, edge_and_after) = batches.split_at_mut(edge);
            let source = &before_edge[near][self.species_index];
            if self.next_instance == source.len() {
                self.species_index += 1;
                self.next_instance = 0;
                continue;
            }
            edge_and_after[0][self.species_index].push(source[self.next_instance]);
            self.next_instance += 1;
            copied += 1;
        }
        copied
    }
}

#[cfg(test)]
mod tests {
    use super::super::scatter_cell_tufts;
    use super::*;
    use crate::presentation::config::TacticalGraphicsConfig;
    use crate::presentation::ground_scatter::grass::GrassCommunity;
    use adventuresim_tactical_core::prelude::{
        GroundSurface, SceneEnvironmentFixture, SceneGround,
    };
    use bevy::tasks::{AsyncComputeTaskPool, TaskPoolBuilder};

    struct DensePlacement;

    impl TuftPlacement for DensePlacement {
        fn lattice_bounds(&self, _cell_spacing: f32) -> (IVec2, IVec2) {
            (IVec2::new(-6, -5), IVec2::new(6, 5))
        }

        fn cell_allows(
            &self,
            _cell_hash: u64,
            _cell: IVec2,
            _cell_spacing: f32,
            _jitter: f32,
        ) -> bool {
            true
        }

        fn coverage(&self, _centre: Vec2) -> u8 {
            255
        }

        fn height(&self, centre: Vec2) -> Option<f32> {
            Some(centre.x * 0.01 + centre.y * 0.02)
        }

        fn community(&self, _centre: Vec2) -> GrassCommunity {
            GrassCommunity::MesicMeadow
        }
    }

    fn shipped_grass_config() -> GrassConfig {
        TacticalGraphicsConfig::parse(include_str!(
            "../../../../../../assets/config/tactical-graphics.yaml"
        ))
        .expect("shipped tactical graphics configuration must be valid")
        .grass
    }

    fn batch_signature(batches: &[Vec<InstanceData>; GrassSpecies::ALL.len()]) -> Vec<u32> {
        batches
            .iter()
            .flat_map(|batch| {
                batch.iter().flat_map(|instance| {
                    [
                        instance.position.x.to_bits(),
                        instance.position.y.to_bits(),
                        instance.position.z.to_bits(),
                        instance.scale.to_bits(),
                        instance.rotation.to_bits(),
                        instance.index,
                        instance.batch_id,
                        instance.seed,
                    ]
                })
            })
            .collect()
    }

    fn all_batch_signatures(batches: &TierSpeciesBatches) -> Vec<Vec<u32>> {
        batches.iter().map(batch_signature).collect()
    }

    fn dense_scene_inputs() -> (SceneTerrain, SceneGround) {
        let terrain = SceneTerrain::from_heightmap(9, 9, 2.0, vec![0.0; 81])
            .expect("flat test terrain is valid");
        let ground = SceneGround::from_samples(
            9,
            9,
            2.0,
            vec![
                GroundSurface {
                    cover_density_bps: 10_000,
                    cover_height_cm: 40,
                    ..Default::default()
                };
                81
            ],
        )
        .expect("dense test ground is valid");
        (terrain, ground)
    }

    #[test]
    fn bounded_dense_cursor_progresses_and_reproduces_full_lattice_output() {
        let grass = shipped_grass_config();
        let placement = DensePlacement;
        let seed = 0x51a7_7eed;
        let lod = GrassMeshLod::Near;
        let spacing = grass.placement.playable_patch_spacing_m;
        let mut expected = std::array::from_fn(|_| Vec::new());
        scatter_cell_tufts(&mut expected, &placement, seed, lod, spacing, &grass);

        let mut actual = std::array::from_fn(|_| Vec::new());
        let mut cursor = ScatterCursor::new(&placement, seed, lod, spacing);
        assert_eq!(cursor.advance(&mut actual, &placement, &grass, 3), 3);
        assert!(!cursor.is_complete());
        assert!(actual.iter().any(|batch| !batch.is_empty()));
        while !cursor.is_complete() {
            cursor.advance(&mut actual, &placement, &grass, 5);
        }

        assert_eq!(batch_signature(&actual), batch_signature(&expected));
    }

    #[test]
    fn bounded_near_edge_copy_preserves_every_near_instance() {
        let grass = shipped_grass_config();
        let placement = DensePlacement;
        let seed = 0x51a7_7eed;
        let mut batches: TierSpeciesBatches =
            std::array::from_fn(|_| std::array::from_fn(|_| Vec::new()));
        scatter_cell_tufts(
            &mut batches[GrassMeshLod::Near.tier_index()],
            &placement,
            seed,
            GrassMeshLod::Near,
            grass.placement.playable_patch_spacing_m,
            &grass,
        );
        let expected = batch_signature(&batches[GrassMeshLod::Near.tier_index()]);
        let mut cursor = NearEdgeCopyCursor::default();
        while !cursor.is_complete() {
            cursor.advance(&mut batches, 7);
        }
        assert_eq!(
            batch_signature(&batches[GrassMeshLod::NearEdge.tier_index()]),
            expected
        );
    }

    #[test]
    fn asynchronous_mask_preparation_completes_the_original_full_scene_batches() {
        AsyncComputeTaskPool::get_or_init(|| TaskPoolBuilder::new().num_threads(1).build());
        let grass = shipped_grass_config();
        let (terrain, ground) = dense_scene_inputs();
        let profile = GrassCommunityProfile::from_environment(
            &SceneEnvironmentFixture::TemperateHills.snapshot("incremental-grass"),
        );
        let base_seed = 17;
        let mask_seed = 19;
        let pigment = TuftPigment {
            color: Color::WHITE,
            density: 1.0,
            dryness: 0.0,
            wind_scale: 0.0,
        };
        let mut generation = GrassGeneration::new(
            Entity::from_raw_u32(17).expect("test entity index is valid"),
            terrain.clone(),
            ground.clone(),
            profile,
            base_seed,
            mask_seed,
            pigment,
            grass.clone(),
        );

        assert!(generation.mask.is_none());
        assert!(generation.mask_task.is_some());
        assert!(!generation.advance());

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while !generation.advance() {
            assert!(
                std::time::Instant::now() < deadline,
                "incremental generation must complete after its background mask is ready"
            );
            std::thread::yield_now();
        }

        let mask = CoverageMask::new(&ground, mask_seed);
        let placement = ScenePlacement {
            terrain: &terrain,
            ground: &ground,
            mask: &mask,
            profile,
            base_seed,
        };
        let mut expected = TierSpeciesBatches::default();
        for lod in [GrassMeshLod::Near, GrassMeshLod::Far] {
            scatter_cell_tufts(
                &mut expected[lod.tier_index()],
                &placement,
                base_seed,
                lod,
                grass.placement.playable_patch_spacing_m,
                &grass,
            );
        }
        for species in GrassSpecies::ALL {
            expected[GrassMeshLod::NearEdge.tier_index()][species.index()] =
                expected[GrassMeshLod::Near.tier_index()][species.index()].clone();
        }
        scatter_cell_tufts(
            &mut expected[GrassMeshLod::Vista.tier_index()],
            &placement,
            base_seed ^ 0x7669_7374_615f_6c6f,
            GrassMeshLod::Vista,
            grass.placement.vista_patch_spacing_m,
            &grass,
        );

        assert_eq!(
            all_batch_signatures(&generation.batches),
            all_batch_signatures(&expected)
        );
    }
}
