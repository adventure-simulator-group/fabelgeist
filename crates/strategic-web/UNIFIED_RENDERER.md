# Shared strategic and tactical renderer

The strategic document owns one `#game-canvas`, Wasm application, WebGPU
device and asset store. Soft navigation replaces `#strategic-page` while
retaining that runtime. The map remains in its existing HTML renderer.
HTML-only pages such as the journal hide the retained scene; returning to a
venue reuses its scene document, roster and equipment.

## Presentation and authority

`/api/scene-assets` prepares the dispatcher's `TacticalSceneInput` for the
active character's authorized location without allocating a tactical server.
The browser passes its original JSON text to Rust, preserving all 64-bit seeds
and building IDs. Settlement geometry has a stable location seed across
missions.

The client uses the tactical scene generator and presentation observers for
terrain, landforms, streets, yards, buildings, doors, windows, furnishings,
gardens, vegetation, weather, sky, and lighting. Service buildings retain their
full dimensions, plans, doors, windows, and interiors. The client arranges these
instances side by side on a clear frontage at the edge of the retained city.
Interior furniture and conversation positions move with their buildings. This
presentation placement does not change the tactical server's scene document.
One perspective camera views the entire frontage, with the remaining city
behind it. HTML names and accessible route links occupy the corresponding street
bays; horizontal scrolling crops the same camera projection.

Business operators resolve to the scene's establishments. Required distant
buildings receive full detail and furnished interiors on the shared frontage
before readiness; the remaining city retains generated facade and shell assets.
Static timber members use a client-generated construction kit. Exact dimensions
and finishes identify shared component meshes; placement, rotation, and wood
texture phase are instance data. Canonical component transforms are shared
across repeated building recipes. Each instance follows its parent building's
GPU visibility and LOD selection. Geometry ranges reference deduplicated lists
of building placements; the GPU expands only selected instances into draws.
The preload queue retains shared compiled recipes instead of expanding every
member into a separate CPU record. Cut masonry, gable face selections, and other
specialized surfaces retain their semantic geometry compilers. Tactical plans,
collision, and operable elements keep their authoritative representation.
Common rectangular civilian background shells compile from their programmes and
the shared roof-plane recipe. This distant representation keeps wall heights,
upper-storey projection, roof pitch, and metric texture scale, with simplified
window marks and no individual dormers, interior framing, or joinery. Churches,
workplaces, fortifications, and other specialized shapes use their semantic
shell compilers. Playable buildings and the service street retain their semantic
shells so their standard renderer LOD crossfades remain aligned. Detailed
facades and playable assets still finish before
readiness; selecting a venue never triggers this omitted-detail work.
Conversations occupy the buildings' furnished circulation space, with camera
clearance checked against architecture and furniture. Placement favors rooms
receiving actual daylight and seats feet on the physical floor. Outdoor
locations use their actual terrain. All cameras use the tactical perspective
and inherit its tone mapping, fog, and environment lighting. Each view uses
the tactical interior exposure calculation at its own world position; active
views jointly select the nearby buildings' indirect daylight fields.
Daylight uploads keep a fixed GPU allocation so existing material bindings
observe updates as nearby buildings change. Shared tactical eye adaptation
compares interior irradiance with outdoor sunlight, allowing up to six stops
of daytime adjustment. Strategic captures use that settled exposure; tactical
play retains gradual adaptation and the same night response.
Inside a sampled room, its daylight field replaces unoccluded outdoor diffuse
ambient and environment lighting. Outdoor reflections are attenuated by the
room's irradiance relative to the outdoor reference; local reflection probes
are not yet generated. Direct lights retain their normal shadowing. The same
policy applies to ordinary and blood-masked surfaces. Exterior PBR is unchanged.

Characters use the tactical animation rig and the same stable ID-derived
proportions and identity morphs. Strategic models have no `Player`, tactical
controller or replicated state. This presentation does not change either
strategic authority or the transient tactical server's authority.

HTML reports visible physical-pixel rectangles and stable domain identities
through `sync-strategic-view`. Character IDs cross JavaScript as decimal
strings, preserving the complete `u64` range. Additional Bevy cameras render
the shared street, selected character and visible portraits to the same canvas.
Portrait cameras view the same character entities as the central scene.
Focused portrait and conversation cameras include only their target character
and the room, so other residents cannot obscure the face or equipment.
These close views use a 128-metre far plane to avoid submitting the distant
city behind the room. The street view retains the tactical skyline range, and
all prepared city assets remain resident.
Clipped projections preserve framing during scrolling and resizing.
The header, character menu and recruitment list also use model portraits.
Portrait-only models use the same venue positions and separate view layers.

The canvas accepts tactical input only while tactical play is active. In
strategic play, HTML retains names, focus, route links, panels, forms and
chat. The canvas is clipped to dedicated visual windows and ignores pointer
events. Talking to a smith selects their character; editing the weapon recipe
selects the forge preview. The primary egui context belongs exclusively to
the gameplay camera.

## Loading and navigation

Initial loading prepares every available venue and prefetches its resident
list. Geometry and character entities remain resident across place changes.
The browser displays loading state until character rigs have presented a pose,
material textures, city geometry, terrain and sky lighting have loaded, and
render pipelines have settled. Required scene asset failures produce an
unavailable state. Place and character selection reuse assets; they do not
import Wasm, create a device or compile
another copy of a building.

Cold preparation reuses generated building plans and collision geometry when
creating their render assets, and uses the retained furnishing layouts for
conversation placement. Repeated circulation points are scored once. Facade
instances compile only the render data they consume; indoor lighting fields
and dynamic opening sets belong to detailed playable instances. LOD passes
reuse their solid compiler's geometry index across all selected solids.

Distant furniture-site preparation retains one generated recipe per complete
building program. Venue promotion reuses these plans, and city mesh compilation
consumes them without regenerating the roofs, frames or collision. Temporary
plans are released as meshes become resident; unused recipes are dropped when
city preparation completes or the location is replaced. They never enter a
scene document or the replication protocol.

Ordinary building generation checks inputs and construction constraints.
Exhaustive structural audits belong to explicit tests and authoring acceptance.
Likewise, `TacticalSceneInput::audit_garden_clearance` is an explicit authoring
and test check: loading does not build detailed meshes for every distant recipe
solely to validate garden bounds. Runtime checks still validate garden
ownership, plots, access geometry, and terrain support. These changes do not
defer interactive assets until a venue is selected.

The retained location is replaced when travelling to another location.
The entire street view and character portraits retain Bevy image targets,
then Bevy's 2D pass composites them onto the same canvas. The street has one
camera and one target, shared by every building. These views
remain still between changes. Their cameras stop rendering once assets and
pipelines settle; the central conversation view stays live. A changed
outfit, camera framing, viewport size or retained location invalidates the
corresponding capture. Scrolling repositions retained images without rebuilding
geometry.
When the layout supplies a portrait size, cold readiness includes portraits for
residents of unvisited venues at those dimensions. A pool of at most two snapshot
camera entities captures these images and stays inactive between captures.
Completed images remain resident. This also avoids exhausting Bevy's limited
distance-visibility camera table with inactive cameras. A later layout requiring
different dimensions still creates a new capture when that portrait is shown.
Unchanged layout does not rebuild camera projections. The bridge exposes
bounded readiness and navigation samples through
`window.strategicRendererMetrics`.

Animation culling considers all active 3D views. Retained characters initialize
their poses before they can be culled. Identity morphs and skeletal proportions
update when appearance or loaded rig data changes, rather than every frame.
The appearance endpoint returns only equipped items for owned characters,
party members, applicants to the active party and people in the same settlement.
Catalog placements, wearer bindings, weapon recipes and holder recipes feed
the shared tactical equipment renderer. On native clients, fitted garments share
generated meshes while retaining each wearer's morph weights and skeleton.
The armor device currently uses blocking readback and is native-only. Browser
equipment follows the tactical web renderer's catalog presentation; runtime
armor and garment fitting in the browser still requires an asynchronous device
path. This city stack does not introduce served equipment binaries.
Cold loading waits
for equipment generation and binding; warm navigation retains the equipment.
Live updates refresh the
appearance data and replace changed outfits.
Native runtime garments derive surface coordinates from the UV-free animation body.
Belt sockets lie on the generated garment surface. Strategic idle equipment
uses conservative culling bounds. Local display descriptors are removed before
network enrollment while the shared building asset cache is retained.

## Resident city rendering

The client generates building meshes and their facade and shell LODs from the
scene recipes. No compiled building geometry is served. Repeated material
textures may be served and shared across instances.

After generation finishes, the static distant city is packed into persistent
GPU buffers. Each mesh asset contributes indexed canonical geometry once; each
building contributes a transform, bounds and available LODs. Material batches
reference whole mesh ranges. One compute lane checks each range and reserves
its visible clusters with one global atomic operation. Workgroups check 64
ranges at once; rejected ranges need no emission or synchronization. Each lane
emits clusters of at most 64 triangles. Each visible entry stores an 8-byte pair
of range index and index-buffer offset; the source has one record per range.
Static building parts enter an assembly queue directly, without temporary
render entities or standard mesh GPU uploads. Their CPU meshes remain cached
for reuse; the renderer uploads only the packed storage buffers. Building roots
remain available for shop signs. Standalone preload uses a 64 ms work budget
between yields, amortizing intervening frames. An individual building finishes
before yielding and can exceed that budget.
Static outdoor vista furniture
joins these buffers, sharing canonical recipe geometry and retaining its
distance fade. Its scenery roots have no tactical physics or animation and are
released with their render children; the generated scene descriptors retain
their recipes and identities. Directional shadows use the parent camera's
distance and the shadow view's caster frustum. Point and spot shadows are shared
across cameras and retain all casters in their light frustum.
Interactive buildings and furniture retain the tactical mesh renderer. Furniture
instances reuse canonical recipe meshes and material handles, allowing Bevy to
instance matching surfaces. Doors and window
casements also share geometry by exact dimensions and leaf kind. Each object
retains its own transform, visibility, outline and tactical identity; moving a
leaf or item does not rebuild geometry. Interior fixtures follow their owner's
detail range. Outdoor stalls and small street furniture have separate distance
ranges. All retain ordinary PBR and interior lighting.
Sign boards and brackets share one unit cube, with physical dimensions held in
instance transforms. Lettering faces share one unit quad; each establishment
retains its own painted text and material. These shared meshes allow ordinary
PBR and shadow passes to instance repeated sign parts.
Fixed compound walls and caps merge by material within 128-metre spatial cells.
The merged meshes preserve world placement, metric UVs and normal-map tangents;
their bounds still participate in camera and shadow culling. Operable gates
remain separate entities. This batching changes presentation only, leaving the
authoritative boundary and opening descriptors intact.
Vertex and index pages are independently capped at 64 MiB. Oversized source
meshes split on triangle boundaries, so no geometry binding requires a device
limit above the baseline 128 MiB storage-buffer binding capacity.

Before rendering, compute passes select buildings against each active camera or
shadow view and compact the selected clusters into indirect draws. Facade versus
shell selection uses projected building radius, with hysteresis to stabilize
small camera movements. Distant-city shadow views use the generated shell and
omit decorative facade overlays, while retaining roof and enclosure geometry.
Their caster selection is independent of camera visibility. View role is
explicit: an orthographic scene camera still selects its normal LOD and obeys
its near plane. Directional shadow views admit casters before the light's near
plane. Dispatch follows
active cameras and their linked shadow views, excluding retained inactive views.
The camera's explicit far plane supplements its infinite-far projection matrix,
so actor views enforce their configured range while skyline views retain theirs.
Ordinary meshes also obey each strategic camera's explicit far plane, after
Bevy's ordinary visibility pass. This filtering leaves shadow caster lists
independent. Solid city surfaces use opaque, depth-only shadow pipelines;
alpha-cutout surfaces retain their texture-tested silhouettes.
Strategic lights retain stable room and character layers. Character meshes
outside the union of active camera layers temporarily stop casting shadows.
Unselected residents and completed portrait snapshots stay resident; activating
a view restores its character's shadow casting before shadow preparation.
Authored shadow exclusions are preserved.

Grass keeps persistent tuft buffers and runs GPU culling immediately before
each camera's main pass. Animated bounds cover wind, blade width and player
interaction. Each view clears and fills its own visible-instance result in GPU
command order, so snapshots and the live camera can safely reuse scratch buffers.
Grass retains its existing policy of omitting shadow and depth prepasses.
CPU scatter preparation caches habitat-cell positions and random fields for
the duration of the pass. Each tuft still evaluates its exact nearest site and
local environmental weights, preserving species boundaries and seeded output.
The shipped graphics preset keeps near grass topology, uses 25 blades per far
tuft and 16 per vista tuft, and limits directional shadows to 32 metres. These
are runtime quality settings in `assets/config/tactical-graphics.yaml`, not
alternate city assets or offline LODs.
Each camera and its cascades form one render group. Visibility lists and draw
arguments are scratch buffers shared by successive groups, sized for the largest
active group rather than every retained view. GPU command-order counter resets
prevent one group from consuming another group's results. Small per-view LOD
histories remain separate. Scratch allocations retain their high-water size
during navigation and are released when the resident city is replaced.
This path uses
ordinary WebGPU compute and indirect drawing, without requiring Bevy's native
GPU-culling feature set.

Geometry remains resident during navigation. Initial readiness includes GPU
buffer preparation and compute pipeline availability. Current city surfaces
preserve base color, repeated albedo and normal textures, roughness and
metallicity. Distant glass uses an opaque approximation; interactive windows
retain their original material. This is material batching of generated mesh
clusters, not yet a shared library of semantic beam, wall and roof modules.

Run the culling regression on an actual WebGPU adapter:

```sh
node --test crates/strategic-web/tests/city-gpu.browser.cjs
node --test crates/strategic-web/tests/interior-lighting.browser.cjs
```

These check frustum rejection, independent LOD history, shadow casters,
hysteresis, sequential scratch reuse and indirect arguments, plus room versus
exterior lighting against the production shaders. They collect no performance
measurements.

## Verification

Run the bridge regression in Chromium:

```sh
node --test crates/strategic-web/tests/strategic-scene.browser.cjs
```

After building the tactical Wasm and synchronizing assets with
`just build-wasm`, run the same fixture against the actual renderer:

```powershell
$env:STRATEGIC_SCENE_FIXTURE_DIR = "$PWD/target/strategic-scene-review/fixtures"
cargo test -p adventuresim-tactical-client --bin adventuresim-tactical-client procedural_carry_recipes
$env:STRATEGIC_RENDER_BENCHMARK = '1'
node --test crates/strategic-web/tests/strategic-scene.browser.cjs
```

Set `STRATEGIC_SCENE_PEOPLE_PER_PLACE=8` to repeat with 88 retained characters
and eight visible portraits. The default fixture uses the 4,423-building
tactical city, 22 characters and two portraits. Both measure cold readiness,
warm building and portrait changes, browser frame intervals and additional
asset requests. Missing assets fail
the test and remain in the recorded results; they are not suppressed.

The WebGPU run uses Edge on Windows, matching the other GPU tests. It checks
adapter availability before starting. The fixture serves production renderer
code and assets with isolated HTML and resident data; it does not access a
database. Results, screenshots, adapter identity and bounded diagnostics go
under `target/strategic-scene-review/`. Cold startup and warm navigation are
reported separately. Fixture timing measures the renderer and browser bridge;
it does not establish production database or network latency.
Readiness telemetry also exposes `assets_ready`, `snapshots_pending`, and
`settled_render_frames`. These distinguish asset preparation, snapshot capture,
and the final stable-frame requirement. The browser polls readiness separately;
its observed completion time is not a GPU timestamp or first-visible-pixel
measurement. For navigation breakdowns, observe GPU submissions as well as
animation-frame callbacks: the window event loop can render from other callbacks
after navigation.
When a production inventory fixture is present, `location-replacement.json`
also records the cost and requests for replacing the retained settlement.

### Draw and timing measurements

Set `STRATEGIC_GPU_PROFILE=1` alongside `STRATEGIC_RENDER_BENCHMARK=1` to
capture steady frames. Use a fresh browser run for each location, setting
`STRATEGIC_PROFILE_PLACE=inn` or `STRATEGIC_PROFILE_PLACE=public-square`.
The frame probe observes animation-frame callbacks; navigation can change the
renderer scheduling and leave that probe without samples. Set
`STRATEGIC_PROFILE_AND_NAVIGATION=1` to continue the ordinary navigation checks
after an indoor capture. The steady captures themselves do not measure
navigation latency. The opt-in
test probe intercepts WebGPU commands without changing scene content. It requests
timestamp-query support, reads indirect draw arguments back after each render
pass, and records direct draws, instance counts and submitted triangles.
Render bundles are rejected until explicitly supported by the probe.

Run `node --test crates/strategic-web/tests/webgpu-probe.browser.cjs` to
validate counters against known direct and GPU-written indirect arguments.
Summarize a capture with:

```sh
node crates/strategic-web/tests/summarize-gpu-profile.cjs target/strategic-scene-review/gpu-profile
```

The output includes pass timestamps, instance histograms, lightly instrumented
animation-frame callback timing and DevTools CPU profiles. Triangle counts are
submitted primitives, including repeated shadow passes; they do not measure
post-clipping visible triangles. Indirect argument copies and query readback
add instrumentation work, so GPU-capture callback times are kept separate from
ordinary callback timing. GPU pass durations exclude the probe's readback
copies and are not equivalent to end-to-end frame latency.

The outdoor capture also temporarily suppresses raster draw calls, measures
60 frames, restores drawing and measures another 60 frames. This diagnostic
retains scene updates, render preparation, state-setting commands and compute
passes. It removes both raster GPU work and draw-call encoding; it does not
isolate either cost individually, and its output is not rendered frame rate.
The CPU, GPU and diagnostic samples run separately. GPU captures have a bounded
timeout, and the browser test still fails on missing production assets.
