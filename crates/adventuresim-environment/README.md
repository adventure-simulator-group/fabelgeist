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
thread and snapshots come from the `fabelgeist_botany::viewer` module; the
Bevy app, its forest renderer and its egui interface live here, outside the
botany crate.

```powershell
cargo run --release -p adventuresim-environment --features viewer --bin environment-viewer
```

The interface is a game's HUD over the full window. The top-left card shows
the scene, the year and the population with its curve. The top-right buttons
pause (Space) or play at 5, 20 or 80 simulated years a second, or as fast as
the simulation goes, and open the menu. Drag to orbit, right-drag to pan and
use the wheel to zoom.

The hotbar along the bottom holds the tool in hand:

- **Look** (Esc) leaves the world alone; the mouse only moves the camera.
- One slot for each species growing, the climbers after the plants, picked
  with 1 to 9. A click then sows that species: the seed is thrown from the
  eye through the clicked pixel and takes root where it lands, on open
  ground or at the foot of the wall it hits. A ring in the species' colour
  marks the ground under the pointer.
- **Clear** (G) takes every plant out of the red ring under the pointer.

A drag still turns the camera with any tool. Esc with **Look** in hand opens
the menu, as does the menu button. **Scenes** starts any scene afresh;
*Castle, unplanted* is bare ground round the castle and village, its species
and climbers ready to sow. **World** sets the climate and whether plants
spread seed, die and shade the plants under them, with the simulation's step
under **Simulation**. **Display** moves the sun, switches shadows and foliage,
and sets how finely branches and leaves are built. F3 shows frame rate,
simulation step time and instance counts.

`--snapshot <png>` grows a scene for `--years` without the simulation thread,
shows it, and saves the window once:

```powershell
cargo run --release -p adventuresim-environment --features viewer --bin environment-viewer -- --snapshot target/boreal.png --scene boreal --years 120
```

`--closeup` frames the tallest plant, `--solo conifer` grows one plant of a
species alone, and `--profile fast` renders with the cheaper preset.
