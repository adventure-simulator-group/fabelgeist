# adventuresim-environment

Living environments for Fabelgeist. Plants are grown rather than placed:
`fabelgeist-botany` simulates whole ecosystems after Makowski et al.'s
*Synthetic Silviculture* (SIGGRAPH 2019), with species competing for light,
seeding, adapting to climate and climbing the buildings among them.

## Forest rendering

The `render` feature draws botany's snapshots in Bevy (`render::ForestPlugin`).
Every branch segment and leaf cluster is a GPU instance drawn through
`bevy_eidolon`, as the game's grass and shrubs are, culled on the GPU by
16-metre chunks; its vertex shader builds a tube along the segment's curved
axis, or the cluster's blades (broadleaf spirals, conifer needle sprays, palm
fronds). Bark and blades are lit by Bevy's PBR
lights; blade outlines become multisample coverage. The terrain and the
buildings are ordinary meshes coloured by their vertices. An application feeds
snapshots through `ForestFeed` and sets tube and blade detail in
`ForestDetail`.

The Bevy forest does not yet cast shadows
(`bevy_eidolon` draws no shadow pass; the trees receive the terrain's and the
buildings'), draws every plant at one level of detail without grove proxies,
and does not grow fillets where branches meet or lay litter on the ground.

## Environment viewer

The `environment-viewer` binary watches a scene grow. Botany's simulation
thread and snapshots come from the `fabelgeist_botany::viewer`
module; the Bevy app, its forest renderer and its egui panel live here,
outside the botany crate.

```powershell
cargo run --release -p adventuresim-environment --features viewer --bin environment-viewer
```

Drag to orbit, right-drag to pan and use the wheel to zoom. Space plays and
pauses, G cuts a gap in the canopy where the camera looks, and R resets the
scene. The **Scene** tab chooses the scene, speed, climate and lighting, and
whether plants spread seed and whether they die; the **Profile** tab sets how
the simulation steps and how finely tubes and blades are built.

**Sowing** plants by hand: pick a species (the climbers follow the plants),
switch **Sow** on (or press S), and click the view. The seed is thrown from
the eye through the clicked pixel and takes root where it lands, on open
ground or at the foot of the wall it hits. *Castle, unplanted* is bare
ground around the castle and village, its species and climbers ready to
sow.

`--snapshot <png>` grows a scene for `--years` without the simulation thread,
shows it, and saves the window once:

```powershell
cargo run --release -p adventuresim-environment --features viewer --bin environment-viewer -- --snapshot target/boreal.png --scene boreal --years 120
```

`--closeup` frames the tallest plant, `--solo conifer` grows one plant of a
species alone, and `--profile fast` renders with the cheaper preset.
