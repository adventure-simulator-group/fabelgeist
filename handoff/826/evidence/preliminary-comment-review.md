# Preliminary independent Comment Review

Status: **findings present; no final clearance**.

## Snapshot and constraints

- HEAD: `71a2d1ffc6203c251f87a4c4e759670058d620b3`.
- Base: `3a50b708381ad00c116445c2c999a49c3943088b` (`origin/main`).
- Working snapshot: the 12 tracked/untracked SHA-256 fingerprints recorded in
  `target/pr826-audit/preliminary-review-snapshot.json` at dispatch. The parent
  confirmed that this source remained frozen during review.
- Applied `/home/brunosegovia/.codex/skills/comment-review/SKILL.md`, root
  `AGENTS.md`, and `crates/AGENTS.md`.
- Source review only. No source edits, tests, cases, helpers, assertions, builds,
  project checks, runtime probes, push, hosted CI, or external posts were made.
- Cache/state repairs F2/F3 and the agreed #774 shared consumer/error contract
  remain pending. This verdict cannot establish their final documentation.

## Findings

### C1: The blocking-validation rationale misstates pinned wgpu polling

Locations:

- `crates/fabelgeist-gpu/src/globals/wgpu_context/mod.rs:18-29`, especially
  lines 21-25; same claim at lines 127-130 and 283-288.
- `crates/fabelgeist-gpu/src/data/gpu/texture/texture_2d/mod.rs:137-140`.
- Related unchanged claim in
  `crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/mod.rs:45-47`.

The comments say an owned device drains instantly, while continuous submissions
to a borrowed device prevent `poll(wait_indefinitely)` returning and necessarily
lead to watchdog removal. Ownership does not prove an empty GPU queue. More
concretely, the pinned `wgpu`/`wgpu-types` version is 29.0.4 (`Cargo.lock:12028`
and `12181`), whose `PollType::Wait` documentation says an omitted submission
index waits for the most recent submission **at the time of the poll**, and
explicitly allows later submissions before the call returns. Its implementation
loads `last_successful_submission_index` once and waits on that target:

- Cargo source `wgpu-types-29.0.4/src/lib.rs:229-264`.
- Cargo source `wgpu-core-29.0.4/src/device/resource.rs:837-887`.

Thus the blanket “never returns while something else keeps submitting” claim
contradicts the dependency contract. Long waits and device loss can occur, but
this source establishes neither instant owned completion nor the stated
continuously moving target. Preserve the project's conservative borrowed-device
policy without presenting that explanation as native behavior.

Suggested `blocking_validation` wording:

> Whether shader, pipeline and texture construction synchronously waits for
> native validation. Native contexts created here enable this; borrowed
> contexts disable it to avoid blocking their caller on GPU work. A blocking
> poll waits for the latest submission observed when polling begins and has no
> timeout. When disabled, this code installs no validation scope, so native
> errors follow the device's configured error handling. Browser builds skip
> synchronous readback. Shader source is separately checked by Naga.

For `submitted_work_done`, retain the useful explicit submission-index
description and remove the incorrect contrast with `wait_indefinitely`. Replace
the constructor rationale with “Borrowed contexts avoid synchronous validation
readback; see `WgpuContext::blocking_validation`.” Phrase the existing borrowed
context test comment as the policy it actually asserts, without promising that
the opposite always hangs. Coordinate any edit to #774's claimed pipeline
function/comment rather than independently changing that shared function.

### C2: Readback documentation is attached to the upload method

Location:
`crates/fabelgeist-gpu/src/data/gpu/texture/texture_2d/mod.rs:383-390`.

“The texture's pixels, read back as an image” and the narrowing/channel expansion
description document `Texture2d::write`, which uploads provided bytes with
`queue.write_texture` and returns `Result<()>`. The described implementation is
instead the unchanged
`crates/fabelgeist-gpu/src/data/gpu/texture/texture_2d/image.rs:97-150`
`read_image` method, currently without this documentation. This defect already
exists on pinned main, but it is in a changed file and directly concerns the
texture/readback contract being reviewed.

Remedy: move this documentation, unchanged except any contract clarification
required by final state changes, onto `read_image`. Document `write` separately
only if its upload contract requires explanation. Do not add tests.

### C3: Error documentation excludes its newly added producer paths

Locations:
`crates/fabelgeist-gpu/src/data/gpu/texture/view_error.rs:1-14`.

The module and enum introduction describe only whole-texture format selection;
`Uninitialized` specifically requires a different-format request and
`MissingView` a same-format request. The dispatched working tree now emits
these variants without any format request:

- `Texture2d::blit`, `texture_2d/mod.rs:493-494`, returns `MissingView`.
- `TextureSrgbConverter::convert`, `srgb_converter.rs:92-101`, returns
  `MissingView` with source/destination context.
- `Texture2d::create_from_image`, `texture_2d/mod.rs:218-225`, uses
  `Uninitialized` for native upload admission.
- `DistanceField::generate_min` and `generate_smooth_min`,
  `fabelgeist-compute/src/distance_field.rs:35,87,117,178`, use
  `Uninitialized` for native copies.

That misleading restriction matters to callers downcasting the public error:
matching a variant no longer establishes a format-selection request or its
branch. The final error owner is still pending #774 agreement, so settle its
semantic scope first. If the current shared variants remain canonical, use:

> Texture resource admission and whole-texture format-view failures.

For the variants:

> The operation requires a native texture, but none is present.

> The operation requires a cached view, but none is present.

Keep format-selection-specific branch priority in the method/guide contract.
If the agreed canonical design instead separates these producer families,
update the documentation to match that design and remove the inappropriate
producers; wording alone must not decide shared ownership.

## Coverage and accepted claims

Read comments and public examples in every changed file, including the
unchanged comments inside those files:

- `.github/workflows/rust-quality.yml`: step names and evidence paths match
  their commands. The texture-view integration target runs serially; doctests
  run separately. No hardware/browser/performance claim is made by the YAML.
- `fabelgeist-gpu/README.md`: new guide link resolves to `texture-views.md`.
- `texture/mod.rs`: new default edge-length comment matches its two 2D
  constructor consumers.
- `view_error.rs`: read all enum/variant documentation, constructor rationale,
  error-source implementation, and the `compile_fail` example. The example
  correctly demonstrates that bare `TextureViewResult` does not implicitly
  convert its error into an `anyhow::Result` return.
- `texture_2d/mod.rs`, `texture_3d/mod.rs`, `texture_cube/mod.rs`, and all three
  new `views.rs` modules: read comments with constructors, branching,
  descriptors, validation scopes, native causes, and readback paths. The
  default-view storage usage comment describes the inherited view usage;
  descriptor-side sRGB linearization remains implemented. Cube face and array
  descriptors retain their dimensions/layers. The comment on optional context
  in `face_view_with_format` matches its validation-only use.
- `srgb_converter.rs`: workgroup comment agrees with dispatch dimensions and
  embedded shader declaration; missing-view error contexts remain distinct.
- `fabelgeist-compute/src/distance_field.rs`: temporary-copy comments agree
  with the sampled-input/storage-output pass construction.
- `globals/wgpu_context/mod.rs`: reviewed initialization, borrowed/owned
  policy, submission wait method, and existing test documentation. See C1.
- `tests/texture_views.rs` and `tests/fixtures/texture_views.json`: reviewed
  public-test descriptions, context-message identity explanation, fixture
  method/prefix relationships, and actual existing assertions by source.
- `texture-views.md`: example types/re-exports agree with current APIs; cached
  branch priority, typed native causes and source-less admission errors match
  the dispatched implementation. Native error collection is gated by the
  context flag; browser builds skip it. The ComputePass `?` conversions retain
  alternate-view native errors. Existing tests cover the listed cases by source.
  The guide explicitly disclaims current validation from historical software
  Vulkan observations and gives no unsupported rendering/performance verdict.

Affected unchanged source inspected to establish truth: `texture_2d/image.rs`,
ComputePass texture branches (`compute/pass/mod.rs:250-344`), shader and pipeline
scope handling, `Cargo.lock`, and installed pinned wgpu 29.0.4 polling docs/core
implementation. I did not validate historical Vulkan evidence, execute the
Markdown example, compile rustdoc, or establish hosted-check results. Minor
80-column wrapping at `texture-views.md:54` remains mechanical cleanup, not an
additional substantive finding.

Final independent Comment Review must repeat after the pending connected
repairs on the exact pushed revision used for all other reviews and checks.
