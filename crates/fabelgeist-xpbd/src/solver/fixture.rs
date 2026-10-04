//! Independent original GPU and callback snapshots across submission policies.
use super::*;
use crate::{ConstraintEdges, ParticleInverseMass};
use fabelgeist_math::Vec3;
use std::convert::Infallible;

#[derive(Clone, Copy)]
enum Case {
    Ordinary,
    NoSubsteps,
    Zero,
    NegativeZero,
    Negative,
    Nan,
    PositiveInfinity,
    NegativeInfinity,
    NoSweeps,
    ThreeSweeps,
    Empty,
    Subnormal,
}
#[derive(Clone, Copy)]
enum Submission {
    Whole,
    Interleaved,
    Direct,
}
#[derive(Clone, Copy)]
enum Phase {
    Before,
    After,
}
struct Hook {
    calls: Vec<(Phase, SubstepDuration)>,
}
impl SubstepHook for Hook {
    type Error = Infallible;
    fn record(
        &mut self,
        _: &mut KernelBatch,
        _: &Particles,
        interval: SubstepDuration,
    ) -> Result<(), Self::Error> {
        self.calls.push((Phase::Before, interval));
        Ok(())
    }
    fn after_solve(
        &mut self,
        _: &mut KernelBatch,
        _: &Particles,
        interval: SubstepDuration,
    ) -> Result<(), Self::Error> {
        self.calls.push((Phase::After, interval));
        Ok(())
    }
}
struct Run {
    particles: Particles,
    constraint: ConstraintSet,
    solver: Solver,
    delta: StepDuration,
    direct: SubstepDuration,
    case: Case,
}
impl Run {
    fn new(context: &WgpuContext, cache: &KernelCache, case: Case) -> Self {
        let seconds = match case {
            Case::Zero => 0.0,
            Case::NegativeZero => -0.0,
            Case::Negative => -1.0 / 60.0,
            Case::Nan => f32::from_bits(0x7fc01234),
            Case::PositiveInfinity => f32::INFINITY,
            Case::NegativeInfinity => f32::NEG_INFINITY,
            Case::Subnormal => f32::from_bits(1),
            _ => 1.0 / 60.0,
        };
        let (positions, masses) = match case {
            Case::Empty => (vec![], vec![]),
            _ => (
                vec![Vec3::new(0.0, 0.1, 0.0), Vec3::new(0.2, 0.2, 0.0)],
                vec![ParticleInverseMass::UNIT_MASS, ParticleInverseMass::PINNED],
            ),
        };
        let particles = Particles::from_positions(context, &positions, &masses).unwrap();
        let constraint = ConstraintSet::distance(
            context,
            cache,
            "solver original fixture".into(),
            &ConstraintEdges::from([[0u32, 1]].as_slice()),
            &[0.1],
            0.0001.into(),
        )
        .unwrap();
        let substeps = match case {
            Case::NoSubsteps => SubstepCount::EMPTY,
            Case::Subnormal => 4.into(),
            _ => 2.into(),
        };
        let iterations = match case {
            Case::NoSweeps => 0.into(),
            Case::ThreeSweeps => 3.into(),
            _ => 1.into(),
        };
        let solver = Solver::with_cache(
            context,
            cache,
            SolverSettings {
                substeps,
                iterations,
                ..Default::default()
            },
        )
        .unwrap();
        Self {
            particles,
            constraint,
            solver,
            delta: seconds.into(),
            direct: seconds.into(),
            case,
        }
    }
    fn submit(&mut self, context: &WgpuContext, submission: Submission) -> Hook {
        let mut hook = Hook { calls: Vec::new() };
        let mut constraints = match self.case {
            Case::Empty => vec![],
            _ => vec![&mut self.constraint],
        };
        match submission {
            Submission::Whole => self
                .solver
                .step(
                    context,
                    &self.particles,
                    &mut constraints,
                    &mut hook,
                    self.delta,
                )
                .unwrap(),
            Submission::Interleaved => self
                .solver
                .step_interleaved(
                    context,
                    &self.particles,
                    &mut constraints,
                    &mut hook,
                    self.delta,
                )
                .unwrap(),
            Submission::Direct => {
                let mut batch =
                    KernelBatch::labelled(context, "solver direct original fixture".into());
                self.solver
                    .record_substep(
                        &mut batch,
                        &self.particles,
                        &mut constraints,
                        &mut hook,
                        self.direct,
                    )
                    .unwrap();
                batch.submit();
            }
        }
        hook
    }
}
struct State(Vec<u8>);
impl State {
    async fn capture(&mut self, context: &WgpuContext, particles: &Particles, hook: Hook) {
        for buffer in [
            &particles.positions,
            &particles.previous,
            &particles.velocities,
        ] {
            let bytes: Vec<u8> = buffer.read(context).await.unwrap();
            self.0.extend(bytes);
        }
        for (phase, interval) in hook.calls {
            let phase: u32 = match phase {
                Phase::Before => 0,
                Phase::After => 1,
            };
            self.0.extend(phase.to_le_bytes());
            self.0.extend(u32::from(interval).to_le_bytes());
        }
    }
}
#[tokio::test]
async fn preserves_original_solver_and_callback_words_across_submission_policies() {
    let context = WgpuContext::new().await.unwrap();
    let cache = KernelCache::new();
    let mut state = State(Vec::new());
    for case in [
        Case::Ordinary,
        Case::NoSubsteps,
        Case::Zero,
        Case::NegativeZero,
        Case::Negative,
        Case::Nan,
        Case::PositiveInfinity,
        Case::NegativeInfinity,
        Case::NoSweeps,
        Case::ThreeSweeps,
        Case::Empty,
        Case::Subnormal,
    ] {
        for submission in [
            Submission::Whole,
            Submission::Interleaved,
            Submission::Direct,
        ] {
            let mut run = Run::new(&context, &cache, case);
            let hook = run.submit(&context, submission);
            state.capture(&context, &run.particles, hook).await;
        }
    }
    assert_eq!(
        state.0.as_slice(),
        include_bytes!("../../tests/fixtures/solver.bin")
    );
}
