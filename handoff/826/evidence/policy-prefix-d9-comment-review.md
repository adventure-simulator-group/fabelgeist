# Frozen policy/prefix d9 Comment Review

Reviewed immutable HEAD `d9c3521f30f52d3437cfef2850203c9df3192b0e`, tree
`660e61a63abe42f2ff07b50114b34729def12ffe`, against parent
`73b6e3a2ddf319dec749e3b1ff4a25e99b9c6612`. Main remains pinned at
`990430b77a9f63548ffcb642074185f61ddb3322`. All eight changed files and
all exact delta hunks were read, including the full 2,206-line current renderer.
JSON records hashes, complete reads and exact unchanged source coverage.

One actionable wording finding remains:

- **D9-C1, P3, render/draws.rs:16.** StrandDraw.instances still claims its
  range is bounded by resident population. DrawPlan now intentionally splits
  the logical prefix without resident access; recording clamps the resulting
  range to each resident level. Say: “Logical u32 instance interval; recording
  clamps it to the resident population.” This matches the repaired boundary.

All three 73b wording findings are resolved. Empty describes draw absence;
card coverage states exact half-open behavior including supplied random 1;
FrameInfo describes primary population, excludes transition/shadow draws and
states the current quad convention. Canonical card/count migration remains
required. Representation is also publicly reexported.

Buffer.prefix checks the requested extent against logical parent and actual
physical allocation. It shares the same native Arc and changes only logical
metadata; zero is admitted. Its no-allocation/copy/map/submit/wait description
matches the code. It makes no native device/usage/mapping admission claim.
Binding admission separately rejects empty logical bytes. MappingExtent remains
the sole aligned covering-range policy for Buffer.read; the same extent reaches
staging, copy, map and mapped view, while the returned host prefix excludes
padding. Fresh complete binding, read and mapping reads support those claims.

Current history distinguishes an absent accumulation result from an actual
history slot. Starting accumulation without current history resets the mean;
the first blend uses weight one. Scene/target/view changes invalidate history,
while movement deliberately retains a current temporal filter. Logical strand
splitting precedes independent resident clamps, preserving the intended boundary
between planned prefix and native draw population.

Current policy selection, quantity and effective finite-input contracts remain
consistent. The unchanged quality/count/catalog/frame-uniform/deformation and
readback sources were independently reconciled by exact byte identity to the
previous complete reads, without claiming fresh full-body coverage of those
files. Constructor provenance, device-handled native errors and provider-limit
qualifications remain intact.

The renderer fixture module retains exact SHA256
`44c66d149be12de0828289a8461e243c235facb1032230263538ebc471cfda5e`.
Dataset fixtures and all three shader blobs are also byte-identical. No new
cases, helpers, assertions or doctests were added.

Whole producer/count/head/native-layout work remains required; prefix admission
does not complete those migrations. Owner #780's constructors, FastPath,
ResourcePipeline, grouped Readback, completion callers, error/cache and lock
integrations remain pending. The all-thirteen inventory and final reviews are
separate from this bounded checkpoint.

No source edits, project execution/checks/builds/tests/scanners/generators,
runtime, new fixtures, push, hosted CI or external posts were performed. No
compiled, browser/hardware/artistic, performance, global-health or rollback
verdict is supplied. Earlier detached-Wasm-view and recorder-scope assurances
remain withdrawn. One wording finding and required families remain open; final
independent reviews and hosted gates must pass on the same final pushed revision.
