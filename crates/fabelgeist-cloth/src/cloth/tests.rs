use fabelgeist_gpu::prelude::BufferUpload;
use fabelgeist_math::Vec2;
use fabelgeist_physics::{Collider, Collisions};
use fabelgeist_xpbd::{Solver, SolverSettings};

use super::*;
use crate::garment::{Panel, Placement, Seam, SeamSide};
use crate::{Fabric, build};

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

    fn collisions(&self) -> Result<Collisions> {
        Collisions::new(&self.context, &self.cache)
    }

    fn solver(&self, settings: SolverSettings) -> Result<Solver> {
        Solver::with_cache(&self.context, &self.cache, settings)
    }
}

fn square(size: f32) -> Vec<Vec2> {
    vec![
        Vec2::new(-size * 0.5, -size * 0.5),
        Vec2::new(size * 0.5, -size * 0.5),
        Vec2::new(size * 0.5, size * 0.5),
        Vec2::new(-size * 0.5, size * 0.5),
    ]
}

/// A flat sheet lying in the XZ plane, some distance up.
fn sheet(size: f32, height: f32, resolution: f32, fabric: Fabric) -> crate::GarmentMesh {
    let panels = vec![Panel::new("sheet", square(size)).placed(Placement {
        translation: Vec3::new(0.0, height, 0.0),
        // The panel plane is XY; a quarter turn about x lays it flat.
        rotation: Vec3::new(-90.0, 0.0, 0.0),
    })];
    build(&panels, &[], resolution, fabric.density).unwrap()
}

#[tokio::test]
async fn a_sheet_falls_and_settles_on_the_ground() -> Result<()> {
    let harness = Harness::new().await?;
    let fabric = Fabric::COTTON;
    let mesh = sheet(0.4, 0.5, 0.03, fabric);
    let mut cloth = Cloth::new(&harness.context, &harness.cache, &mesh, fabric)?;

    let mut collisions = harness.collisions()?;
    collisions.set_colliders(
        &harness.context,
        vec![Collider::ground(0.0).with_thickness(0.002)],
    )?;

    let solver = harness.solver(SolverSettings {
        substeps: 12.into(),
        ..cloth.settings()
    })?;
    for _ in 0..180 {
        cloth.step(&harness.context, &solver, &mut collisions, 1.0 / 60.0)?;
    }

    let settled = cloth.read_positions(&harness.context).await?;
    assert!(
        settled.iter().all(|p| p.is_finite()),
        "the solve produced a non-finite position"
    );
    for (index, position) in settled.iter().enumerate() {
        assert!(
            position.y > -0.01,
            "particle {index} fell through the ground to {}",
            position.y
        );
        assert!(
            position.y < 0.05,
            "particle {index} never landed: it is at {}",
            position.y
        );
    }
    Ok(())
}

/// The property that makes it cloth rather than a bag of points: the edges
/// hold their rest length while everything moves.
#[tokio::test]
async fn a_falling_sheet_does_not_stretch() -> Result<()> {
    let harness = Harness::new().await?;
    let fabric = Fabric::COTTON;
    let mesh = sheet(0.4, 0.5, 0.03, fabric);
    let mut cloth = Cloth::new(&harness.context, &harness.cache, &mesh, fabric)?;

    let mut collisions = harness.collisions()?;
    collisions.set_colliders(&harness.context, vec![Collider::ground(0.0)])?;

    let solver = harness.solver(SolverSettings {
        substeps: 15.into(),
        ..cloth.settings()
    })?;
    for _ in 0..120 {
        cloth.step(&harness.context, &solver, &mut collisions, 1.0 / 60.0)?;
    }

    let settled = cloth.read_positions(&harness.context).await?;
    let worst = mesh
        .edges
        .iter()
        .zip(&mesh.rest_lengths)
        .map(|(edge, &rest)| {
            ((settled[edge[0] as usize] - settled[edge[1] as usize]).length() - rest).abs() / rest
        })
        .fold(0.0f32, f32::max);
    assert!(
        worst < 0.1,
        "the worst edge is {:.1}% off its rest length",
        worst * 100.0
    );
    Ok(())
}

/// A sheet pinned along one edge should hang and stay hanging, not slide off
/// its own pins.
#[tokio::test]
async fn a_pinned_sheet_hangs() -> Result<()> {
    let harness = Harness::new().await?;
    let fabric = Fabric::COTTON;

    let panels = vec![Panel::new("flag", square(0.3)).placed(Placement {
        translation: Vec3::new(0.0, 0.0, 0.0),
        rotation: Vec3::default(),
    })];
    let mesh = build(&panels, &[], 0.025, fabric.density).unwrap();

    // Pin everything along the top.
    let top = mesh
        .positions
        .iter()
        .map(|p| p.y)
        .fold(f32::NEG_INFINITY, f32::max);
    let mut inverse_masses = mesh.inverse_masses();
    let mut pinned = 0;
    for (index, position) in mesh.positions.iter().enumerate() {
        if position.y > top - 1e-4 {
            inverse_masses[index] = fabelgeist_shell::ParticleInverseMass::PINNED;
            pinned += 1;
        }
    }
    assert!(pinned > 5, "only {pinned} particles are pinned");

    let mut cloth = Cloth::new(&harness.context, &harness.cache, &mesh, fabric)?;
    cloth
        .particles
        .write(&harness.context, &mesh.positions, &inverse_masses)?;

    let mut collisions = harness.collisions()?;
    let solver = harness.solver(SolverSettings {
        substeps: 15.into(),
        ..cloth.settings()
    })?;
    for _ in 0..240 {
        cloth.step(&harness.context, &solver, &mut collisions, 1.0 / 60.0)?;
    }

    let settled = cloth.read_positions(&harness.context).await?;
    for (index, position) in settled.iter().enumerate() {
        if inverse_masses[index].mobility() == fabelgeist_shell::ParticleMobility::Prescribed {
            assert!(
                (*position - mesh.positions[index]).length() < 1e-4,
                "pinned particle {index} moved"
            );
        }
    }
    // The free edge should still be about a panel's height below the pins --
    // hanging, not stretched into a string.
    let bottom = settled.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
    assert!(
        bottom < top - 0.25 && bottom > top - 0.35,
        "the free edge is at {bottom}, which is not 0.3 below the pinned edge at {top}"
    );
    Ok(())
}

/// Bending stiffness has to be visible: a stiff fabric draped over an edge
/// should stick out further than a limp one. The two values here sit either
/// side of the transition -- measured by sweeping the compliance and watching
/// how far the flap reaches, which is also where the `Fabric` presets came
/// from.
#[tokio::test]
async fn bending_stiffness_changes_the_drape() -> Result<()> {
    let harness = Harness::new().await?;

    let mut overhangs = Vec::new();
    for bend_compliance in [1e-7f32, 1e-2] {
        let fabric = Fabric::COTTON.with_bend_compliance(bend_compliance);
        let panels = vec![Panel::new("flap", square(0.3)).placed(Placement {
            translation: Vec3::new(0.0, 0.5, 0.0),
            rotation: Vec3::new(-90.0, 0.0, 0.0),
        })];
        let mesh = build(&panels, &[], 0.025, fabric.density).unwrap();

        // Pin the half of the sheet at negative z, so the other half is a flap
        // hanging off a straight fold line.
        let mut inverse_masses = mesh.inverse_masses();
        for (index, position) in mesh.positions.iter().enumerate() {
            if position.z < 0.0 {
                inverse_masses[index] = fabelgeist_shell::ParticleInverseMass::PINNED;
            }
        }

        let mut cloth = Cloth::new(&harness.context, &harness.cache, &mesh, fabric)?;
        cloth
            .particles
            .write(&harness.context, &mesh.positions, &inverse_masses)?;

        let mut collisions = harness.collisions()?;
        let solver = harness.solver(SolverSettings {
            substeps: 15.into(),
            ..cloth.settings()
        })?;
        for _ in 0..240 {
            cloth.step(&harness.context, &solver, &mut collisions, 1.0 / 60.0)?;
        }

        let settled = cloth.read_positions(&harness.context).await?;
        assert!(settled.iter().all(|p| p.is_finite()));
        // How far the free end still reaches out horizontally.
        overhangs.push(
            settled
                .iter()
                .map(|p| p.z)
                .fold(f32::NEG_INFINITY, f32::max),
        );
    }

    assert!(
        overhangs[0] > overhangs[1] + 0.05,
        "stiff reached {:.3} and limp reached {:.3}; bending is doing nothing",
        overhangs[0],
        overhangs[1]
    );
    Ok(())
}

/// Seams have to close. Two panels a decent distance apart, sewn down both
/// sides, should pull together into a tube.
#[tokio::test]
async fn seams_pull_panels_together() -> Result<()> {
    let harness = Harness::new().await?;
    let fabric = Fabric::COTTON;

    let panels = vec![
        Panel::new("front", square(0.3)).placed(Placement {
            translation: Vec3::new(0.0, 0.0, 0.08),
            rotation: Vec3::default(),
        }),
        Panel::new("back", square(0.3)).placed(Placement {
            translation: Vec3::new(0.0, 0.0, -0.08),
            rotation: Vec3::default(),
        }),
    ];
    let seams = vec![
        Seam::new(
            SeamSide { panel: 0, edge: 1 },
            SeamSide { panel: 1, edge: 1 },
        ),
        Seam::new(
            SeamSide { panel: 0, edge: 3 },
            SeamSide { panel: 1, edge: 3 },
        ),
    ];
    let mesh = build(&panels, &seams, 0.025, fabric.density).unwrap();

    let before: f32 = mesh
        .seams
        .iter()
        .map(|&[a, b]| (mesh.positions[a as usize] - mesh.positions[b as usize]).length())
        .sum::<f32>()
        / mesh.seams.len() as f32;
    assert!(before > 0.1, "the panels start {before} apart");

    let mut cloth = Cloth::new(&harness.context, &harness.cache, &mesh, fabric)?;
    let mut collisions = harness.collisions()?;
    // No gravity: this is about the seams, not about falling.
    let solver = harness.solver(SolverSettings {
        substeps: 15.into(),
        gravity: Vec3::default(),
        damping: 2.0,
        ..Default::default()
    })?;
    for _ in 0..240 {
        cloth.step(&harness.context, &solver, &mut collisions, 1.0 / 60.0)?;
    }

    let settled = cloth.read_positions(&harness.context).await?;
    assert!(settled.iter().all(|p| p.is_finite()));
    let after: f32 = mesh
        .seams
        .iter()
        .map(|&[a, b]| (settled[a as usize] - settled[b as usize]).length())
        .sum::<f32>()
        / mesh.seams.len() as f32;

    assert!(
        after < before * 0.15,
        "the seams closed from {before} to {after}, which is not closed"
    );
    Ok(())
}

/// Self-collision keeps two layers apart. A sheet folded onto itself should
/// end up about a thickness thick, not zero.
#[tokio::test]
async fn self_collision_keeps_layers_apart() -> Result<()> {
    let harness = Harness::new().await?;
    // Exaggerated thickness, so that the separation is measurable against the
    // mesh resolution rather than lost in it.
    let fabric = Fabric::COTTON.with_bend_compliance(1e-2);
    let fabric = Fabric {
        thickness: 0.02,
        ..fabric
    };

    // Two sheets stacked almost on top of each other, both falling to the
    // ground: they must not end up in the same plane.
    let panels = vec![
        Panel::new("lower", square(0.2)).placed(Placement {
            translation: Vec3::new(0.0, 0.10, 0.0),
            rotation: Vec3::new(-90.0, 0.0, 0.0),
        }),
        Panel::new("upper", square(0.2)).placed(Placement {
            translation: Vec3::new(0.0, 0.13, 0.0),
            rotation: Vec3::new(-90.0, 0.0, 0.0),
        }),
    ];
    let mesh = build(&panels, &[], 0.02, fabric.density).unwrap();
    let mut cloth = Cloth::new(&harness.context, &harness.cache, &mesh, fabric)?;

    let mut collisions = harness.collisions()?;
    collisions.set_colliders(&harness.context, vec![Collider::ground(0.0)])?;

    let solver = harness.solver(SolverSettings {
        substeps: 15.into(),
        ..cloth.settings()
    })?;
    for _ in 0..240 {
        cloth.step(&harness.context, &solver, &mut collisions, 1.0 / 60.0)?;
    }

    let settled = cloth.read_positions(&harness.context).await?;
    assert!(settled.iter().all(|p| p.is_finite()));

    let split = mesh.panel_offsets[1] as usize;
    let mean = |slice: &[Vec3]| slice.iter().map(|p| p.y).sum::<f32>() / slice.len() as f32;
    let lower = mean(&settled[..split]);
    let upper = mean(&settled[split..]);

    assert!(
        upper - lower > fabric.thickness * 0.5,
        "the two layers settled {:.4} apart; the fabric is {:.4} thick, so they have merged",
        upper - lower,
        fabric.thickness
    );
    Ok(())
}

/// With self-collision off, the same two layers are free to merge. This is
/// what makes the test above about self-collision rather than about the sheets
/// simply not reaching each other.
#[tokio::test]
async fn layers_merge_without_self_collision() -> Result<()> {
    let harness = Harness::new().await?;
    let fabric = Fabric {
        thickness: 0.02,
        ..Fabric::COTTON.with_bend_compliance(1e-2)
    };

    let panels = vec![
        Panel::new("lower", square(0.2)).placed(Placement {
            translation: Vec3::new(0.0, 0.10, 0.0),
            rotation: Vec3::new(-90.0, 0.0, 0.0),
        }),
        Panel::new("upper", square(0.2)).placed(Placement {
            translation: Vec3::new(0.0, 0.13, 0.0),
            rotation: Vec3::new(-90.0, 0.0, 0.0),
        }),
    ];
    let mesh = build(&panels, &[], 0.02, fabric.density).unwrap();
    let mut cloth = Cloth::new(&harness.context, &harness.cache, &mesh, fabric)?;
    cloth.self_collision.enabled = false;

    let mut collisions = harness.collisions()?;
    collisions.set_colliders(&harness.context, vec![Collider::ground(0.0)])?;

    let solver = harness.solver(SolverSettings {
        substeps: 15.into(),
        ..cloth.settings()
    })?;
    for _ in 0..240 {
        cloth.step(&harness.context, &solver, &mut collisions, 1.0 / 60.0)?;
    }

    let settled = cloth.read_positions(&harness.context).await?;
    let split = mesh.panel_offsets[1] as usize;
    let mean = |slice: &[Vec3]| slice.iter().map(|p| p.y).sum::<f32>() / slice.len() as f32;
    let gap = mean(&settled[split..]) - mean(&settled[..split]);
    assert!(
        gap < fabric.thickness * 0.5,
        "the layers stayed {gap:.4} apart with self-collision off, so the other test proves nothing"
    );
    Ok(())
}

#[tokio::test]
async fn a_sheet_drapes_over_a_sphere() -> Result<()> {
    let harness = Harness::new().await?;
    let fabric = Fabric::COTTON;
    let radius = 0.15f32;

    let mesh = sheet(0.5, 0.4, 0.025, fabric);
    let mut cloth = Cloth::new(&harness.context, &harness.cache, &mesh, fabric)?;

    let mut collisions = harness.collisions()?;
    collisions.set_colliders(
        &harness.context,
        vec![
            Collider::sphere(Vec3::new(0.0, 0.15, 0.0), radius)
                .with_friction(0.4)
                .with_thickness(0.003),
            Collider::ground(0.0).with_friction(0.4),
        ],
    )?;

    let solver = harness.solver(SolverSettings {
        substeps: 15.into(),
        ..cloth.settings()
    })?;
    for _ in 0..300 {
        cloth.step(&harness.context, &solver, &mut collisions, 1.0 / 60.0)?;
    }

    let settled = cloth.read_positions(&harness.context).await?;
    assert!(settled.iter().all(|p| p.is_finite()));

    let center = Vec3::new(0.0, 0.15, 0.0);
    for (index, position) in settled.iter().enumerate() {
        assert!(
            (*position - center).length() >= radius - 0.005,
            "particle {index} is inside the sphere"
        );
        assert!(
            position.y > -0.01,
            "particle {index} fell through the floor"
        );
    }

    // Something has to be resting on top: a sheet that slid off entirely has
    // not draped.
    let on_top = settled
        .iter()
        .filter(|p| p.y > center.y + radius * 0.5)
        .count();
    assert!(on_top > 10, "only {on_top} particles are on the sphere");
    Ok(())
}

#[tokio::test]
async fn resets_to_the_flat_layout() -> Result<()> {
    let harness = Harness::new().await?;
    let fabric = Fabric::COTTON;
    let mesh = sheet(0.3, 0.5, 0.04, fabric);
    let mut cloth = Cloth::new(&harness.context, &harness.cache, &mesh, fabric)?;

    let mut collisions = harness.collisions()?;
    let solver = harness.solver(cloth.settings())?;
    for _ in 0..30 {
        cloth.step(&harness.context, &solver, &mut collisions, 1.0 / 60.0)?;
    }
    let moved = cloth.read_positions(&harness.context).await?;
    assert!(moved[0].y < mesh.positions[0].y - 0.01, "it did not fall");

    cloth.reset(&harness.context)?;
    let reset = cloth.read_positions(&harness.context).await?;
    for (index, (a, b)) in reset.iter().zip(&mesh.positions).enumerate() {
        assert!((*a - *b).length() < 1e-6, "particle {index} did not reset");
    }
    // And its velocity, or it would carry on falling from the flat layout.
    let velocities = cloth.particles.read_velocities(&harness.context).await?;
    assert!(velocities.iter().all(|v| v.length() < 1e-6));
    Ok(())
}

#[tokio::test]
async fn rejects_an_empty_garment() -> Result<()> {
    let harness = Harness::new().await?;
    let empty = crate::GarmentMesh::default();
    assert!(Cloth::new(&harness.context, &harness.cache, &empty, Fabric::COTTON).is_err());
    Ok(())
}

/// Which power of the hinge area makes bending stiffness independent of the
/// mesh resolution. Run by hand:
///
/// ```text
/// PRISM_BEND_EXPONENT=0.5 cargo test -p fabelgeist-cloth --lib bend_scaling_grid -- --nocapture --ignored
/// ```
///
/// The right exponent is the one where the two resolutions' curves lie on top
/// of each other -- the same compliance producing the same droop.
#[tokio::test]
#[ignore]
async fn bend_scaling_grid() -> Result<()> {
    let harness = Harness::new().await?;
    for resolution in [0.04f32, 0.02] {
        let mut line = format!("h={resolution:<5}");
        for compliance in [1e-7f32, 1e-6, 1e-5, 1e-4, 1e-3, 1e-2] {
            let fabric = Fabric::COTTON.with_bend_compliance(compliance);
            let panels = vec![Panel::new("flap", square(0.3)).placed(Placement {
                translation: Vec3::new(0.0, 0.5, 0.0),
                rotation: Vec3::new(-90.0, 0.0, 0.0),
            })];
            let mesh = build(&panels, &[], resolution, fabric.density).unwrap();
            let mut inverse_masses = mesh.inverse_masses();
            for (index, position) in mesh.positions.iter().enumerate() {
                if position.z < 0.0 {
                    inverse_masses[index] = fabelgeist_shell::ParticleInverseMass::PINNED;
                }
            }
            let mut cloth = Cloth::new(&harness.context, &harness.cache, &mesh, fabric)?;
            cloth
                .particles
                .write(&harness.context, &mesh.positions, &inverse_masses)?;
            let mut collisions = harness.collisions()?;
            let solver = harness.solver(SolverSettings {
                substeps: 8.into(),
                ..cloth.settings()
            })?;
            for _ in 0..100 {
                cloth.step(&harness.context, &solver, &mut collisions, 1.0 / 60.0)?;
            }
            let settled = cloth.read_positions(&harness.context).await?;
            let reach = settled
                .iter()
                .map(|p| p.z)
                .fold(f32::NEG_INFINITY, f32::max);
            line.push_str(&format!("  {reach:.3}"));
        }
        println!("{line}");
    }
    Ok(())
}

/// Submitting per substep must not change the answer.
///
/// `step_interleaved` exists so a compositor sharing the device can be
/// scheduled between submissions. It records exactly the same dispatches in
/// exactly the same order -- only the submission boundaries move -- so the
/// result has to be the same. If it ever is not, the batching is hiding a
/// missing barrier.
#[tokio::test]
async fn interleaved_submission_gives_the_same_result() -> Result<()> {
    let harness = Harness::new().await?;
    let fabric = Fabric::COTTON;
    let mesh = sheet(0.4, 0.5, 0.035, fabric);

    let mut settled = Vec::new();
    for interleaved in [false, true] {
        let mut cloth = Cloth::new(&harness.context, &harness.cache, &mesh, fabric)?;
        let mut collisions = harness.collisions()?;
        collisions.set_colliders(
            &harness.context,
            vec![Collider::ground(0.0).with_friction(0.4)],
        )?;
        let solver = harness.solver(SolverSettings {
            substeps: 10.into(),
            ..cloth.settings()
        })?;

        for _ in 0..90 {
            if interleaved {
                cloth
                    .step_interleaved(&harness.context, &solver, &mut collisions, 1.0 / 60.0)
                    .await?;
            } else {
                cloth.step(&harness.context, &solver, &mut collisions, 1.0 / 60.0)?;
            }
        }
        settled.push(cloth.read_positions(&harness.context).await?);
    }

    let (batched, interleaved) = (&settled[0], &settled[1]);
    assert_eq!(batched.len(), interleaved.len());
    let worst = batched
        .iter()
        .zip(interleaved)
        .map(|(a, b)| (*a - *b).length())
        .fold(0.0f32, f32::max);
    assert!(
        worst < 1e-4,
        "the two submission schemes drifted apart by {worst} m, which means \
         one of them is missing a barrier the other has"
    );
    Ok(())
}

#[tokio::test]
async fn coincident_non_neighbours_separate_and_pinned_particles_stay_fixed() -> Result<()> {
    let harness = Harness::new().await?;
    let particles = fabelgeist_xpbd::Particles::from_positions(
        &harness.context,
        &[Vec3::default(), Vec3::default()],
        &[0.0.into(), 1.0.into()],
    )?;
    let mut collision = crate::SelfCollision::new(
        &harness.context,
        &harness.cache,
        2,
        &[vec![], vec![]],
        0.005,
    )?;
    let mut batch = KernelBatch::labelled(&harness.context, "coincident contact");
    collision.record(&mut batch, &particles, true)?;
    batch.submit();
    let p = particles.read_positions(&harness.context).await?;
    assert!(p[0].length() < 1e-6);
    assert!((p[1] - p[0]).length() > 0.0099);
    Ok(())
}

#[tokio::test]
async fn interactive_step_blocks_a_triangle_interior_crossing() -> Result<()> {
    let harness = Harness::new().await?;
    let mesh = GarmentMesh {
        positions: vec![
            Vec3::new(-1., 0., -1.),
            Vec3::new(0., 0., 1.),
            Vec3::new(1., 0., -1.),
            Vec3::new(0., 0.02, 0.),
        ],
        triangles: vec![[0, 1, 2]],
        masses: vec![0.0.into(), 0.0.into(), 0.0.into(), 1.0.into()],
        ..Default::default()
    };
    let mut cloth = Cloth::new(&harness.context, &harness.cache, &mesh, Fabric::COTTON)?;
    let solver = harness.solver(SolverSettings {
        substeps: 1.into(),
        gravity: Vec3::default(),
        damping: 0.,
        ..cloth.settings()
    })?;
    let mut collisions = harness.collisions()?;
    let mut velocities = vec![0.0f32; 16];
    velocities[13] = -4.0;
    cloth
        .particles
        .velocities
        .write(&harness.context, BufferUpload::from_elements(&velocities))?;
    cloth
        .step_interleaved(&harness.context, &solver, &mut collisions, 0.01)
        .await?;
    let positions = cloth.read_positions(&harness.context).await?;
    assert!(positions[3].y >= Fabric::COTTON.thickness * 0.99);
    assert_eq!(&positions[..3], &mesh.positions[..3]);
    Ok(())
}
