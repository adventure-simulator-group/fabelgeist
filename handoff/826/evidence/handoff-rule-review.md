# Independent handoff Rule Review of PR #826

The handoff delta is exactly the claimed wording-preserving line wrap. No
additional semantic/code rule gap was found in the owned receiving graph.
One small owned documentation rule gap remains: two other prose lines still
need wrapping. Required #780/#774 families and hosted acceptance remain
incomplete, so this report gives no whole-PR or final clearance.

## Pin and independently verified identity

- Reviewed HEAD: `7e69c836d168c804f5c7260b6c0bc3747f66337c`.
- Base: `990430b77a9f63548ffcb642074185f61ddb3322`.
- Prior source checkpoint: `0bc4fd52bc376a22c6f7e73bc7f9ea427f662f17`.
- All 73 main-to-HEAD paths matched their exact HEAD blobs; working tree clean.
- Every `ledger.json.handoff_wave.files` fingerprint matched, with the correct
  handoff revision.
- Independent digest of sorted changed-path/SHA-256 records:
  `a38ae78c9fdb91bcf76ed72cb06e271a00695fc0ed9ed7164a6623840a9f4c80`.
- The only prior-to-handoff changed path is GPU `texture-views.md`. Its actual
  diff splits one prose line into two; tokenized wordstreams are identical.
  All source/tests/configuration remain byte-identical to the prior snapshot.

The Rule Review skill/reference and root/crates AGENTS rules remain applicable
and unchanged. Current source anchors and their producer/recipient relationships
were revalidated; the ledger's prior verdict was not substituted for evidence.
No source edits, tests, builds, project checks, scanners, generators, runtime,
push, CI or external posts were performed. Only this ignored report was
written. No tests/cases/helpers/assertions were added.

## H1 — P3: two remaining guide prose lines exceed the required wrap

Locations: `crates/fabelgeist-gpu/texture-views.md:59` and `:66`.

The first combines the cached-view format explanation with the whole-cube
dimension explanation; the second combines error conversion prose and the
ComputeBindGroup sentence. Both are ordinary prose, not a code block, table or
link that needs preservation. They still exceed the 80-column requirement in
root `AGENTS.md:55`. I missed these in the prior source review; the current
guide reread exposed them.

Correction and acceptance: wrap those existing lines without changing their
wordstream. No semantic/API/test change is needed. This small issue was not
introduced by the handoff's one-line delta, but remains in the owned guide and
prevents calling the complete rule review clear at this snapshot.

## Current source-backed contract receipts

| Responsibility | Owner → admission → retained state → recipient/boundary |
| --- | --- |
| Texture/cache/format states | Whole constructors admit controlled native/cache/metadata/device records. Clear-only transitions retain all supported missing/detached states. Matching requests preserve cache-first Arc sharing; alternates require allocation and actual SDK device identity. Generic/face/array views and direct copy/read/write/clear/blit/converter recipients consume the same controlled facts. SelectedTextureView carries effective format/dimension/device into parameter selection and static assembly. |
| Uniform shape and native words | The sole UniformMember producer in reflection/uniforms.rs admits direct or flat supported shapes and retains name/native offsets/span. UniformValueShape is defined once in shader.rs and consumed by both packers before writing. Existing From<i32>→Unsigned(value as u32) and From<u32> preserve their native 32-bit word; IntegerWord accepts Sint32/Uint32 without a competing signed representation. Supported numeric/matrix bytes and static-zero versus strict-fast policies remain intact. |
| Unsupported interface states | Multiple blocks per shared group, nested/array/wide/non-square unsupported host values and used bound resource payloads absent from PassParameter return canonical typed pipeline admission errors before committing misleading state. Used Immediate is checked after compute GlobalUse union and before ResourceBinding exclusion; it has an explicit typed unsupported payload without fictitious group/binding values. Invalid compute TaskPayload/RayPayload/IncomingRayPayload use is rejected by pinned Naga validation. Private/workgroup and unused globals remain skipped appropriately. |
| Actual native resource roles | Validated usage union across compute entry points determines retained resources; sampling_set union determines filtering in the one shared layout. Sampler comparison, sampled/depth multisampling, scalar sample kinds and storage ReadOnly/WriteOnly/ReadWrite/Atomic are preserved. Load-only floats avoid false filtering; provider feature/format/actual-resource incompatibility remains native rejection. Sparse bind-group indices are preserved. |
| Error and pipeline lifecycle | ShaderError retains actual parse/validation/translation/native sources. Immutable shader/device/interface records and complete entry-point cache keys prevent stale mutable source identity in the owned provider. Observed Result outcomes remain distinct from device-handled creation, and rejection never becomes a usable cache hit. Compiled state/device admission and generic conversions retain the source chain. Source/cache producers above this layer remain dependencies. |
| Static allocation and fast/batch | One immutable-pipeline assembler serves public/static recording. Logical prefixes reuse Buffer binding admission. Uniform extents are checked before packing, unmapped allocation/upload have balanced native scopes, and native causes remain concrete. Fast missing/wrong Buffer and missing/unsupported Uniform categories use canonical ComputePassError. Arena byte offsets/addition and Prepared narrowing are checked; copy logical bounds, empty batch precedence/counting, folded launch and callback ordering remain. |
| Modules, primitives and derivation | New modules own real reflection, packing, allocation, compilation or arena responsibilities. Named sampler/uniform records reuse canonical shared owners. Raw shader text/native names/ABI offsets/enums remain explicit boundaries; closed roles and failures use enums. Fieldless ArenaError derives preserve the previous messages/source semantics through the existing dependency. No new compatibility API, numerical tolerance, display-prose branch, duplicate owner or unsafe cause/auto-trait adapter was found. |

These receipts were checked against current source and unchanged pinned SDK
definitions, including actual Device identity, conditional native/fragile
non-atomic-Wasm Error source bounds, Naga FunctionInfo usage/sampling and the
native resource role mappings. They do not establish native/Wasm compilation,
atomics-enabled browser compatibility or actual GPU execution.

The complete affected main-diff graph remains accounted for: core GPU owners,
all algorithm dispatch/texture/error ports, BVH/mesh/hair/physics adapters,
existing tests/fixture/workflow and owning documentation. No source relationship
changes follow from this guide's whitespace-only delta. Unrelated numerical
kernels and retained claimant bodies were not independently repaired or granted
clearance.

## Required dependencies and future validation

All #780 producer/count/layout/source-device-cache and mesh caller families
remain required. Current Kernel raw workgroup/count producers, source-only
caches/lock unwraps, primitive-to-grid/byte receiver adapters and delegated
Kernel/module/field comments are not complete merely because the lower native
receiving contracts are sound.

All retained #774 hair/fitting/body/armor/metal/engraving and connected runtime
equipment cause families remain required. The restored claimed hair array
recipients still need agreed WorkgroupGrid integration. The later direct human
core transfer and coordinator reconciliation do not authorize duplicate edits
inside those retained functions. Exact owner source/API/revision integration is
still necessary; none is silently deferred.

Required hosted fmt/lint/native/Wasm/project checks and applicable baseline
reductions after extraction remain pending. No quality registry scope, Cargo
lint inheritance or baseline ceiling was weakened. Apply actual reductions and
do not raise ceilings under `crates/AGENTS.md:167-178`; source inspection does
not prove the numerical quality gate. Existing test population and documented
historical software-GPU/browser/hardware/rendering/performance limits remain.

Correctness: no additional semantic/code rule gap found in the owned graph.
Rule completeness: H1 still needs its prose-only repair. Migration completeness:
incomplete at every required external owner family and unrun validation. Final
all-thirteen coverage, three independent reviews and required hosted CI must
agree on the same final pushed SHA. This handoff checkpoint is not that verdict.
