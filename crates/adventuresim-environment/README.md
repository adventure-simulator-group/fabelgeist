# adventuresim-environment

Living environments for Fabelgeist. Plants are grown rather than placed:
`fabelgeist-botany` simulates whole ecosystems after Makowski et al.'s
*Synthetic Silviculture* (SIGGRAPH 2019), with species competing for light,
seeding, adapting to climate and climbing the buildings among them.

## Environment viewer

The `environment-viewer` binary watches a scene grow. Botany's simulation
thread, instanced renderer and orbit camera come from the
`fabelgeist_botany::viewer` module; the egui interface over them lives here,
outside the botany crate.

```powershell
cargo run --release -p adventuresim-environment --features viewer --bin environment-viewer
```

Drag to orbit, right-drag to pan and use the wheel to zoom. Space plays and
pauses, G cuts a gap in the canopy where the camera looks, and R resets the
scene. The **Scene** tab chooses the scene, speed, climate and lighting, and
whether plants spread seed and whether they die; the **Profile** tab trades
detail for speed.

**Sowing** plants by hand: pick a species (the climbers follow the plants),
switch **Sow** on (or press S), and click the view. The seed is thrown from
the eye through the clicked pixel and takes root where it lands, on open
ground or at the foot of the wall it hits. *Castle, unplanted* is bare
ground around the castle and village, its species and climbers ready to
sow.

Without a window, `--snapshot <png>` grows a scene for `--years` and renders
one frame:

```powershell
cargo run --release -p adventuresim-environment --features viewer --bin environment-viewer -- --snapshot target/boreal.png --scene boreal --years 120
```

`--closeup` frames the tallest plant, `--solo conifer` grows one plant of a
species alone, and `--profile fast` renders with the cheaper preset.
