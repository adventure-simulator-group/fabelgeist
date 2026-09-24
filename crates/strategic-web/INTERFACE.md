# Strategic interface interaction contract

The place facade and its sign identify the physical location. The adventurer
navigation strip identifies the current task and provides access to location,
party inventory, journal, character switching, and conversation. Location
context remains part of the existing route rather than becoming client state.

Wide strategic screens reserve the full-height central area for the character
model. Character and inventory documents remain in side panels, with their
own scrolling when needed. The scene placeholder keeps that reservation
until the 3D model is available. Conversation opens over the right panel,
clear of the model. Narrow screens stack the panels when there is no room
for this arrangement.
Conversation can be collapsed without losing its contents. Opening a social
interaction expands it. Personal inventory reveals its discard pane when
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

Run `cargo test -p strategic-web`, `npm test --prefix crates/strategic-web`,
and `npm run test:browser --prefix crates/strategic-web`. The interface browser
checks cover dialog geometry and focus, time validation, and management
layout. Visual captures are optional through `UX_CAPTURE_DIR` and belong in
ignored local output directories. Check real rendered pages at desktop and
narrow sizes as well as the fixture-based regression checks.
