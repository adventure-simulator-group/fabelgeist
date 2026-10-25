# Lower d449 Comment Review

Three wording findings remain at the frozen checkpoint. The reviewed mapping,
validation-policy, staged-prefix and source-preserving error explanations
otherwise match their actual paths within the stated ownership preconditions.
This does not clear incomplete peer integration or establish compilation.

## Exact identity and coverage

- HEAD: `d4491e9de215ba1b607fd2989cbfd5e3cbbdd21d`.
- Tree: `c770dc8e66ecac829c48bc443fa8523c71bf330e`.
- Parent: `1f7ecbe13f6f1cff79520e8ad4bdd9c460704bfd`.
- Main: `990430b77a9f63548ffcb642074185f61ddb3322`.
- Tracked worktree remained clean.

All seventeen receiving-delta files were read completely and independently
fingerprinted in the JSON report. Relevant unchanged consumer reads and carried
evidence are distinguished there. The GPU buffer guide was freshly read in full;
native Blitter, existing Metal bake and application caches were read in targeted
excerpts alongside their earlier exact-source coverage. This is a checkpoint
Comment Review, not a fresh audit of all 210 incoming paths.

## D449-C1: Document queue-writing provenance — P2

Locations: Buffer `write` at line 45, `write_at` at line 51 and composed
`from_upload` at line 71; canonical context contract can be owned by
`WgpuContext::from_parts` at line 52.

The new observer scopes `context.device`, while pinned SDK29 queue writes report
errors through the queue's own error sink. Publicly constructible context and
buffer handles cannot prove their logical-device relationship. Read and submit
already state this necessary precondition; writes do not.

Document that the destination belongs to `context.device`, and `context.queue`
belongs to that same logical device. A canonical context-pair contract may be
linked from the write APIs. Do not imply a runtime provenance query or brand
check exists. Borrowed/browser paths still have no synchronous validation verdict.

`Buffer::new` itself uses only the device. Do not invent a paired-queue requirement
for its actual allocation observation. Describe allocation origin and policy:
creation `Ok` does not certify an arbitrary context queue pairing. `from_upload`
does use the queue and therefore needs the complete write precondition.

## D449-C2: Remove unsupported consumer rationale — P2

At `globals/wgpu_context/initialization.rs:72`, the new comment says Texture3d
presentation renders into this sRGB format. Tracked source contains no read or
use of `context.blitter`; it is declared and initialized only. The actual
Texture2d blit path constructs its own Blitter. An optional public blitter
configured for a format does not establish that named presentation behavior.

Remove or qualify this rationale to describe actual supported policy. This is
not a request to invent a consumer or accept a format behavior change.

## D449-C3: Correct host upload lifetime — P2

Locations: `buffer/upload.rs:13` and `fabelgeist-gpu/README.md:3-4`.

Both say native host bytes are retained through queue submission. The actual
BufferUpload is consumed by `write_buffer`; SDK29 `api/queue.rs:128-139` says
data is immediately copied into staging and may be discarded after that call.
The queued GPU transfer begins on a later submit.

Say bytes are borrowed through the write call and distinguish the host copy from
queued execution. Do not imply either retained source lifetime until submit or
GPU completion on return. This is retained prose at a now-touched boundary and
its directly affected guide.

## Verified contracts without additional owned wording findings

Buffer read now documents logical complete-element output, zero/ZST admission,
physical-padding exclusion, exact creating device/paired queue, producer-submit
handling and external unmapped/exclusive mapping ownership. It distinguishes
native blocking progress from browser callback completion and does not certify
later writes or global device health.

Typed host allocation precedes the mapped view. The AnyBitPattern, destination
alignment and initialize-before-set_len safety explanation matches the copy.
Fallible reservation and actual host address/allocation sources replace the old
capacity panic path.

MappingRequest owns callback failure/cancellation cleanup. It stores no SDK scope
guard across await. The successful view is dropped before release. Native Drop
uses a Validation-only cleanup scope, including borrowed contexts, to avoid
replacing a primary failure with a native NotMapped cleanup error. That exception
is explicit in context policy docs, performs no poll and supplies no operation
verdict. The source does not promise capture of every error class or universal
runtime safety. Caller-owned overlapping mapping state remains excluded.

The synchronous observer closes its current-thread scope before returning and
does not poll. Read completion separately uses explicit native Wait; browser
completion awaits callbacks. Zero logical length returns empty only after host
element admission. Native copy capability/alignment remains admission, rather
than a promise to silently pad arbitrary copies.

BufferReadError, GenerateError GPU anyhow source, separate Readback/Staging
sources and focused MetalDeviceError preserve actual causes without diagnosing
every failure as irrecoverable device loss. BrowserContextError keeps its original
opaque Canvas JsValue for inspection; it correctly supplies no invented standard
Error source. The pinned wasm-bindgen 0.2.126 Send/Sync implementations apply only
without target atomics. No atomic-Wasm or browser compilation verdict is given.

Armor cache docs no longer promise exactly-once construction or arbitrary shared
resource safety. Metal constructor prose describes obtaining six handles and
reusing them. Armor scratch retains byte lengths with a minimum four-byte word;
Metal scratch retains native-word counts with a minimum one word.

Staging tickets retain session identity. Exact-count prefix decoding rejects
foreign tickets, ZSTs and short bytes, including zero-count behavior without
synthesizing missing data. Status admits one u32 before indexing. Allocation
initialization is overwritten by the admitted full prefix, so the missing-byte
claim matches the source. These statements apply to the repaired receiving API;
old consumers still require migration.

Initialization now retains native adapter/device/surface errors and observes
available immediate native validation separately from completion. Existing inline
Buffer/context test bodies/oracles and the preexisting upload doctest remain
unchanged. No cases, helpers or assertions were introduced by this repair.

## Exact peer dependencies and limits

Readback `42d781f3dd02934894f090ca7710c5397522b2c1` is the independently verified
retained-device `record(batch, sources)`/`read()` API consumed by staging.
Count/stride/query receiving contracts were inspected at
`87e28123e55479e32783c6a7f1559151277f6589`. Later material/count source at
`5b03044ce073b63143c4f45001fe8f4309c41023` supplies the supported 32–1024 tile
admission and native word/count bridge. These exact-source inspections are not
acceptance or integration of the complete later owner export.

#780's full producer/error/cache/caller/lock work remains required. Current Metal
bake, body/part ticket consumers and application caches still have old contracts
or stringify errors. Local receiving cause preservation cannot establish an
end-to-end source chain until those owners finish integration. The incoming
210-path/thirteen-lens inventory remains incomplete and was not delegated here.

Pinned SDK29 scope, queue-write, mapping, native cleanup and polling evidence is
identified in JSON; fresh and carried reads are explicit. No source edits, local
checks/builds/tests/scanners/generators/runtime, new fixtures/helpers/assertions,
push, CI requests or external posts occurred. No compilation, hosted pass,
hardware/browser/performance, global completion, rollback or whole-PR clearance
is awarded. Final integrated reviews and checks must use the same pushed SHA.
