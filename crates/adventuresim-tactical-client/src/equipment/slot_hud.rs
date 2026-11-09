//! The QWERTY slot grid shown at the bottom of the screen while a hand grabs:
//! one cell per input with the item at its outermost layer, the held and
//! other hand below. The page draws it (`tactical-hud.js`); this module builds
//! the view and pushes it when it changes, and `null` when no hand grabs.
//!
//! Slots are chosen with their keys. With the cursor free, the page also
//! sends a click on a cell or on the other hand back as a [`SlotHudPress`].
use super::*;
use crate::hud::{self, HudView, LastPushed};
use bevy::ecs::system::SystemParam;
use serde::{Deserialize, Serialize};

const SLOT_GRID_ROWS: u8 = 4;
const SLOT_GRID_COLUMNS: u8 = 7;
/// Atlas icon of an empty hand.
const EMPTY_HAND_ICON: &str = "mailed-fist";
/// Atlas icon of an item the catalogue does not know.
const UNKNOWN_ITEM_ICON: &str = "help";

type SlotItemQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static ItemOf,
        Option<&'static EquipSlot>,
        &'static ItemProperties,
        &'static EquipmentTopology,
    ),
>;

#[derive(SystemParam)]
pub(super) struct SlotHudSources<'w, 's> {
    items: SlotItemQuery<'w, 's>,
    scene_items: Query<'w, 's, (Entity, &'static ItemProperties), With<TacticalSceneItem>>,
    weapons: Query<'w, 's, (Entity, &'static WeaponAppearance)>,
    holders: Query<'w, 's, (Entity, &'static WeaponHolderAppearance)>,
}

#[derive(Debug, PartialEq, Serialize)]
struct SlotCellView {
    row: u8,
    column: u8,
    /// The input that addresses the cell; a click sends it back.
    input: &'static str,
    /// The input as printed on the key.
    key: String,
    /// The held item can go here (or, with a free hand, this can be drawn).
    eligible: bool,
    selected: bool,
    /// Flashes red: a refused placement, on a cell that cannot take it.
    refused: bool,
    icon: Option<IconView>,
    /// Cells showing one item that spans several locations share this, and
    /// the page joins adjacent ones.
    join: Option<u64>,
    tooltip: String,
}

#[derive(Debug, PartialEq, Serialize)]
struct SlotHudView {
    rows: u8,
    columns: u8,
    cells: Vec<SlotCellView>,
    /// The grabbing hand's item, or the empty hand.
    active: IconView,
    other: IconView,
    active_tooltip: &'static str,
    /// The scene item a free hand would pick up on release.
    pickup: Option<PickupView>,
}

#[derive(Debug, PartialEq, Serialize)]
struct PickupView {
    icon: IconView,
    tooltip: String,
}

fn catalog_icon<'a>(item: &'a ItemProperties, fallback: &'static str) -> &'a str {
    item_catalog::definition(&item.id)
        .map_or(fallback, |definition| definition.presentation.icon.as_str())
}

fn item_icon(entity: Entity, slug: &str, portraits: &HashMap<Entity, IconView>) -> IconView {
    portraits
        .get(&entity)
        .cloned()
        .unwrap_or_else(|| IconView::atlas(slug))
}

fn hand_icon(
    item: Option<(Entity, &ItemProperties)>,
    portraits: &HashMap<Entity, IconView>,
) -> IconView {
    item.map_or_else(
        || IconView::atlas(EMPTY_HAND_ICON),
        |(entity, properties)| {
            item_icon(entity, catalog_icon(properties, EMPTY_HAND_ICON), portraits)
        },
    )
}

fn other_hand(hand: EquipmentHand) -> EquipmentHand {
    match hand {
        EquipmentHand::Left => EquipmentHand::Right,
        EquipmentHand::Right => EquipmentHand::Left,
    }
}

impl SlotHudView {
    fn new(
        session: &GrabSession,
        hand: EquipmentHand,
        actor: Entity,
        sources: &SlotHudSources,
        portraits: &HashMap<Entity, IconView>,
    ) -> Self {
        let items = &sources.items;
        let in_hand = |hand: EquipmentHand| {
            items
                .iter()
                .find(|(_, owner, slot, _, _)| {
                    owner.0 == actor && slot.is_some_and(|slot| *slot == hand.slot())
                })
                .map(|(entity, _, _, properties, _)| (entity, properties))
        };
        let held = in_hand(hand);
        let other = other_hand(hand);
        let pickup = match session.selection {
            Some(GrabSelection::SceneItem(entity)) if held.is_none() => sources
                .scene_items
                .get(entity)
                .ok()
                .map(|(entity, item)| PickupView {
                    icon: item_icon(entity, catalog_icon(item, UNKNOWN_ITEM_ICON), portraits),
                    tooltip: format!("Release to pick up {}", item.id),
                }),
            _ => None,
        };
        Self {
            rows: SLOT_GRID_ROWS,
            columns: SLOT_GRID_COLUMNS,
            cells: INPUT_ADDRESS_MAPPINGS
                .iter()
                .filter(|mapping| {
                    mapping.keyboard_row < SLOT_GRID_ROWS
                        && mapping.keyboard_column < SLOT_GRID_COLUMNS
                })
                .map(|mapping| {
                    slot_cell(
                        session,
                        mapping,
                        actor,
                        held.map(|(_, item)| item),
                        items,
                        portraits,
                    )
                })
                .collect(),
            active: hand_icon(held, portraits),
            other: hand_icon(in_hand(other), portraits),
            active_tooltip: match hand {
                EquipmentHand::Left => "Active left hand",
                EquipmentHand::Right => "Active right hand",
            },
            pickup,
        }
    }
}

fn slot_cell(
    session: &GrabSession,
    mapping: &adventuresim_core::equipment::InputAddressMapping,
    actor: Entity,
    held: Option<&ItemProperties>,
    items: &SlotItemQuery,
    portraits: &HashMap<Entity, IconView>,
) -> SlotCellView {
    let slot =
        slot_selection::hud_slot(session, mapping, HeldPreviewItem::from(held), |location| {
            hud_layers(actor, location, items)
        });
    let location = slot.map_or(mapping.locations[0], |slot| slot.location);
    let layers = hud_layers(actor, location, items);
    let outermost = outermost_occupied_depth(&layers)
        .and_then(|depth| layers.get(depth))
        .and_then(|layer| items.get(layer.addressed_entity()).ok());
    let eligible = slot.is_some();
    let key = mapping.input.to_uppercase();
    let tooltip = match (held, outermost) {
        (Some(held), _) if eligible => format!("{key}: place or swap {}", held.id),
        (Some(held), _) => format!("{key}: {} cannot be placed here", held.id),
        (None, Some((_, _, _, item, _))) => format!("{key}: draw {}", item.id),
        (None, None) => format!("{key}: empty"),
    };
    let (icon, join) = outermost.map_or((None, None), |(entity, _, _, item, topology)| {
        let spans_locations = topology
            .occupancies
            .iter()
            .filter(|occupancy| {
                matches!(
                    occupancy.anchor,
                    TacticalEquipmentAnchor::CharacterLocation(_)
                )
            })
            .count()
            > 1;
        (
            Some(item_icon(
                entity,
                catalog_icon(item, UNKNOWN_ITEM_ICON),
                portraits,
            )),
            spans_locations.then(|| entity.to_bits()),
        )
    });
    SlotCellView {
        row: mapping.keyboard_row,
        column: mapping.keyboard_column,
        input: mapping.input,
        key,
        eligible,
        selected: matches!(
            session.selection,
            Some(GrabSelection::Slot { location: found, .. }) if found == location
        ),
        refused: session.invalid_flash_remaining > 0.0 && !eligible,
        icon,
        join,
        tooltip,
    }
}

/// A click on the slot HUD, sent by the browser page.
#[derive(Event, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub(crate) enum SlotHudPress {
    /// Selects the cell's slot, as its key would.
    Slot { input: String },
    /// Moves or swaps the held item with the other hand.
    OtherHand,
}

pub(super) fn apply_slot_hud_press(
    press: On<SlotHudPress>,
    mut session: ResMut<GrabSession>,
    player: Query<Entity, With<ClientPlayer>>,
    items: SlotItemQuery,
) {
    let (Some(hand), Ok(actor)) = (session.active, player.single()) else {
        return;
    };
    match &*press {
        SlotHudPress::OtherHand => {
            session.selection = Some(GrabSelection::Hand(other_hand(hand)));
        }
        SlotHudPress::Slot { input } => {
            let Some(mapping) = INPUT_ADDRESS_MAPPINGS
                .iter()
                .find(|mapping| mapping.input == input)
            else {
                return;
            };
            let held = items
                .iter()
                .find(|(_, owner, slot, _, _)| {
                    owner.0 == actor && slot.is_some_and(|slot| *slot == hand.slot())
                })
                .map(|(_, _, _, properties, _)| properties);
            let slot = slot_selection::hud_slot(
                &session,
                mapping,
                HeldPreviewItem::from(held),
                |location| hud_layers(actor, location, &items),
            );
            if let Some(slot) = slot {
                slot_selection::apply_slot(&mut session, mapping, slot);
            }
        }
    }
}

pub(super) fn push_slot_hud(
    session: Res<GrabSession>,
    mut icon_cache: ResMut<WeaponIconCache>,
    player: Query<Entity, With<ClientPlayer>>,
    sources: SlotHudSources,
    mut last: Local<LastPushed<String>>,
) {
    let view = match (session.active, player.single()) {
        (Some(hand), Ok(actor)) => {
            let mut portraits =
                procedural_icons(&mut icon_cache, &sources.weapons, &sources.holders);
            portraits.extend(armor_icons(&sources.items, &sources.scene_items));
            Some(SlotHudView::new(
                &session, hand, actor, &sources, &portraits,
            ))
        }
        _ => None,
    };
    let Ok(json) = serde_json::to_string(&view) else {
        return;
    };
    if let Some(json) = last.update(json) {
        hud::view(HudView::Slots, json);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_presses_are_typed() {
        assert!(matches!(
            serde_json::from_str::<SlotHudPress>(r#"{"kind":"slot","input":"q"}"#).unwrap(),
            SlotHudPress::Slot { input } if input == "q"
        ));
        assert!(matches!(
            serde_json::from_str::<SlotHudPress>(r#"{"kind":"other-hand"}"#).unwrap(),
            SlotHudPress::OtherHand
        ));
        assert!(serde_json::from_str::<SlotHudPress>(r#"{"kind":"drop"}"#).is_err());
    }

    #[test]
    fn every_input_fits_the_slot_grid() {
        assert!(INPUT_ADDRESS_MAPPINGS.iter().all(|mapping| {
            mapping.keyboard_row < SLOT_GRID_ROWS && mapping.keyboard_column < SLOT_GRID_COLUMNS
        }));
    }

    #[test]
    fn unknown_icons_fall_back_to_the_first_atlas_cell() {
        assert_eq!(
            IconView::atlas("not-an-icon"),
            IconView::atlas(EQUIPMENT_ICON_SLUGS[0])
        );
        assert!(matches!(
            IconView::atlas(EMPTY_HAND_ICON),
            IconView::Atlas {
                column: 4,
                row: 3,
                ..
            }
        ));
    }
}
