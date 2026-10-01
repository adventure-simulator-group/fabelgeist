//! Deduplicate wearer fits and keep GPU waits outside the frame update.
use super::*;
use crate::animation::skeletal_proportions::CharacterSkeletalProportions;
use adventuresim_core::character_proportions::CharacterProportions;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future};

/// Recently fitted variants retained across travel; live entities own their mesh
/// handles independently, so evicting a cache entry never removes visible armor.
const CACHED_FIT_LIMIT: usize = 128;

#[cfg(test)]
#[path = "generation_tests.rs"]
mod tests;

#[derive(Component, Clone, Debug, PartialEq, Eq, Hash)]
pub(in crate::equipment) struct FitKey {
    shape: BodyShapeKey,
    item: String,
    placement: String,
}

impl FitKey {
    fn for_presentation(
        canonical: &body::CanonicalBody,
        presentation: &RuntimeEquipmentPresentation,
        previous: Option<&Self>,
        owner: Option<&ItemOf>,
        fitted_to_owner: bool,
        characters: &Query<(&CharacterId, Option<&CharacterSkeletalProportions>)>,
    ) -> Option<Self> {
        let shape = if let Some(previous) = previous.filter(|_| !fitted_to_owner) {
            previous.shape.clone()
        } else if let Some(owner) = owner.filter(|_| fitted_to_owner) {
            let (id, explicit) = characters.get(owner.0).ok()?;
            BodyShapeKey::new(
                &canonical.names,
                Some(id.0),
                explicit
                    .map(|p| p.0)
                    .unwrap_or_else(|| CharacterProportions::from_character_id(id.0)),
                canonical.rig.reference,
            )
        } else {
            BodyShapeKey::new(
                &canonical.names,
                None,
                canonical.rig.reference,
                canonical.rig.reference,
            )
        };
        let placement = if fitted_to_owner {
            presentation.placement_id.clone()
        } else if let Some(previous) = previous {
            previous.placement.clone()
        } else {
            // A never-worn display item uses the first authored fit placement;
            // hand slots and world placement are not anatomical fitting regions.
            adventuresim_core::item_catalog::definition(&presentation.item_id)
                .and_then(|item| item.equipment.as_ref())
                .and_then(|equipment| equipment.placements.first())
                .map(|placement| placement.id.clone())
                .unwrap_or_else(|| presentation.placement_id.clone())
        };
        Some(Self {
            shape,
            item: presentation.item_id.clone(),
            placement,
        })
    }
}

/// Item-owned so placeholder rebuilds on drop do not lose the fitted shape.
#[derive(Component, Clone)]
pub(in crate::equipment) struct LastFit(FitKey);

pub(super) struct FittedBody {
    body: Arc<RuntimeBody>,
    inverse_bindposes: Handle<SkinnedMeshInverseBindposes>,
}

pub(super) struct PendingFit {
    key: FitKey,
    task: Task<anyhow::Result<GeneratedArmor>>,
}

impl RuntimeEquipmentBodyCache {
    fn trim(&mut self) {
        while self.models.len() > CACHED_FIT_LIMIT {
            let Some(key) = self
                .models
                .keys()
                .min_by_key(|key| self.last_used.get(*key).copied().unwrap_or(0))
                .cloned()
            else {
                break;
            };
            self.models.remove(&key);
            self.last_used.remove(&key);
        }
        self.bodies.retain(|shape, _| {
            self.models.keys().any(|key| &key.shape == shape)
                || self
                    .pending
                    .as_ref()
                    .is_some_and(|pending| &pending.key.shape == shape)
        });
        self.last_used.retain(|key, _| {
            self.models.contains_key(key)
                || self
                    .pending
                    .as_ref()
                    .is_some_and(|pending| &pending.key == key)
        });
    }
    fn prepare_shape(
        &mut self,
        key: &BodyShapeKey,
        bindposes: &mut Assets<SkinnedMeshInverseBindposes>,
    ) -> anyhow::Result<()> {
        if self.bodies.contains_key(key) {
            return Ok(());
        }
        let canonical = self.body.as_ref().context("canonical body not ready")?;
        let mut body = canonical.body.clone();
        key.apply(&mut body, &canonical.targets)?;
        let fitted_bindposes = canonical.rig.fit(&mut body, key.proportions())?;
        self.bodies.insert(
            key.clone(),
            FittedBody {
                body: Arc::new(body),
                inverse_bindposes: bindposes.add(fitted_bindposes),
            },
        );
        Ok(())
    }

    fn start(&mut self, key: FitKey) {
        let body = self.bodies[&key.shape].body.clone();
        let bracer = self.bracer_design.clone().expect("prepared bracer");
        let breastplate = self
            .breastplate_design
            .clone()
            .expect("prepared breastplate");
        let request = key.clone();
        self.pending = Some(PendingFit {
            key,
            task: AsyncComputeTaskPool::get().spawn(async move {
                let started = web_time::Instant::now();
                let result = adventuresim_character_creator::runtime_equipment::generate(
                    &body,
                    &request.item,
                    &request.placement,
                    &bracer,
                    &breastplate,
                )
                .await;
                info!(
                    item = request.item,
                    elapsed_ms = started.elapsed().as_secs_f64() * 1000.0,
                    vertices = result.as_ref().map_or(0, |armor| armor.positions.len()),
                    "runtime wearer fit"
                );
                result
            }),
        });
    }

    fn finish(&mut self, meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) {
        let Some(pending) = self.pending.as_mut() else {
            return;
        };
        let Some(result) = block_on(future::poll_once(&mut pending.task)) else {
            return;
        };
        let PendingFit { key, .. } = self.pending.take().expect("completed fit");
        let result = result
            .map(|generated| CachedEquipment {
                sockets: sockets::garment_sockets(
                    &self.bodies[&key.shape].body,
                    &generated,
                    &key.item,
                ),
                mesh: runtime_equipment_mesh(&generated, meshes),
                material: runtime_equipment_material(&key.item, materials),
            })
            .map_err(|error| format!("{error:#}"));
        if let Err(error) = &result {
            error!(item = key.item, "runtime wearer fit failed: {error}");
        }
        self.models.insert(key, result);
    }
}

impl CachedEquipment {
    fn attach(
        &self,
        commands: &mut Commands,
        entity: Entity,
        presentation: &RuntimeEquipmentPresentation,
        body: &FittedBody,
    ) {
        if !self.sockets.is_empty() {
            commands
                .entity(presentation.item)
                .insert(EquipmentAttachmentSockets(self.sockets.clone()));
        }
        let part = ProceduralEquipmentPart::new(
            presentation.item,
            body.inverse_bindposes.clone(),
            body.body.joint_names.clone(),
        );
        commands.entity(entity).with_child(part.render_bundle(
            format!("Runtime equipment {}", presentation.item_id),
            self.mesh.clone(),
            self.material.clone(),
        ));
        commands
            .entity(entity)
            .insert((ProceduralEquipmentResolved, Visibility::Inherited));
    }
}

#[expect(
    clippy::too_many_arguments,
    clippy::type_complexity,
    reason = "equipment job integration and wearer-owned fitting inputs"
)]
pub(in crate::equipment) fn generate_runtime_equipment_models(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut bindposes: ResMut<Assets<SkinnedMeshInverseBindposes>>,
    mut cache: ResMut<RuntimeEquipmentBodyCache>,
    presentations: Query<(
        Entity,
        &RuntimeEquipmentPresentation,
        Option<&FitKey>,
        Has<ProceduralEquipmentResolved>,
        Option<&Children>,
    )>,
    parts: Query<(), With<ProceduralEquipmentPart>>,
    items: Query<(
        Option<&ItemOf>,
        Has<TacticalSceneItem>,
        Option<&LastFit>,
        Option<&EquipSlot>,
    )>,
    characters: Query<(&CharacterId, Option<&CharacterSkeletalProportions>)>,
) {
    cache.finish(&mut meshes, &mut materials);
    for (entity, presentation, current, resolved, children) in &presentations {
        if cache.failed {
            commands
                .entity(entity)
                .insert((ProceduralEquipmentFailed, ProceduralEquipmentResolved));
            continue;
        }
        let Some(canonical) = cache.body.as_ref() else {
            return;
        };
        if cache.bracer_design.is_none() || cache.breastplate_design.is_none() {
            return;
        }
        let Ok((owner, scene, last_fit, slot)) = items.get(presentation.item) else {
            continue;
        };
        let fitted_to_owner = !scene && holding_side(slot).is_none() && owner.is_some();
        let previous = last_fit.map(|fit| &fit.0).or(current);
        let Some(key) = FitKey::for_presentation(
            canonical,
            presentation,
            previous,
            owner,
            fitted_to_owner,
            &characters,
        ) else {
            continue;
        };
        cache.use_clock = cache.use_clock.wrapping_add(1);
        let clock = cache.use_clock;
        cache.last_used.insert(key.clone(), clock);
        if current == Some(&key) && resolved {
            continue;
        }
        if current != Some(&key) {
            if let Some(children) = children {
                for child in children.iter() {
                    if parts.contains(child) {
                        commands.entity(child).despawn();
                    }
                }
            }
            commands
                .entity(entity)
                .insert(key.clone())
                .remove::<(ProceduralEquipmentResolved, ProceduralEquipmentFailed)>();
            commands
                .entity(presentation.item)
                .insert(LastFit(key.clone()))
                .remove::<EquipmentAttachmentSockets>();
        }
        if let Some(result) = cache.models.get(&key) {
            let Ok(model) = result else {
                commands
                    .entity(entity)
                    .insert((ProceduralEquipmentResolved, ProceduralEquipmentFailed));
                continue;
            };
            let body = &cache.bodies[&key.shape];
            model.attach(&mut commands, entity, presentation, body);
        } else if cache.pending.is_none() {
            match cache.prepare_shape(&key.shape, &mut bindposes) {
                Ok(()) => cache.start(key),
                Err(error) => {
                    cache.models.insert(key, Err(format!("{error:#}")));
                }
            }
        }
    }
    cache.trim();
}
