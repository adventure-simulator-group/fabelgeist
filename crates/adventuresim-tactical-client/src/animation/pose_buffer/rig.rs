//! Admission and cached metadata for an authored animation rig.

use super::*;

pub(super) struct RigDefinition {
    pub(super) family: String,
    pub(super) joints: Vec<RigJoint>,
}

#[derive(Clone)]
pub(super) struct RigJoint {
    pub(super) target: AnimationTargetId,
    pub(super) bind: LocalPose,
    pub(super) parent: Option<RigJointOrdinal>,
    pub(super) name: Option<RigJointName>,
    pub(super) body_region: AnimationBodyRegion,
    pub(super) role: Option<BoneRole>,
    pub(super) pelvis: bool,
}

struct AdmittedRigBinding {
    entity: Entity,
    target: AnimationTargetId,
    bind: LocalPose,
    identity: Option<AuthoredRigJoint>,
    parent: Option<Entity>,
}

impl RigDefinitions {
    #[expect(
        clippy::type_complexity,
        reason = "binds semantic animation targets to the reference hierarchy"
    )]
    pub(super) fn bind(
        &mut self,
        owner: Entity,
        family: String,
        targets: &Query<(
            Entity,
            &AnimationTargetId,
            &AuthoredBindTransform,
            Option<&AuthoredRigJoint>,
            Option<&ChildOf>,
        )>,
        transforms: &Query<&Transform>,
    ) -> Option<PoseBufferRig> {
        let found = targets
            .iter()
            .filter(|(_, _, bind, _, _)| bind.owner == owner)
            .map(|(entity, target, bind, name, parent)| AdmittedRigBinding {
                entity,
                target: *target,
                bind: LocalPose::from_transform(bind.local),
                identity: name.cloned(),
                parent: parent.map(ChildOf::parent),
            })
            .collect::<Vec<_>>();
        if found.is_empty() {
            return None;
        }
        let definition = self
            .0
            .entry(family.clone())
            .or_insert_with(|| {
                let mut ordered = found.iter().collect::<Vec<_>>();
                ordered.sort_by(|left, right| {
                    left.identity
                        .as_ref()
                        .map(|joint| &joint.name)
                        .cmp(&right.identity.as_ref().map(|joint| &joint.name))
                });
                let indices = ordered
                    .iter()
                    .enumerate()
                    .map(|(index, binding)| (binding.entity, RigJointOrdinal::from(index)))
                    .collect::<HashMap<_, _>>();
                Arc::new(RigDefinition {
                    family: family.clone(),
                    joints: ordered
                        .into_iter()
                        .map(|binding| RigJoint {
                            target: binding.target,
                            bind: binding.bind,
                            parent: binding
                                .parent
                                .and_then(|parent| indices.get(&parent).copied()),
                            name: binding.identity.as_ref().map(|joint| joint.name.clone()),
                            body_region: binding
                                .identity
                                .as_ref()
                                .map_or(AnimationBodyRegion::Upper, |joint| joint.body_region),
                            role: binding.identity.as_ref().and_then(|joint| joint.role),
                            pelvis: binding.identity.as_ref().is_some_and(|joint| joint.pelvis),
                        })
                        .collect(),
                })
            })
            .clone();
        let by_target = found
            .into_iter()
            .map(|binding| (binding.target, binding.entity))
            .collect::<HashMap<_, _>>();
        let entities = definition
            .joints
            .iter()
            .map(|joint| by_target.get(&joint.target).copied())
            .collect::<Vec<_>>();
        let current = definition
            .joints
            .iter()
            .zip(&entities)
            .map(|(joint, entity)| {
                entity
                    .and_then(|entity| transforms.get(entity).ok().copied())
                    .map(LocalPose::from_transform)
                    .unwrap_or(joint.bind)
            })
            .collect::<Vec<_>>();
        let pose_rig = PoseBufferRig::new(definition, entities, current);
        Some(pose_rig)
    }
}

impl PoseBufferRig {
    fn new(
        definition: Arc<RigDefinition>,
        entities: Vec<Option<Entity>>,
        current: Vec<LocalPose>,
    ) -> Self {
        let joint_count = definition.joints.len();
        Self {
            definition,
            entities,
            previous: current.clone(),
            next: current,
            sample_accumulator: 0.0,
            interpolation_alpha: 0.0,
            last_evaluation_tick: None,
            decay_delta_seconds: 0.0,
            offsets: vec![JointInertialOffset::default(); joint_count],
            target_linear_velocities: vec![Vec3::ZERO; joint_count],
            target_angular_velocities: vec![Vec3::ZERO; joint_count],
            terrain_plants: [None; 2],
            plan: None,
            active: false,
            frozen: false,
            settled: true,
            physics_owned: false,
        }
    }
}
