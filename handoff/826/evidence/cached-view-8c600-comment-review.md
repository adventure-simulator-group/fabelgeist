# Cached-view command Comment Review

Head: `8c600f727d1eecf21dd241f02739b825edef5000`.
Tree: `1c141f5f1a28cb532cb51b472ba7e8fcc01486ae`.
Parent: `47f37a07687bc47347cedb9e274c2f273f9deae6`.

No actionable comment or documentation mismatch was found in this bounded
source review. Broader integration and final validation remain incomplete.

## Coverage and source identity

Both changed files and six dependencies were freshly read fully. Four unchanged
source/documentation files carry exact parent-review identity, explicitly marked
in the JSON. All three supplied working-proof file hashes match this commit.
The existing Texture2d fixture suffix and whole Blitter module are independently
byte-identical to the parent. No fixture or shader was added or changed.

## Contract review

### Cached-view admission and state

Texture2d.blit checks source cached view, target cached view, source Device and
target Device in that order before any new native operation.
TextureView.clear_raw checks cached view then Device. Texture2d.clear_raw
delegates through as_view, whose existing From conversion clones the exact
cached Arc and device/format/extent metadata. No native allocation is required
by these operations.

No actionable mismatch; MissingView precedes WrongDevice and detached allocation
with retained view remains supported.

### Immediate native observation

Texture2d.blit encloses Blitter::new, encoder creation and Blitter::blit in
context.observe_validation. TextureView.clear_raw encloses encoder creation and
complete render-pass recording/drop. These closures are synchronous and create
resources from the receiving context.device. The existing helper reads native
Validation only under context policy without device.poll or GPU completion wait.

No new completion or rollback guarantee; immediate observation is separate from
deferred encoder acceptance.

### Finalization, submission and causes

Both native paths then call unchanged context.submit. Its own scopes handle
encoder.finish and queue.submit separately. Captured wgpu errors remain
ComputePassError::Native with a direct error source. The queue must belong to
that same logical Device. Browser/borrowed contexts keep device error handling
and Ok is no synchronous native verdict.

No actionable mismatch. Other device error classes, later health and GPU
completion remain outside this return contract.

### Clear math and public behavior

Old2d depth branch and existing View depth branch both choose1 only when RGB are
all zero, otherwise color.x. Every old2d integer max_val branch returned255;
View uses the same f32 per-channel multiply255 then f64 conversion. Native
attachment/load/store semantics remain the same. Public clear/from-color names,
defaults and target publication after raw success remain. Diagnostic labels now
use the shared View clear labels.

Source preservation supports consolidation without a new clear policy or
compatibility path; no executed native/color verdict.

### Blitter and remaining documentation

Whole Blitter source, two WGSL programs, sampler, layouts, pipelines, black-
clear load and triangle-strip draw remain byte-identical. Full changed file
prose and existing image packing/roundtrip rationale were read with actual Image
implementation. Cached view removal/accessor and policy/error docs remain exact
carried parent source.

No actionable current source-comment mismatch in this bounded family.

### Existing fixtures

Texture2d existing cfg(test) suffix was fully read and independently compared
byte for byte with parent; hash matches the supplied working proof. No fixture
section exists in TextureView. This delta adds no
test/case/helper/assertion/doctest.

Source/provenance only. No test or project check was run.

## Limits and dependencies

- This independent Comment Review covers the exact two-file cached-view command
  delta and its actual view/conversion/native-policy/error/Blitter dependencies.
  No actionable Comment Review finding was identified.

- Eight files were freshly read fully. Four additional unchanged source/doc
  files carry exact identity from the preceding full Cube review with its
  bounded limits; they are not described as fresh reads.

- Native immediate creation/recording gaps are structurally addressed under the
  existing policy. Source review does not prove compiled acceptance,
  native/browser runtime behavior, rendering quality or global device health.

- No new poll/wait is added by these command observation closures; texture
  construction and alternate-view operations retain their separate pre-existing
  wait policy.

- No source edits, local project
  checks/build/lint/test/scanner/generator/runtime, added
  tests/cases/helpers/assertions/doctests, push, hosted CI or external post
  performed.

- Other canonical receiving, extraction and producer/caller/error/cache/lock
  families remain required. Root must complete all13 receiving waves,
  independent final Code/Rule/Comment reviews and required hosted checks on one
  final pushed SHA. No global/family/final clearance.

