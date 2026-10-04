use super::*;
use crate::{ParticleInverseMass, SubstepIndex};
use fabelgeist_math::Vec3;
use std::error::Error;

#[derive(Debug, PartialEq, Eq)]
struct HookStopped(SolverHookPhase);
impl std::fmt::Display for HookStopped {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "stopped at {:?}", self.0)
    }
}
impl Error for HookStopped {}

struct RecordingHook {
    calls: Vec<SolverHookPhase>,
    stop: Option<SolverHookPhase>,
}
impl RecordingHook {
    fn visit(&mut self, phase: SolverHookPhase) -> Result<(), HookStopped> {
        self.calls.push(phase);
        if self.stop == Some(phase) {
            Err(HookStopped(phase))
        } else {
            Ok(())
        }
    }
}
impl SubstepHook for RecordingHook {
    type Error = HookStopped;
    fn record(
        &mut self,
        _: &mut KernelBatch,
        _: &Particles,
        _: SubstepDuration,
    ) -> Result<(), Self::Error> {
        self.visit(SolverHookPhase::BeforeSolve)
    }
    fn after_solve(
        &mut self,
        _: &mut KernelBatch,
        _: &Particles,
        _: SubstepDuration,
    ) -> Result<(), Self::Error> {
        self.visit(SolverHookPhase::AfterSolve)
    }
}

#[tokio::test]
async fn hook_failures_retain_phase_substep_and_concrete_cause() {
    let context = WgpuContext::new().await.unwrap();
    let particles = Particles::from_positions(
        &context,
        &[Vec3::default()],
        &[ParticleInverseMass::UNIT_MASS],
    )
    .unwrap();
    let solver = Solver::new(
        &context,
        SolverSettings {
            substeps: 3.into(),
            ..Default::default()
        },
    )
    .unwrap();
    for phase in [SolverHookPhase::BeforeSolve, SolverHookPhase::AfterSolve] {
        let mut hook = RecordingHook {
            calls: Vec::new(),
            stop: Some(phase),
        };
        let error = solver
            .step(
                &context,
                &particles,
                &mut [],
                &mut hook,
                (1.0 / 60.0).into(),
            )
            .unwrap_err();
        assert_eq!(error.substep, SubstepIndex::FIRST);
        assert!(
            matches!(&error.source, SolverSubstepError::Hook {phase: actual, source} if *actual == phase && *source == HookStopped(phase))
        );
        assert!(error.source.source().unwrap().is::<HookStopped>());
        assert_eq!(hook.calls.last(), Some(&phase));
        assert_eq!(error.to_string(), format!("stopped at {phase:?}"));
    }
}

#[tokio::test]
async fn constraint_failures_keep_set_color_stage_and_provider_cause() {
    let context = WgpuContext::new().await.unwrap();
    let cache = KernelCache::new();
    let particles = Particles::from_positions(
        &context,
        &[Vec3::default(); 2],
        &[ParticleInverseMass::UNIT_MASS; 2],
    )
    .unwrap();
    let solver = Solver::with_cache(&context, &cache, SolverSettings::default()).unwrap();
    let kernel = cache
        .get(
            &context,
            &crate::wgsl::constraint_kernel(crate::wgsl::DISTANCE),
        )
        .unwrap();
    let incidence = crate::ConstraintIncidence::from_native_records(&[[0u32, 1]]).unwrap();
    let mut set = ConstraintSet::new(
        &context,
        "missing rest".into(),
        kernel,
        &cache,
        &incidence,
        0.0.into(),
    )
    .unwrap();
    let error = solver
        .step(
            &context,
            &particles,
            &mut [&mut set],
            &mut NoSubstepHook,
            (1.0 / 60.0).into(),
        )
        .unwrap_err();
    let SolverSubstepError::Constraint(source) = error.source else {
        panic!("wrong solver failure")
    };
    assert_eq!(source.set, crate::ConstraintName::from("missing rest"));
    assert!(matches!(
        source.stage,
        crate::ConstraintDispatchStage::Solve(_)
    ));
    assert!(
        source
            .source()
            .unwrap()
            .is::<fabelgeist_compute::KernelDispatchError>()
    );
}

#[tokio::test]
async fn ordered_composition_keeps_participant_errors_and_record_only_behavior() {
    let context = WgpuContext::new().await.unwrap();
    let particles = Particles::from_positions(
        &context,
        &[Vec3::default()],
        &[ParticleInverseMass::UNIT_MASS],
    )
    .unwrap();
    let mut batch = KernelBatch::new(&context);
    let mut first = RecordingHook {
        calls: Vec::new(),
        stop: Some(SolverHookPhase::BeforeSolve),
    };
    let mut second = RecordingHook {
        calls: Vec::new(),
        stop: None,
    };
    let error = HookChain::new(&mut first, &mut second)
        .record(&mut batch, &particles, 0.01.into())
        .unwrap_err();
    assert!(matches!(
        error,
        HookChainError::First(HookStopped(SolverHookPhase::BeforeSolve))
    ));
    assert!(second.calls.is_empty());
    first.stop = None;
    second.stop = Some(SolverHookPhase::BeforeSolve);
    let error = HookChain::new(&mut first, &mut second)
        .record(&mut batch, &particles, 0.01.into())
        .unwrap_err();
    assert!(matches!(
        error,
        HookChainError::Second(HookStopped(SolverHookPhase::BeforeSolve))
    ));
    assert!(error.source().unwrap().is::<HookStopped>());
    second.stop = None;
    let mut chain = HookChain::new(&mut first, &mut second);
    chain.record(&mut batch, &particles, 0.01.into()).unwrap();
    chain
        .after_solve(&mut batch, &particles, 0.01.into())
        .unwrap();
    for phase in first.calls.iter().chain(&second.calls) {
        assert_eq!(*phase, SolverHookPhase::BeforeSolve);
    }
}

#[tokio::test]
async fn selected_and_nested_participants_retain_heterogeneous_causes() {
    let context = WgpuContext::new().await.unwrap();
    let particles = Particles::from_positions(
        &context,
        &[Vec3::default()],
        &[ParticleInverseMass::UNIT_MASS],
    )
    .unwrap();
    let solver = Solver::new(&context, SolverSettings::default()).unwrap();
    let mut recording = RecordingHook {
        calls: Vec::new(),
        stop: Some(SolverHookPhase::BeforeSolve),
    };
    let mut idle_before = NoSubstepHook;
    let mut idle_after = NoSubstepHook;
    let selected: &mut dyn SubstepHook<Error = HookStopped> = &mut recording;
    let mut inner = HookChain::new(selected, &mut idle_after);
    let mut outer = HookChain::new(&mut idle_before, &mut inner);
    let selected: &mut dyn SubstepHook<Error = _> = &mut outer;
    let error = solver
        .step(&context, &particles, &mut [], selected, 0.01.into())
        .unwrap_err();
    assert_eq!(error.substep, SubstepIndex::FIRST);
    let SolverSubstepError::Hook { phase, source } = error.source else {
        panic!("nested participant lost its hook classification")
    };
    assert_eq!(phase, SolverHookPhase::BeforeSolve);
    assert!(matches!(
        source,
        HookChainError::Second(HookChainError::First(HookStopped(
            SolverHookPhase::BeforeSolve
        )))
    ));
    assert!(
        source
            .source()
            .unwrap()
            .source()
            .unwrap()
            .is::<HookStopped>()
    );
    assert_eq!(recording.calls, [SolverHookPhase::BeforeSolve]);
}

#[tokio::test]
async fn skipped_steps_never_invoke_hooks_but_direct_substeps_do() {
    let context = WgpuContext::new().await.unwrap();
    let particles = Particles::from_positions(
        &context,
        &[Vec3::default()],
        &[ParticleInverseMass::UNIT_MASS],
    )
    .unwrap();
    let mut solver = Solver::new(&context, SolverSettings::default()).unwrap();
    let mut hook = RecordingHook {
        calls: Vec::new(),
        stop: None,
    };
    for seconds in [0.0f32, -0.0, -1.0, f32::NEG_INFINITY] {
        solver
            .step(&context, &particles, &mut [], &mut hook, seconds.into())
            .unwrap();
    }
    solver.settings.substeps = SubstepCount::EMPTY;
    solver
        .step(&context, &particles, &mut [], &mut hook, 0.01.into())
        .unwrap();
    assert!(hook.calls.is_empty());
    let mut batch = KernelBatch::new(&context);
    solver
        .record_substep(&mut batch, &particles, &mut [], &mut hook, 0.0.into())
        .unwrap();
    assert_eq!(
        hook.calls,
        [SolverHookPhase::BeforeSolve, SolverHookPhase::AfterSolve]
    );
}
