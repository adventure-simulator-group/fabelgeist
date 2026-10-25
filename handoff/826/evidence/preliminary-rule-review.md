# Preliminary independent Rule Review of PR #826

No new rule regression was found in the current independent draft repairs.
Correctness is source-reviewed only; migration completeness is **incomplete**.
F2, F3 and the D774 ownership/canonical-contract dependency remain actionable.
This report grants no final clearance.

## Snapshot and method

- Base: `3a50b708381ad00c116445c2c999a49c3943088b`.
- HEAD: `71a2d1ffc6203c251f87a4c4e759670058d620b3`.
- Working snapshot: the twelve file fingerprints, including all three untracked
  `views.rs` files, in `preliminary-review-snapshot.json`.
- At completion, HEAD and all twelve fingerprints still matched. Manifest
  SHA-256: `1be2e038b70f58cb2b3c8e123c0cb875e766697b4514238215cf76975984669d`.
- Read root `AGENTS.md`, `crates/AGENTS.md`, the Rule Review skill and its
  bespoke-type reference. No deeper GPU/compute AGENTS files were found.
- Inspected current source and the base diff before reading the ledger. The
  ledger and #774 exact claim receipt were then used to reconcile findings.

This reviewer did not implement the changes. Only this ignored report was
written. No build, test, lint, generator, scanner, project runtime, hosted job,
push or external post was performed. The requested hosted-only validation and
ban on new tests/cases/helpers/assertions override the skill's general advice to
run checks or probes locally.

## Actionable existing migration gaps

### F2: cache/native/metadata admission remains independently mutable

Severity: P2. This is an existing in-scope migration gap, not a newly introduced
regression. `Texture2d` at `texture_2d/mod.rs:17-23`, `Texture3d` at
`texture_3d/mod.rs:11-17`, and `TextureCube` at `texture_cube/mod.rs:14-20`
expose independently writable format, cache, native handle, dimensions and
usage. Their selectors trust `format == self.format` before native admission
(`texture_2d/views.rs:15-21`, `texture_3d/views.rs:15-21`,
`texture_cube/views.rs:19-25`).

A caller can create an R32Float texture, assign its public format to Rgba8Unorm,
then request Rgba8Unorm and receive the original R32Float cache as success.
Replacing the cache with another texture's cache likewise defeats provenance.
`TextureView::from` at `texture_view/mod.rs:296-306` propagates the contradiction
to another record. This violates `crates/AGENTS.md:31-35,49-51`: semantic
records must enforce their relevant invariants and reject invalid combinations.
The typed error itself cannot repair admission to the successful state.

Complete one controlled resource/cache metadata contract and adapt every
receiver. Preserve the supported detached cache and cache-absence precedence;
forcing both optional handles to be present would change the current contract.
Do not add independent per-shape equivalent wrappers, per
`crates/AGENTS.md:69-71`. Device provenance is also unresolved: each alternate
selector scopes/polls the supplied context's device while creating on the
retained texture. The signature does not establish that they share a device.
This representable weakness belongs to F2; no actual cross-device caller was
observed in this review. It is not a new regression in the draft.

Acceptance: private/coordinated construction prevents arbitrary relabeling and
cache replacement; explicit removal retains all supported availability states;
the selected native view carries effective format and dimension consistently;
the device that owns the native error sink is established for validation.

### F3/D774: shared consumers and retained causes are not complete

Severity: P2. `ComputePass::record` selects a view but derives `actual_format`
from the allocation in all whole-texture arms and the generic view arm
(`compute/pass/mod.rs:263-352`). Its following validation consumes that separate
fact (`362-400`). `bind_group.rs:208-257` repeats the allocation-format/cache
split. Linear allocation format and selected sRGB interpretation are distinct
facts even when both fields are individually valid; arbitrary public field
replacement makes the disagreement wider. This is the relying-consumer half
of F2 under `crates/AGENTS.md:31-35,49-51`, not a request for a second format
enum or an unrelated reflection redesign.

The selected alternate-view calls propagate `TextureViewError` with `?`, which
retains it. Direct missing-cache paths still emit generic prose
(`compute/pass/mod.rs:412-425`, `bind_group.rs:247-257`). The shared pipeline
retains a native rejection as `Option<String>` (`compute/pipeline/mod.rs:20`,
`114-117`), erases parser causes in `validate_interface` (`126-128`), and emits
another formatted error in `compile` (`176-183`). These fail the requested
connected native-cause completion contract until the canonical dependency is
integrated. They are existing incomplete paths, not defects introduced by the
current texture draft. No source branch on those display strings was found.

The exact #774 receipt at
`774-records/records/parallel-ownership/rig_garments-claims.json:4949-5008`
claims the whole pipeline retained-cause record and listed lifecycle methods,
`shader::parse_naga`, and `ComputePass::record`. It requires one whole-function
repair with retained concrete causes. Ownership agreement is still a proposal,
not authorization. This review makes no competing error owner and grants no
deferral. The user's explicit ownership requirement governs this dependency.

Acceptance: obtain explicit agreement, consume the canonical shared error and
selected-view contracts, finish all four receiving texture variants plus the
public bind helper, and re-review the full producer-to-recipient path. The
pipeline cause/cache lifecycle must be completed by its agreed owner before
the broader native-cause family can pass.

## Source-backed family traces and dispositions

| Family | Trace and result |
| --- | --- |
| Whole-view failures | `view_error.rs:10-107` owns closed missing/uninitialized/native method variants, `TextureFormat` requests and boxed `wgpu::Error` causes. All three `views.rs` producers admit failures there; Display is presentation and `source()` returns the actual native error. `TextureViewResult` meets the owning alias rule at `crates/AGENTS.md:118-122`. No new primitive classification or prose-driven branching was found. This increment is correct by source inspection. |
| Cache selection | Matching format clones the cached Arc or returns MissingView before requiring an allocation. Different format requires allocation, preserves the original request in rejection, and creates a fresh Arc. Storage usage maps an sRGB descriptor request to its existing linear counterpart. Cube dimension/layer count remain Cube/six. `TextureFormat` and `CubeFace` are reused canonical owners; native SDK enums and labels remain explicit adapters. Completeness is partial because of F2/F3. |
| Native causes and context policy | Initial 2D/3D/cube constructors and cube-face construction gate scopes on the existing context policy; polling only occurs after a scope was created. Rejections preserve `wgpu::Error` with anyhow context rather than formatting the replacement error. Browser/borrowed contexts have no synchronous native rejection verdict, as documented. No new context policy owner was added. Device provenance still requires F2 resolution. |
| Retained records and adapters | Whole-texture constructors initialize cache/allocation metadata together; Default intentionally admits absent handles. Clone preserves sharing. `TextureView::from`, cube-face construction, `GpuResource`, `PassParameter`, `ResourceDescriptor::from_resource` and the two bind paths carry these values to native shader/binding/copy boundaries. Existing formats remain useful logical interpretation types; native allocation format must not replace that meaning. The currently public admission bypass remains actionable F2. |
| Missing recipients | `Texture2d::blit` checks both caches before native work. DistanceField minimum operations check the input native handle before parsing/allocation/copy and remove encountered unwraps. Constructor-created upload ports use typed absence errors. `TextureSrgbConverter::convert` uses the shared MissingView error and retains source/destination context. This conforms to `crates/AGENTS.md:126-127`; initialized operation order is retained by source comparison. |
| Default pixel policy | `texture/mod.rs:20` owns the 256-pixel default, consumed by 2D default construction/color and cube-from-textures. This centralizes a shared tuning value under `crates/AGENTS.md:23-30`; it does not introduce an unnecessary primitive wrapper. Existing constructor units and rounding are preserved. |
| Auto traits and module ownership | Redundant unsafe Send/Sync implementations were removed; SDK fields now determine those traits. The focused view modules own selection/subresource construction and preserve inherent public API visibility. The large parent modules shrink; no baseline ceiling or suppression was broadened. This is consistent with `crates/AGENTS.md:95-110`. Actual native/Wasm trait compatibility remains unexecuted. |
| Validation and prose | The workflow invokes existing integration tests and doctests, writes the checkout SHA and uploads evidence. Existing tests/fixture were read, not expanded. `texture-views.md` distinguishes historical software-GPU observations from current acceptance and retains browser/hardware/performance/rendering limits. Root README/wiki are untouched. This follows root `AGENTS.md:50-55`. Hosted results remain outstanding and must match the final revision. |

The inspection included whole-view selectors, constructors, face/array view
construction, whole-to-generic conversion, the directly affected missing-state
recipients above, texture error propagation through ComputePass dispatch, and
the actual format/cache binding recipients. Unrelated numerical/shader kernels,
ZIP/NumPy, sorting and #774's broader implementation were not independently
audited or repaired. This is a bounded preliminary Rule Review, not a complete
new thirteen-lens wave or a final review of still-unwritten F2/F3 changes.

## Verdict and final requirements

Correctness of reviewed draft repairs: no newly introduced rule regression
identified; execution unverified. Migration completeness: incomplete at F2,
F3 and D774. No optional additional abstraction is required by this review.

After the dependency agreement and connected repairs, repeat independent Rule
Review with the final complete source snapshot. All thirteen lenses, independent
Code/Rule/Comment reviews and required hosted project checks must pass on the
same final pushed revision. No final pass can be inferred from this report.
