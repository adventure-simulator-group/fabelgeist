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
snapshots through `ForestFeed` and sets the detail in `ForestDetail`.

Each chunk is drawn at one of three levels of detail, picked each
frame from how large its largest plant appears on screen: in full detail
(10-sided tubes, 10 blades a cluster) when that plant's radius spans at least
220 pixels, in middle detail (6 sides, 6 blades) down to 40 pixels, and
coarsest (4 sides, 2 blades) below. A cell's plants are split into chunks by
size, so shrubs follow their own level, and a chunk keeps its level until its
plants are 15% past the threshold, since changing it uploads the chunk again.

A snapshot is made ready on a background task (its records copied to bytes,
its instances sorted into chunks), then drawn in one frame. The records are
written on the render thread into storage buffers that persist from snapshot
to snapshot; only the bytes in use are
written, and only when they changed, as do the chunks' instances, so a paused
simulation, which keeps handing over the same snapshot, costs nothing to draw.

The Bevy forest does not yet cast shadows
(`bevy_eidolon` draws no shadow pass; the trees receive the terrain's and the
buildings'), stands no proxies in for far groves, and does not grow fillets
where branches meet or lay litter on the ground.

## Environment viewer

See [Forest component measurements](BENCHMARKS.md) for the bounded,
machine-readable generation and rendering benchmark workflow.

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

`--snapshot <png>` grows a scene for `--years` without the simulation thread
and renders it once headless, without a window, at 1600 by 1000 pixels:

```powershell
cargo run --release -p adventuresim-environment --features viewer --bin environment-viewer -- --snapshot target/boreal.png --scene boreal --years 120
```

`--closeup` frames the tallest plant, `--solo conifer` grows one plant of a
species alone, and `--profile fast` renders with the cheaper preset.

`--frames 300` times that many frames before the capture, waiting for the GPU
after each, and prints the time a frame takes and each render pass's GPU time.
`--orbit` turns the camera once round the scene meanwhile, which changes
chunks' levels of detail as a player looking round does, and `--refeed 5`
hands the forest the snapshot again every fifth frame, as the simulation hands
it new ones:

```powershell
cargo run --release -p adventuresim-environment --features viewer --bin environment-viewer -- --snapshot target/castle.png --scene castle --years 120 --frames 300 --orbit --refeed 5
```
