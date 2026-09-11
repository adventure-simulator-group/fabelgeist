//! Installed catalog equipment fixtures for the gameplay animation renderer.
use super::*;
use adventuresim_core::item_catalog::{self, EquipmentPlacement, ItemDefinition};
use clap::ValueEnum;
mod readiness;
use readiness::{EquipmentVisualRequirements, EquipmentVisualState, EquipmentVisualStatus};

#[derive(Clone, Copy, Debug, ValueEnum, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ArmorHarness {
    Plate,
    PlateTassets,
    Mail,
    Padded,
    CloseHelmet,
}

impl ArmorHarness {
    fn item_ids(self) -> impl Iterator<Item = &'static str> {
        let items: &'static [&'static str] = match self {
            Self::CloseHelmet => &["close_helmet"],
            Self::Plate | Self::PlateTassets => &[
                "morion",
                "gorget",
                "cuirass",
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
        };
        // These waist defenses occupy the same rigid-armor catalog slot.
        let waist = match self {
            Self::Plate => Some("fauld"),
            Self::PlateTassets => Some("tassets"),
            _ => None,
        };
        items.iter().copied().chain(waist)
    }

    fn visual_requirements(self) -> EquipmentVisualRequirements {
        match self {
            Self::CloseHelmet => EquipmentVisualRequirements {
                names: &["skull", "bevor", "visor"],
                morph_targets: Some(
                    adventuresim_core::character_morph::IDENTITY_MORPH_COUNT
                        + adventuresim_core::skeletal_fit::SkeletalFitMorph::ALL.len(),
                ),
            },
            _ => EquipmentVisualRequirements::default(),
        }
    }

    fn placements(
        self,
    ) -> impl Iterator<Item = (&'static ItemDefinition, &'static EquipmentPlacement)> {
        self.item_ids().flat_map(|item_id| {
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
        head: Option<Vec3>,
    ) -> Option<Transform> {
        let harness = self.harness?;
        const REVIEW_CAMERA_DISTANCE_METRES: f32 = 2.4;
        const REVIEW_CAMERA_ELEVATION_METRES: f32 = 0.2;
        const HELMET_REVIEW_DISTANCE_METRES: f32 = 0.72;
        const HELMET_FOCUS_ABOVE_HEAD_METRES: f32 = 0.07;
        let (focus, distance, elevation) = if matches!(harness, ArmorHarness::CloseHelmet) {
            (
                head? + Vec3::Y * HELMET_FOCUS_ABOVE_HEAD_METRES,
                HELMET_REVIEW_DISTANCE_METRES,
                0.0,
            )
        } else {
            (
                subject.translation,
                REVIEW_CAMERA_DISTANCE_METRES,
                REVIEW_CAMERA_ELEVATION_METRES,
            )
        };
        let offset = match view {
            CaptureView::Gameplay => return None,
            CaptureView::Side => Vec3::new(distance, elevation, 0.0),
            CaptureView::Front => Vec3::new(0.0, elevation, -distance),
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
    let was_ready = capture.ready;
    capture.ready = armor.iter().count() == harness.placements().count();
    for (entity, item) in &armor {
        match visuals.state(entity, harness.visual_requirements()) {
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
    if capture.ready && !was_ready {
        let parts = armor
            .iter()
            .map(|(entity, item)| {
                serde_json::json!({
                    "item_id": item.id, "parts": visuals.summary(entity),
                })
            })
            .collect::<Vec<_>>();
        fs::write(
            capture.output.join("armor-readiness.json"),
            serde_json::to_vec_pretty(&parts).expect("serialize armor readiness"),
        )
        .expect("write armor readiness");
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
    fn helmet_review_tracks_head_without_overriding_gameplay_camera() {
        let capture = ArmorCapture {
            harness: Some(ArmorHarness::CloseHelmet),
            output: PathBuf::new(),
            ready: false,
            waited: 0,
            failed: false,
        };
        let subject = Transform::from_xyz(2.0, 0.95, 3.0);
        let head = subject.translation + Vec3::Y * 0.7;
        assert!(
            capture
                .review_camera(CaptureView::Gameplay, &subject, Some(head))
                .is_none()
        );
        assert!(
            capture
                .review_camera(CaptureView::Front, &subject, None)
                .is_none()
        );
        let front = capture
            .review_camera(CaptureView::Front, &subject, Some(head))
            .unwrap();
        let side = capture
            .review_camera(CaptureView::Side, &subject, Some(head))
            .unwrap();
        assert_ne!(front.translation, side.translation);
        for camera in [front, side] {
            assert!(camera.translation.distance(head) < 1.0);
            assert!(
                camera
                    .forward()
                    .dot((head - camera.translation).normalize())
                    > 0.99
            );
        }
        let moved = capture
            .review_camera(CaptureView::Front, &subject, Some(head + Vec3::Y))
            .unwrap();
        assert!((moved.translation - front.translation).abs_diff_eq(Vec3::Y, 1e-5));
    }

    #[test]
    fn tassets_fixture_changes_only_the_conflicting_waist_defense() {
        let plate = ArmorHarness::Plate.item_ids().collect::<BTreeSet<_>>();
        let tassets = ArmorHarness::PlateTassets
            .item_ids()
            .collect::<BTreeSet<_>>();
        assert_eq!(
            plate.difference(&tassets).copied().collect::<Vec<_>>(),
            ["fauld"]
        );
        assert_eq!(
            tassets.difference(&plate).copied().collect::<Vec<_>>(),
            ["tassets"]
        );
        assert!(matches!(
            ArmorHarness::from_str("plate-tassets", false).unwrap(),
            ArmorHarness::PlateTassets
        ));
    }

    #[test]
    fn review_harnesses_use_nonconflicting_catalog_placements() {
        for (harness, count) in [
            (ArmorHarness::Plate, 22),
            (ArmorHarness::PlateTassets, 22),
            (ArmorHarness::Mail, 9),
            (ArmorHarness::Padded, 9),
            (ArmorHarness::CloseHelmet, 1),
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
