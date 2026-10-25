# Armor/Metal facade Rule Review

Pinned Root `d7da77bb9d21200170b522b8e6a1f7aa4aef2bab`, tree
`868959fcaec6f7b624c0c462c6bea4a6a70528d4`, compared with main990.
All six requested owning files and their diffs were read completely. The JSON
separates complete source reads from partial recipient/interface evidence;
hashes do not substitute for reading omitted generator bodies.

No new actionable rule finding was demonstrated in the owned facade,
staging/session or error bodies. This is a partial receiving migration: the
current Root graph still has actual missing definitions and incompatible
callers, so there is no compilation or family-completeness verdict.

| Family | Source-backed contract |
| --- | --- |
| Armor context and kernels | facade.rs:35 builds query and both normal-weighting kernels against one actual context/cache before publishing ArmorGpu. Upload/scratch use that context; build_in and nearest_points propagate submission failure before readback. |
| Query counts | facade.rs:145 preserves empty-query precedence, then rejects empty targets. Host lengths narrow through canonical QueryCount and MeshVertexCount; canonical result layout and target selection check native scalar-word addresses before recording. Returned u32 values are the native query-index ABI. |
| Staging identity | staging.rs:14–63 retains one private Arc session through issued tickets and results. prefix:71 checks session identity and slot access before decoding. Borrowing the ticket preserves identity; no integer-only or compatibility ticket path remains. |
| Host decoding | staging.rs:83–105 rejects zero-sized elements, checks count times host element size, rejects short bytes, reserves aligned typed storage and copies exactly the admitted prefix. Zero elements remain valid. status:110 indexes only after proving one complete word. |
| Grouped readback | staging.rs:126–133 records on the Armor batch, propagates submit failure, waits for mapping and only then publishes bytes with their session. Canonical b11 Readback obtains its actual allocation/polling device from that batch. |
| Metal word/byte units | metal/mod.rs:96 uses canonical BufferElementCount and BufferElementStride. The native stride is one u32 word; canonical byte_length widens both u32 leaves before multiplication. Empty allocation retains one physical word. |
| Metal producers | Canonical b11 MetalTextureSize admits sides 32–1024, derives typed texels and checks words-per-texel multiplication. Its bake carries those counts through scratch and InvocationCount, submits successfully before reads and preserves material/cut errors. These producers are required unreceived dependencies. |
| Error ownership | GenerateError::Gpu retains actual anyhow source; StagingError::Decode retains BufferReadError; grouped readback retains ReadbackError. MetalDeviceError adds native context with its actual source, without classifying every failure as device loss or replacing it with display text. |

Required integration obligations remain concrete and actionable:

- Root lacks some canonical counts, stride, MetalTextureSize and query owners.
  Its existing Readback still requires context arguments, while staging already
  consumes the agreed batch-owned API. Receive the complete canonical graph.
- Generator/caller migrations remain incomplete. For example, PartBuild::finish
  and Correspondence::finish still use removed get and infallible status;
  bracer, breastplate, anime and Creator fit readbacks still pass tickets by
  value. Scratch callers retain raw byte/label arguments. Root metal bake still
  accepts u32/String and ignores submit results. Migrate the whole contracts;
  restore no old overload, alias or fallback.
- Creator Armor/Metal initialization caches erase causes into String. Preserve
  the explicitly agreed initialize-once outcome policy while retaining actual
  reusable errors through their UI recipients. The two hair initializers remain
  required too. Pipeline Progress retry documentation applies to that pipeline
  API and imposes no blanket application retry policy. This resolution is also
  appended to the historical b11 inventory without rewriting its finding.
- Armor's manifest declares anyhow and Mesh, but current Root lock records
  lack both direct edges. Canonical final manifests and both lock graphs must
  agree; the b11 Creator lock also needs those edges. No blind lock overwrite.
- Broader physical producers, final all-thirteen waves, three independent
  reviews and required hosted checks remain open on the same final pushed SHA.

The native boundary retains its documented limits. Root GPU enables wgpu's
native std feature, which controls the actual per-thread error-scope key;
SDK cfg(std) is a different alias. wgpu 29.0.4 ErrorSource is Send/Sync only under
cfg(send_sync): native or the supported fragile non-atomic Wasm configuration.
No atomics-Wasm guarantee is inferred. Context submission captures Validation
under owned native policy; borrowed/browser contexts retain device handling.
Successful submission is not completion or later device-health certification.
Foreign buffers and externally supplied context parts retain the actual-device
and paired-queue preconditions of their native APIs.

Item ordering and responsibility splits follow crates/AGENTS.md:90–94. All
owned Rust files are at most 185 raw lines and every complete method is below
100 raw lines; none requires a raised or new debt record. Existing Result and
error distinctions remain narrow. No production unwrap, expect or unsafe was
added in these owning files. The DevicePart body and complete Metal fixture
file are byte-identical to main; the reviewed changes introduce no tests,
cases, helpers, assertions or doctests. Armor's modeling guide exempts this
native/API maintenance from a visual art cycle.

No source edits, project scanner, generator, build, check, lint, test or runtime
execution occurred. Source/SDK inspection and TOML/Git bookkeeping are the
entire evidence. The held Advect checkpoint 3ee is not Armor/Metal ownership
or receipt authority. This bounded Rule Review grants no global clearance.
