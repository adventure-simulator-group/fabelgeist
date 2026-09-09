//! Installed catalog equipment fixtures for the gameplay animation renderer.
use super::*;
use adventuresim_core::item_catalog::{self, EquipmentPlacement, ItemDefinition};
use clap::ValueEnum;
mod readiness;
use readiness::{EquipmentVisualState, EquipmentVisualStatus};

#[derive(Clone, Copy, Debug, ValueEnum, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ArmorHarness {
    Plate,
    Mail,
    Padded,
}

impl ArmorHarness {
    fn item_ids(self) -> &'static [&'static str] {
        match self {
            Self::Plate => &[
                "morion",
                "gorget",
                "cuirass",
                "fauld",
                "spaulder",
                "rerebrace",
                "couter",
                "vambrace",
                "mitten_gauntlet",
                "cuisse",
                "poleyn",
                "greave",
                "sabaton",
            ],
            Self::Mail => &[
                "mail_coif",
                "mail_shirt",
                "mail_sleeve",
                "mail_skirt",
                "mail_chausses",
                "leather_boot",
            ],
            Self::Padded => &[
                "arming_cap",
                "arming_doublet",
                "quilted_sleeve",
                "padded_skirt",
                "padded_chausses",
                "leather_boot",
            ],
        }
    }

    fn placements(
        self,
    ) -> impl Iterator<Item = (&'static ItemDefinition, &'static EquipmentPlacement)> {
        self.item_ids().iter().flat_map(|item_id| {
            let definition = item_catalog::definition(item_id).expect("authored harness item");
            definition
                .equipment
                .as_ref()
                .expect("harness equipment definition")
                .placements
                .iter()
                .map(move |placement| (definition, placement))
        })
    }
}

#[derive(Component)]
pub(super) struct CapturedArmor;

#[derive(Resource)]
pub(super) struct ArmorCapture {
    harness: Option<ArmorHarness>,
    output: PathBuf,
    ready: bool,
    waited: u32,
    failed: bool,
}

impl ArmorCapture {
    /// Keep the complete articulated harness legible in the two inspection views.
    pub(super) fn review_camera(
        &self,
        view: CaptureView,
        subject: &Transform,
    ) -> Option<Transform> {
        self.harness?;
        const REVIEW_CAMERA_DISTANCE_METRES: f32 = 2.4;
        const REVIEW_CAMERA_ELEVATION_METRES: f32 = 0.2;
        let focus = subject.translation;
        let offset = match view {
            CaptureView::Gameplay => return None,
            CaptureView::Side => Vec3::new(
                REVIEW_CAMERA_DISTANCE_METRES,
                REVIEW_CAMERA_ELEVATION_METRES,
                0.0,
            ),
            CaptureView::Front => Vec3::new(
                0.0,
                REVIEW_CAMERA_ELEVATION_METRES,
                -REVIEW_CAMERA_DISTANCE_METRES,
            ),
        };
        Some(Transform::from_translation(focus + offset).looking_at(focus, Vec3::Y))
    }

    pub(super) fn new(harness: Option<ArmorHarness>, output: PathBuf) -> Self {
        if let Some(harness) = harness {
            let pieces = harness.placements().map(|(item, placement)| serde_json::json!({"item_id": item.id, "placement_id": placement.id})).collect::<Vec<_>>();
            let manifest = serde_json::json!({"harness": harness, "pieces": pieces, "renderer": "gameplay_equipment_glb_skin_morph"});
            fs::write(
                output.join("armor-fixture.json"),
                serde_json::to_vec_pretty(&manifest).expect("serialize armor fixture"),
            )
            .expect("write armor fixture");
        }
        Self {
            harness,
            output,
            ready: harness.is_none(),
            waited: 0,
            failed: false,
        }
    }

    pub(super) fn spawn(&self, commands: &mut Commands, subject: Entity) {
        let Some(harness) = self.harness else { return };
        for (item, placement) in harness.placements() {
            let physical = &item.equipment.as_ref().expect("harness equipment").physical;
            commands.spawn((
                CapturedArmor,
                Name::new(format!("Armor review {}/{}", item.id, placement.id)),
                ItemOf(subject),
                ItemProperties {
                    id: item.id.clone(),
                    weight: item.weight_kg,
                },
                EquipmentTopology {
                    placement_id: Some(placement.id.clone()),
                    occupancies: placement
                        .occupancy
                        .iter()
                        .enumerate()
                        .map(|(index, requirement)| EquipmentTopologyOccupancy {
                            occupancy_id: format!(
                                "armor-review:{}:{}:{index}",
                                item.id, placement.id
                            ),
                            anchor: TacticalEquipmentAnchor::CharacterLocation(
                                requirement.location,
                            ),
                            channel: requirement.channel,
                            order: requirement.order,
                            requirement_index: index as u16,
                            capacity_index: 0,
                        })
                        .collect(),
                },
                TacticalEquipmentPhysical {
                    dimensions_m: Vec3::from_array(physical.dimensions_m),
                    grip_to_tip_m: physical.grip_to_tip_m,
                    striking_head_length_m: 0.0,
                    anchor_offset_m: Vec3::from_array(physical.anchor_offset_m),
                },
                Transform::default(),
            ));
        }
    }

    fn fail(&mut self, reason: &str, exit: &mut MessageWriter<AppExit>) {
        self.failed = true;
        fs::write(self.output.join("failure.txt"), reason).expect("write armor capture failure");
        error!(%reason, "Armor capture cannot use unresolved equipment");
        exit.write(AppExit::error());
    }
}

pub(super) fn update_readiness(
    mut capture: ResMut<ArmorCapture>,
    armor: Query<(Entity, &ItemProperties), With<CapturedArmor>>,
    visuals: EquipmentVisualStatus,
    mut exit: MessageWriter<AppExit>,
) {
    let Some(harness) = capture.harness else {
        return;
    };
    if capture.failed {
        return;
    }
    capture.ready = armor.iter().count() == harness.placements().count();
    for (entity, item) in &armor {
        match visuals.state(entity) {
            EquipmentVisualState::Ready => {}
            EquipmentVisualState::Failed => {
                capture.fail(
                    &format!("Procedural equipment asset failed for {}", item.id),
                    &mut exit,
                );
                return;
            }
            EquipmentVisualState::Missing | EquipmentVisualState::Loading => capture.ready = false,
        }
    }
    if !capture.ready {
        capture.waited += 1;
        if capture.waited >= CAPTURE_LOAD_FRAME_LIMIT {
            capture.fail("Timed out waiting for every armor mesh, material, morph, and wearer skin to resolve", &mut exit);
        }
    }
}

pub(super) fn ready(capture: Res<ArmorCapture>) -> bool {
    capture.ready && !capture.failed
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_core::equipment::{EquipmentGraph, EquipmentGraphPlacement};

    #[test]
    fn review_harnesses_use_nonconflicting_catalog_placements() {
        for (harness, count) in [
            (ArmorHarness::Plate, 22),
            (ArmorHarness::Mail, 9),
            (ArmorHarness::Padded, 9),
        ] {
            let mut graph = EquipmentGraph::default();
            for (index, (_, placement)) in harness.placements().enumerate() {
                graph
                    .equip(
                        index as u64,
                        EquipmentGraphPlacement {
                            body: placement.occupancy.clone(),
                            parents: vec![],
                        },
                    )
                    .unwrap();
            }
            assert_eq!(graph.nodes.len(), count);
        }
    }
}
