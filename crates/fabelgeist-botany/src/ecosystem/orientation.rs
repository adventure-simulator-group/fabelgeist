//! Building native environment requests and applying a complete result batch.
use super::*;

impl Ecosystem {
    pub(super) async fn orient_new_modules(&mut self, report: &mut StepReport) -> Result<()> {
        if report.spawned > 0 {
            let (offsets, total) = self.offsets();
            let set = self.sphere_set(&offsets, total);
            let mut requests = Vec::with_capacity(report.spawned as usize);
            for (plant, &offset) in self.plants.iter().zip(&offsets) {
                let s = &self.species[plant.species.index()];
                for (u, module) in plant.modules.iter().enumerate() {
                    if !module.fresh {
                        continue;
                    }
                    let (center, radius) =
                        self.extents[plant.species.index()][module.prototype.index()];
                    let root = plant
                        .attachment(&self.prototypes, ModuleIndex::from_word(u as u32))
                        .ok_or(BotanyError::MissingAttachment {
                            plant: plant.id,
                            module: module.id,
                        })?;
                    let q = module.orientation;
                    // Laterals aim at the species' set-point angle (Digby
                    // and Firn's gravitropic set-point, which the paper
                    // cites); the leader -- the trunk's own continuation --
                    // aims straight up.
                    let cos_target = if module.order == 0 {
                        1.0
                    } else {
                        s.tropism_angle.cos()
                    };
                    // Whether, turned any way, it could reach a solid.
                    let near = self
                        .obstacles
                        .as_ref()
                        .is_some_and(|o| o.distance(root) < center.length() + radius + 0.5);
                    requests.push(OrientRequest {
                        root: root.to_array(),
                        radius,
                        local_center: center.to_array(),
                        index: (offset + u) as u32,
                        rotation: [q.x, q.y, q.z, q.w],
                        axis: self.prototypes.get(module.prototype).axis.to_array(),
                        cos_target,
                        collision_weight: s.collision_weight,
                        tropism_weight: s.tropism_weight,
                        parent: set.parents[offset + u],
                        obstacles: u32::from(near),
                    });
                }
            }
            let oriented = self
                .environment
                .orient(&set, &requests, &self.settings.environment)
                .await?;
            let limit = self.settings.bud_crowding_limit;
            let expected = requests.len();
            let actual = oriented.len();
            if actual != expected {
                return Err(BotanyError::OrientationCount { expected, actual });
            }
            let mut next = oriented.into_iter();
            for plant in &mut self.plants {
                let mut keep = vec![true; plant.modules.len()];
                let mut dormant = false;
                for (u, module) in plant.modules.iter_mut().enumerate() {
                    if module.fresh {
                        let result = next
                            .next()
                            .ok_or(BotanyError::OrientationCount { expected, actual })?;
                        module.orientation = result.rotation;
                        module.fresh = false;
                        if result.crowding > limit {
                            keep[u] = false;
                            dormant = true;
                        }
                    }
                }
                if dormant {
                    report.spawned -= keep.iter().filter(|k| !**k).count() as u32;
                    plant.discard(&keep);
                }
            }
        }
        Ok(())
    }
}
