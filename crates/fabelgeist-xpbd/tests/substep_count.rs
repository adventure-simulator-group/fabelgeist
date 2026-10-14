use anyhow::Result;
use fabelgeist_compute::prelude::*;
use fabelgeist_gpu::prelude::*;
use fabelgeist_math::Vec3;
use fabelgeist_xpbd::{
    ConstraintSet, Particles, Solver, SolverSettings, SubstepCount, SubstepHook,
};

#[test]
fn admission_preserves_full_cardinality_and_consumer_policy() {
    for native in [0, 1, 2, 32, u32::MAX] {
        let count = SubstepCount::from(native);
        assert_eq!(u32::from(count), native);
        assert_eq!(count, SubstepCount::from_native(native));
        assert_eq!(format!("{count:?}"), format!("{native:?}"));
        assert_eq!(count.is_empty(), native == 0);
        assert_eq!(u32::from(count.at_least_one()), native.max(1));
        assert_eq!(count.at_least_one().at_least_one(), count.at_least_one());
    }
    let range = SubstepCount::from(1)..=SubstepCount::from(32);
    assert!(!range.contains(&SubstepCount::from(0)));
    assert!(range.contains(&SubstepCount::from(32)));
    assert!(!range.contains(&SubstepCount::from(u32::MAX)));
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HookPhase {
    BeforeSolve,
    AfterSolve,
}

#[derive(Debug, PartialEq, Eq)]
struct HookObservation {
    phase: HookPhase,
    // Exact IEEE-754 duration bits are the native observation representation.
    duration_bits: u32,
}

#[derive(Clone, Copy)]
enum ExecutionMode {
    Batched,
    Interleaved,
}

#[derive(Debug, PartialEq)]
struct Outcome {
    trace: Vec<HookObservation>,
    snapshots: Vec<Vec<u32>>,
    positions: Vec<Vec3>,
    velocities: Vec<Vec3>,
}

struct Observations<'a> {
    context: &'a WgpuContext,
    trace: Vec<HookObservation>,
    positions: Vec<Buffer>,
}
impl Observations<'_> {
    fn capture(
        &mut self,
        batch: &mut KernelBatch,
        particles: &Particles,
        dt: f32,
        phase: HookPhase,
    ) -> Result<()> {
        let bytes = u64::from(particles.count()) * 16;
        let snapshot = Buffer::new(
            self.context,
            bytes.into(),
            BufferDefinition::storage()
                .with_usage(BufferUse::CopyDestination)
                .with_usage(BufferUse::CopySource),
        )?;
        batch.copy_buffer(&particles.positions, &snapshot, bytes)?;
        self.trace.push(HookObservation {
            phase,
            duration_bits: dt.to_bits(),
        });
        self.positions.push(snapshot);
        Ok(())
    }
}
impl SubstepHook for Observations<'_> {
    fn record(&mut self, batch: &mut KernelBatch, particles: &Particles, dt: f32) -> Result<()> {
        self.capture(batch, particles, dt, HookPhase::BeforeSolve)
    }
    fn after_solve(
        &mut self,
        batch: &mut KernelBatch,
        particles: &Particles,
        dt: f32,
    ) -> Result<()> {
        self.capture(batch, particles, dt, HookPhase::AfterSolve)
    }
}

#[tokio::test]
async fn disabled_and_active_counts_preserve_hook_constraint_and_batch_order() -> Result<()> {
    let context = WgpuContext::new_compute().await?;
    let cache = KernelCache::new();
    for native in [0u32, 1, 4] {
        let mut outcomes = Vec::new();
        for execution in [ExecutionMode::Batched, ExecutionMode::Interleaved] {
            let particles = Particles::from_positions(
                &context,
                &[Vec3::default(), Vec3::new(2.0, 0.0, 0.0)],
                &[0.0.into(), 1.0.into()],
            )?;
            let mut constraint =
                ConstraintSet::distance(&context, &cache, "tether", &[[0, 1]], &[1.0], 0.0)?;
            let solver = Solver::with_cache(
                &context,
                &cache,
                SolverSettings {
                    substeps: native.into(),
                    gravity: Vec3::default(),
                    damping: 0.0,
                    ..Default::default()
                },
            )?;
            let mut hook = Observations {
                context: &context,
                trace: vec![],
                positions: vec![],
            };
            if matches!(execution, ExecutionMode::Interleaved) {
                solver.step_interleaved(
                    &context,
                    &particles,
                    &mut [&mut constraint],
                    &mut hook,
                    1.0 / 60.0,
                )?;
            } else {
                solver.step(
                    &context,
                    &particles,
                    &mut [&mut constraint],
                    &mut hook,
                    1.0 / 60.0,
                )?;
            }
            assert_eq!(hook.trace.len(), native as usize * 2);
            for pair in hook.trace.as_chunks::<2>().0 {
                let bits = ((1.0f32 / 60.0) / native as f32).to_bits();
                assert_eq!(
                    pair,
                    &[
                        HookObservation {
                            phase: HookPhase::BeforeSolve,
                            duration_bits: bits
                        },
                        HookObservation {
                            phase: HookPhase::AfterSolve,
                            duration_bits: bits
                        },
                    ]
                );
            }
            let mut snapshots = Vec::new();
            for buffer in &hook.positions {
                let values: Vec<f32> = buffer.read(&context).await?;
                snapshots.push(values.iter().map(|v| v.to_bits()).collect::<Vec<_>>());
            }
            let positions = particles.read_positions(&context).await?;
            if native == 0 {
                assert_eq!(positions[1].x.to_bits(), 2.0f32.to_bits());
            } else {
                assert_eq!(snapshots[0][4], 2.0f32.to_bits());
                assert_eq!(snapshots[1][4], 1.0f32.to_bits());
                assert_eq!(positions[1].x.to_bits(), 1.0f32.to_bits());
            }
            let velocities = particles.read_velocities(&context).await?;
            outcomes.push(Outcome {
                trace: hook.trace,
                snapshots,
                positions,
                velocities,
            });
        }
        assert_eq!(outcomes[0], outcomes[1]);
    }
    Ok(())
}
