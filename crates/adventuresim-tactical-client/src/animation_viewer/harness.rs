//! Installed catalog equipment fixtures for the gameplay animation renderer.
use super::*;
use adventuresim_core::item_catalog::{self, EquipmentPlacement, ItemDefinition};
use clap::ValueEnum;
mod material;
mod museum;
mod readiness;
mod skinning;
mod topology;
use readiness::{EquipmentVisualRequirements, EquipmentVisualState, EquipmentVisualStatus};

#[derive(Clone, Copy, Debug, ValueEnum, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ArmorHarness {
    Plate,
    PlateTassets,
    PlateUnderlayers,
    Underlayers,
    Mail,
    Padded,
    CloseHelmet,
    MuseumHenry,
    MuseumNuremberg,
}

impl ArmorHarness {
    fn item_ids(self) -> impl Iterator<Item = &'static str> {
        let items: &'static [&'static str] = match self {
            Self::CloseHelmet => &["close_helmet"],
            Self::MuseumHenry => museum::HENRY_ITEMS,
            Self::MuseumNuremberg => museum::NUREMBERG_ITEMS,
            Self::Plate | Self::PlateTassets | Self::PlateUnderlayers => &[
                "morion",
                "gorget",
                "cuirass",
                "pauldron",
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
                "padded_chausses",
                "leather_boot",
            ],
            Self::Underlayers => &[],
        };
        // These waist defenses occupy the same rigid-armor catalog slot.
        let waist = match self {
            Self::Plate | Self::PlateUnderlayers => Some("fauld"),
            Self::PlateTassets => Some("tassets"),
            _ => None,
        };
        let underlayers: &[&str] = if matches!(self, Self::Underlayers | Self::PlateUnderlayers) {
            &[
                "arming_doublet",
                "padded_chausses",
                "mail_voiders",
                "mail_brayette",
                "mail_knee_voider",
            ]
        } else {
            &[]
        };
        items
            .iter()
            .copied()
            .chain(waist)
            .chain(underlayers.iter().copied())
            .chain(matches!(self, Self::Underlayers).then_some("mail_standard"))
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
            Self::Underlayers
            | Self::PlateUnderlayers
            | Self::MuseumHenry
            | Self::MuseumNuremberg => EquipmentVisualRequirements {
                names: &[],
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
            let manifest = serde_json::json!({"harness": harness, "pieces": pieces, "renderer": "gameplay_equipment_glb_skin_morph", "identity": "deterministic_character_id_variation"});
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
        let mut previous = Vec::new();
        for (item, placement) in harness.placements() {
            let physical = &item.equipment.as_ref().expect("harness equipment").physical;
            let occupancies = topology::occupancies(item, placement, &previous);
            let entity = commands
                .spawn((
                    CapturedArmor,
                    Name::new(format!("Armor review {}/{}", item.id, placement.id)),
                    ItemOf(subject),
                    ItemProperties {
                        id: item.id.clone(),
                        weight: item.weight_kg,
                    },
                    EquipmentTopology {
                        placement_id: Some(placement.id.clone()),
                        occupancies,
                    },
                    TacticalEquipmentPhysical {
                        dimensions_m: Vec3::from_array(physical.dimensions_m),
                        grip_to_tip_m: physical.grip_to_tip_m,
                        striking_head_length_m: 0.0,
                        anchor_offset_m: Vec3::from_array(physical.anchor_offset_m),
                    },
                    Transform::default(),
                ))
                .id();
            previous.push((item, entity, placement));
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
                fs::write(
                    capture.output.join("armor-readiness-failure.json"),
                    serde_json::to_vec_pretty(&serde_json::json!({
                        "item_id": item.id, "parts": visuals.summary(entity),
                    }))
                    .expect("serialize failed armor readiness"),
                )
                .expect("write failed armor readiness");
                capture.fail(
                    &format!(
                        "Procedural equipment asset or primary skin weights failed for {}",
                        item.id
                    ),
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
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn voiders_attach_to_doublet_while_retaining_the_wearer_for_skin_binding() {
        for harness in [ArmorHarness::Underlayers, ArmorHarness::PlateUnderlayers] {
            let mut world = World::new();
            let wearer = world.spawn_empty().id();
            let capture = ArmorCapture {
                harness: Some(harness),
                output: PathBuf::new(),
                ready: false,
                waited: 0,
                failed: false,
            };
            world
                .run_system_once(move |mut commands: Commands| {
                    capture.spawn(&mut commands, wearer);
                })
                .unwrap();
            let mut items = world.query::<(Entity, &ItemProperties, &EquipmentTopology, &ItemOf)>();
            let doublet = items
                .iter(&world)
                .find(|(_, item, _, _)| item.id == "arming_doublet")
                .unwrap()
                .0;
            let (_, _, topology, owner) = items
                .iter(&world)
                .find(|(_, item, _, _)| item.id == "mail_voiders")
                .unwrap();
            assert_eq!(owner.0, wearer);
            assert_eq!(topology.occupancies.len(), 1);
            assert!(matches!(
                &topology.occupancies[0].anchor,
                TacticalEquipmentAnchor::ItemAttachment { parent, attachment_point_id }
                    if *parent == doublet && attachment_point_id == "mail_voiders"
            ));
            assert_eq!(
                harness.visual_requirements().morph_targets,
                Some(
                    adventuresim_core::character_morph::IDENTITY_MORPH_COUNT
                        + adventuresim_core::skeletal_fit::SkeletalFitMorph::ALL.len()
                )
            );
            let mut knees = 0;
            for (_, _, topology, owner) in items
                .iter(&world)
                .filter(|(_, item, _, _)| item.id == "mail_knee_voider")
            {
                assert_eq!(owner.0, wearer);
                let TacticalEquipmentAnchor::ItemAttachment { parent, .. } =
                    topology.occupancies[0].anchor
                else {
                    panic!("knee mail must use the supporting hose");
                };
                let parent_item = world.get::<ItemProperties>(parent).unwrap();
                let parent_topology = world.get::<EquipmentTopology>(parent).unwrap();
                assert_eq!(parent_item.id, "padded_chausses");
                assert_eq!(topology.placement_id, parent_topology.placement_id);
                knees += 1;
            }
            assert_eq!(knees, 2);
        }
    }

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
            (ArmorHarness::Padded, 6),
            (ArmorHarness::Underlayers, 8),
            (ArmorHarness::PlateUnderlayers, 29),
            (ArmorHarness::CloseHelmet, 1),
            (ArmorHarness::MuseumHenry, 21),
            (ArmorHarness::MuseumNuremberg, 26),
        ] {
            let mut graph = EquipmentGraph::default();
            let mut previous = Vec::new();
            for (index, (item, placement)) in harness.placements().enumerate() {
                let entity = Entity::from_raw_u32(index as u32).unwrap();
                let occupancies = topology::occupancies(item, placement, &previous);
                let parents = occupancies
                    .iter()
                    .filter_map(|occupancy| {
                        if let TacticalEquipmentAnchor::ItemAttachment {
                            parent,
                            attachment_point_id,
                        } = &occupancy.anchor
                        {
                            Some(adventuresim_core::equipment::EquipmentGraphEdge {
                                parent_inventory_item_id: previous
                                    .iter()
                                    .position(|(_, entity, _)| entity == parent)
                                    .unwrap()
                                    as u64,
                                attachment_point_id: attachment_point_id.clone(),
                                capacity_index: occupancy.capacity_index,
                            })
                        } else {
                            None
                        }
                    })
                    .collect();
                graph
                    .equip(
                        index as u64,
                        EquipmentGraphPlacement {
                            body: placement.occupancy.clone(),
                            parents,
                        },
                    )
                    .unwrap_or_else(|error| {
                        panic!("{harness:?} {}/{}: {error}", item.id, placement.id)
                    });
                previous.push((item, entity, placement));
            }
            assert_eq!(graph.nodes.len(), count);
        }
    }
}
