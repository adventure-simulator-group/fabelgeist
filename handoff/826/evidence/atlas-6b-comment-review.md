# Frozen atlas 6b Comment Review

Reviewed immutable HEAD `6b22364249ae1503fe070dd7b5ce64b8e6e0e25d`, tree
`4868f4a5c6ee9eef4a85ed9994a02790bc61249b`, against parent
`04a68c493d152c5250e17bedd09d33497ab67bf3`. Main remains pinned at
`990430b77a9f63548ffcb642074185f61ddb3322`. All nine changed files were
read completely, including the full 2,103-line renderer. JSON records exact
hashes, canonical Color/Seed/RNG evidence and affected-source coverage limits.

One actionable wording finding remains:

- **6B-C1, P3, render/mod.rs:363.** Constructor docs place all allocation
  failures under device handling, but the constructor now returns actual host
  allocation and atlas-admission errors from CardAtlas.transparent and
  UploadedAtlas.new. Qualify native pipeline/resource creation and queue
  submission as device-handled; distinguish returned host/image admission
  errors. Preserve the limits on native observation and global health.

Both 04 findings are corrected. GPU groom docs explain exact installed head
capture, device/extent checks and absent coverage resetting to bare skin. Head
installation now expressly qualifies native allocation handling.

Atlas recipes admit positive variants and positive signed-raster tile edges.
Construction checks atlas-width multiplication, native row bytes, host element
count and byte/isize extents. Smaller mip layouts inherit admitted bounds.
Explicit image/scratch/column reservations are fallible and retain actual
allocation causes; this does not promise recovery from every runtime or native
memory failure.

The retained representation is RGBA8 unorm: coverage, shade, depth, coverage.
The canonical Color mapping preserves that native format and channel count.
Canonical Seed xor_word preserves the old per-variant seed mix before the same
RNG and draw sequence. Mips retain floor-halving, rounded integer 2x2 means and
the existing stop at either unit axis or twelve total levels. No complete 1x1
chain or altered filtering guarantee is introduced. Public getters/iteration
expose actual immutable stored mips, rather than replaceable width/mip records.

Uploader size admission uses the actual device limit, the paired queue contract
and exact retained mip dimensions/rows. Native creation/write errors retain
device handling; successful return is not observed validation or global health.
UploadedAtlas denotes native view/variant metadata after queue writes, without
an explicit completion promise. The pinned SDK copies host data at write time;
GPU transfer begins on later submission. No wait is issued by this constructor.
The renderer starts with an initialized transparent singleton, preserving a
usable atlas state before any groom is installed.

Candidate construction precedes publication of logical renderer scene state.
Admission rejection retains the old logical scene, without promising rollback
of queued GPU side effects or invalid handles returned through a non-panicking
native error handler. Existing missing target/frame, exact head/capture and
readback limitations remain intact.

The renderer fixture retains exact SHA256
`44c66d149be12de0828289a8461e243c235facb1032230263538ebc471cfda5e`;
all three shader blobs are unchanged. CardMesh's production impl is
byte-identical. The existing mixed fixture preserves its case, helper,
assertions and oracles: only checked constructor operands and getters adapt to
the new API. No new cases/helpers/assertions/doctests were added.

Required producer paths remain incomplete. Host LOD and GPU atlas producers
still call the old paint API; they must consume the canonical settings/seed/mip
contract. Native strand/card/simulation source/count/layout, creator and lock
integration also remain required. The upload-prefix fit prose must be checked
again with native strand admission: the current uploader can force one strand
even if a whole strand exceeds the device limit. This report does not clear
those pending families.

No source edits, project execution/checks/builds/tests/scanners/generators,
runtime, push, hosted CI or external posts were performed. No compilation,
browser/hardware/artistic, performance, global-health, rollback or universal
allocation verdict is supplied. Earlier withdrawn assurances remain withdrawn.
One wording finding and required families remain open. Final integrated
independent reviews and hosted gates remain required on the same pushed revision.
