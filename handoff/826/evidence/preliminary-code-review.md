# Preliminary independent Code Review of PR #826

This review found no new discrete correctness defect introduced by the current
draft repairs. It does **not** grant final clearance: F2, F3 and the D774
ownership/dependency agreement remain unfinished, and no validation was run.

## Revision and evidence

- Base: `3a50b708381ad00c116445c2c999a49c3943088b`.
- HEAD: `71a2d1ffc6203c251f87a4c4e759670058d620b3`.
- Working changes: exactly the twelve paths and SHA-256 values in
  `target/pr826-audit/preliminary-review-snapshot.json`.
- Snapshot manifest SHA-256:
  `1be2e038b70f58cb2b3c8e123c0cb875e766697b4514238215cf76975984669d`.
- At review completion, HEAD matched and every snapshot fingerprint matched.

Reviewed actual source and the base diff before reading ledger findings. Then
compared the observations with `target/pr826-audit/ledger.json`. Repository-root
and `crates/AGENTS.md` rules were read. This is an independent review agent;
the agent did not implement the reviewed changes.

## New findings

None. The reviewed changes preserve the initialized descriptor policy and
replace the encountered missing-handle panics with early error returns. The
remaining issues below are already inventoried dependencies, not new defects
attributed to these draft repairs.

## Existing dependencies that still prevent clearance

### F2: mutable cache, native handle and metadata remain independent

The public fields at `texture_2d/mod.rs:17-23`, `texture_3d/mod.rs:11-17`
and `texture_cube/mod.rs:14-20` still permit replacing `format`, `view`,
`texture`, dimensions or usage independently. A caller can create an R32Float
texture, assign `format = Rgba8Unorm`, and receive the original R32Float cached
view from a same-format request for Rgba8Unorm. All three extracted selectors
still trust the stored label before inspecting the native resource
(`texture_2d/views.rs:15-21`, `texture_3d/views.rs:15-21`,
`texture_cube/views.rs:19-25`). Likewise, a caller can install another
texture's cached view. The typed error does not establish these relationships.

`TextureView::from(&Texture2d)` at `texture_view/mod.rs:296-306` copies those
independent facts into the generic view. The coordinated-state migration and
all of its consumers must therefore finish before this can be cleared. Keep
the explicitly required detached-cache and missing-cache precedence behaviors;
merely forcing both handles to be present would change the current contract.

An additional representable provenance weakness belongs to F2: callers can
pass device B's context to a selector operating on device A's texture. The
selector pushes and polls B's validation scope while native view creation uses
A's error sink. The pinned `wgpu-29.0.4/src/api/texture.rs:93-100` delegates
creation to the texture, and `src/backend/wgpu_core.rs:493-497,2279-2303`
retains and uses that texture's error sink. Thus an invalid alternate request
need not be captured by the supplied context's scope. No current repository
caller crossing devices was observed; this is an existing representable API
weakness, not a newly introduced regression or proof of runtime misuse. The
coordinated-state contract should explicitly settle device provenance.

### F3/D774: binding facts and retained causes await the canonical contract

`ComputePass::record` selects a native view but separately obtains
`actual_format` from `tex.texture`, including all three whole-texture arms and
the generic view arm (`compute/pass/mod.rs:263-352`). Its format check at
`362-400` therefore validates allocation metadata rather than the selected
view's interpretation. The public helper in `bind_group.rs:195-235` repeats
the split authority. This matters for sRGB/linear view interpretations and
for the mismatched aggregate states admitted by F2. Its missing-view result
also remains an unstructured message at `compute/pass/mod.rs:412-425`.

The pipeline retains a native rejection only as `e.to_string()` at
`compute/pipeline/mod.rs:114-117`; `validation_error` stores `Option<String>`
at line 20. `validate_interface` also formats a parser error into another
message at `126-128`. These paths cannot satisfy a source-preserving canonical
error contract until the agreed #774 implementation is consumed.

These functions are explicitly claimed by #774. This review neither changed
them nor recommended an independent competing type. Ownership agreement and
the complete canonical contract must precede implementation and a fresh final
review. No independent deferral or acceptance of these incomplete paths is
implied here.

## Source-backed review receipts

- All three `view_with_format` methods were read in full. A matching format
  clones the cached Arc or returns `MissingView`, before native admission. An
  alternate format without a texture returns `Uninitialized`. Storage-bound
  sRGB requests retain the original request in errors while selecting their
  linear descriptor counterpart. Cube dimension and six-layer selection are
  preserved. Extracting the methods into private modules does not narrow their
  inherent public API visibility.
- `view_error.rs:10-107` retains the concrete requested format and boxed native
  `wgpu::Error`; `source()` returns that owned native error. The Display strings
  remain presentation. The three producer conversions preserve the original
  cause rather than formatting a replacement.
- `texture_2d/mod.rs:70-181`, `texture_3d/mod.rs:47-129`,
  `texture_cube/mod.rs:50-155` and `texture_cube/views.rs:61-115` were checked
  for native creation and cache initialization. The borrowed-device policy is
  applied before pushing the scope; blocking polls occur only when that scope
  exists. Owned-device creation/face failures now retain the concrete native
  source through anyhow context. Browser and borrowed contexts intentionally
  have no synchronous native-validation verdict, as documented.
- `Texture2d::blit` at `texture_2d/mod.rs:492-506` checks both caches before
  creating the blitter/encoder. DistanceField minimum operations at
  `distance_field.rs:29-107` and `110-212` check the input native handle before
  parsing, allocating or recording a copy; initialized algorithm/shader order
  is retained. The constructor upload unwrap replacements are unreachable
  error guards for constructor-created textures, without a new success path.
- `srgb_converter.rs:90-101` uses the shared missing-view classification and
  preserves existing source/destination context through anyhow.
- The removed unsafe Send/Sync implementations are not replaced by assertions
  or wrappers. Native SDK auto traits remain the authority. The existing Wasm
  dependency explicitly enables
  `fragile-send-sync-non-atomic-wasm` (`fabelgeist-gpu/Cargo.toml:33-36`). Actual
  native and Wasm consumer compilation still requires hosted validation.
- `PassParameter`, `GpuResource`, `ResourceDescriptor`, whole-to-generic-view
  conversion, `ComputePass::record` and `create_bind_groups` were inspected at
  their texture admission/selection/use ports. The direct alternate-view
  producer calls use `?`, so conversion into anyhow retains the bespoke source.
  Unchanged direct record/dispatch recipients in texture_ops, mesh GPU,
  stencil, reshape, scatter, reduce, map, transpose, stream, gather, broadcast,
  gradient, dual-contouring execution, divergence, advection, marching cubes,
  scan, matmul and Kernel were inspected at their error-propagation ports.
  They use `?` or a direct generic result return; no additional view-error
  stringification was found there. Kernel's cached path excludes textures and
  samplers and falls back to `ComputePass::record` for them.
- Existing `tests/texture_views.rs` and its JSON fixture were read in full;
  no tests, cases, helpers or assertions were added or edited by this reviewer.
  The workflow change invokes the existing integration target and doctests,
  records its checkout SHA and uploads evidence. Running on a pull-request
  merge checkout remains distinguishable from the PR head via that source
  file; final acceptance must reconcile the actual tested revision.
- `texture-views.md` preserves the lack of browser, hardware-wide,
  performance and calibrated-rendering validation. Its historical observations
  explicitly do not validate the current source.

## Limits and required next review

This was source-only: no tests, builds, lints, project checks, scanners or
hosted jobs were run, and no external review was posted. No source was edited.
The report is the only written artifact. The review did not audit unrelated
numerical kernels, the ZIP/NumPy stack, the sorting migration or #774's owned
implementation as an independent repair task.

This snapshot is an intermediate, unpushed draft. F2/F3 changes will materially
alter the reviewed state and affected consumers. Repeat independent Code Review
on the complete final pushed revision alongside all thirteen audit lenses,
Rule Review, Comment Review and required hosted CI before claiming clearance.
