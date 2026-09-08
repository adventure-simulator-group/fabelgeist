use super::*;

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
            Option<&Name>,
            Option<&ChildOf>,
        )>,
        transforms: &Query<&Transform>,
    ) -> Option<PoseBufferRig> {
        let found = targets
            .iter()
            .filter(|(_, _, bind, _, _)| bind.owner == owner)
            .map(|(entity, target, bind, name, parent)| {
                (
                    entity,
                    *target,
                    LocalPose::from_transform(bind.local),
                    name.map(|name| name.as_str().to_owned()),
                    parent.map(ChildOf::parent),
                )
            })
            .collect::<Vec<_>>();
        if found.is_empty() {
            return None;
        }
        let definition = self
            .0
            .entry(family.clone())
            .or_insert_with(|| {
                let mut ordered = found
                    .iter()
                    .map(|(entity, target, bind, name, parent)| {
                        (*entity, target, bind, name, *parent)
                    })
                    .collect::<Vec<_>>();
                ordered.sort_by(|left, right| left.3.cmp(right.3));
                let indices = ordered
                    .iter()
                    .enumerate()
                    .map(|(index, (entity, ..))| (*entity, index))
                    .collect::<HashMap<_, _>>();
                Arc::new(RigDefinition {
                    family: family.clone(),
                    joints: ordered
                        .into_iter()
                        .map(|(_entity, target, bind, name, parent)| RigJoint {
                            target: *target,
                            bind: *bind,
                            parent: parent.and_then(|parent| indices.get(&parent).copied()),
                            name: name.clone(),
                            lower_body: name.as_deref().is_some_and(is_lower_body_animation_target),
                        })
                        .collect(),
                })
            })
            .clone();
        let by_target = found
            .into_iter()
            .map(|(entity, target, _, _, _)| (target, entity))
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
        }
    }
}
