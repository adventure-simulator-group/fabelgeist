# Forest component measurements

The environment viewer can record the work needed to grow and draw one
deterministic forest snapshot. Use this to attribute generation, packing,
upload and rendering costs before changing the foliage architecture.

These measurements cover the native offscreen forest component. They do not
establish browser-area arrival time, network transfer, landscape-scale
coverage, or integration with the tactical client's other foliage.

## Prepare before reserving the machine

Build the viewer in release mode. Use one absolute Cargo target directory
for the baseline and candidate, with one compiler workload at a time:

```powershell
cargo build --release -p adventuresim-environment --features viewer --bin environment-viewer --target-dir C:/path/to/shared-target
```

The runner uses an existing executable and never builds. Without `--run`, it
writes the contract without starting the viewer:

```powershell
python scripts/benchmark_foliage.py --executable C:/path/to/shared-target/release/environment-viewer.exe --provenance target/forest-host.json --output target/forest-plan
```

Before measuring, record the hardware, OS, driver, power mode, revision, dirty
files, compiler, backend and quality settings. The runner hashes the binary
and records source state. Keep a copy of the source diff alongside the raw
evidence when measuring uncommitted changes.

Pass that frozen JSON file with `--provenance`. It must contain nonempty
`hardware`, `software`, `tool_versions` and `render_settings` records. Include
the selected adapter/backend, power mode and driver; verify them against the
pilot's actual process log before repeating runs. Keep unresolved fields
explicitly unknown during the pilot, not as assumed hardware facts.

## Prepared unique and shared-tree controls

The local prototype separates removal of growth replay from reuse of tree
geometry. Prepare each source forest offline with an explicit species-wide
variant budget; this command does not measure arrival:

```powershell
environment-viewer --prepare-forest target/prepared/mixed --scene mixed --years 120 --profile fast --variants-per-species 16
environment-viewer --prepare-forest target/prepared/boreal --scene boreal --years 120 --profile fast --variants-per-species 16
```

Each new output directory contains unique records, a separate model library,
and regional placements bound to that library by a SHA-256 content digest.
All required files must be available to a fresh client. File reads, decoding,
record uploads and mesh readiness count in the prepared arrival path. Local
file reads do not establish network-transfer costs.

Capture these assets without replay or performance measurements:

```powershell
environment-viewer --prepared-forest target/prepared/mixed --representation unique --snapshot target/unique.png
environment-viewer --prepared-forest target/prepared/mixed --representation shared --snapshot target/shared.png
```

Prepared captures require completed draws and consecutive identical GPU
readbacks. For a small all-plants renderer oracle, prepare with `--all-plants`
and compare both representations with `--fixed-lod full`, `middle`, and
`coarse`. Use a small deterministic fixture for this exact geometry control.
Preparation rejects address meshes larger than 256 MiB; metadata and GPU
integer-address bounds also apply.

Shared trees retain species, stable identity and root. Offline matching uses
height, radial width, branch/leaf complexity and vertical crown occupancy;
these shape measurements are separate from conservative render bounds.
Placements scale the chosen shape to source height/width. Asymmetric crowns
retain their dominant direction; other substitutions receive deterministic
yaw from stable plant identity. An all-plants oracle retains unit scale/yaw.
An explicit species budget must accommodate observed empty, bare and leafed
geometry states. A model's source light samples and crown shape remain an
approximation when reused elsewhere. Review silhouettes, repetition, canopy
coverage and lighting before interpreting timings.

The shared renderer instances whole trees in capacity/size/16-metre-cell
batches. Power-of-two branch and leaf capacities share one integer-address
grid per class and LOD across source models and species. Placements select
actual model record spans. Padded slots return clipped degenerate vertices
before reading botanical records. They still cost vertex invocations and
placement metadata reads; report allocated slots separately from live records.
All three grid LODs remain resident. Frustum culling is per spatial batch;
distance culling is per tree. Conservative bounds come from actual transformed
models, not grid capacity. Merged batches can change adaptive LOD decisions,
so use fixed common LOD for renderer equivalence and review adaptive views.
Placements also carry horizontal/vertical scale, yaw and translation.
Procedural bark and crown occlusion use model coordinates; surface normals
use the inverse transpose of the placement scale.

After correctness passes and contributors clear the machine, use the same
runner with `--prepared-root target/prepared --representation unique` or
`shared`. Omitting those options measures replay. Freeze the same asset files
and effective detail for each comparison. The runner records required asset
hashes and bytes; the viewer reports library records, placement instances,
spatial batches, capacity slots and all-LOD mesh payloads separately. Per-LOD
record/slot totals describe batches before GPU culling, not visible vertex
invocations. Peak memory remains
unmeasured. This prototype does not implement distant impostors, ecological
cohort updates or production area streaming.

### Indexed-padding diagnostic

For shared prepared snapshots, `--tree-layout model-padded` and
`--tree-layout model-live-indices` require an explicit `--fixed-lod`.
Both construct the same per-model/part meshes and model/cell/size batches.
Both retain full capacity vertex arrays. The second uses only the canonical
index prefix for actual model records; it does not index padded records.
Assets, poses, shaders, conservative bounds and culling policy stay fixed.
`--tree-layout capacity-batches` is the normal sharing control.

Before cloning meshes, admission bounds the retained source grids plus all
requested runtime grids to the existing 256 MiB address-payload limit in
every mode. This is a payload bound, not a peak-memory measurement. Model
layouts intentionally duplicate vertices and can cost more startup/memory;
their purpose is to isolate indexed padding with matched batch partitions.

Report schema 6 names per-LOD referenced work `indexed_record_slots`.
`capacity_slots_implied_by_shared_placements` still describes per-placement
capacity potential. `shared_meshes` records the actual requested grid count,
vertex/index bytes, layout and a SHA-256 of batch keys, bounds and complete
instance payloads. Canonical `library_mesh_vertex_index_bytes` describes the
source library, which is a different payload in model modes. These counts
exclude common primitive meshes and allocator overhead. Check matching
batch digests and all per-LOD records/instances before interpreting a padded
versus live-index comparison. Changing index lengths still changes buffer
allocation/bin placement; it does not isolate one shader instruction.

### Structural-observer diagnostic

`--probe-validation every-frame` and
`--probe-validation sampling-endpoints` compare the cost of repeated
structural checks on a prepared snapshot with an explicit `--fixed-lod`.
Use the same binary, assets, camera, mesh layout and requested frame count
for each pair. The default remains `every-frame`.

Both policies require a fresh full check after warmup and another after
sampling. Each request is latched before that render frame's draws and
acknowledged only after its GPU wait and full validation. The first draw
milestone cannot satisfy either request. Both policies reject new uploads
or expected-geometry changes after the start request. Final screenshots
and reports wait for the final acknowledgement and two stable readbacks.

Only during sampling, `sampling-endpoints` omits the repeated pipeline,
mesh, batch-membership and count-histogram scans. It keeps per-draw atomic
counters, GPU waits and constant-cost failure polling. It cannot establish
intermediate-frame structural validity or detect transient pipeline errors
that disappear before the final scan. This is a diagnostic for frozen
inputs, not a validation policy for gameplay or moving-camera captures.

`probe_validation` records the policy, completed lifecycle and
successful full/skipped structural check counts. An untimed capture using
either explicit policy exercises the same start/end handshake and one
intermediate render, without `--frames` or `--benchmark-report`. Compare
those captures before timing. Sampling starts after the start transition;
final validation and screenshot work occur after the last frame timestamp.

The difference between policies measures repeated structural observation
within this serialized harness. It does not measure total probe overhead
or ordinary presentation throughput: draw counters and GPU waits remain.
Use a contemporary every-frame control; do not pool historical results
from the previous lifecycle with this comparison.

### Render-stage profiling

For prepared snapshots with an explicit `--fixed-lod`,
`--render-profile control` leaves the original render systems in place.
`--render-profile instrumented` records render-phase wall spans and wraps
the original tree-culling functions with native GPU timestamp markers.
The original functions still encode the dispatches. Each original system
must be found exactly once; setup fails otherwise. The wrappers run inside
the render graph's rendering phase, before the camera driver. Their three
piped stages run on one worker and queue begin/dispatch/end buffers in order.

Both modes save a `.profile.json` sidecar beside the final screenshot.
Schema 6 measurement reports also include `renderer_profile`; it is null
when no profiling mode was requested. Untimed captures exercise profiling
and report admission without supplying frame intervals. Validate those
captures before coordinating a timing window.

CPU samples begin when Queue observes the frozen Sampling phase. They cover
Queue, Prepare, graph encoding, graph submission and the existing blocking
GPU poll. A complete frame needs all five spans. These are inclusive wall
times, including scheduling gaps, not CPU busy times. They are not matched
by ordinal to the main-world frame-interval vector. Submission includes
encoder finishing and uncovered-target handling as well as queue submission.
The GPU wait remains a stall span, not additional GPU execution time.

The branch and leaf culling histories include CPU encoding-body and GPU
timestamp observations. CPU cull spans exclude deferred encoder finishing
and actual submission; marker work can contribute. These and the other
render diagnostics retain Bevy's asynchronous bounded histories. They may
include warmup or final frames and are not an exact 1800-frame distribution.
Preserve their paths and raw history values. Instrumented mode requires
enabled encoder timestamp-query features and all four cull diagnostic paths.
Missing evidence fails capture rather than becoming a zero-duration result.

Profiling adds clocks, schedule boundaries, command buffers and GPU queries.
Measure its effect with a contemporary control in the same build and layout.
Do not subtract the control difference to manufacture a production result,
sum overlapping CPU/GPU/wait spans, or infer a critical path from opaque-pass
timing alone. Ordinary viewing and captures omit these hooks by default.

## Measure on an otherwise idle machine

Coordinate with other contributors before starting. Do not overlap these
runs with compilation, tests, captures or other performance workloads.
Start with one bounded pilot to validate the evidence path:

```powershell
python scripts/benchmark_foliage.py --executable C:/path/to/shared-target/release/environment-viewer.exe --provenance target/forest-host.json --output target/forest-pilot --fixture mixed-context --repetitions 1 --frames 60 --timeout 120 --run
```

After the pilot has valid draw observations and visible foliage, use the
default five process runs and 600 frames for each fixture. Each output folder
must be new. A timeout or missing evidence stops the series at its first
invalid run; diagnose that contract before continuing.

Each run records:

- Scene construction, simulation GPU setup, growth and snapshot packing.
- Structural record counts and bytes, queued and submitted writes, dropped
  or superseded writes, and GPU errors.
- The first GPU-complete frame in which all source chunks reached their
  forest draw commands. The source chunk identities and instance totals must
  agree with the render batches.
- Effective renderer detail and camera, raw GPU-waited frame intervals, and
  the time of the first and stable screenshot readbacks.
- Raw process output, the launch command, numbered readbacks and a matching
  final pair. The runner retains image hashes for provenance.

The draw-command observer runs after the existing draw command sequence. A
skipped draw does not increment it. Its GPU wait and bookkeeping add overhead;
these intervals describe serialized component diagnostics, not ordinary
presentation throughput.

## Interpret the evidence

Encoded indirect draws can still have zero visible instances. The probe does
not observe every culling dispatch or read back the indirect argument buffers.
An independent visual check must confirm the expected forest, crown coverage,
species and density before using a sample to justify an improvement. A fast
empty frame is invalid evidence.

The two matching final readbacks establish stability, not botanical quality.
Review the numbered predecessor of `forest.png` identified in `result.json`.
The earlier numbered readbacks preserve failed settling attempts.

The stable-readback time is a conservative capture-end bound. It includes the
existing one-second and thirty-frame settling period and readback overhead;
it also occurs after the requested steady-frame interval. Repeated PNG writes
can extend it. PNG encoding and writing the final image occur after that
timestamp. Do not call this exact first-visible-frame latency or subtract
the settling time to manufacture one.

The three built-in fixtures are small deterministic patches and one close-up.
They diagnose component costs, not distant landscapes. Each fixture uses its
own recorded camera. Compare the same fixture, seed, age, camera, density and
effective detail between revisions. Freeze explicit camera poses before a
candidate changes the content bounds or automatic framing.

The machine-readable probe requires a frozen camera and snapshot. It rejects
`--orbit` and `--refeed`; normal viewer frame timing still supports them. A
moving-camera LOD comparison needs its own recorded path and capture poses.

Fresh processes retain filesystem and driver caches; the runner does not
claim to clear them. Measure resident-client unseen destinations, first-use
assets and cached returns separately when the production arrival path is
connected. Include all visible vegetation and concurrent area initialization
in that final test. Require every acceptance run to meet the two-second
budget, with less than one second as the headroom goal.
