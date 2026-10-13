# Fabelgeist tactical client

The atmosphere extraction backport is required by the selected Bevy 0.19 engine.
The build script reads the resolved version from the workspace lockfile and
requires removing that backport and its guard when the project selects Bevy 0.20
or later. Releasing a new upstream engine does not change the selected engine or
make an otherwise unchanged build fail. This check does not access the network.

## Runtime equipment fitting

Armor and clothing are generated locally from each wearer's evaluated, unposed
body. Identity and skeletal fit morphs are evaluated on the body once; skeletal
proportion offsets then place its vertices and fitting landmarks. Equipment
contains no body-shape morph targets. Its inverse bind poses belong to that
wearer's rest skeleton, so animation does not apply proportion changes twice.

The client queues fitting outside frame updates on native builds and awaits
WebGPU initialization and readback in the browser. The compute device creates
no canvas; Bevy renders the resulting geometry in the existing game canvas.
Offline exporters retain their reusable morph-equipped asset workflow.
Compute and rendering dependencies must use the same `wgpu` release: linking
two releases into the browser client duplicates their vendored WebGPU bindings.

Initial loading exercises every authored armor placement on the canonical body,
plus layered shoulders and alternate cuirass and tasset constructions. These
temporary meshes are discarded; the device and compiled compute kernels stay
alive across city travel. Strategic scene readiness waits for this preparation,
even when the initial city has no equipped NPCs. Preparation failures surface as
loading errors. Newly encountered wearers still receive their own fitted meshes;
initial preparation does not cache canonical fits as substitutes for them.

Fits are cached by item design, anatomical placement, and evaluated body shape.
The last 128 variants remain available across travel; live entities retain their
own mesh handles after cache eviction. Changing wearer or proportions requests a
new fit. Holding or dropping a previously fitted item preserves its shape and
anatomical placement across placeholder rebuilds. Carried and dropped meshes
are centered rigid objects; only worn equipment binds to the wearer's skeleton.

Measure fitting separately from rendering with the real canonical body:

```powershell
cargo run -p adventuresim-tactical-client --example runtime_equipment_bench -- assets/animations/biped/unarmed/base.glb 3
```

The JSON records separate device opening, first use, and repeated fits. The
vambrace and breastplate comparisons use the same batched readback for one body
and the full set of body morph endpoints; they isolate morph generation cost.
The browser probe example exercises the same asynchronous runtime generator.
Build it with `cargo build -p adventuresim-tactical-client --example
armor_browser_probe --target wasm32-unknown-unknown --no-default-features`.
Run the matching `wasm-bindgen` CLI with `--target web` into an output directory,
copy `examples/armor_browser_probe.html` there as `index.html`, and copy
`assets/animations/biped/unarmed/base.glb` from the repository as `body.glb`.
Serve that directory on localhost. The page reports initial preparation time,
geometry validity, and first and repeated fitting times for twelve equipment
types after preparation. Kernel counts identify any missed first-use work.
Gorget and mail-coif section measurements intersect body triangles, so fitting
does not require dense vertices near each anatomical measurement plane.

`animation-viewer --armor-harness wearer-fit` renders a gorget, cuirass, and
paired vambraces on the actual animated wearer for fitting inspection.
Device builders generate pauldrons, wrapped tassets, anime breastplates, and
puff-and-slash clothing as well. Unsupported recipes fail readiness instead of
loading served equipment meshes or silently substituting another design.

Generated equipment uses ordinary skeletal attachment during animation. Metal
parts retain their authored rigid joint ownership; cuirass courses share one
chest attachment, and pauldron distal lames follow the upper arm. There is no
runtime plate solver, body collision mesh, or per-frame contact search in this
generation path. Generation-time layer fitting and closed-shell validation do
not establish collision-free animation. Independent plate articulation and
contact correction are separate work.

## Browser release builds

`just build-wasm` keeps the gameplay client on the workspace `release` profile
and builds the art demo separately with the size-oriented `wasm-release`
profile and no default features. The demo therefore excludes client audio and
debug tooling without removing either from ordinary client builds. The build
requires the exact `wasm-bindgen` version in `Cargo.lock` and the Binaryen
version pinned in the root `package-lock.json`; run `npm ci` to install the
latter.

The build applies `wasm-opt -Oz` only to the art demo after `wasm-bindgen` and
writes raw, gzip level 6, and Brotli level 11 sizes to
`crates/adventuresim-stdb-module/static/wasm/bundle-sizes.json`. In the browser,
the developer console's `[art-demo startup]` record separates module download
from Wasm compilation and initialization, making a cold-cache startup directly
repeatable in browser developer tools.

Outdoor PBR lighting uses Bevy's live atmosphere for sunlight transmission,
horizon occlusion, the solar disc and aerial perspective. Generated sky
radiance supplies diffuse and specular environment lighting at intensity one;
there is no added global ambient visibility floor. The environment map is
cached only after GPU completion and invalidated by scene, weather, time,
selected atmosphere, its position/scale, or scattering-medium changes. Camera
exposure does not invalidate unexposed sky radiance. Probe retirement runs
after Bevy's deferred preparation commands. The live atmosphere remains enabled
after the environment bake.

The pinned Bevy 0.19.1 atmosphere shaders use local, source-attributed
corrections: generation samples texel centres, and the forward/inverse lookup
coordinate maps agree. Canonical shader handles and imports stay intact, so
visible sky and environment baking use the same mapping. A shader reload
invalidates the bake; capture readiness waits for the corrected GPU pipelines.
The analytic planet ground samples sunlight transmission at its surface
radius, retaining the dependence on solar angle.
With multisample anti-aliasing, atmospheric scattering and transmission use
each sample's depth and viewport position, so mixed terrain/sky pixels receive
the corresponding transport before color resolution.

This environment map samples the atmospheric medium; the separate animated
cloud renderer is not included in it. Cloud radiance, cloud beam transmission,
and scene bounce lighting require further transport integration. The existing
weather source attenuation is an approximation, not a full cloud-scattering
solution.

The tactical client renders transient server-authoritative combat state with
Bevy. Skeletal animation is presentation-only: the server replicates compact
`SkeletonState` posture, locomotion, stance, action, and timing coordinates;
the client selects and blends authored poses, then applies procedural look and
terrain leg IK.

The equipment HUD shows procedural weapons, holders, and armor as color
portraits on black squares. Weapon recipes render into a transient image cache;
armor loads the baked portrait for its manifest placement. See the
[portrait workflow](../adventuresim-weapon-model/README.md#equipment-portraits)
for lighting and regeneration commands.

EGUI renders the centered incapacitation wheel without taking pointer input.
The segmented arc surrounds the retained Bevy UI crosshair, starts at 12
o'clock, and uses the strategic condition colors for live pain and blood loss
plus enrolled fear, fatigue, hunger, thirst, and temperature; tactical
imbalance is white. Zero incapacitation draws nothing and a full revolution is
the visible maximum.

The gameplay camera is likewise client presentation. A single retained rig
blends from centered lowered-guard exploration to raised-guard right-shoulder
aiming without smoothing manual yaw or pitch. Focus translation uses bounded
anisotropic critical damping and a screen-space sweet spot sized to the current
boom distance. A box enclosing the eye and near-plane corners sweeps from the
controller center around hard geometry. The final framed position is swept
again, so shoulder offsets and focus lag cannot bypass the collision check.
Tight spaces blend toward a close shoulder view with a small height offset and
reduced focus lag. Collision and close framing are resolved together before
recovery, including corners where the close view meets a second obstacle.
Retraction is immediate; recovery waits briefly and then uses critical damping
and distance hysteresis. Soft occluders remain excluded from camera collision.
Raised aiming resolves the center-screen camera target and the subsequent
muzzle path separately. Debug builds use `F6` to show rig, collision, smoothing,
occlusion classification, and aim-ray telemetry.

The tactical workspace targets Bevy 0.19, Avian 0.7, Ahoy 0.2, Replicon
0.41, Aeronet 0.21, Enhanced Input 0.26, and Flair 0.8. The engine upgrade does
not move animation or movement authority: `SkeletonState` and controller state
remain server-owned, while authored pose evaluation, lighting, and the Bevy
world-asset scene attachment are client presentation. Native and Wasm builds
share those semantics through their existing explicit feature sets.

## Client-generated geological geometry

Collider-bearing rocks replicate compact recipes containing a seed, silhouette
archetype, lithology, dimensions, and conservative collision radius. The
tactical server creates only the bounded sphere collider and transform. The
client samples each recipe on a small uniform grid and extracts its render
surface with the private Surface Nets implementation in
`src/presentation/volumetric.rs`; it rejects non-finite fields and tests the
result for deterministic, outward, closed topology. Loose-stone ground cover
reuses four shared client-generated variants without collision or foliage wind.

This extractor is infrastructure for later bounded cliffs, overhangs, and cave
patches, not an implemented cave system. Future scene authority must send a
compact deterministic field recipe, never a server-generated render mesh.
Authored clips are sampled by the client-owned pose buffer. The semantic router
reads presentation state, resolves weighted clip samples through the animation
pack catalog, and applies fixed-rate FK with per-joint inertialization before
procedural terrain, body-response, and weapon-constraint passes.
## Animation export contract

The humanoid base rig is independent from authored motions:

```text
assets/animations/biped/unarmed/base.glb
assets/animations/biped/unarmed/walk.glb
assets/animations/biped/unarmed/swing.glb
assets/animations/biped/unarmed/thrust.glb
assets/animations/biped/unarmed/offhand.glb
```

Only `base.glb` supplies a spawnable scene. Its default scene must retain the
skinned character mesh; `prepare_rig_base.py` strips only authoring helpers such
as the placeholder weapon cylinder. The client attaches this authored scene to
both the client-controlled character and replicated remote characters. Each
other file contains exactly one coherent motion, named or unnamed, and never
has its scene attached. The
30fps `AnimationPackCatalog` explicitly owns every semantic pose through a
file/frame anchor and includes unnamed endpoint/closure frames. Source motion
files belong under `assets_src/biped/unarmed/`; `assets_src/base.*` remains the
rig-source special case until `assets_src/biped/unarmed/base.casc` has a
matching
base GLB export.

Publish and verify every currently available runtime animation without changing
source exports:

```powershell
python scripts/prepare_rig_base.py assets_src/biped/unarmed/base.glb assets/animations/biped/unarmed/base.glb
python scripts/prepare_animation_assets.py
python scripts/prepare_animation_assets.py --check
```

Motion publication validates the one-animation, duration, and canonical
bone-path contracts. Runtime motion GLBs preserve only the canonical hierarchy,
bind transforms, and animation data; mesh, skin, material, texture, and image
payloads remain solely in the spawnable runtime `base.glb`. The publisher keeps
only catalog-addressable frames for ordinary motions, removes tracks that equal
the bind transform, and collapses other constant tracks to one key. Walk and run
store five cubic anchor keys rather than Cascadeur's exported in-betweens.
The `.casc` projects are tracked authoring sources. Their reproducible
`assets_src/**/*.glb` exports are ignored; export them locally before publishing
the tracked runtime GLBs.

Use these conventions:

- glTF coordinates and meters: +Y up, -Z forward, +X anatomical left;
- the scene root stays at the origin and gameplay movement is not baked into it;
- the armature bind pose is a T-pose, which is the final runtime fallback;
- each motion GLB contains exactly one animation and preserves its documented
  semantic anchors;
- locomotion uses only its contact and passing/flight anchor frames. The
  runtime constructs contact -> passing -> mirrored contact -> mirrored
  passing -> contact by sampling the two exact catalog frames on distinct Bevy
  graph nodes, selecting pre-mirrored endpoint clips, and blending complete
  poses with linear quarter-cycle weights;
  every exported in-between key and every later exporter key is ignored; and
- packs in one fallback chain use identical bone names and hierarchy.

Lower-body reflection is evaluated in character space so anatomical left and
right retain their lateral spacing while exchanging gait roles. Authored
upper-body carriage remains intact; explicit hand targets and weapon
constraints apply only when gameplay supplies them. Root, pelvis, spine, neck,
and head translations are clamped around the authored bind pose before look
and final IK, while authored joint rotations remain intact. During active
locomotion, sparse anchor blending advances linearly with phase so the pose
cannot ease to a hold and then accelerate between anchors. Gait parity is
binary per endpoint and is applied before blending; the runtime never
fractionally reflects an already blended skeleton, which would collapse the
forward/back separation of bilateral limbs.
Authored root/pelvis Y is normalized, then the
shared gait profile supplies grounded bounce or a gravity-shaped run flight
arc with two phase-aligned peaks per cycle without moving the gameplay
controller. A contact-edge calibration translates the complete visual rig so
the supported sole meets the rig floor, then retains that baseline through the
stride without reconstructing either leg. Idle poses blend back
to their authored central-bone transforms. The 33mm hierarchy compensation is
measured for upright, lowered-guard `humanoid_unarmed` locomotion only; guard
movement and specialized packs receive no inferred compensation.

## Deterministic animation capture

Republish after changing any motion with
`python scripts/prepare_animation_assets.py`; CI-style verification uses the
same command with `--check`. The publisher requires Python 3 and NumPy.

The native `animation-viewer` binary is a deterministic gameplay-presentation
fixture rather than a separate pose renderer. It installs the gameplay player,
camera, scene presentation, authored FK, pre-mirrored gait endpoints, whole-body
fallback mirroring, look, and terrain-IK plugins, then advances the shared
authoritative locomotion projector at its real 64Hz fixed tick. Default-off
scenarios retain authored ordinary leg motion with a vertically fixed gameplay
root; the explicit cross-slope scenario opts into the seeded terrain-IK pass.
Coverage includes two-cycle 2.0m/s walk, 3.75m/s blend, 5.5m/s run, raised-guard
full/half-speed movement, and start/stop, guard-entry, and guard-release
transitions. Every logical tick is captured first from the raw gameplay
third-person camera, then from side and front diagnostic cameras with a skeleton
overlay and yellow supported-foot / pink swing-foot markers. The simulation is
frozen while those three views are rendered, so they describe one pose. The
output includes per-view PNG sequences, `manifest.json` bone and support
telemetry, and an `index.html` normal/half/quarter-speed reviewer with
representative contact sheets. A missing rig or unresolved locomotion clip times
out with `failure.txt` rather than hanging.

Run it from the repository root:

```powershell
cargo run -p adventuresim-tactical-client --bin animation-viewer -- --output target/animation-captures/locomotion-review
```

Use `--armor-harness close-helmet` to generate the close helmet through
normal gameplay equipment loading. Capture waits for fitted geometry,
materials, wearer skin bindings, and the absence of equipment morph targets.
Front and side views follow the head at inspection distance; the
gameplay view keeps its usual framing. `armor-readiness.json` records the
resolved parts and weights. `--scenario ordinary-camera-pitch` exercises
lowered-guard idle and head pitch; `--scenario raised-guard-stationary-turn`
exercises guard and turning. Add `--hidden` for automated captures without a
visible desktop window.

For supplementary boundary inspection, use `--camera-orbit-degrees 120` to
rotate the front and side cameras around their existing focus. Gameplay framing
stays unchanged. `--diffuse-armor` removes metallic highlights and makes the
supporting cuirass translucent while keeping body anatomy visible. The capture
records these options in `inspection.json`; neither option changes fitting.

Use `--armor-harness puffed` for paired puff-and-slash sleeves and hose. Their
ring and panel geometry, anatomical section fitting, shell extrusion, and skin
correspondence run on the client GPU. Component materials retain the authored
outer fabric and undercloth colors. Worn parts share the evaluated wearer rig;
carried or dropped material parts retain one common item origin.

Use `--asset-root` when invoking it outside the repository root,
`--scenario steady-walk-2.0` for a focused iteration, and
`--frames-per-sample` to change the render settle interval (not the simulated
64Hz sample interval). Open `index.html` after capture and review each scenario
at normal speed before using slow motion. The manifest tracks pelvis, chest,
head, shoulders, elbows, hands, hips, knees, and feet; finite transforms; loop
seams; per-frame displacement and rotation spikes; knee direction;
terrain-relative foot clearance/support/slip; contact-sole grounding;
controller-height stability;
phase-indexed contact/pass height; visual peak count; run flight duration and
sole clearance; and pelvis/head stability. These
values are regression signals and do not establish biomechanical correctness
without visual review.
Capture fails for teleport-scale continuity, ground penetration, duplicate
front/side/third-person image output, missing artifacts, or excessive
supported-foot per-frame slip and planted-interval drift, and records the
responsible frames.

The fixture synthesizes deterministic controller observations at the shared
server projection boundary; it does not run the transport or physics character
controller. Only the cross-slope probe follows rendered
terrain height; flat scenarios verify that animation never changes controller Y.

## Real-client animation diagnostics

Use the supervised diagnostic profile when a problem appears in gameplay but
not in `animation-viewer`:

```powershell
just tactical-play diagnostic
```

This launches the ordinary native client, server, transport, replicated
physics controller, and rendering stack. Once the controlled character is
available, the client turns 90 degrees right, holds forward at 0.5 analogue
input for two seconds, raises its guard for half a second, starts a real
preferred attack, captures a PNG during the attack, exercises full-speed
movement and posture transitions, stops, and exits. The supervisor then stops
its isolated server and database and returns successfully. The profile run
directory contains the generated
`animation-input-script.json`, the per-render-frame `animation-state.jsonl`,
the attack PNG, and the ordinary client/server logs. `just tactical-status`
prints that run directory.

The JSONL record includes the requested command and input, controller
transform, replicated authoritative `SkeletonState`, client-predicted
presentation shadow, semantic `AnimationEvaluation`, resolved clip weights
and sample times, endpoint parity, whole-body mirror coordinates,
phase prediction/correction deltas,
authoritative phase measurements, pending drift correction, any presentation
crossfade, wall-clock time, and the latest render-schedule completion counter.
After final pose evaluation and transform propagation, each record also
contains the global translation, rotation, and scale of every authored
animation target. PresentMon remains the independent authority for actual
swapchain presentation. This is the diagnostic boundary at the pose actually
submitted for rendering; it does not replace the real network or animation
path.

Only the bounded `diagnostic` profile enables the per-frame JSONL log by
default. Interactive `animation` and `combat` sessions avoid an unbounded log;
launch the native client with an explicit `--animation-log PATH` when a manual
session needs one.

On Windows the bounded diagnostic launcher also starts PresentMon when it is
available and writes `presentmon-<session>.csv` beside the JSONL log. This
records ETW display/presentation timing independently of Bevy's update loop.
Pass `presentation_trace=off` to disable it or
`presentation_trace=required` to fail startup when PresentMon cannot run (or
to force it for an interactive profile). `PRESENTMON_PATH` overrides PATH and
standard-location discovery.

For presentation A/B tests, pass `present_mode=auto-vsync`,
`auto-no-vsync`, `fifo`, `fifo-relaxed`, `mailbox`, or `immediate` to
`just tactical-play`. The default remains `auto-vsync`; unsupported explicit
modes are reported by the graphics backend rather than silently selected by
the launcher.

The native client also accepts custom files through `--input-script PATH` and
`--animation-log PATH`. A script has this shape:

```json
{
  "commands": [
    { "type": "rotate", "degrees_right": 90.0 },
    { "type": "move", "direction": "forward", "input_speed": 0.5, "duration_seconds": 2.0 },
    { "type": "guard", "raised": true },
    { "type": "wait", "duration_seconds": 0.5 },
    { "type": "attack", "duration_seconds": 0.25 },
    { "type": "screenshot", "path": "C:/capture/attack.png" },
    { "type": "wait_for_signal", "path": "C:/capture/ready.json" },
    { "type": "wait", "duration_seconds": 0.5 }
  ]
}
```

Movement directions are `forward`, `backward`, `left`, and `right`.
`guard` changes the persistent aiming state. `attack` presses the ordinary
preferred-attack control once and then observes neutral input for
`duration_seconds`; it requires raised guard under normal gameplay rules.
`screenshot` captures the gameplay window directly through Bevy, without OBS.
`wait_for_signal` holds neutral input until its file exists, which lets a
capture supervisor release movement only after recording is ready. Add
`--exit-after-script` for bounded unattended captures.

The gameplay camera runs with bloom disabled and fixed four-sample MSAA.
`minimal` omits the remaining optional presentation features:

```powershell
just tactical-play diagnostic 24920 no-shadows
just tactical-play diagnostic 24920 minimal
```

The normal client uses a 64×64 generated atmosphere environment map.

The same presets are available on the native client through
`--graphics-preset`.

Walk support telemetry remains continuous through its cycle. As locomotion
blends toward run, support narrows around each foot contact. At 5.5m/s the
quarter-cycle run flight unloads both legs for roughly 90-110ms and presents
0.10-0.30m of sole clearance. Contact-phase sole clearance must remain between
-0.02m and 0.04m. When terrain IK is explicitly enabled, high
support retains the foot's world-space horizontal plant until release. Only a
meaningfully supported leg enters the analytic terrain solve; the swing leg
keeps its authored FK and action poses opt out until they expose explicit foot
contact semantics. Idle continues to support both feet.

The replicated skeleton also carries the shared 64 Hz locomotion sample tick,
observed world velocity/acceleration, alternating contact sequence, and landing
sequence/impact. The client transforms acceleration through the current body
frame and advances retained lean only once per authoritative tick, including
bounded coalesced gaps. A hard stop retains the effective authored locomotion
pose and releases it to exact idle over a fixed-tick 0.18-second crossfade,
preventing the sparse run/idle clips from switching in one frame. Landing
response compresses once on a real airborne landing, retains both
pre-compression world foot plants through recovery, and solves the actual
hip/knee chains back to them; it never translates or stretches thigh roots.
Stationary and stopping ordinary locomotion blends both feet back to full
support.

Rendering uses a client-only shadow of `SkeletonState`. Between authoritative
samples it advances gait phase from the most recently measured physical speed
and smooths the displayed local/world velocity. Minor authoritative phase
differences are treated as packet-timing jitter. Larger persistent drift is
low-pass filtered before a slow bounded circular correction is applied, while
posture, actions, contacts, landings, and large discontinuities snap to
authority. This removes packet-cadence pose holds and packet-by-packet speed
modulation without predicting gameplay events or changing the replicated
component.

Contact and landing messages are deduplicated presentation hooks for future
audio/VFX. Plausible contact gaps reconstruct at most eight ordered alternating
contacts; resets, backward sequences, and larger gaps resynchronize silently.
Missed landing updates collapse to the latest observation rather than bursting.
An event's sample tick is the tick where its replicated sequence was observed,
not an invented historical contact timestamp.

Terrain conformity starts off. In debug builds, press `F8` to opt into its
height, slope, and pelvis corrections. The HUD reports whether it is on or off;
authored FK, gait endpoint blending, torso stabilization, and procedural guard
stepping remain active. Ordinary flat-ground locomotion does not run the terrain
leg solver while the toggle is off.

In debug builds, `F7` switches both peers between normal and quarter-speed game
time. The server retains the latest validated analogue movement request across
missing unreliable input packets and restores Ahoy's fixed-loop input from it
before each movement step. That intent drives the controller only. Current
post-physics planar speed selects the idle/walk/run blend and determines stride
cadence, while acceleration is reserved for procedural body response. The
clock toggle therefore cannot directly change walk/run selection.

The procedural humanoid pass recognizes these case-sensitive bone names:

```text
body_world           root                 c_spine0 / c_spine1
c_spine2 / c_spine3  c_neck               c_head / c_camera
l_clavicle / r_clavicle                   l_uparm / r_uparm
l_lowarm / r_lowarm  l_wrist / r_wrist    l_weapon / r_weapon
l_upleg / r_upleg    l_lowleg / r_lowleg  l_foot / r_foot
l_ball / r_ball
```

Finger, face, foot-articulation, and distributed twist bones retain authored FK.
They still participate in full-pose mirroring. `l_weapon`, `r_weapon`, and
`c_camera` are export-added attachment joints on the canonical MHR hierarchy.
Equipment removes each target's rolled authored bind frame before following its
live deformation; worn placeholders derive their +Y axis from semantic joint
pairs, while held weapons retain the character-space +Y tip convention.

The final client-only pose pass distributes bounded look across the actual
spine/neck chain, converts bounded pelvis compensation through its real parent,
and solves legs and optional hand targets across the MHR hierarchy without
overwriting authored twist locals. Foot slope alignment uses the authored bind
transform to derive its sole-up axis rather than assuming an MHR joint-local
cardinal axis. A primary hand socket drives a held weapon, then an optional
weapon-local secondary grip drives the off hand. These targets and constraints
are client-only and never extend replicated `SkeletonState`.

## Missing assets

Ordinary pose lookup first follows the pack's single fallback chain, then its
deterministic similar-pose chain. Attack availability is stricter because it is
a gameplay capability. A pack that defines `swing` or `thrust` owns that
complete main-hand set, and an absent family stays unavailable; only a pack
with neither motion inherits its parent's main-hand attacks. `offhand` resolves
independently through the parent chain. Missing, unloaded, zero-animation,
multiple-animation, or short motion files are unavailable.
Every local or remote character also gets a generated T-pose safety net until
the base scene is available. Bind locals are reset before every animation
evaluation so partial clips cannot accumulate stale or procedural transforms.

## City ground and outdoor furniture

City streets and developed yards use one production material policy across
playable and distant ground. Metre-space cobble and gravel detail blends with
compacted earth, broken edges, static traffic wear, and weather-dependent
dampness. Accepted vendor, receiving, and horse-stop footprints contribute
local wear masks. Static traffic history adds seven carriage gauges, lateral
variation, and tangent-continuous turns at shared endpoints and interior
crossings. Turning front and rear axles leave overlapping marks in both travel
directions, constrained to the visible road/market union. Broad churn covers
most of the central carriageway; wheel marks break up within those deposits.

Traffic, churn, and road-union shoulders are baked once into 64-metre tiles at
four texels per metre. Neighboring tiles share world-space filter gutters, and
all overlapping ground patches sample the same masks. Ground meshes split at
tile boundaries while retaining canonical terrain support. The furniture capture
gate checks the ground masks are GPU resident, and its junction views show
turning continuity and axle variation. Ground meshes sample the same presented
terrain surface; material relief does not change tactical collision or create
physical ruts.

Vista construction shades each environmental sample once for its exact weather
snapshot, then interpolates those linear colours across terrain vertices and
LOD seams. The temporary sample fields stay local to one ring construction and
follow the scene sample bounds. Geometry, colour interpolation order, substrate
pigments and distant sward coverage remain unchanged. Playable-edge pigment is
prepared once with the same scene environment.

Whole foundations are selected conservatively against each ring’s output
rectangles before internal sides are matched. Their bounds include closed-cell
vertices and exposed cut faces. Every intersecting property retains its
original face order and exact clipping; the physical support and collision
representation remain complete. Selection stays local to preparation.

The vista partitioner scans ordered accepted faces once per ring and prepares
each face's rectangle bounds once. Per-rectangle buffers retain source triangle
order; complete chunks retain clipping, normals, pigments and mesh indices.
These buffers contain only current mesh output and are released after
preparation. They do not retain another terrain or collision representation.

Operable windows retain checked scene positions/directions and positive leaf
dimensions through installation and presentation. The closure mesh cache accepts
`LeafDimensions` and converts to the cuboid mesh contract at its explicit native
cache/mesh adapter. Fixed glazing still admits thin or degenerate cuboid geometry.
Cache keys, shared handles and material selection keep their native layouts.
Core's paired `SceneWindowPose` supplies the converted leaf and native rotation.
Building closure visibility and shutter lighting carry `SceneBuildingId` and
`OpeningAssemblyId` through their joins. Material palette selection extracts the
native building number only at the existing deterministic seed/hash adapter.
Fixed bar presence uses the named `WindowBarPresence` enum in both scene APIs
and the serialized `bars` field.

Window animation and catch decisions remain transient server state. The mutable
controller's current angle and ordinary toggle `bool` are a documented #770
handoff under #765; distinguishing controller missing/blocked outcomes is outside
the selected #794 geometry change. Generator admission belongs to #766 and the
shared architectural-to-scene conversion belongs to #767.

Outdoor furniture arrives as compact immutable recipe references and normal
entity transforms. Shared mesh handles and the building material palette
render each accepted instance. Small furniture fades over 180-230 metres;
canvas stalls remain visible to 350-450 metres. Market and frontage placement
extends through the nearest vista ring. Distant furniture shares the production
recipe renderer but receives no physics or ordinary entity replication.
The core owns the clipped vista cells and vertex-height policy used by both
placement and terrain rendering, including seams and LOD morphs.
`python scripts/capture_furniture_review.py --output target/furniture-review`
captures the production implementation with GPU residency and material checks.

## Art-demo cloud assets

The fixed city and oak exhibits embed deterministic initial cloud-shell bakes,
so selecting either exhibit performs no procedural cloud bake on the browser
main thread. Each bake has a generated `.scene-digest` identity; the exhibit
refuses stale output rather than silently baking during navigation. Regenerate
and verify both assets after changing cloud bake logic or either tactical-scene
fixture:

```sh
cargo test -p adventuresim-tactical-client --bin art-demo \
  regenerate_art_demo_cloud_assets -- --ignored --nocapture
```

Production tactical scenes do not use these assets and retain runtime cloud
animation.

## Terrain comparison captures

The `terrain-grounding` capture profile requires explicit frozen world-space
cameras through `--city-cameras`. It renders at 2240 × 1260 physical pixels,
independent of desktop scaling. The image manifest reports those same physical
dimensions. Other capture profiles retain their standard 1280 × 720 images;
scene-performance measurements use a separate 2560 × 1440 offscreen target.
Cameras, time, weather, exposure and physical member selection must be fixed
when comparing terrain support across generated products and building LODs.
Explicit capture members are prepared for playable presentation only after the
unchanged input has reconstructed its accepted terrain. Their exact programmes,
placements and floor elevations are retained, and their distant drawing entries
are suppressed. The existing authoritative projectors generate each selected
owner's enclosure and garden against that same accepted terrain. The saved
input and scene digest identify the authoritative
source; `fixed-camera-contract.json` records this presentation selection.
Ordinary images and wireframe/collision diagnostics are separate evidence.
