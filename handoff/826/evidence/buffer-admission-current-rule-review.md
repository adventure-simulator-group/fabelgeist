# Independent Buffer admission Rule Review

Pinned 6e56adeab1101743941c6d50eef4340008b910ea; source account and all13
inventory are in buffer-admission-current-pre-repair-all13.json. All accounted
source identities equal cac5. Source-only review; no implementation change or
project check.

## BA-R1 — P2: retain Buffer admission kind and typed request

Locations: crates/fabelgeist-gpu/src/data/gpu/buffer/mod.rs:41-42,
crates/fabelgeist-gpu/src/data/gpu/buffer/mod.rs:69-77.

Newly recognized existing lower-boundary bespoke-error contract gap; not a code
regression or failed range rejection

Zero allocation, offset-add overflow and write beyond logical extent return
anyhow! message roots. The lower API does not retain an inspectable closed kind
or the named canonical byte/offset request record. GpuResource, static uniform
assembly, Armor/Metal/Hair wrappers reuse these errors and preserve the incoming
anyhow source; this is not the final UI boundary.

Callers cannot inspect the three Buffer admission failures as owning semantic
variants or recover their typed request/capacity context. They would need
display prose or an unrelated generic error wrapper to distinguish them. No
observed consumer currently branches on prose, and existing tests verify correct
rejection; this is a semantic API/completeness finding, not demonstrated memory
corruption/native error loss.

Rules: crates/AGENTS.md:21-34 (error classification/state/quantities have
semantic owners); crates/AGENTS.md:42-55 (named typed record fields and existing
leaves); crates/AGENTS.md:57-70 (closed carried roles, one canonical owner);
crates/AGENTS.md:85-86 (stable typed error/code from authoritative boundary;
presentation once).

Correction: Define one focused Buffer-owned structured admission error with
distinct zero-allocation, write-offset-overflow and write-out-of-bounds
variants. Carry existing BufferByteLength for requested/upload/logical byte
quantities and BufferByteOffset for address; named fields, no count/device
wrapper. Keep the existing public anyhow carrier if desired and put the typed
error inside it. Actual allocate_native/write_to wgpu::Error sources remain
unchanged. Do not force all errors into a read-specific owner or invent a native
cause for checked_add returningNone.

## Trace and acceptance

ByteLength/Offset and BufferUpload -> new/write_at local checks ->
structured-local-error MISSING -> existing anyhow carrier -> direct
GpuResource/static uniform operation observation/Armor
GenerateError::Gpu/MetalDeviceError/HairGpuError::Backend. Prefix/read/binding
separate typed failures already exist and do not need duplicate classification.

All three local rejection producers return that owner, with Display rendering
the current useful diagnostics from typed fields.

new/from_upload and write/write_at preserve
zero/empty/range/end-equality/no-write-before-failure behavior.

Actual existing
anyhow/GenerateError::Gpu/MetalDeviceError/HairGpuError::Backend/operation-observation
wrappers preserve the lower type and source chain; no new String conversion or
independent caller repair.

No new tests/cases/helpers/assertions/doctests. Existing fixtures and API
signatures may remain exact; repeat full source review and required hosted gates
on final pushed SHA.

## Deduplication and limits

Not RGP-P1 sampler, RGP-P2 binding Result alias or RGP-P3 stale pipeline import.
Neither historical83 Rule report recorded these three local errors. Their
same-source no-additional-owned-finding conclusion is reopened for this bounded
missed classification; historical reports remain preserved. No duplicate report
of already-typed read/prefix/binding errors.

Source-only focused pre-repair discovery; no implementation/global/compile/final
clearance. All13 records completed for this classification with wider current
graph explicitly open.

All31 source identities match exact Root6e56 and cac5 independently;28 full
fresh file bodies, three bounded complete relevant-function reads. No fresh
whole-module credit for large Stereo/Sim/Scalp files.

159 located allocation/from_upload/write_at callsites are a discovery census
only; no full-body or broader producer correctness credit for all159. Existing
anyhow API need not change, so no independent shared caller repair is implied.

Earlier83 Rule reports source coverage remains preserved; the missed same-source
classification is explicitly reopened. No duplicate read/prefix/binding/native
sampler findings.

Local errors occur before native operations independent of provider mode; native
currentblocking_validation/browser handler guarantees unchanged. Actual
paired-device/queue/mapping alias preconditions remain documented. No new
SendSync/platform execution claim from error classification.

No source edits, tests/cases/helpers/assertions/doctests, local
build/check/lint/scanner/generator/runtime/probes, push/CI or external messages.
Only source/Git/hash/bookkeeping and ignored reports.

Root remains implementation owner. Preserve canonical peer count/stride and all
physical/caller/lock owner dependencies. Complete connected family/all13/three
independent reviews/hosted checks on final pushed SHA; no merge.
