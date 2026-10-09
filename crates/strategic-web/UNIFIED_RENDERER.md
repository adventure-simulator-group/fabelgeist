# Shared strategic and tactical renderer

The strategic document owns one `#game-canvas`, Wasm application, WebGPU
device and asset store. Soft navigation replaces `#strategic-page` while
retaining that runtime. The map remains in its existing HTML renderer.
HTML-only pages such as the journal hide the retained scene; returning to a
venue reuses its scene document, roster and equipment.
Forge teardown removes only the forge preview. The strategic scene bridge owns
view visibility and retains the current view across ordinary page replacement.

## Client generation workers

Cold preparation starts a bounded pool of up to four CPU workers, reserving a
logical core for the document where available. They share the compiled Wasm
module, but each owns its generation memory. Workers never boot Bevy, create a
canvas, or request a GPU device. Dependent preparation phases reuse initialized
workers from the same pool. They terminate after the complete preparation, or
immediately on failure.

Occupied building plans, interiors, detail meshes, LODs, and tangents are
independent jobs alongside the distinct background exterior prototypes. Terrain
generation then consumes their prepared plans and compact doorway/footprint
records. Workers return locally generated CBOR products through transferable
buffers while the renderer loads textures. Occupied mesh vertex and index arrays
use packed byte strings, avoiding scalar-by-scalar decoding on the main thread.
After terrain preparation, independent landscape jobs clip street and yard
meshes and scatter playable and vista grass. Their inputs include the graphics
configuration; packed geometry and instance attributes preserve native results.
Rust parses the original scene JSON and verifies each returned product against
its requested identity. Entity installation, render assets, and GPU upload
remain in the persistent application.

Readiness requires all generation jobs, installation, and existing asset gates.
A failed worker or invalid product fails loading explicitly. Warm venue and
portrait switches use retained assets and do not schedule generation jobs.
`window.strategicGenerationMetrics` reports worker count, product bytes, summed
worker time, installation decoding time, and elapsed preparation time.

## Locally generated asset cache

IndexedDB retains immutable generation products across document reloads. Nothing
is baked offline or downloaded as geometry. Repeated material textures continue
to use the normal asset server. A cache key includes the complete job input and
the SHA-256 digest of the actual Wasm executable, so generator, dependency,
compiler, or serialization changes invalidate earlier products automatically.
The original JSON seed representation remains intact throughout key generation.

Products are gzip-compressed locally, carry a content checksum, and are decoded
and identity-checked by Rust. The cache-format revision is also part of the key.
Decompression enforces the recorded output size and product-size limit. Corrupt
entries are removed and regenerated. Disabled storage, blocked
opens, timeouts, and quota errors degrade to client generation. Writes do not
delay readiness; `window.strategicGenerationCacheSettled` lets diagnostics wait
for persistence before testing a reload. Best-effort eviction targets a 512 MiB
budget after each preparation; products larger than 128 MiB are not stored.
Oldest writes are evicted first. These records never contain live tactical tick
state.

Opening and reading storage use a two-second deadline. Background writes and
eviction have a separate thirty-second deadline because their completion events
can queue behind generation frames. If a deadline fires after an IndexedDB
transaction has finished, its queued completion event resolves the operation.
An aborted transaction deadline affects that operation only; subsequent reads
and background writes remain available. Storage failures such as denied access
or exhausted quota still disable the cache for that preparation.

Cache reads have bounded concurrency; each miss starts its worker without waiting
for unrelated reads. Resident exterior meshes skip their disk products entirely.
Opening the cache reads a key inventory once, so products known to be absent
skip individual IndexedDB read transactions. A timed-out inventory leaves
ordinary lookups available. Products written by another document after that
inventory may be regenerated until the next cache opening.
Up to 64 previous occupied-building semantic recipes remain available alongside
the current city's newly prepared recipes; their installed meshes stay in the
renderer's building asset cache. A fully cached scene starts no generation
workers. Render-asset installation, texture
residency, GPU uploads, equipment, and portrait preparation still run, so a cache
hit is not equivalent to whole-game readiness. Generation metrics distinguish
hits, misses, summed lookup duration, decoding, worker time, and complete
preparation time. Concurrent lookup and worker durations are not additive elapsed
time. Worker count includes instances started in all preparation phases.

The document retains landscape products for the three most recently prepared
scene/configuration pairs. Returning to one reuses street and yard mesh handles,
nearby traffic-mask images, and grass placements. Grass instance buffers and
entities are still installed again. Changing the graphics configuration requires
new landscape products. This retention is separate from the disk cache and the
bounded snapshot cache.

Installation also reuses deterministic intermediate values within the document.
A building chooses its material palette once per assembly traversal and shares
it across modular components. Ground-mask rasterization computes each noise
lattice corner once, then interpolates the same values for neighboring pixels.
Neither changes the generated appearance or requires stored render products.

Interior furnishing retains navigation obstruction counts while trying candidate
groups. Accepted furniture remains in the graph; rejecting a group removes only
its contributions. The same reachability and access-path checks still decide
placement in both the scene worker and promoted strategic venues.

Grass derives one seed for each spatially identified tuft and draws its jitter,
species, rotation, and shader variation from fixed slots in that tuft's stream.
This avoids repeatedly hashing the same identity for individual visual fields.
Tuft density, representation, coverage, and fade distances remain unchanged.
Street and yard meshes query a bounds hierarchy over the presented terrain
triangles before clipping, preserving terrain seams and exact surface heights.
Street traffic clearance uses cached road directions and a spatial index for
market bounds, avoiding repeated normalization and unrelated market scans.

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
member into a separate CPU record. Assembly groups recipes by geometry and
palette, resolves component geometry once per group, and passes shared owner
lists directly to range packing. Even temporary draw records stay grouped;
duplicate components retain their draw multiplicity. Cut masonry, gable face
selections, and other specialized surfaces retain their semantic geometry
compilers. Tactical plans, collision, and operable elements keep their
authoritative representation.
Buildings outside the tactical boundary use three shared exterior variants per
architectural family. Occupation, service size, and individual building seeds do
not create additional exterior meshes. The settlement's canonical prosperity
tier travels with each distant placement: poorer settlements omit ornate
merchant-house exteriors and use plain or weathered finishes; richer settlements
can use brick infill and decorative finishes. Geometry and finish select the
same variant, keeping the prototype pool bounded. Landmarks retain their
frontage direction. Each model may shrink uniformly to fit its reserved plot,
but never expands the plot or changes the city's placement.
Outdoor furniture also uses the prototype's scaled footprint and doors; its
occupation still selects the kind of street activity. Preparing this scenery
does not reconstruct the original occupied buildings.
The strategic street reserves its camera approach before placing background
tree stands, so changes to scene identity cannot put foliage in front of tabs.

The occupied recipe remains available for buildings promoted to the strategic
street or playable tactical area. Those buildings retain their original plans,
equipment, furnishings, collision, and detailed assets. All exterior prototypes
are generated on the client and participate in the existing local cache; no
prebuilt geometry is served. Scene documents use the current schema directly.

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

Distant furniture-site preparation consumes compact physical footprints and
doorway reservations produced with the facade meshes. Those records remain
resident across city changes, avoiding another round of distant building plans.
Occupied plans are transferred to terrain generation after their independent
worker jobs finish, so pad preparation and mesh compilation share the same plan.
Native preparation can hand its recipe memoization directly to venue promotion
and mesh compilation. Unused prepared facade products are released after city
installation. Compiler memoization never enters a scene document or replication.

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
outfit, camera framing, or viewport size invalidates the corresponding capture.
Return trips can reuse completed captures when the location and complete scene
digest, including its environment, match. The inactive-city cache holds at most
two cities and 64 MiB of image pixels; unfinished captures are discarded.
Scrolling repositions retained images without rebuilding geometry.
When the layout supplies a portrait size, cold readiness includes portraits for
residents of unvisited venues at those dimensions. A pool of at most four snapshot
camera entities captures these images and stays inactive between captures.
New and reused capture cameras receive the tactical environment before camera
preparation and render extraction, avoiding an initial frame with default
rendering settings and its unnecessary pipeline specializations.
Each city's initial mesh draws wait for CPU installation and the final atmosphere
environment. Cameras, lighting preparation, asset uploads, and character pose
initialization continue during this wait. This avoids specializing meshes for
temporary lighting. The gate stays open for that city after installation, so
later weather changes do not blank retained views. A replacement city resets the
gate before it can inherit the previous city's ready state. GPU preparation,
snapshot completion, and settled-frame checks still follow before reporting
readiness.
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
reference whole mesh ranges. One compute lane checks each range's owner list and
reserves each visible owner's clusters with one global atomic operation.
Workgroups check 64 ranges at once; rejected owners need no emission. Each lane
emits clusters of at most 64 triangles. Each visible entry stores an 8-byte pair
of range index and encoded placement/cluster index. Ranges share geometry across
their owner lists; the vertex shader resolves the owner and local cluster.
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

Set `STRATEGIC_TRAVEL_BENCHMARK=1` with `STRATEGIC_RENDER_BENCHMARK=1` to
measure A → B → A without reloading the document or recreating the renderer.
The destination varies terrain seed and scene identity while retaining validated
building layouts. This isolates scene replacement and cross-city asset reuse;
it does not measure a completely different set of occupied building programs.
`travel.json` records elapsed arrival time and generation metrics separately.
The usual missing-asset assertion still applies after results are recorded.

To measure new occupied layouts, generate validated fixtures with the ignored
`distinct_city_inputs_validate_occupied_layouts` tactical-client test, setting
`STRATEGIC_TRAVEL_FIXTURE_DIR` to an absolute output directory. Set
`STRATEGIC_TRAVEL_SCENE_INPUT` and `STRATEGIC_TRAVEL_SECOND_INPUT` to its two JSON
files. This measures A → B → C → A with different occupied programs at both
destinations. Preserve the exact files for before/after comparisons; the test
uses fixed authored seeds rather than searching for easier layouts at runtime.

Preparing a destination before arrival requires a server-authorized scene input.
The current scene endpoint only serves the character's current settlement, so
the client does not bypass that rule to speculate about future destinations.

### Draw and timing measurements

For startup CPU attribution, set `STRATEGIC_STARTUP_PROFILE=1` alongside
`STRATEGIC_RENDER_BENCHMARK=1`. Add `STRATEGIC_RELOAD_BENCHMARK=1` to capture
both empty-cache startup and the subsequent cached reload. The test server
instruments its JavaScript responses without modifying production files or
disabling the browser HTTP cache. Captures contain DevTools CPU samples, resource
timings, long tasks, generation stages, and readiness milestones.

```sh
node crates/strategic-web/tests/summarize-startup-profile.cjs target/strategic-scene-review
```

CPU sampling adds overhead; use separate unprofiled runs for latency comparisons.
Summary buckets are disjoint, while inclusive function costs overlap. Asynchronous
cache-read and decompression spans also overlap and must not be added as elapsed
time. Browser idle samples do not identify GPU wait time. The gap between asset
readiness and snapshot readiness includes capture scheduling and rendering; it
is not a measurement of portrait generation alone. Asset readiness can become
false again while new capture pipelines compile, so first-observed milestones
are not boundaries between independent, nonoverlapping phases.

Startup profiles also record WebGPU pipeline-creation calls and sampled queue
completion callbacks. Callback latency includes main-thread scheduling and
earlier queued work; synchronous pipeline-creation calls can return before the
browser/backend finishes preparing the pipeline.

Add `STRATEGIC_STARTUP_BROWSER_TRACE=1` to capture browser GPU-service events,
including Dawn pipeline creation and DirectX shader compilation where the
backend exposes them. The `*-browser.json` files use the Chrome trace format;
`city-document-start` aligns them with the page's performance clock. Nested
pipeline and shader-compiler spans overlap and must not be added together.
Shader sources and pipeline descriptors are included in startup trace events
to attribute compiler work to material and vertex-layout variants.

Add `STRATEGIC_STARTUP_WORKERS=1` to sample generation workers separately. The
diagnostic pauses workers at startup to attach the profiler and collects their
profiles before the pool terminates them. These worker profiles have their own
clocks and sampling overhead; their summed CPU time is not elapsed readiness.

Add `STRATEGIC_STARTUP_GPU_TIMESTAMPS=1` to sample pass timestamps in every eighth
command encoder, with at most two readbacks outstanding. This opt-in diagnostic
requires timestamp-query support and observes submissions outside animation
callbacks too. Pass durations exclude the diagnostic's query-copy/readback work;
the samples do not cover every pass or measure the whole GPU workload. Run
`node --test crates/strategic-web/tests/startup-gpu-timestamps.browser.cjs` to
verify sampling against known WebGPU passes.

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
