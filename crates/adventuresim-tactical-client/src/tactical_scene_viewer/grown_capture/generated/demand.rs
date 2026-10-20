//! Freeze real camera demand before arming a fault or allowing a submission.
use super::super::ordinary_batches::inventory::{
    CurrentInventory, DrawDemand, PartIdentity, RootOrigin,
};
use super::*;
use bevy::{mesh::MeshTag, render::batching::NoAutomaticBatching};

#[derive(Resource, Default)]
pub(in super::super) struct Demand {
    pose: Option<HeldDemand>,
}

struct HeldDemand {
    view: usize,
    visible: Vec<Entity>,
    fault: Fault,
}

struct FaultTarget {
    root: Entity,
    part: Entity,
    group: Vec<Entity>,
    identity: PartIdentity,
    origin: RootOrigin,
}

enum Fault {
    Healthy,
    Applied {
        control: controls::BatchControl,
        target: FaultTarget,
    },
}

impl Demand {
    pub(in super::super) fn allows_hidden(&self, root: Entity) -> bool {
        self.pose.as_ref().is_some_and(|pose|matches!(&pose.fault,
            Fault::Applied {control:controls::BatchControl::HiddenLastRoot | controls::BatchControl::HiddenVistaRoot,target} if target.root==root))
    }

    pub(in super::super) fn allows_singleton(&self, part: Entity) -> bool {
        self.pose.as_ref().is_some_and(|pose|matches!(&pose.fault,
            Fault::Applied {control:controls::BatchControl::ForcedSingletons,target} if target.group.contains(&part)))
    }

    fn observe(
        &mut self,
        world: &mut World,
        view: usize,
        inventory: &mut CurrentInventory,
    ) -> Result<bool> {
        if let Some(held) = &self.pose
            && held.view == view
        {
            match &held.fault {
                Fault::Applied {
                    control:
                        controls::BatchControl::HiddenLastRoot | controls::BatchControl::HiddenVistaRoot,
                    ..
                } => {
                    // Required membership stays frozen after actual visibility drops.
                    inventory.demand = DrawDemand::FrozenCamera {
                        members: held.visible.clone(),
                    };
                }
                _ => ensure!(
                    held.visible == inventory.visible,
                    "held generated visibility demand changed"
                ),
            }
            return Ok(true);
        }
        let control = world.resource::<Review>().control;
        let fault = if view == 0
            || matches!(
                control,
                controls::BatchControl::Ordinary
                    | controls::BatchControl::ChangedSceneOwner
                    | controls::BatchControl::ParentedOriginalRoot
            ) {
            Fault::Healthy
        } else {
            let target = FaultTarget::from_visible(inventory, world, control)?;
            let report = serde_json::json!({"view":view,"control":control,
                "required_visible_entities":inventory.visible.iter().map(|entity|entity.to_bits()).collect::<Vec<_>>(),
                "target_root":target.root.to_bits(),"target_part":target.part.to_bits(),
                "target_group":target.group.iter().map(|entity|entity.to_bits()).collect::<Vec<_>>(),
                "part_identity":target.identity,"target_origin":target.origin,"frozen_before_fault":true});
            let output = world.resource::<SceneCaptureState>().output.clone();
            fs::write(
                output.join("grown-fault-demand.json"),
                serde_json::to_vec_pretty(&report)?,
            )?;
            target.apply(world, control)?;
            fs::write(
                output.join("grown-fault-applied.json"),
                serde_json::to_vec_pretty(&serde_json::json!({
                "view":view,"control":control,"fault_applied":true,
                "fresh_submission_required":true,"target_root":target.root.to_bits(),"target_part":target.part.to_bits()}))?,
            )?;
            Fault::Applied { control, target }
        };
        let eligible = matches!(fault, Fault::Healthy);
        self.pose = Some(HeldDemand {
            view,
            visible: inventory.visible.clone(),
            fault,
        });
        // The first faulted frame has already passed visibility. Withhold it.
        Ok(eligible)
    }
}

impl FaultTarget {
    fn from_visible(
        inventory: &CurrentInventory,
        world: &World,
        control: controls::BatchControl,
    ) -> Result<Self> {
        for member in inventory.members.iter().rev().filter(|member| {
            inventory.visible.contains(&member.entity)
                && (!control.requires_vista()
                    || inventory.roots.iter().any(|root| {
                        root.entity == member.root
                            && matches!(root.origin, RootOrigin::Vista { .. })
                    }))
                && (control != controls::BatchControl::MissingInactiveTag
                    || world
                        .get::<crate::presentation::GrownPartBinding>(member.entity)
                        .is_some())
        }) {
            let group: Vec<_> = inventory
                .members
                .iter()
                .filter(|other| {
                    other.part == member.part && inventory.visible.contains(&other.entity)
                })
                .map(|other| other.entity)
                .collect();
            if group.len() > 1 {
                let mut target = Self {
                    root: member.root,
                    part: member.entity,
                    group,
                    identity: member.part,
                    origin: inventory
                        .roots
                        .iter()
                        .find(|root| root.entity == member.root)
                        .context("fault target root ownership missing")?
                        .origin,
                };
                if control == controls::BatchControl::MissingInactiveTag {
                    let held = world.resource::<census::HeldScene>();
                    let root = held
                        .roots
                        .iter()
                        .find(|root| root.attachment.root == member.root)
                        .context("inactive control physical root")?;
                    let part = root
                        .parts()
                        .find(|part| part.binding.level != member.part.level)
                        .context("inactive control requires another retained level")?;
                    target.part = part.member.entity;
                    target.identity = part.member.part;
                    target.group = vec![part.member.entity];
                }
                return Ok(target);
            }
        }
        anyhow::bail!("generated fault requires a real visible multi-root part group")
    }

    fn apply(&self, world: &mut World, control: controls::BatchControl) -> Result<()> {
        match control {
            controls::BatchControl::HiddenLastRoot | controls::BatchControl::HiddenVistaRoot => {
                world.entity_mut(self.root).insert(Visibility::Hidden);
            }
            controls::BatchControl::MissingLastTag
            | controls::BatchControl::MissingVistaTag
            | controls::BatchControl::MissingInactiveTag => {
                world.entity_mut(self.part).remove::<MeshTag>();
            }
            controls::BatchControl::ForcedSingletons => {
                for entity in &self.group {
                    let mut part = world.entity_mut(*entity);
                    part.insert(NoAutomaticBatching);
                    // Bevy's retained material bins requeue changed Mesh3d,
                    // not a changed batching marker alone. Keep the same asset.
                    part.get_mut::<Mesh3d>()
                        .context("singleton control requires actual part mesh")?
                        .set_changed();
                }
            }
            controls::BatchControl::ChangedLevel => {
                let scene = world.resource::<census::HeldScene>().assets.scene();
                let request = world
                    .get::<crate::presentation::TreeContentRequest>(scene)
                    .context("generated level control actual request")?;
                let crate::presentation::TreeContentRequest::Grown(request) = request else {
                    anyhow::bail!("generated level control requires actual Grown request");
                };
                let changed = request.for_fixed_level(match self.identity.level {
                    Level::Full => Level::Middle,
                    Level::Middle | Level::Coarse => Level::Full,
                });
                world
                    .entity_mut(scene)
                    .insert(crate::presentation::TreeContentRequest::Grown(changed));
            }
            _ => anyhow::bail!("non-held generated control reached fault arming"),
        }
        Ok(())
    }
}

pub(in super::super) fn hold(
    world: &mut World,
    view: usize,
    inventory: &mut CurrentInventory,
) -> Result<bool> {
    world.resource_scope(|world, mut demand: Mut<Demand>| demand.observe(world, view, inventory))
}
