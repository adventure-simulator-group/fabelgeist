//! Collision, checked against geometry that can be worked out on paper.

use fabelgeist_bvh::gpu::BvhKernels;
use fabelgeist_compute::prelude::*;
use fabelgeist_gpu::prelude::*;
use fabelgeist_math::Vec3;
use fabelgeist_xpbd::{Particles, Solver, SolverSettings};

use crate::collider::{Collider, Shape};
use crate::mesh::{MeshCollider, MeshSurface};
use crate::{Collisions, HookChain};

/// A UV sphere, and the analytic surface it approximates.
fn sphere_mesh(rings: usize, segments: usize, radius: f32) -> (Vec<Vec3>, Vec<[u32; 3]>) {
    let mut positions = Vec::new();
    for ring in 0..=rings {
        let phi = std::f32::consts::PI * ring as f32 / rings as f32;
        for segment in 0..segments {
            let theta = std::f32::consts::TAU * segment as f32 / segments as f32;
            positions.push(Vec3::new(
                radius * phi.sin() * theta.cos(),
                radius * phi.cos(),
                radius * phi.sin() * theta.sin(),
            ));
        }
    }
    let mut triangles = Vec::new();
    for ring in 0..rings {
        for segment in 0..segments {
            let next = (segment + 1) % segments;
            let a = (ring * segments + segment) as u32;
            let b = (ring * segments + next) as u32;
            let c = ((ring + 1) * segments + segment) as u32;
            let d = ((ring + 1) * segments + next) as u32;
            // Wound so the face normals point outwards, which is what the
            // resolve kernel uses to decide which side a particle is on.
            triangles.push([a, b, c]);
            triangles.push([b, d, c]);
        }
    }
    (positions, triangles)
}

// ----- the host mirror of the analytic shapes -----

#[test]
fn plane_distance_is_signed() {
    let collider = Collider::ground(0.0);
    let (distance, normal) = collider.signed_distance(Vec3::new(3.0, 2.0, -1.0));
    assert_eq!(distance, 2.0);
    assert_eq!(normal, Vec3::new(0.0, 1.0, 0.0));

    let (below, _) = collider.signed_distance(Vec3::new(0.0, -0.5, 0.0));
    assert_eq!(below, -0.5);
}

#[test]
fn sphere_distance_is_signed() {
    let collider = Collider::sphere(Vec3::default(), 2.0);
    let (outside, normal) = collider.signed_distance(Vec3::new(5.0, 0.0, 0.0));
    assert_eq!(outside, 3.0);
    assert_eq!(normal, Vec3::new(1.0, 0.0, 0.0));

    let (inside, _) = collider.signed_distance(Vec3::new(1.0, 0.0, 0.0));
    assert_eq!(inside, -1.0);

    // Dead centre has no outward direction; it must still be finite.
    let (_, degenerate) = collider.signed_distance(Vec3::default());
    assert!(degenerate.is_finite());
}

#[test]
fn capsule_distance_covers_the_caps_and_the_shaft() {
    let collider = Collider::capsule(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 2.0, 0.0), 0.5);

    // Beside the shaft.
    let (shaft, normal) = collider.signed_distance(Vec3::new(1.5, 1.0, 0.0));
    assert!((shaft - 1.0).abs() < 1e-6);
    assert!((normal - Vec3::new(1.0, 0.0, 0.0)).length() < 1e-6);

    // Past the top cap: measured from the endpoint, not from the axis.
    let (cap, _) = collider.signed_distance(Vec3::new(0.0, 4.0, 0.0));
    assert!((cap - 1.5).abs() < 1e-6);

    // Inside the shaft.
    let (inside, _) = collider.signed_distance(Vec3::new(0.1, 1.0, 0.0));
    assert!((inside - (-0.4)).abs() < 1e-6);

    // A capsule with coincident endpoints is a sphere, not a division by zero.
    let degenerate = Collider::capsule(Vec3::default(), Vec3::default(), 1.0);
    let (distance, direction) = degenerate.signed_distance(Vec3::new(3.0, 0.0, 0.0));
    assert!((distance - 2.0).abs() < 1e-6);
    assert!(direction.is_finite());
}

#[test]
fn box_distance_covers_faces_edges_and_the_interior() {
    let collider = Collider::new(Shape::Box {
        center: Vec3::default(),
        half_extents: Vec3::new(1.0, 2.0, 3.0),
        rotation: [0.0, 0.0, 0.0, 1.0],
    });

    // Straight out from a face.
    let (face, normal) = collider.signed_distance(Vec3::new(3.0, 0.0, 0.0));
    assert!((face - 2.0).abs() < 1e-6);
    assert!((normal - Vec3::new(1.0, 0.0, 0.0)).length() < 1e-6);

    // Off a corner: the distance is the diagonal, not any one axis.
    let (corner, _) = collider.signed_distance(Vec3::new(2.0, 3.0, 4.0));
    assert!((corner - 3.0f32.sqrt()).abs() < 1e-5);

    // Inside, the nearest face is the one with the least clearance.
    let (inside, inside_normal) = collider.signed_distance(Vec3::new(0.9, 0.0, 0.0));
    assert!((inside - (-0.1)).abs() < 1e-6);
    assert!((inside_normal - Vec3::new(1.0, 0.0, 0.0)).length() < 1e-6);

    // A quarter turn about z swaps which extent faces x.
    let rotated = Collider::new(Shape::Box {
        center: Vec3::default(),
        half_extents: Vec3::new(1.0, 2.0, 3.0),
        rotation: [
            0.0,
            0.0,
            (std::f32::consts::FRAC_PI_4).sin(),
            (std::f32::consts::FRAC_PI_4).cos(),
        ],
    });
    let (turned, _) = rotated.signed_distance(Vec3::new(3.0, 0.0, 0.0));
    assert!((turned - 1.0).abs() < 1e-4, "expected 1.0, got {turned}");
}

// ----- and the GPU response -----

struct Harness {
    context: WgpuContext,
    cache: KernelCache,
}

impl Harness {
    async fn new() -> Result<Self> {
        Ok(Self {
            context: WgpuContext::new().await?,
            cache: KernelCache::new(),
        })
    }

    fn solver(&self, settings: SolverSettings) -> Result<Solver> {
        Solver::with_cache(&self.context, &self.cache, settings)
    }

    fn collisions(&self) -> Result<Collisions> {
        Collisions::new(&self.context, &self.cache)
    }
}

fn settings(substeps: u32) -> SolverSettings {
    SolverSettings {
        substeps,
        damping: 1.0,
        ..Default::default()
    }
}

#[tokio::test]
async fn particles_settle_on_the_ground() -> Result<()> {
    let harness = Harness::new().await?;
    let positions: Vec<Vec3> = (0..64)
        .map(|i| Vec3::new(i as f32 * 0.01, 1.0, 0.0))
        .collect();
    let particles =
        Particles::from_positions(&harness.context, &positions, &vec![1.0; positions.len()])?;

    let mut collisions = harness.collisions()?;
    collisions.set_colliders(
        &harness.context,
        vec![Collider::ground(0.0).with_thickness(0.01)],
    )?;

    let solver = harness.solver(settings(10))?;
    for _ in 0..180 {
        solver.step(
            &harness.context,
            &particles,
            &mut [],
            &mut collisions,
            1.0 / 60.0,
        )?;
    }

    let settled = particles.read_positions(&harness.context).await?;
    for (index, position) in settled.iter().enumerate() {
        assert!(
            position.y >= -1e-3,
            "particle {index} fell through the ground to {}",
            position.y
        );
        assert!(
            (position.y - 0.01).abs() < 2e-3,
            "particle {index} rests at {} rather than on the 0.01 shell",
            position.y
        );
    }
    Ok(())
}

#[tokio::test]
async fn particles_stay_outside_a_sphere() -> Result<()> {
    let harness = Harness::new().await?;
    let radius = 0.5f32;

    // A grid dropped straight onto the top of a sphere.
    let mut positions = Vec::new();
    for x in -8..=8 {
        for z in -8..=8 {
            positions.push(Vec3::new(x as f32 * 0.05, 1.5, z as f32 * 0.05));
        }
    }
    let particles =
        Particles::from_positions(&harness.context, &positions, &vec![1.0; positions.len()])?;

    let mut collisions = harness.collisions()?;
    collisions.set_colliders(
        &harness.context,
        vec![
            Collider::sphere(Vec3::default(), radius).with_thickness(0.005),
            Collider::ground(-1.0),
        ],
    )?;

    let solver = harness.solver(settings(15))?;
    for _ in 0..150 {
        solver.step(
            &harness.context,
            &particles,
            &mut [],
            &mut collisions,
            1.0 / 60.0,
        )?;
    }

    let settled = particles.read_positions(&harness.context).await?;
    for (index, position) in settled.iter().enumerate() {
        let distance = position.length();
        assert!(
            distance >= radius - 1e-3 || position.y < -0.9,
            "particle {index} is inside the sphere at {distance} from the centre"
        );
    }
    Ok(())
}

/// Friction is the difference between a garment that stays on a shoulder and
/// one that slides off it. On a slope, a high coefficient should hold a
/// particle and a zero one should let it go.
#[tokio::test]
async fn friction_holds_a_particle_on_a_slope() -> Result<()> {
    let harness = Harness::new().await?;

    // A plane tilted 20 degrees -- shallow enough that mu = 0.8 holds
    // (tan 20 deg is about 0.36) and mu = 0 does not.
    let angle = 20.0f32.to_radians();
    let normal = Vec3::new(-angle.sin(), angle.cos(), 0.0);

    let mut travelled = Vec::new();
    for friction in [0.0f32, 0.8] {
        let start = Vec3::new(0.0, 0.02, 0.0);
        let particles = Particles::from_positions(&harness.context, &[start], &[1.0])?;

        let mut collisions = harness.collisions()?;
        collisions.set_colliders(
            &harness.context,
            vec![
                Collider::plane(normal, 0.0)
                    .with_friction(friction)
                    .with_thickness(0.002),
            ],
        )?;

        let solver = harness.solver(SolverSettings {
            substeps: 20,
            damping: 0.0,
            ..Default::default()
        })?;
        for _ in 0..120 {
            solver.step(
                &harness.context,
                &particles,
                &mut [],
                &mut collisions,
                1.0 / 60.0,
            )?;
        }

        let end = particles.read_positions(&harness.context).await?[0];
        // Distance slid along the slope, ignoring the drop onto it.
        let along = Vec3::new(angle.cos(), angle.sin(), 0.0);
        travelled.push((end - start).dot(along).abs());
    }

    assert!(
        travelled[1] < travelled[0] * 0.2,
        "friction barely helped: frictionless slid {:.4}, mu=0.8 slid {:.4}",
        travelled[0],
        travelled[1]
    );
    assert!(
        travelled[0] > 0.05,
        "the frictionless particle should have slid a long way, not {:.4}",
        travelled[0]
    );
    Ok(())
}

#[tokio::test]
async fn particles_stay_outside_a_mesh() -> Result<()> {
    let harness = Harness::new().await?;
    let radius = 0.4f32;
    let (mesh_positions, triangles) = sphere_mesh(24, 48, radius);

    let mut positions = Vec::new();
    for x in -6..=6 {
        for z in -6..=6 {
            positions.push(Vec3::new(x as f32 * 0.05, 1.2, z as f32 * 0.05));
        }
    }
    let particles =
        Particles::from_positions(&harness.context, &positions, &vec![1.0; positions.len()])?;

    let mut collisions = harness.collisions()?;
    collisions.set_colliders(&harness.context, vec![Collider::ground(-1.0)])?;
    collisions.set_mesh(Some(MeshCollider::new(
        &harness.context,
        &harness.cache,
        BvhKernels::with_cache(&harness.context, &harness.cache)?,
        &mesh_positions,
        &triangles,
        MeshSurface {
            thickness: 0.01,
            friction: 0.4,
        },
    )?));

    let solver = harness.solver(settings(20))?;
    for _ in 0..200 {
        solver.step(
            &harness.context,
            &particles,
            &mut [],
            &mut collisions,
            1.0 / 60.0,
        )?;
    }

    let settled = particles.read_positions(&harness.context).await?;
    let mut resting = 0;
    for (index, position) in settled.iter().enumerate() {
        let distance = position.length();
        if position.y < -0.9 {
            // Slid off the sphere and onto the floor; that is allowed.
            continue;
        }
        resting += 1;
        assert!(
            distance >= radius - 5e-3,
            "particle {index} is {distance} from the centre, inside the {radius} mesh"
        );
    }
    assert!(
        resting > 20,
        "only {resting} particles stayed on the mesh; the test proves nothing"
    );
    Ok(())
}

/// The mesh hierarchy has to follow the mesh. If a refit is skipped, the
/// hierarchy still describes where the body used to be and the cloth collides
/// with thin air.
#[tokio::test]
async fn a_moving_mesh_still_collides() -> Result<()> {
    let harness = Harness::new().await?;
    let radius = 0.3f32;
    let (base_positions, triangles) = sphere_mesh(16, 32, radius);

    let start = Vec3::new(0.0, 0.5, 0.0);
    let particles = Particles::from_positions(&harness.context, &[start], &[1.0])?;

    let mut collisions = harness.collisions()?;
    let mut mesh = MeshCollider::new(
        &harness.context,
        &harness.cache,
        BvhKernels::with_cache(&harness.context, &harness.cache)?,
        &base_positions,
        &triangles,
        MeshSurface {
            thickness: 0.01,
            friction: 0.0,
        },
    )?;

    // Move the sphere up under the particle, refitting as it goes.
    for step in 1..=10 {
        let offset = Vec3::new(0.0, step as f32 * 0.02, 0.0);
        let moved: Vec<Vec3> = base_positions.iter().map(|&p| p + offset).collect();
        mesh.write_positions(&harness.context, &moved)?;
        mesh.refit(&harness.context)?;
    }
    let final_offset = Vec3::new(0.0, 0.2, 0.0);

    collisions.set_mesh(Some(mesh));
    let solver = harness.solver(settings(20))?;
    for _ in 0..120 {
        solver.step(
            &harness.context,
            &particles,
            &mut [],
            &mut collisions,
            1.0 / 60.0,
        )?;
    }

    let settled = particles.read_positions(&harness.context).await?[0];
    let distance = (settled - final_offset).length();
    assert!(
        distance >= radius - 5e-3,
        "the particle fell into the moved sphere: {distance} from its centre, radius {radius}"
    );
    // Frictionless on the apex is an unstable equilibrium, so sliding off and
    // falling is the right answer -- but it has to have been *held* first. Two
    // seconds of free fall from 0.5 is about -19 metres; anywhere near the
    // sphere means the refit hierarchy found it.
    assert!(
        settled.y > -1.0,
        "the particle was never deflected at all: it is at {settled}, near free fall"
    );
    Ok(())
}

/// A particle already deep inside the mesh has to come out. This is the case a
/// garment hits constantly -- the initial layout puts panels through the body
/// before the first step -- and it is not something a substep can fix on its
/// own: the substep looks only as far as the shell and the distance just
/// travelled, and from the middle of a body there is no triangle in that
/// range. `push_out` is the pass that handles it.
#[tokio::test]
async fn a_particle_started_inside_is_pushed_out() -> Result<()> {
    let harness = Harness::new().await?;
    let radius = 0.4f32;
    let (mesh_positions, triangles) = sphere_mesh(20, 40, radius);

    // Well inside, but off-centre so there is a nearest surface to leave by.
    let start = Vec3::new(0.15, 0.05, 0.0);
    let particles = Particles::from_positions(&harness.context, &[start], &[1.0])?;

    let mut collisions = harness.collisions()?;
    collisions.set_mesh(Some(MeshCollider::new(
        &harness.context,
        &harness.cache,
        BvhKernels::with_cache(&harness.context, &harness.cache)?,
        &mesh_positions,
        &triangles,
        MeshSurface {
            thickness: 0.01,
            friction: 0.0,
        },
    )?));

    // A radius wide enough to see the surface from the middle of the sphere.
    collisions.push_out(&harness.context, &particles, radius, 4)?;

    let recovered = particles.read_positions(&harness.context).await?[0];
    assert!(
        recovered.length() >= radius - 5e-3,
        "push_out left the particle inside: {} from the centre",
        recovered.length()
    );

    // And once out, an ordinary run keeps it out.
    let solver = harness.solver(SolverSettings {
        substeps: 20,
        gravity: Vec3::default(),
        damping: 5.0,
        ..Default::default()
    })?;
    for _ in 0..120 {
        solver.step(
            &harness.context,
            &particles,
            &mut [],
            &mut collisions,
            1.0 / 60.0,
        )?;
    }

    let settled = particles.read_positions(&harness.context).await?[0];
    assert!(
        settled.length() >= radius - 5e-3,
        "the particle sank back inside: {} from the centre",
        settled.length()
    );
    Ok(())
}

#[tokio::test]
async fn pinned_particles_ignore_collision() -> Result<()> {
    let harness = Harness::new().await?;
    let start = Vec3::new(0.0, -1.0, 0.0);
    let particles = Particles::from_positions(&harness.context, &[start], &[0.0])?;

    let mut collisions = harness.collisions()?;
    collisions.set_colliders(&harness.context, vec![Collider::ground(0.0)])?;

    let solver = harness.solver(settings(10))?;
    for _ in 0..60 {
        solver.step(
            &harness.context,
            &particles,
            &mut [],
            &mut collisions,
            1.0 / 60.0,
        )?;
    }

    let settled = particles.read_positions(&harness.context).await?[0];
    assert!(
        (settled - start).length() < 1e-6,
        "a pinned particle was moved to {settled}, below the ground on purpose"
    );
    Ok(())
}

#[tokio::test]
async fn rejects_a_mesh_that_indexes_past_its_vertices() -> Result<()> {
    let harness = Harness::new().await?;
    let result = MeshCollider::new(
        &harness.context,
        &harness.cache,
        BvhKernels::with_cache(&harness.context, &harness.cache)?,
        &[Vec3::default(), Vec3::default(), Vec3::default()],
        &[[0, 1, 7]],
        MeshSurface::default(),
    );
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn updating_collider_counts_is_rejected() -> Result<()> {
    let harness = Harness::new().await?;
    let mut collisions = harness.collisions()?;
    collisions.set_colliders(&harness.context, vec![Collider::ground(0.0)])?;
    assert!(
        collisions
            .update_colliders(
                &harness.context,
                &[Collider::ground(0.0), Collider::ground(1.0)]
            )
            .is_err(),
        "changing the count has to go through set_colliders"
    );
    Ok(())
}

/// Several hooks in one substep, which is how self-collision will sit
/// alongside body collision.
#[tokio::test]
async fn a_hook_chain_runs_every_hook() -> Result<()> {
    let harness = Harness::new().await?;
    let particles =
        Particles::from_positions(&harness.context, &[Vec3::new(0.0, 1.0, 0.0)], &[1.0])?;

    let mut collisions = harness.collisions()?;
    collisions.set_colliders(&harness.context, vec![Collider::ground(0.0)])?;

    let mut calls = 0usize;
    let solver = harness.solver(settings(4))?;
    {
        let mut counter = |_: &mut KernelBatch, _: &Particles, _: f32| {
            calls += 1;
            Ok(())
        };
        let mut chain = HookChain::new(vec![&mut collisions, &mut counter]);
        solver.step(
            &harness.context,
            &particles,
            &mut [],
            &mut chain,
            1.0 / 60.0,
        )?;
    }
    assert_eq!(calls, 4);
    Ok(())
}
