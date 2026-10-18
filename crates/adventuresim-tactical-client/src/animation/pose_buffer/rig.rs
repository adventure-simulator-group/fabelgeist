//! Admission and cached metadata for an authored animation rig.

use super::*;

pub(super) struct RigDefinition {
    pub(super) family: String,
    pub(super) joints: RigJointTable<RigJoint>,
    pub(super) name_ordinals: std::sync::OnceLock<HashMap<RigJointName, RigJointOrdinal>>,
}

#[derive(Clone)]
pub(super) struct RigJoint {
    pub(super) target: AnimationTargetId,
    pub(super) bind: LocalPose,
    pub(super) parent: Option<RigJointOrdinal>,
    pub(super) name: Option<RigJointName>,
    pub(super) body_region: AnimationBodyRegion,
    pub(super) role: Option<BoneRole>,
}

struct AdmittedRigBinding {
    entity: Entity,
    target: AnimationTargetId,
    bind: LocalPose,
    identity: Option<AuthoredRigJoint>,
    parent: Option<Entity>,
}

impl RigDefinition {
    pub(super) fn joint_ordinal(&self, name: &RigJointName) -> Option<RigJointOrdinal> {
        self.name_ordinals
            .get_or_init(|| {
                let mut ordinals = HashMap::new();
                for (ordinal, joint) in self.joints.indexed() {
                    if let Some(name) = &joint.name {
                        ordinals.entry(name.clone()).or_insert(ordinal);
                    }
                }
                ordinals
            })
            .get(name)
            .copied()
    }
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
                        })
                        .collect(),
                    name_ordinals: Default::default(),
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
            .collect::<RigJointTable<_>>();
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
            .collect::<RigJointTable<_>>();
        let pose_rig = PoseBufferRig::new(definition, entities, current);
        Some(pose_rig)
    }
}

impl PoseBufferRig {
    pub(super) fn begin_observation(
        &mut self,
        clock: &ProceduralAnimationClock,
        time: &Time,
    ) -> AnimationValueResult<Option<AnimationElapsed>> {
        let elapsed = match clock.fixed_step() {
            Some(FixedAnimationStep { tick, .. }) if self.last_evaluation_tick == Some(tick) => {
                self.decay_delta_seconds = AnimationElapsed::ZERO;
                return Ok(None);
            }
            Some(step) => {
                self.last_evaluation_tick = Some(step.tick);
                step.delta_seconds
            }
            None => {
                self.last_evaluation_tick = None;
                AnimationElapsed::from(time.delta())
            }
        }
        .capped_by_seconds(0.1)?;
        self.decay_delta_seconds = elapsed;
        Ok(Some(elapsed))
    }

    fn new(
        definition: Arc<RigDefinition>,
        entities: RigJointTable<Option<Entity>>,
        current: RigJointTable<LocalPose>,
    ) -> Self {
        let joint_count = definition.joints.len();
        Self {
            definition,
            entities,
            previous: current.clone(),
            next: current,
            sample_accumulator: AnimationElapsed::ZERO,
            interpolation_alpha: 0.0,
            last_evaluation_tick: None,
            decay_delta_seconds: AnimationElapsed::ZERO,
            offsets: RigJointTable::from(vec![JointInertialOffset::default(); joint_count]),
            target_linear_velocities: RigJointTable::from(vec![Vec3::ZERO; joint_count]),
            target_angular_velocities: RigJointTable::from(vec![Vec3::ZERO; joint_count]),
            terrain_plants: [None; 2],
            plan: None,
            active: false,
            visibility: PoseVisibility::Visible,
            inertial_state: InertialState::Settled,
            ownership: PoseOwnership::Authored,
        }
    }
}
