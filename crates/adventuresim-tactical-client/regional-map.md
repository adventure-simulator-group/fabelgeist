# Regional environment presentation

The browser runtime retains a geographic camera and one terrain surface beside
its actor and tactical scenes. It uses the same Bevy application, WebGPU device,
canvas, mesh store, environment material and lighting assets. Opening the map
does not prepare settlement generation jobs or create a worker runtime.

Browser commands use `regional-map` with a nested command. `open` supplies the
terrain source digest, a checked WGS84 origin in microdegrees, the vertical view
span in metres and the shared physical-pixel canvas rectangle. `pan` carries a
two-component physical-pixel displacement. `zoom` carries a positive bounded
ratio, and `rotate` carries a checked angle in radians. `resize`, `reset`,
`hide` and `install-environment` complete the camera and environment protocol.
The environment product carries terrain and canonical regional connections;
the terrain
product retains the fixed lattice and checked source contract described in
[regional terrain capture](../strategic-web/regional-terrain.md).

The initial camera has a fixed 60-degree overhead pitch. Its mesh frame is east,
absolute elevation and south, making north point up with zero rotation. Shared
geographic projection converts camera movement to the sampler's east/north
frame. Terrain vertices retain their absolute source elevations. Triangles
touching missing vertices are omitted; a wholly uncovered window has no mesh.
The nearest covered sample supplies only the camera's elevation datum.

The camera requests the smallest named terrain window that covers the view with
a margin. Window centres move in steps of sixteen source cells, preventing a
new HTTP capture for each pointer movement. Terrain capture and browser request
ownership remain separate from camera control. While a replacement loads, the
last installed surface can continue drawing in its own geographic frame.

The surface uses the existing vista shader and ground pigment. Its selected
weather and sward pigment come from the active scene environment. Celestial
light proxies copy the existing sun and moon directions in the sky owner's
east/up/south frame. They do not calculate another weather or time state. The
camera inherits the existing exposure, tone mapping and environment-map handles,
while omitting
street-distance fog and perspective atmosphere settings. The initial regional
surface has no tactical shadow maps.

Hiding the view or entering tactical play deactivates its camera and root while
preserving its pose, CPU product and GPU asset handles. Reopening the same source
and home origin resumes that pose. Replacing actor scenery does not remove map
entities. Replacing map terrain releases the previous mesh and material, keeping
GPU residency bounded to one surface. The actor installation gate does not
blank the map while a city loads.

Static city geometry uses separate `scene` and `regional-map` presentation
owners. Each retains one city buffer set, pending instances, readiness,
level-of-detail history and camera-group visibility scratch. The owners share
material pipelines and compiled building prototypes. Their phase anchors use
separate render layers, so actor views do not draw the focused map city.
Replacing a city's buffers releases that owner's previous phase anchors.
Resetting actor scenery leaves the map owner's assets resident; a complete
presentation reset releases both. The regional owner is prepared for focused
city installation, but no map city document is requested or installed yet.

`wasm_regional_map_status` publishes camera pose, the requested and installed
windows, coverage, typed failure classification and readiness. Readiness settles
after four frames with the shared environment ready and no waiting render
pipelines; an explicitly empty surface can settle with `covered: false`. The
pipeline count currently belongs to the
shared device, so unrelated compilation can delay this initial readiness gate.
It does not cause additional settlement preparation on warm reopening.

`install-overlay` admits a source-matched, bounded collection of canonical
settlement and case-site identities, checked coordinates and one selected route.
The producer must already have admitted the observer's knowledge; the renderer
does not discover locations. Duplicate places, non-map place kinds, mismatched
marker ranks and unbounded routes are rejected at decoding.

Each overlay command includes a positive document-local `revision`, bounded to
JavaScript's exact integer range. The renderer ignores revisions at or below
the last installed overlay and echoes the admitted revision in status. A new
source or home resets that ownership. `presentation_ready` acknowledges four
settled frames for the current overlay independently of route capacity, while
`ready` also requires that the route fit its geometry budget. The browser must
match the source, home and revision before exposing pins or its canvas window.
It also matches the presented window against its last admitted HTTP product;
enqueueing an installation command does not acknowledge GPU presentation.
Status also echoes the physical canvas rectangle. Resizing restarts readiness,
and readiness requires that Bevy's applied viewport match that rectangle. This
excludes the intermediate viewport resize while changing display density.

The status includes covered marker positions in absolute logical canvas pixels,
projected by Bevy's actual cropped camera. HTML owns the corresponding links,
accessible names and tooltips. Small settlements disappear at wider view spans;
current, selected and connected places retain priority. Markers outside the
viewport or on an uncovered triangle have no projected position.

Selected routes use a retained ribbon mesh and the existing unlit material
pipeline. Each centreline is clipped to the terrain window and split at every
grid and diagonal triangle edge. Covered pieces follow that triangle's absolute
elevation plane; missing triangles interrupt the route. Computed routes are
continuous and estimates have visible gaps. Width follows the physical-pixel
scale, so pan and rotation reuse geometry while zoom or resize may replace it.
GPU geometry is bounded to 262,144 route vertices. Excess complexity publishes
`route-capacity` and does not retry geometry every frame.

The camera retains a continuous geographic position. Microdegrees are derived
only for terrain requests and status, so tiny street-scale drags accumulate
without rounding away each movement. Pan distances use the displayed terrain
window's east/north frame, and camera focus height uses its exact covered
triangle plane. Only the camera datum can remain at the window datum over a
source hole; no ground geometry or pin is supplied there.

Terrain-window selection includes the tilted viewport's rotated ground
footprint and a margin for snapped window centres. `frame-route` fits the
admitted selected route using the current viewport aspect and yaw, preserving
rotation and applying space around the route. An absent route has no effect.

The strategic map interface uses this renderer through its existing fullscreen
canvas and compositor. Road data and focused city refinement extend its camera and
geographic frame. City refinement must reuse canonical placement and grading
rather than placing city geometry directly on the ungraded lattice.

## Real browser verification

Build the gameplay Wasm client, then run the matching `wasm-bindgen` CLI with
`--target web --no-typescript` into an ignored output directory. Set
`REGIONAL_MAP_WASM_DIR` to that directory and run:

```powershell
node --test crates/strategic-web/tests/regional-map.browser.cjs
```

The test serves an isolated local fixture without a database. It prepares the
existing woodland environment and exercises the actual Bevy renderer on WebGPU,
including terrain drawing, pan, zoom, rotation, hiding, retained reopening and
invalid command rejection. Optional `REGIONAL_MAP_ASSET_DIR` supplies synchronized
browser assets; `REGIONAL_MAP_REVIEW_DIR` selects the ignored screenshot and
telemetry directory. Without Wasm bindings this check skips explicitly.

The fixture has no actors. Four unauthored movement clips currently remain
unavailable in the repository; the check records those missing URLs while
rejecting any other missing asset or renderer error. It validates map readiness
independently of actor equipment readiness rather than substituting animations.
