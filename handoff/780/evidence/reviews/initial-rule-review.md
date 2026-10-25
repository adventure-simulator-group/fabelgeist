Independent Rule Review of `e14407cc494615f0b16522fb7aa7672a3589a3c3` is **not clear**. Compared with current main `3a50b708381ad00c116445c2c999a49c3943088b` and #779 `71f9cab8a5f97760e2e61ca4c74876b715f8aebc`. HEAD and clean working state reconfirmed after inspection.

Actionable findings:

1. **[P2] Finish upstream cardinality and width propagation.**  
   `vertex_normals.rs:224-236,305-337`, BVH `gpu/mod.rs:85-91,203-243`, shell `selfcollision.rs:37-38,177-216`, underlayer `workspace.rs:19-20,57-74`, `gap.rs:27,47,74-75`, and `weld.rs:32,58-59` retain raw counts, capacities and widths and convert only at the sorter call. This leaves sorting quantities interchangeable throughout their producers, state and arithmetic. `crates/AGENTS.md` requires semantic fields and operations to carry bespoke owners, beyond a typed recipient signature. Close these connected families through meaningful producer/state/operation APIs; convert collection lengths at admission and project words only at GPU packing. Keep launch `InvocationCount` distinct from sorting cardinality.

2. **[P2] Prevent scratch buffers from invalidating their admitted capacity.**  
   `sort/mod.rs:206-210` exposes `keys`, `values` and `histogram` for replacement while retaining private capacity. A caller can replace `histogram` with the keys buffer or a smaller buffer; `record` checks only capacity and external key/value lengths (`298-310`) before dispatching with the replaced scratch. The admitted record consequently does not guarantee its actual layout. This is a demonstrated representable invalid state, not an observed runtime failure. Make owned scratch fields private and keep allocation/replacement inside the owner. Repository search found no external production consumer requiring these fields.

3. **[P2] Reconcile the canonical launch API instead of keeping the native-grid adapter.**  
   `sort/quantities.rs:61-65` lowers `SortTileCount` to `[u32; 3]`; `sort/mod.rs:331-344` passes raw grids. #779’s canonical `WorkgroupGrid`, `WorkgroupShape` and `InvocationCount` already establish the launch authority. The user explicitly requires consuming that API without temporary primitive adapters. Preserve `SortTileCount` as the sorting layout owner and produce the canonical `WorkgroupGrid` for dispatch. Do not treat tiles as invocation counts or duplicate #779’s grid implementation.

4. **[P2] Carry canonical byte lengths through the copy facade, subject to Adler’s claim.**  
   `sort/mod.rs:351-352` extracts the already typed `needed: BufferByteLength` into `u64` before calling the internal `KernelBatch::copy_buffer` facade (`kernel/mod.rs`). That facade performs domain byte validation before the native encoder call, so extraction happens before the actual boundary. `crates/AGENTS.md` requires reuse and propagation of the existing owner. The coherent repair is the canonical byte API through the facade, not a sorter-specific forwarding helper. **Blocked pending ownership agreement:** `774/records/parallel-ownership/rig_garments-claims.json` explicitly claims `KernelBatch::copy_buffer` and the native-offset portion of `KernelBatch::dispatch`. Do not edit those claimed functions unilaterally.

5. **[P2] Close count arithmetic before constructing sort requests.**  
   `vertex_normals.rs:236,316` multiplies raw triangle counts by three; underlayer `workspace.rs:57-60` narrows collection counts and multiplies face counts by eight, and `gap.rs:47` recomputes that product. `SelfCollision::new:71-74` derives table size through unchecked multiplication/power-of-two arithmetic. The sorter’s infallible `From<u32>` cannot recover truncation or overflow that happened upstream. For example, vertex-normal construction with triangle capacity `1_431_655_766` overflows `triangle_capacity * 3` before allocation. Use checked, role-specific construction/operations and preserve the supported zero/sentinel behavior. This is reached pre-existing debt in the migration’s explicitly requested producers, rather than a new arithmetic regression.

Current-main integration requirement:

- Main’s `sort/mod.rs:150-179,200-215,282-287` has already replaced the sorter’s serial `scan_code()` with canonical `PrefixSum`, retained `PrefixSumScratch`, and `ScanElementCount`. The reviewed PR still contains the old scan implementation. Integration must retain main’s canonical implementation and migrate its histogram cardinality/scratch relationship; choosing the PR’s old scan would regress current main and create competing implementations.
- Main currently computes `RADIX * tiles` in `u32`. At the maximum admitted sorting count, `tiles == 16_777_216` and this product is `2^32`; the connected migration must explicitly handle that representability boundary.
- Main also contains newer kernel compilation and post-submission callback behavior. Preserve these when reconciling #779 rather than replacing current main with its older kernel file.

Source-backed traces inspected:

| Family | Trace and established guarantees | Completeness |
|---|---|---|
| Sort cardinality/layout | `SortItemCount::from` admits all `u32`; sentinel `max(1)`; widened word-byte arithmetic; tile ceiling; scratch allocation; `record` no-op/capacity/byte precedence; native count/tile uniforms | Partial: producer and scratch relationship findings |
| Key schedule | Width clamps to `1..=32`; pass count rounds to whole bytes; iterator emits shifts `0,8,16,24`; ping-pong copies on odd pass count | Internal operations sound by inspection; upstream widths remain primitive |
| BVH | Morton owner fixes three 10-bit axes; codes shader writes codes and identity payloads; sort; hierarchy tie-breaks equal codes by index; refit consumes sorted primitive indices | Sorting migration partial |
| Vertex normals | Triangle-corner shader emits vertex keys and original corner payloads; stable sort; ranges; gather sums contributions in retained triangle order | Sorting migration partial; checked multiplication unresolved |
| Cloth | `Particles::count()` returns raw retained count; hash writes bucket keys and particle identity payloads; sort; range table; collision reads sorted indices | Sorting migration partial |
| Underlayer weld | Workspace collection admission; quantized-position hash and ascending vertex payloads; stable sort; equal-hash links compare exact quantized positions | Sorting migration partial |
| Underlayer gap | Eight slots per face, unused sentinel bucket; derived 17-bit schedule; sort; ranges reject sentinel; query consumes triangle payloads | Sorting migration partial |
| Main prefix sum | `ScanElementCount` → scratch recursion levels → byte/scratch admission → scan/add-offset passes; native WGSL preserves exclusive wrapping sum semantics | Canonical owner identified; integration not yet implemented |
| Dispatch/bytes | #779 typed grid/shape/invocation owners; current main kernel facade; existing `BufferByteLength`; #774 exact function claims | Reconciliation and ownership dependency unresolved |

Read the Rule Review skill and bespoke-type reference, root/crates rules, character-creator rules and relevant armor guidance. Inspected complete production implementations for the sorting paths above, sort test sources, fixture structure/documentation, current main’s prefix-sum production implementation, and #779’s canonical API. No builds, scanners, project checks, CI, source edits, added tests or external posts were performed. Consumer behavioral tests and every individual fixture row were not exhaustively inspected. This is initial discovery review, not final clearance or executed behavioral validation.
