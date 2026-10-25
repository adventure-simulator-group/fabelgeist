# Armor and metal facade inventory

Pinned Root `d7da77bb9d21200170b522b8e6a1f7aa4aef2bab`, tree
`868959fcaec6f7b624c0c462c6bea4a6a70528d4`. The six Root-owned
facade, staging, module, error and manifest files were freshly read in full,
with distinct admission, submission, mapping and decoding bodies traced.
All thirteen bounded lens cells are complete before the proposed edit.
Connected numerical producer acceptance remains explicitly incomplete.

## Agreed Root repair

**AM-DOC-1:** Document existing native buffer preconditions on
`ArmorGpu::read`, `read_async`, `read_staged` and `read_staged_async`.
Every supplied buffer must belong to the context's device, and its queue
must be paired with that device. Direct reads reference `Buffer::read`'s
mapping ownership contract. Grouped sources must be available for their
actual GPU copies under existing mapping ownership.

This clarifies the existing contract. Add no validator, provenance metadata,
new type, exclusive-read guarantee or alias guarantee. Preserve producing
submission error handling and current progress policies. The acceptance
criterion is a documentation-only change with every signature, body, native
command, shader and fixture byte unchanged.

## Receiving source conclusions

No additional Root implementation defect was identified in the examined
facades. Root preserves actual native/parser/poll/map causes through
`GenerateError::Gpu`, `Readback`/`Staging` variants and `MetalDeviceError`.
Root build/query/grouped read paths handle submission before consuming results.
Staged tickets own a session identity and private index; foreign tickets,
zero-sized elements, overflowing or short prefixes fail. Zero prefixes stay
valid and aligned typed destinations do not fabricate missing bytes.

Native buffer construction/writes use the canonical lower owner. Direct
readback retains explicit native waiting, browser async callbacks and mapping
cleanup. SDK source distinguishes deferred copy validation on a fresh encoder
from immediate ended-encoder failures and device-class creation errors; no
unsupported Cube-style Validation timing repair is proposed for Buffer.read.
The six Root owner files are byte-identical to f0 and to immutable b11/3ee
receiving context, independently checked.

## Required dependencies

The current local graph does not yet compose. Required peer-owned migrations
include grouped Readback signatures; session-bearing non-Copy tickets and
fallible status/prefix callers; typed query/count/stride/Grid producers; metal
size/material/engraving/map/bake/export/preview APIs; four Creator error caches;
and handling every producing submit result before dependent reads/publication.
Concrete current examples and canonical 3ee source relationships are recorded
in the JSON. They are required work, not deferred acceptance.

The allocation/batch/read census contains 326 edges across 60 paths. All
locations and per-file read boundaries are recorded. Unreceived broad
numerical producer bodies are not semantically accepted from those hits; this
review does not duplicate their owner's audit or authorize overwrites. Final
canonical composition, hosted project checks and independent reviews on the
same final pushed revision remain required.

No source was edited and no project checks, scanners, generators, tests,
builds or runtime were executed. No tests or assertions were added. This is
not whole-family, compilation, all-lens global or final PR clearance.
