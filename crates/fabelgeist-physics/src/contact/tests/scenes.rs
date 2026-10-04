use crate::collider::{Collider, Shape};
use crate::{Collisions, MeshCollider, MeshSurface};
use fabelgeist_bvh::gpu::BvhKernels;
use fabelgeist_compute::kernel::{KernelBatch, KernelCache};
use fabelgeist_gpu::prelude::WgpuContext;
use fabelgeist_math::Vec3;
use fabelgeist_xpbd::{ParticleInverseMass, Particles};

#[derive(Clone, Copy, Debug)]
pub(super) enum FixtureScene {
    Empty,
    Disabled,
    Absent,
    Plane,
    Sphere,
    Capsule,
    Box,
    Mesh,
    Combined,
    Growth,
}
pub(super) const SCENES: [FixtureScene; 10] = [
    FixtureScene::Empty,
    FixtureScene::Disabled,
    FixtureScene::Absent,
    FixtureScene::Plane,
    FixtureScene::Sphere,
    FixtureScene::Capsule,
    FixtureScene::Box,
    FixtureScene::Mesh,
    FixtureScene::Combined,
    FixtureScene::Growth,
];
impl FixtureScene {
    pub(super) fn colliders(self) -> Vec<Collider> {
        match self {
            Self::Empty | Self::Absent | Self::Mesh => vec![],
            Self::Disabled | Self::Plane => vec![Collider::ground(0.0)],
            Self::Sphere => vec![Collider::sphere(Vec3::default(), 0.3)],
            Self::Capsule => vec![Collider::capsule(
                Vec3::new(0.0, -0.3, 0.0),
                Vec3::new(0.0, 0.3, 0.0),
                0.2,
            )],
            Self::Box => vec![Collider::new(Shape::Box {
                center: Vec3::default(),
                half_extents: Vec3::new(0.2, 0.1, 0.3),
                rotation: [0.0, 0.0, 0.0, 1.0],
            })],
            Self::Combined => vec![
                Collider::ground(-0.04),
                Collider::sphere(Vec3::default(), 0.3),
            ],
            Self::Growth => {
                let mut colliders = vec![Collider::ground(-1.0); 17];
                colliders[0] = Collider::ground(0.0);
                colliders
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) enum FixturePhase {
    Initial,
    Updated,
    PushOut,
}
pub(super) struct FixtureState {
    pub(super) particles: Particles,
    collision: Collisions,
    colliders: Vec<Collider>,
}
impl FixtureState {
    pub(super) fn new(context: &WgpuContext, cache: &KernelCache, scene: FixtureScene) -> Self {
        let positions = if matches!(scene, FixtureScene::Empty) {
            vec![]
        } else {
            vec![
                Vec3::new(0.1, -0.04, 0.05),
                Vec3::new(-0.1, -0.06, 0.05),
                Vec3::new(0.15, 0.08, -0.1),
                Vec3::new(0.3, -0.1, 0.2),
            ]
        };
        let masses = if positions.is_empty() {
            vec![]
        } else {
            vec![
                ParticleInverseMass::UNIT_MASS,
                0.0.into(),
                (-0.0).into(),
                2.0.into(),
            ]
        };
        let particles = Particles::from_positions(context, &positions, &masses).unwrap();
        let mut collision = Collisions::new(context, cache).unwrap();
        collision.enabled = !matches!(scene, FixtureScene::Disabled);
        collision.particle_radius = 0.002;
        let colliders = scene.colliders();
        collision.set_colliders(context, colliders.clone()).unwrap();
        if matches!(scene, FixtureScene::Mesh | FixtureScene::Combined) {
            let mesh = MeshCollider::new(
                context,
                cache,
                BvhKernels::with_cache(context, cache).unwrap(),
                &[
                    Vec3::new(-1.0, 0.0, -1.0),
                    Vec3::new(1.0, 0.0, -1.0),
                    Vec3::new(1.0, 0.0, 1.0),
                    Vec3::new(-1.0, 0.0, 1.0),
                ],
                &[[0, 2, 1], [0, 3, 2]],
                MeshSurface::default(),
            )
            .unwrap();
            collision.set_mesh(Some(mesh));
        }
        Self {
            particles,
            collision,
            colliders,
        }
    }
    pub(super) fn record(
        &mut self,
        context: &WgpuContext,
        batch: &mut KernelBatch,
        phase: FixturePhase,
    ) {
        match phase {
            FixturePhase::Initial => self.collision.record(batch, &self.particles).unwrap(),
            FixturePhase::Updated => {
                for collider in &mut self.colliders {
                    collider.thickness = 0.006;
                    collider.friction = 0.6;
                }
                self.collision
                    .update_colliders(context, &self.colliders)
                    .unwrap();
                self.collision.particle_radius = 0.004;
                self.collision.record(batch, &self.particles).unwrap();
            }
            FixturePhase::PushOut => self
                .collision
                .record_push_out(batch, &self.particles, 0.5)
                .unwrap(),
        }
    }
}
