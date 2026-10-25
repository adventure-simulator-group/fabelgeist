# Independent Rule Review: dispatch successor

Pinned revision `51b21447649f0aa53be19a20cdfc066863617deb`, tree
`114c758d7049d1af96fc1d257a8be52bd81ea56e`, compared with
`65e8aebedb6490bc2945eac6d48479ea6a2cd10a`. Actual parent is
`cf7646a96ac5ba89696eac925f14593e0b13c884`. Main remains pinned at
`990430b77a9f63548ffcb642074185f61ddb3322`.

Both prior implementation/rule findings are fixed. One P3 documentation
contract remains at this pin. Native cached receiver and producer integration
remain required; no compilation, combined family or final clearance is granted.

## Finding

**P3 51B-GUARD-DOC-01 — describe the accepted guarded geometry route.**
`pass/grid.rs:54–57` says a folded-index kernel must admit through
WorkgroupShape::covering_folded_x. The explicitly accepted peer PrefixSum already
counts workgroups and uses a source-proven uniform block guard before lane
address products/barriers. It consumes typed grid geometry directly. Requiring
the invocation projection would conflate units or introduce an unused public
API. Qualify the geometry-only contract to allow either the checked shape
projection or a proven overflow-safe shader guard, while retaining receiving
Device admission. This follows the root requirement to synchronize documented
contracts with implementation. The report uses pinned51 bytes; a subsequent
working-tree documentation change is not credited here.

## Verified repairs and traces

| Family | Source-backed evidence and limit |
| --- | --- |
| Empty-first projection | shape.rs:86–89 derives the grid and returns Empty before testing nonlinearity at 90. Admitted shape [4,2,1] with zero invocations again gives [1,0,1]; live work still rejects that nonlinear shape. Live padded span checks and ordinary covering behavior are unchanged. |
| Owning result | grid.rs:8 defines WorkgroupGridResult and validate_device:74 uses it. This satisfies crates/AGENTS.md:118–122 while shape's explicit standard results preserve distinct shape/folded errors. The error now correctly names the receiving device. |
| Units and operations | InvocationCount, WorkgroupCount, nonzero Shape and geometric Grid remain distinct canonical owners. Ceiling division is unchanged; live folded projection checks the entire padded u64 span against 2^32 addresses. Native per-axis receiving limits are separate from the named 65,535 packing policy. No extra public group-count projection remains. |
| Actual static/cached routes | Lower ComputePass::record checks actual context Device before pipeline/resource/commands, and standalone dispatch uses that route and canonical context submission. Raw Grid::record still requires caller admission. Root's current KernelBatch has only its intentional empty guard before pipeline/FastPath preparation; agreed #780 device validation belongs immediately after that guard and remains required. No competing root repair was made. |
| Peer scan proof | Exact peer95 PrefixSum::record_level propagates canonical counts and ScanBlockCount::workgroups returns typed Grid geometry. WGSL computes a guarded group ordinal, checks block against ceil(count/256) uniformly, then forms block*256+local_id.x and reaches barriers. Valid block/lane addresses fit u32 even at maximum input count. Padded groups and sentinel results return uniformly before writes, sums access or barriers. That source supports withdrawing the unused extra public method. This is peer source proof, not evidence that root's old scan has been integrated. |

The complete immutable peer production PrefixSum body and WGSL generator were
read at `95f9d31b7d9fbb56aa8330c72fffb46fb8e0c0be`, together with the actual
folded-index helper. The helper rejects overflow before forming its result;
block_count is workgroup-uniform and at most 16,777,216. For a valid group,
block*256 plus a lane through 255 is at most u32::MAX. The guard also protects
sums indexing, and its uniform branch precedes every workgroup barrier.

The previous65 report's requirement for a new public group-count projection was
too strong before this complete guard reconciliation. That requirement is
withdrawn. It is replaced by consumption of the agreed actual source contract,
not by a claim of integrated scan or cached receiver clearance. The previous
geometry-documentation assurance likewise needs the qualification above.

## Coverage and limits

All five current dispatch files and both exact deltas were re-read in full.
Three are unchanged from65 and two changed. One hundred and seven declared
prior source identities match current bytes, with original whole/slice coverage
retained rather than unread whole-body credit. Exact receiver, guard, feature,
SDK and governing rule hashes and thirteen semantic relationships are in JSON.

Tests/native-cases and the compile-fail snippet remain identical to95, pre-repair
9a91 and original #779's October 5 source. No case, oracle, helper, assertion or
doctest was authored or executed. Grid107, shape141 and lower pass81 lines remain
below module ceilings; changed methods remain below100. Item order, type
ownership, alias/derive/error roles and actual native adapter preconditions are
coherent. No ceiling is raised; hosted baseline reductions remain pending.

No source edit, project execution/scanner/check/build/test/runtime/generator,
push or CI was performed. Only read-only source/Git/hash accounting and ignored
report writing occurred. Normal native submission and pure admission results
retain their documented limits; no GPU completion/global health assurance is
invented. Cached/static parity, canonical caller/native layout/Sim source,
creator/lock integration and inherited rule repairs remain required. Full audit
waves, all three independent reviews and hosted checks must pass on the same
final pushed revision before clearance.
