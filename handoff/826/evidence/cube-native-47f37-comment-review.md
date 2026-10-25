# Cube native family Comment Review

Head: `47f37a07687bc47347cedb9e274c2f273f9deae6`.
Parent: `9b9d422e011ac23ee1e453483eb4356d0dd14b9f`.
Main: `990430b77a9f63548ffcb642074185f61ddb3322`.

No actionable comment or documentation mismatch was found in this bounded
source review. This is not compiled, native, global or final PR clearance.

## Coverage and source identity

All six changed/new files and 15 affected dependency/documentation files were
read fully at the exact head. Per-file SHA256 and Git blobs are in the JSON.
All six working-source-proof hashes match the committed bytes. The two format
files match canonical e216 exactly. All three authored raw WGSL strings match
the parent byte for byte. No fixture, case, assertion or doctest was added.

## Contract review

### Immediate construction/recording

shader.rs:record_shader calls context.observe_validation around shader, uniform,
layout, pipeline, encoder and completed pass recording. Blit does likewise
around record_blit; direct copy observes encoder creation and copy recording.
Helper reads only Validation under native policy and contains no device.poll.

No actionable comment mismatch. Separate submit scopes remain necessary for
deferred encoder validation.

### Finalization/submission

Every successful observer path calls existing context.submit. Its logical-
Device/paired-Queue preconditions, per-phase Validation scopes, no-completion-
wait statement and browser/borrowed limitations remain explicit. Native errors
are wrapped in ComputePassError::Native with original wgpu::Error exposed by
source.

No claim of GPU completion, rollback, global health or universal native success.

### Format/error ordering

Raw shader methods obtain view_2d_array first: absent allocation then wrong
Device then native view observation. Only after that do they request exact
effective storage spelling. That one effective format also enters native storage
layout. Unsupported spelling returns StorageFormatError retaining exact format;
no fallback. Earlier view/native observation error can precede unsupported-
spelling error.

Precise bounded source contract; no claim unsupported format always wins before
every native view operation.

### Blit/copy admission

set_side_texture checks destination allocation and Device before direct-copy
eligibility, then source Device. Fallback blit selects source view before target
face view. Native physical usage and format support remain SDK validation
responsibilities. Layout/group records retain exact native association;
ComputeBindGroup has only reflected index/group and a compute-pass binding
adapter.

Preserved admission priority and responsibility ownership; no duplicated generic
compute layout authority.

### Missing/cache states

Cached-format whole views retain cache-first behavior; raw Cube render requires
retained allocation through view_2d_array. Face views retain native Device
provenance and requested/effective-format behavior. Cache removal and allocation
removal remain distinct.

No new absence-state claim, cache repair or independent format authority.

### Programs/uniforms/dispatch

All three authored raw WGSL strings match parent exactly. Native face uniform
timef32/faceu32/twozero remains16bytes; all-face uniform fourf32 remains16bytes.
Workgroups remain ceil(size/8) by ceil(size/8), depth1 or six CubeFace values.
Blit filter/sampler/REPLACE/Load/Store/triangle policy unchanged.

Source identity and descriptor read only, not executed behavior or visual
validation.

### Public API and fixtures

Public TextureCube methods remain inherent methods and consuming wrappers return
target after raw method success. Methods moved into responsibility modules
without forwarding compatibility APIs. Six changed paths contain no test
module/cases/assertions/doctests; existing shader templates are byte-preserved.

No new fixture or runtime verdict. Broader fallible storage-format caller
migration remains required.

## Limits and required dependencies

- This is independent source-only Comment Review of this exact six-path Cube
  family and materially affected dependency prose. No actionable comment finding
  was identified in this bounded scope.

- The formerly known immediate Cube native-construction/recording gap is
  addressed structurally under existing policy, but no native execution or
  hosted result is claimed.

- Existing view creation still uses its pre-existing ValidationScope wait
  policy. The new immediate observer adds no device poll; this is not a claim
  the entire public Cube operation never waits.

- Borrowed contexts/browser builds remain device-handled. A normal return there
  is no synchronous validation verdict; other error classes remain device-
  configured, and submitted work may outlive the call.

- Full canonical storage/projection caller integration, held extraction tranche,
  other producer/error/cache/native families and previous SP-C1/C2/C3 findings
  outside this six-path delta remain required; this Cube verdict does not clear
  them.

- No source edits, local project
  execution/check/build/lint/test/scanner/generator/runtime,
  test/case/helper/assertion/doctest additions, push, hosted CI or external post
  performed.

- Root still owns complete all13 discovery/receiving waves, final independent
  Code/Rule/Comment reviews and required hosted checks on the same final pushed
  revision; no global/family/compiled/final PR clearance.

