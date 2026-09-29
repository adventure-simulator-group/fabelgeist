//! The solver, checked against a host implementation of the same equations
//! and against physics that can be worked out on paper.

use fabelgeist_compute::prelude::*;
use fabelgeist_gpu::prelude::*;
use fabelgeist_math::Vec3;

use crate::{ConstraintSet, Particles, Solver, SolverSettings};

/// The XPBD distance projection, written out on the host.
///
/// Deliberately a separate implementation rather than a call into the same
/// code: an oracle that shares the code under test agrees with it about its
/// bugs too. Colours are applied in the same order the GPU dispatches them, so
/// the two should agree to floating-point noise, not merely in spirit.
struct Reference {
    positions: Vec<Vec3>,
    previous: Vec<Vec3>,
    velocities: Vec<Vec3>,
    inverse_masses: Vec<f32>,
}

struct ReferenceConstraints {
    /// Particle pairs, in colour order.
    edges: Vec<[u32; 2]>,
    rest_lengths: Vec<f32>,
    ranges: Vec<u32>,
    compliance: f32,
}

impl Reference {
    fn step(
        &mut self,
        constraints: &mut ReferenceConstraints,
        settings: &SolverSettings,
        delta: f32,
    ) {
        let substep = delta / settings.substeps as f32;

        for _ in 0..settings.substeps {
            for index in 0..self.positions.len() {
                self.previous[index] = self.positions[index];
                if self.inverse_masses[index] == 0.0 {
                    self.velocities[index] = Vec3::default();
                    continue;
                }
                let mut velocity = self.velocities[index] + settings.gravity * substep;
                velocity = velocity * (-settings.damping * substep).exp();
                let speed = velocity.length();
                if speed > settings.max_speed && speed > 0.0 {
                    velocity = velocity * (settings.max_speed / speed);
                }
                self.velocities[index] = velocity;
                self.positions[index] += velocity * substep;
            }

            let mut lambdas = vec![0.0f32; constraints.edges.len()];
            for _ in 0..settings.iterations.max(1) {
                for color in 0..constraints.ranges.len() - 1 {
                    let range =
                        constraints.ranges[color] as usize..constraints.ranges[color + 1] as usize;
                    for c in range {
                        let [a, b] = constraints.edges[c].map(|i| i as usize);
                        let inverse_a = self.inverse_masses[a];
                        let inverse_b = self.inverse_masses[b];

                        let delta_vector = self.positions[a] - self.positions[b];
                        let distance = delta_vector.length();
                        if distance < 1e-9 {
                            continue;
                        }
                        let normal = delta_vector / distance;
                        let value = distance - constraints.rest_lengths[c];
                        let alpha_tilde = constraints.compliance / (substep * substep);
                        let denominator = inverse_a + inverse_b + alpha_tilde;
                        if denominator < 1e-12 {
                            continue;
                        }
                        let delta_lambda = (-value - alpha_tilde * lambdas[c]) / denominator;
                        lambdas[c] += delta_lambda;
                        self.positions[a] += normal * (inverse_a * delta_lambda);
                        self.positions[b] += normal * (-inverse_b * delta_lambda);
                    }
                }
            }

            for index in 0..self.positions.len() {
                if self.inverse_masses[index] == 0.0 {
                    self.velocities[index] = Vec3::default();
                } else {
                    self.velocities[index] =
                        (self.positions[index] - self.previous[index]) / substep;
                }
            }
        }
    }
}

/// A hanging chain: particle 0 pinned, the rest strung below it.
fn chain(links: usize, spacing: f32) -> (Vec<Vec3>, Vec<f32>, Vec<[u32; 2]>, Vec<f32>) {
    let positions: Vec<Vec3> = (0..=links)
        .map(|i| Vec3::new(i as f32 * spacing, 0.0, 0.0))
        .collect();
    let mut inverse_masses = vec![1.0f32; positions.len()];
    inverse_masses[0] = 0.0;
    let edges: Vec<[u32; 2]> = (0..links).map(|i| [i as u32, i as u32 + 1]).collect();
    let rest_lengths = vec![spacing; links];
    (positions, inverse_masses, edges, rest_lengths)
}

async fn run(
    positions: &[Vec3],
    inverse_masses: &[f32],
    edges: &[[u32; 2]],
    rest_lengths: &[f32],
    compliance: f32,
    settings: SolverSettings,
    steps: usize,
    delta: f32,
) -> Result<(Vec<Vec3>, Vec<Vec3>)> {
    let context = WgpuContext::new().await?;
    let cache = KernelCache::new();

    let mut particles = Particles::from_positions(&context, positions, inverse_masses)?;
    let mut set = ConstraintSet::distance(
        &context,
        &cache,
        "distance",
        edges,
        rest_lengths,
        compliance,
    )?;
    let solver = Solver::with_cache(&context, &cache, settings)?;

    for _ in 0..steps {
        solver.step(&context, &particles, &mut [&mut set], &mut (), delta)?;
    }

    let gpu_positions = particles.read_positions(&context).await?;
    let gpu_velocities = particles.read_velocities(&context).await?;
    let _ = &mut particles;

    // The same run on the host, using the colour order the set chose.
    let coloring = set.coloring();
    let ordered_edges: Vec<[u32; 2]> = coloring.order.iter().map(|&i| edges[i as usize]).collect();
    let ordered_rest: Vec<f32> = coloring
        .order
        .iter()
        .map(|&i| rest_lengths[i as usize])
        .collect();

    let mut reference = Reference {
        positions: positions.to_vec(),
        previous: positions.to_vec(),
        velocities: vec![Vec3::default(); positions.len()],
        inverse_masses: inverse_masses.to_vec(),
    };
    let mut reference_constraints = ReferenceConstraints {
        edges: ordered_edges,
        rest_lengths: ordered_rest,
        ranges: coloring.ranges.clone(),
        compliance,
    };
    for _ in 0..steps {
        reference.step(&mut reference_constraints, &settings, delta);
    }

    let _ = gpu_velocities;
    Ok((gpu_positions, reference.positions))
}

fn assert_close(gpu: &[Vec3], host: &[Vec3], tolerance: f32) {
    assert_eq!(gpu.len(), host.len());
    for (index, (a, b)) in gpu.iter().zip(host).enumerate() {
        let error = (*a - *b).length();
        assert!(
            error < tolerance,
            "particle {index}: GPU {a} vs host {b}, off by {error}"
        );
    }
}

#[tokio::test]
async fn matches_the_host_solver_on_a_chain() -> Result<()> {
    let (positions, inverse_masses, edges, rest_lengths) = chain(40, 0.05);
    let settings = SolverSettings {
        substeps: 8,
        ..Default::default()
    };
    let (gpu, host) = run(
        &positions,
        &inverse_masses,
        &edges,
        &rest_lengths,
        0.0,
        settings,
        20,
        1.0 / 60.0,
    )
    .await?;
    // Twenty steps of eight substeps is 160 sweeps of accumulated
    // floating-point difference; a millimetre over a two-metre chain is noise.
    assert_close(&gpu, &host, 1e-3);
    Ok(())
}

/// Non-zero compliance is the whole point of XPBD, so it gets its own check
/// against the host rather than riding on the stiff case.
#[tokio::test]
async fn matches_the_host_solver_with_compliance() -> Result<()> {
    let (positions, inverse_masses, edges, rest_lengths) = chain(24, 0.08);
    let settings = SolverSettings {
        substeps: 12,
        ..Default::default()
    };
    let (gpu, host) = run(
        &positions,
        &inverse_masses,
        &edges,
        &rest_lengths,
        1e-6,
        settings,
        15,
        1.0 / 60.0,
    )
    .await?;
    assert_close(&gpu, &host, 1e-3);
    Ok(())
}

/// The accumulated multiplier only does anything when a substep runs more than
/// one sweep -- with a single sweep `lambda` is still zero when it is read, and
/// deleting the term outright changes nothing. So this runs four sweeps per
/// substep, which is where XPBD and plain PBD part company, and it is the only
/// test in this file that fails if `xpbd_solve` loses its `alpha_tilde *
/// lambda` term. (Checked by breaking it on purpose: every other test here,
/// including the settled-length ones, passes without it.)
#[tokio::test]
async fn matches_the_host_solver_across_repeated_sweeps() -> Result<()> {
    let (positions, inverse_masses, edges, rest_lengths) = chain(24, 0.08);
    let settings = SolverSettings {
        substeps: 6,
        iterations: 4,
        ..Default::default()
    };
    let (gpu, host) = run(
        &positions,
        &inverse_masses,
        &edges,
        &rest_lengths,
        1e-5,
        settings,
        15,
        1.0 / 60.0,
    )
    .await?;
    assert_close(&gpu, &host, 1e-3);
    Ok(())
}

/// A pinned particle has zero inverse mass, and every correction is scaled by
/// it. If that ever stops holding, a garment falls off the body.
#[tokio::test]
async fn pinned_particles_never_move() -> Result<()> {
    let (positions, inverse_masses, edges, rest_lengths) = chain(30, 0.05);
    let (gpu, _) = run(
        &positions,
        &inverse_masses,
        &edges,
        &rest_lengths,
        0.0,
        SolverSettings::default(),
        30,
        1.0 / 60.0,
    )
    .await?;
    assert!(
        (gpu[0] - positions[0]).length() < 1e-6,
        "the pinned particle moved to {}",
        gpu[0]
    );
    Ok(())
}

/// Free fall: no constraints, so after `t` seconds everything should have
/// dropped `g t^2 / 2`, up to the damping.
#[tokio::test]
async fn free_fall_matches_the_analytic_drop() -> Result<()> {
    let context = WgpuContext::new().await?;
    let cache = KernelCache::new();

    let positions = vec![Vec3::default(); 100];
    let particles = Particles::from_positions(&context, &positions, &vec![1.0; positions.len()])?;
    let settings = SolverSettings {
        substeps: 20,
        damping: 0.0,
        ..Default::default()
    };
    let solver = Solver::with_cache(&context, &cache, settings)?;

    let steps = 60;
    let delta = 1.0f32 / 60.0;
    for _ in 0..steps {
        solver.step(&context, &particles, &mut [], &mut (), delta)?;
    }

    let after = particles.read_positions(&context).await?;
    let elapsed = steps as f32 * delta;
    let analytic = 0.5 * settings.gravity.y * elapsed * elapsed;
    // Symplectic Euler overshoots the closed form by half a substep of
    // velocity each step; over a second at twenty substeps that is about half
    // a percent.
    let error = (after[0].y - analytic).abs() / analytic.abs();
    assert!(
        error < 0.02,
        "fell to {} where the closed form says {analytic}",
        after[0].y
    );
    // And the velocity is g t.
    let velocities = particles.read_velocities(&context).await?;
    assert!(
        (velocities[0].y - settings.gravity.y * elapsed).abs() < 0.2,
        "velocity {} is not near {}",
        velocities[0].y,
        settings.gravity.y * elapsed
    );
    Ok(())
}

/// A stiff chain hanging under gravity settles at its rest length. This is the
/// property a garment depends on: seams that hold and panels that do not
/// stretch into nonsense.
#[tokio::test]
async fn a_stiff_chain_holds_its_length() -> Result<()> {
    let links = 30;
    let spacing = 0.05f32;
    let (positions, inverse_masses, edges, rest_lengths) = chain(links, spacing);

    let context = WgpuContext::new().await?;
    let cache = KernelCache::new();
    let particles = Particles::from_positions(&context, &positions, &inverse_masses)?;
    let mut set =
        ConstraintSet::distance(&context, &cache, "distance", &edges, &rest_lengths, 0.0)?;
    let solver = Solver::with_cache(
        &context,
        &cache,
        SolverSettings {
            substeps: 20,
            damping: 2.0,
            ..Default::default()
        },
    )?;

    for _ in 0..300 {
        solver.step(&context, &particles, &mut [&mut set], &mut (), 1.0 / 60.0)?;
    }

    let settled = particles.read_positions(&context).await?;
    for (index, edge) in edges.iter().enumerate() {
        let length = (settled[edge[0] as usize] - settled[edge[1] as usize]).length();
        assert!(
            (length - spacing).abs() < spacing * 0.05,
            "link {index} is {length} long, not {spacing}"
        );
    }

    // And it hangs down, rather than staying out along x where it started.
    let free_end = settled[links];
    assert!(
        free_end.y < -spacing * links as f32 * 0.8,
        "the chain did not fall: the free end is at {free_end}"
    );
    Ok(())
}

/// Compliance has to *mean* something: a softer fabric must stretch further
/// under the same load, and a stiffer one less.
#[tokio::test]
async fn compliance_orders_the_stretch() -> Result<()> {
    let context = WgpuContext::new().await?;
    let cache = KernelCache::new();
    let links = 20;
    let spacing = 0.05f32;

    let mut lengths = Vec::new();
    for compliance in [0.0f32, 1e-6, 1e-4] {
        let (positions, inverse_masses, edges, rest_lengths) = chain(links, spacing);
        let particles = Particles::from_positions(&context, &positions, &inverse_masses)?;
        let mut set = ConstraintSet::distance(
            &context,
            &cache,
            "distance",
            &edges,
            &rest_lengths,
            compliance,
        )?;
        let solver = Solver::with_cache(
            &context,
            &cache,
            SolverSettings {
                substeps: 20,
                damping: 3.0,
                ..Default::default()
            },
        )?;
        for _ in 0..400 {
            solver.step(&context, &particles, &mut [&mut set], &mut (), 1.0 / 60.0)?;
        }
        let settled = particles.read_positions(&context).await?;
        // Total length of the hanging chain once it has come to rest.
        let total: f32 = edges
            .iter()
            .map(|e| (settled[e[0] as usize] - settled[e[1] as usize]).length())
            .sum();
        lengths.push(total);
    }

    let rest = spacing * links as f32;
    assert!(
        lengths[0] < lengths[1] && lengths[1] < lengths[2],
        "stretch is not ordered by compliance: {lengths:?}"
    );
    assert!(
        (lengths[0] - rest).abs() < rest * 0.02,
        "the stiff chain stretched to {} from {rest}",
        lengths[0]
    );
    Ok(())
}

/// Colouring is what makes the parallel dispatch safe. A grid where every
/// interior particle carries four constraints is where a missing colour would
/// show up as a race.
#[tokio::test]
async fn a_colored_grid_solves_without_racing() -> Result<()> {
    let (width, height) = (24u32, 24u32);
    let spacing = 0.04f32;
    let index = |x: u32, y: u32| y * width + x;

    let mut positions = Vec::new();
    let mut inverse_masses = Vec::new();
    for y in 0..height {
        for x in 0..width {
            positions.push(Vec3::new(x as f32 * spacing, 0.0, y as f32 * spacing));
            // The top row is pinned, so the sheet hangs.
            inverse_masses.push(if y == 0 { 0.0 } else { 1.0 });
        }
    }

    let mut edges = Vec::new();
    let mut rest_lengths = Vec::new();
    for y in 0..height {
        for x in 0..width {
            if x + 1 < width {
                edges.push([index(x, y), index(x + 1, y)]);
                rest_lengths.push(spacing);
            }
            if y + 1 < height {
                edges.push([index(x, y), index(x, y + 1)]);
                rest_lengths.push(spacing);
            }
        }
    }

    let context = WgpuContext::new().await?;
    let cache = KernelCache::new();
    let particles = Particles::from_positions(&context, &positions, &inverse_masses)?;
    let mut set = ConstraintSet::distance(&context, &cache, "stretch", &edges, &rest_lengths, 0.0)?;
    assert!(
        set.color_count() >= 4,
        "a grid needs at least four colours, got {}",
        set.color_count()
    );

    let solver = Solver::with_cache(
        &context,
        &cache,
        SolverSettings {
            substeps: 15,
            damping: 2.0,
            ..Default::default()
        },
    )?;
    for _ in 0..200 {
        solver.step(&context, &particles, &mut [&mut set], &mut (), 1.0 / 60.0)?;
    }

    let settled = particles.read_positions(&context).await?;
    assert!(
        settled.iter().all(|p| p.is_finite()),
        "the solve produced a non-finite position"
    );
    // Under a race, some edges keep only one of two competing corrections and
    // end up badly stretched. Every edge staying near its rest length is the
    // evidence that the colours held.
    let worst = edges
        .iter()
        .zip(&rest_lengths)
        .map(|(e, &rest)| {
            ((settled[e[0] as usize] - settled[e[1] as usize]).length() - rest).abs() / rest
        })
        .fold(0.0f32, f32::max);
    assert!(
        worst < 0.15,
        "worst edge is {:.1}% off its rest length",
        worst * 100.0
    );
    Ok(())
}

#[tokio::test]
async fn rejects_mismatched_inputs() -> Result<()> {
    let context = WgpuContext::new().await?;
    let cache = KernelCache::new();

    assert!(
        ConstraintSet::distance(&context, &cache, "bad", &[[0, 1]], &[1.0, 2.0], 0.0).is_err(),
        "one edge with two rest lengths must be rejected"
    );

    let mut particles = Particles::new(&context, 4)?;
    assert!(
        particles
            .write(&context, &[Vec3::default(); 2], &[1.0])
            .is_err(),
        "two positions with one inverse mass must be rejected"
    );
    assert!(
        particles
            .write(&context, &[Vec3::default(); 8], &[1.0; 8])
            .is_err(),
        "eight particles must not fit a capacity of four"
    );
    Ok(())
}

/// An empty set is a no-op rather than an error: a garment with no seams is a
/// perfectly ordinary garment.
#[tokio::test]
async fn handles_empty_constraint_sets() -> Result<()> {
    let context = WgpuContext::new().await?;
    let cache = KernelCache::new();
    let particles = Particles::from_positions(&context, &[Vec3::default()], &[1.0])?;
    let mut set = ConstraintSet::distance(&context, &cache, "none", &[], &[], 0.0)?;
    assert_eq!(set.constraint_count(), 0);

    let solver = Solver::with_cache(&context, &cache, SolverSettings::default())?;
    solver.step(&context, &particles, &mut [&mut set], &mut (), 1.0 / 60.0)?;
    Ok(())
}

/// The substep hook is where collisions go, so it has to actually run, and run
/// once per substep.
#[tokio::test]
async fn the_substep_hook_runs_every_substep() -> Result<()> {
    let context = WgpuContext::new().await?;
    let cache = KernelCache::new();
    let particles = Particles::from_positions(&context, &[Vec3::default()], &[1.0])?;
    let settings = SolverSettings {
        substeps: 7,
        ..Default::default()
    };
    let solver = Solver::with_cache(&context, &cache, settings)?;

    let mut calls = 0usize;
    let mut seen_substep = 0.0f32;
    {
        let mut hook = |_: &mut KernelBatch, _: &Particles, substep: f32| {
            calls += 1;
            seen_substep = substep;
            Ok(())
        };
        solver.step(&context, &particles, &mut [], &mut hook, 1.0 / 60.0)?;
    }
    assert_eq!(calls, 7);
    assert!((seen_substep - (1.0 / 60.0) / 7.0).abs() < 1e-9);
    Ok(())
}
