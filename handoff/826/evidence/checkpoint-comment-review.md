# Integration checkpoint Comment Review

Status: **actionable findings; preliminary only**.

## Revision and scope

- Reviewed checkpoint: `a03ccb5da481872dff2aa2bdbb0222b93c7314a6`.
- Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.
- The checkout was clean at entry and remained at the checkpoint during the
  source reads. The committed diff, rather than prior reports, defined scope.
- Applied the Comment Review skill, root `AGENTS.md` and `crates/AGENTS.md`.
- This review covers the integration's changed/moved comments, guide and API
  examples, with their implementing paths, existing fixtures and materially
  affected consumers. It is not a complete new thirteen-lens audit wave.
- Shared GPU ownership is transferred following #774; the former ownership
  blocker is not carried forward. #780's checked count/shape/cache/source
  migration remains active and must be integrated before final review.

## Actionable findings

### C1 — The fast-path barrier/performance narrative is not the current contract

Locations:

- `crates/fabelgeist-compute/src/kernel/fast.rs:3-30` and `191-192`.
- Related repeated causal claim at `kernel/mod.rs:409-416`.

The module says reusing a bind group makes wgpu stop emitting barriers between
dependent compute passes and that recreating the group restores them. It also
states timings, failure rates and allocator exhaustion without an identified
revision, device, experiment or current validation evidence. These are presented
as unconditional current native behavior rather than a bounded historical
observation. The paragraph still calls the current allocator a “ring” at line
30, while `fast/arena.rs` gives each batch its own chunks and never wraps slots
within a batch.

Pinned `wgpu-core` 29.0.4 provides contradictory evidence for the general causal
explanation: `src/command/compute.rs:281-323` explicitly documents a fresh usage
scope and merging active bind groups at each dispatch; `dispatch` calls
`flush_bindings` at `835-840`, independent of bind-group identity. Across passes,
the same file inserts barriers from the tracked resources at `775-780`.
This source does not prove that no backend-specific bug ever occurred. It does
show that the comment cannot explain the current wgpu contract by asserting
that identity alone suppresses resource registration/barriers.

Keep per-dispatch bind-group construction and the existing BVH acceptance test;
this documentation review does not authorize changing that behavior or adding
tests. Replace the causal and numerical claims with a bounded explanation, for
example:

> The fast path shares its pipeline and allocates uniform slots in a
> batch-owned arena. Each dispatch creates its own bind group. Preserve this
> policy when changing the path, and validate dependent dispatches with the
> existing BVH refit acceptance test.

If the historical observation is necessary to explain the policy, label it
historical and cite durable reproducible evidence with its revision/backend.
Do not claim a backend mechanism or current timing that this review cannot
establish. Change remaining “ring” wording to the actual per-batch arena.

### C2 — The uniform capacity comment promises allocation-free packing

Location: `kernel/fast.rs:50-55`.

“The largest uniform block the cached path packs without allocating” is false
for an admitted `Mat4`: `kernel/fast/uniforms.rs:43-50` allocates a
`Vec::with_capacity(16)` while packing that value. `FastPath::prepare` also
allocates its resource entry vectors. The 256-byte threshold controls the size
of the uniform-block stack scratch, not all host allocations.

Suggested replacement:

> Maximum uniform-block byte length admitted by the fast path's stack scratch.
> Larger blocks use the static dispatch path.

This corrects the documented guarantee without inventing an allocation-free
requirement. If the active uniform/source migration removes that allocation,
review the resulting implementation before making a broader performance claim.

### C3 — Cube-face context now participates on browser builds too

Location:
`crates/fabelgeist-gpu/src/data/gpu/texture/texture_cube/views.rs:85-86`.

“Only read to check the view natively; the web cannot block on it” is stale.
The context is now read by `ensure_device` at lines 95-97 on every platform;
only the scope and synchronous readback are native-only. The moved
`allow(unused_variables)` annotation also no longer has that stated reason.

Suggested comment:

> Check device identity when supplied; native builds may also read back
> validation according to the context policy.

The now-used browser variable needs no unused-variable annotation. Preserve the
device check and the context-free method's retained-allocation behavior.

### C4 — The guide describes an unconstructible logical-format/storage state

Location: `crates/fabelgeist-gpu/texture-views.md:55-59`.

“A logical sRGB texture can have a linear storage view” does not describe a
state admitted by these immutable owners. The three constructors add
`STORAGE_BINDING` only if `format.supports_storage() && !format.is_srgb()`:

- `texture_2d/mod.rs:111-113`.
- `texture_3d/mod.rs:75-77`.
- `texture_cube/mod.rs:89-91`.

The new private metadata and removal-only APIs cannot relabel a linear storage
texture as logical sRGB. The actual alternate-view case is a **linear** storage
texture receiving an **sRGB view request**: descriptor linearization preserves
the original request for rejection diagnostics, but the owner's logical format
stays linear. `From<&Texture2d> for TextureView` uses the owner's logical format
and its cached view's effective format; it does not retain an earlier alternate
request. Existing fixture
`cached_and_alternate_views_preserve_storage_policy_and_sharing` also constructs
the stored texture as `Rgba8Unorm`, then requests the sRGB counterpart.

Suggested replacement:

> For a linear storage texture, an sRGB view request selects the linear
> counterpart at the descriptor boundary. Bindings validate that effective
> view format. Converting a texture to `TextureView` retains its logical image
> interpretation and cached view's effective format.

Retaining both facts is a useful design contract; describe a reachable producer
path rather than implying that all logical/native combinations are admitted.

### C5 — KernelCache's compile-once claim excludes its documented race

Location: `crates/fabelgeist-compute/src/kernel/mod.rs:172-177`.

The type says it compiles each distinct source once. `get` at lines 188-201
checks under a read lock, compiles outside the write lock, and only then uses
`entry(...).or_insert(...)`; simultaneous misses can each run `Kernel::new`.
The local comment explicitly acknowledges the race. The compile mutex serializes
compilations but does not repeat this cache lookup before the second compilation.

For that implementation, accurate wording is:

> Reuses admitted kernels by source. Concurrent first lookups may compile the
> same source before one result is published to the cache.

The active #780 cache/source migration may change both cache identity and the
race policy. Resolve this finding as part of that connected family and document
its final behavior. It remains unresolved at this checkpoint; no final clearance
or accepted deferral is implied.

## Reviewed claims that match this checkpoint

The previous blanket queue-draining rationale is removed from the pipeline
method, and the context policy documentation now matches pinned wgpu 29.0.4's
latest-submission-at-poll-entry semantics. Native readback is conditional;
browser/borrowed paths do not claim synchronous native acceptance. The report
does not treat device-handled creation as an observed successful validation.

Read each important changed or moved comment with these implementations:

- Texture owners, three view modules, `selection.rs`, `texture_view/mod.rs`,
  and `view_error.rs`: private allocation/cache/metadata, removal operations,
  cache-first behavior after allocation detachment, native-device equality,
  effective selected format/dimension, source-less admission errors and retained
  native causes. The compile-fail example retains the correct concrete result
  mismatch. Cube-array rejection has its own native cause variant.
- Shader parser/error and immutable `ComputeShader`; reflected sparse group
  layout creation; immutable `ComputePipeline` and cache outcome variants:
  concrete causes survive, cache keys use full entry-point names, observed
  rejection is retained across clones, and a readback-enabled context recreates
  a prior device-handled cache entry instead of accepting it as observed.
  Preparation reflects/creates binding layouts but does not compile the native
  compute pipeline. The new comments reflect that distinction.
- Shared binding assembly/public adapter and `ComputeBindGroup::bind`: group
  indices are retained rather than enumerated after gaps are omitted. Static
  uniform packing preserves the reflected offsets and existing matrix policy.
  Selected texture facts, rather than allocation format, are validated.
- `Buffer::binding_resource`: logical zero is allowed as metadata but rejected
  as a binding; oversized logical prefixes fail before native binding creation.
- `ComputePass`, grid/shape/count owners, `KernelBatch` and moved fast helpers:
  batch emptiness returns before parameter admission/counter increment;
  standalone static dispatch still admits bindings for an empty grid. Slot
  addressing and native dynamic-offset conversion are checked. Per-batch
  ownership and after-submission callback order match the source.
- The moved read-image documentation is attached to `read_image` and agrees
  with its byte/channel conversions, rather than documenting `write`.
- Existing texture-view fixture JSON, public fixture adapters and unchanged
  assertions: tiny constructor inputs, absence/branch priority, native request
  and cause retention, standard error context, cached identity and alternate
  storage-view checks agree with their descriptions. Only API adapters changed.
- The sort test's zero expected value is now the default recorded-dispatch
  count; the input and zero-dispatch oracle remain unchanged.
- GPU README guide link and workflow validation step labels/commands: the link
  resolves, integration tests are serial, doctests run separately, and hosted
  artifacts record source and logs. No passing result is inferred from YAML.

Inventoried every changed Rust comment through the complete committed comment
diff. Also read the materially affected adapter hunks in GPU BVH, hair, mesh,
physics and the compute primitives (advect, broadcast, distance-field/JFA,
divergence, dual contouring, gather, gradient, map, marching cubes, matmul,
prefix sum, reduce, reshape, scan, scatter, sort, stencil, stereo, stream,
transpose). Their accessor/grid/byte adapters do not themselves introduce new
commented contracts. Read nearby dependent-buffer/copy/empty-sort comments in
BVH, hair, physics and sort to assess whether the changed boundaries invalidate
those explanations. No claim of a repository-wide review of every unchanged
comment is made.

## Validation limits and completion requirements

This was independent source review only. No tests, cases, helpers or assertions
were added; no source file was edited. No local project checks, builds, scanners,
runtime execution, push, CI request or external post was performed. This ignored
report is the sole authored artifact.

Pinned dependency source was inspected locally to verify polling and resource
tracking semantics; no current hardware/browser/performance verdict follows.
Historical numerical observations in comments were not reproduced or accepted
as current evidence. Existing tests were read, not run, and no fixture changes
were requested.

The active #780 count/shape/cache/source migration must finish, findings must be
resolved, and a full independent Comment Review must run again on the same final
pushed SHA as the other reviews and required hosted checks. This checkpoint
does not establish source completeness, a passing test result or final clearance.
