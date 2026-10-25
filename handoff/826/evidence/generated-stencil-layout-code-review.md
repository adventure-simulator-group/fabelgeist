# Generated Stencil layout: independent Code Review

One actionable P2 finding remains in the frozen 50-source snapshot. This is
bounded source review, not compilation or whole-PR clearance.

Snapshot: `generated-working-stencil-layout-snapshot/manifest.json`, working
base `919ba9b9415c06981ff5dc01d21666f49407eb3e`. All 50 source files, 6,162
lines, were freshly read completely. Every frozen hash matches its manifest.
The JSON report records each hash, baseline relationship and dependency read.

## GSL-C1: preserve deliberately projected texture axes

`stencil/neighborhood/layout.rs:126-137` compares output coverage using the
neighborhood's dimension alone. A `Neighbors3D<f32>` program with a 4×4 2D
input and a 4×4×2 3D output now fails with `OutputCoverage` on z.

The native source deliberately declares the input grid as `(width,height,1)`
and projects every 3D sample to `.xy`. The former frozen Stencil has the same
projection. It repeats the input plane along output z; z never becomes an input
texture address. Both resources fit ordinary native limits. Thus the new check
rejects retained successful behavior rather than exposing an unsupported new
feature.

The minimum repair carries the actual fetch projection into output coverage.
Keep admission for every emitted signed coordinate conversion, neighbor
addition/subtraction and boundary expression before that projection: dropping z
from a later fetch does not make overflowing earlier z arithmetic acceptable.
Preserve sampled-axis bounds, Buffer flattening and capacity, fourth-dimensional
prefix limits, empty-output ordering and actual device limits. Root has
acknowledged this trigger and the signed-arithmetic qualification. The reviewer
made no source edit.

## Other reviewed contracts

Ordered signatures retain their original declaration and admitted role together.
Scatter's unsigned value and Stream's final input-loading decision survive role
selection. Both shared and reserved neighborhood type envelopes use safe string
boundaries. Local errors now retain typed function, binding and native-type
payloads; actual regex, coordinate parser, shader, pipeline and device causes
remain distinct and propagate without text classification.

Host declared axes match native grid policy for declared sizes, texture extents
and the named default side. Buffer outputs unflatten the actual flat index for
2D/3D grids. Input capacity uses the same admitted program's reflected array
stride, including padding and custom records. Checked positive axes, widened
products, signed last-index bounds and unsigned flat-divisor admission protect
the generated sample arithmetic, subject to the GSL-C1 repair above.

Texture fetch projections retain their existing lower-dimensional lane behavior.
Added neighborhood admission skips its input/grid checks for empty output; this
does not promise that an empty logical native binding succeeds. The lower owner
still admits pipeline, bindings, missing/device/view/format state and submission.

All fluid source and adapters were read fully. The supplemental current native
literal comparison is 45/50/47, with only local policy messages excluded after
moving them into typed errors. Earlier 46/51/48 counts are historical. Existing
Map fixtures remain exact; Gather, Stream and extracted Stencil fixtures have
only the necessary constructor/getter adapters and formatting. No case, helper
or assertion was added. Existing consumer and binding bodies were also read.

## Required limits and dependencies

Root must complete connected pre-repair inventory and repair GSL-C1, then repeat
successor reviews. Peer canonical module/export/stride/count/Reduce and caller
composition remains required and unreceived; known missing local integration is
not called a newly introduced source defect or accepted as clear.

No production edits/copy, project build, check, lint, test, scanner, generator,
runtime/probe, new fixture, push or CI occurred. No generated program was run.
Exact native source carries are individually labelled; this is not a fresh full
GPU crate review. Full combined integration, all thirteen lenses, independent
reviews and hosted checks on the same final pushed revision remain open.
