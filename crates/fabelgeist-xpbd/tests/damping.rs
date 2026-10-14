use fabelgeist_gpu::prelude::{Result, WgpuContext};
use fabelgeist_math::Vec3;
use fabelgeist_xpbd::dynamics::DampingRate;
use fabelgeist_xpbd::{ParticleInverseMass, ParticleVelocities, Particles, Solver, SolverSettings};

struct Harness {
    context: WgpuContext,
}

impl Harness {
    async fn new() -> Result<Self> {
        Ok(Self {
            context: WgpuContext::new_compute().await?,
        })
    }

    async fn advance(&self, damping: DampingRate) -> Result<Vec<Vec3>> {
        // Metre positions and metre-per-second velocities in solver space.
        let particles = Particles::from_positions(
            &self.context,
            &[Vec3::default(), Vec3::new(1.0, 0.0, 0.0)],
            &[ParticleInverseMass::UNIT_MASS, ParticleInverseMass::PINNED],
        )?;
        let velocities = ParticleVelocities::from([Vec3::new(1.0, 0.0, 0.0); 2].as_slice());
        particles
            .velocities
            .write(&self.context, velocities.upload())?;
        let solver = Solver::new(
            &self.context,
            SolverSettings {
                substeps: 1,
                damping,
                gravity: Vec3::default(),
                ..Default::default()
            },
        )?;
        // One authored 32 Hz step, passed to the existing native duration API.
        solver.step(&self.context, &particles, &mut [], &mut (), 1.0 / 32.0)?;
        particles.read_positions(&self.context).await
    }
}

#[test]
fn native_admission_preserves_storage_words_and_scalar_debug() {
    // Native IEEE-754 fixture words include both zero signs and NaN payloads.
    for word in [
        0,
        0x8000_0000,
        1,
        0x8000_0001,
        0x3dcc_cccd,
        0x3f19_999a,
        0xbf80_0000,
        0x41a0_0000,
        0x7f7f_ffff,
        0xff7f_ffff,
        0x7f80_0000,
        0xff80_0000,
        0x7fc1_2345,
        0xffc5_4321,
    ] {
        let native = f32::from_bits(word);
        let rate = DampingRate::per_second(native);
        let decoded = DampingRate::from(native);
        assert_eq!(f32::from(rate).to_bits(), word);
        assert_eq!(f32::from(decoded).to_bits(), word);
        assert_eq!(format!("{rate:?}"), format!("{native:?}"));
        assert_eq!(format!("{rate:.3?}"), format!("{native:.3?}"));
        assert_eq!(rate.is_finite(), native.is_finite());
    }
}

#[test]
fn comparison_retains_native_zero_and_nan_behavior() {
    let positive_zero = DampingRate::per_second(0.0);
    let negative_zero = DampingRate::per_second(-0.0);
    assert_eq!(positive_zero, negative_zero);
    assert_eq!(f32::from(negative_zero).to_bits(), 0x8000_0000);
    assert!(DampingRate::per_second(-1.0) < positive_zero);
    let nan = DampingRate::from(f32::from_bits(0x7fc1_2345));
    assert_ne!(nan, nan);
    assert_eq!(nan.partial_cmp(&positive_zero), None);
}

#[tokio::test]
async fn positive_drag_reduces_motion_and_negative_drag_amplifies_it() -> Result<()> {
    let harness = Harness::new().await?;
    let zero = harness.advance(DampingRate::per_second(0.0)).await?;
    let positive = harness.advance(DampingRate::per_second(2.0)).await?;
    let negative = harness.advance(DampingRate::per_second(-1.0)).await?;
    assert_eq!(zero[0].x, 1.0 / 32.0);
    assert!(positive[0].x < zero[0].x);
    assert!(negative[0].x > zero[0].x);
    for positions in [zero, positive, negative] {
        assert_eq!(positions.len(), 2);
        assert_eq!(positions[1], Vec3::new(1.0, 0.0, 0.0));
    }
    Ok(())
}
