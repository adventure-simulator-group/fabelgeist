# Strategic interface interaction contract

The place facade and its sign identify the physical location. Circular character
portraits have attached, always-visible rounded spokes for profile,
conversation, and inventory. Curved shoulders tuck under the portrait rim;
separate edges and shadows distinguish the overlapping tabs. The profile tab
sits in front of conversation, then inventory, like stacked folder tabs. This
order stays fixed when selecting a view. The selected tab has a bright outer
lip; controls retain keyboard focus outlines and forced-color borders.
Clicking the portrait opens its profile. One tab and its portrait ring carry
the active-view highlight; the location retains a
quieter context marker while a character view is open. Routes remain
authoritative, including nested treatment views and remembered location context.

NPC portraits use the same circle and attached conversation spoke. Only
supported views appear. Party membership actions belong inside profiles,
separate from view navigation. The circular incapacitation wheel remains a
distinct badge on the portrait. Its key is available in the character menu.
The journal remains in the header; character switching stays in its menu.

Wide strategic screens reserve the central area above chat for the character
model. Character and inventory documents remain in side panels, with their
own scrolling when needed. The scene placeholder keeps that reservation
until the 3D model is available. Chat is always visible at the bottom center,
between the full-height side panels. Center-menu content reserves room for it.
Narrow screens keep chat docked at the bottom while the menus scroll above it.
They stack the panels when there is no room
for this arrangement.
Inventory buy, sell, and transfer controls occupy dedicated table columns at
the panels' inner edges: right on the left panel, left on the right panel.
These columns stay visible while other columns scroll horizontally. Item
names remain separate from actions, and controls never extend into the stage.
The shared chat dock sits outside the replaceable menu panels, so opening the
journal retains the current messages and composer. Contextual conversations
populate that same dock; menus without a conversation still render chat.
There is no visibility toggle or collapsed preference. Chat remains resizable.
Personal inventory reveals its discard pane when
items are staged; canceling restores the pending quantities. Cooking presents
only usable ingredients and cooking vessels, with selected portions shown
before submission. Selection is a draft; the server commits the action.

Rest previews use the server-owned daily lodging rate and strategic calendar.
They identify the active adventurer's lodging bill and wake time; they do not
predict additional spending by scheduled activities. Departure uses explicit
24-hour HH:MM input, validated by both the browser and route boundary.

Forms identify their action, prerequisites, duration, and known costs. An
unavailable action keeps its reason readable. Failed resident loading offers
retry. Medical presentation contains only observer-authorized projections;
possible diseases remain estimates and a single observation establishes no
trend. None of these presentations introduce additional game authority.

SSR action overlays are mounted directly beneath the replaceable strategic
page to escape scene clipping and stacking contexts. They contain keyboard
focus and preserve close/return behavior. The persistent Bevy canvas remains
outside the replaced page; navigation must not recreate it.

Headings retain the display face; dense tables and controls use a system UI
face with aligned numerals. Copy sits on opaque dark surfaces. Preserve
tactile buttons, visible keyboard focus, readable disabled reasons, and
non-color selection cues. Prefer contextual details over repeating long
explanations in each row.

Keep common inventory names and units visible. The Column key explains
quantity targets and missing values; it opens by click or keyboard. Stats
panels default to compact icons, with short names retained for ambiguous
attributes and skills. Each panel's Show labels control restores names and
contextual keys; its preference persists across navigation and browser visits.
The controls still work for the current view when browser storage is blocked.

Each region's health bar fills its column, between an anatomical icon and a
plus sign. Arms and legs remain in left/right pairs side by side; right-side
icons mirror their left counterparts. Names appear with Show labels
or when the reading is opened. Treatment controls appear alongside the opened
reading. Sound health is green; unclassified impairment is dotted slate-grey.
Yellow remains associated with assessed choleric impairment and projected
incapacitation, rather than an unspecified disease diagnosis. The expanded
reading contains only the observer-authorized breakdown already used by the
meter. Five divisions and filled length communicate usable ability;
hatching identifies unavailable ability. Summary icons, skill icons, and skill
bar segments share the rank palette: dark green at zero, then lime green,
yellow, orange, red, and purple through ranks one to five. Zero shares its
color token with intact item condition. Positive fractional skills retain
the color of their current bar band; filled lengths remain continuous. Exact
ranks appear on hover or keyboard focus; click, Enter, or Space pins the
reading until toggled off or dismissed with Escape. The optional skill key
shows all six colors with numbers.
Reuse a small set of visual patterns so players can transfer what they learn
between panels. Keep rank colors distinct
in meaning from health-condition and combat-wheel colors. Capability summaries
show colored icons, with names in the labeled view. Neither view exposes
exact ranks until the player inspects a stat.

Named equipment placements retain their keyboard shortcuts. Humour names,
supply names, procedure thresholds, and encumbrance values are readable without
hovering. The central character stage remains reserved on spacious layouts.

Run `cargo test -p strategic-web`, `npm test --prefix crates/strategic-web`,
and `npm run test:browser --prefix crates/strategic-web`. The interface browser
checks cover dialog geometry and focus, time validation, and management
layout. Visual captures are optional through `UX_CAPTURE_DIR` and belong in
ignored local output directories. Set that variable to an absolute directory
for both the Rust tests and the browser tests to export and capture actual
Maud templates with deterministic sample data. These captures suppress the
live renderer and network services; they verify document presentation, not
live reducers or Bevy scenes.
