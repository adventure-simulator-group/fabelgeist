# Independent Rule Review: policy, renderer and buffer prefix

Pinned source: `d9c3521f30f52d3437cfef2850203c9df3192b0e`, tree
`660e61a63abe42f2ff07b50114b34729def12ffe`. Requested comparison:
`73b6e3a2ddf319dec749e3b1ff4a25e99b9c6612`; actual parent is
`8b32a1860079c64073028033b9fc4629a64ef451`. Main remains pinned at
`990430b77a9f63548ffcb642074185f61ddb3322`.

Correctness has one actionable capacity gap and one stale field contract.
Migration completeness remains open. This checkpoint grants no compilation,
combined family, whole PR, hosted check or final pushed revision clearance.

## Findings

- **P2 D9-PREFIX-WRITE-01 — logical capacity is bypassed by `write`.**
  `crates/fabelgeist-gpu/src/data/gpu/buffer/mod.rs:53–55` sends the upload
  directly to the native allocation. An eight-byte COPY_DST allocation can be
  narrowed with `prefix(4)`, then receive eight bytes through `write`; the native
  allocation is large enough, so provider validation cannot enforce the logical
  four-byte extent. `write_at(START, upload8)` correctly rejects this same write
  at lines 68–78. A zero prefix similarly admits nonempty writes through `write`.
  This is a source counterexample, not an executed runtime result. The connected
  unchanged consumer violates the semantic propagation and invariant contracts
  in `crates/AGENTS.md:31–39,49–56`. Make both public queue-write forms honor the
  existing logical capacity operation, preserving the canonical native upload
  adapter and actual error cause. Root already owns this complete family.
- **P3 D9-DRAW-DOC-01 — planned ranges no longer guarantee residency.**
  `crates/fabelgeist-hair/src/render/draws.rs:16` describes `instances` as bounded
  by the resident population. `DrawPlan::new` now deliberately splits the shared
  logical range without a renderer; the actual resident clamp is at
  `render/mod.rs:1938`. Update that field contract to match these two stages,
  under the root requirement to synchronize documentation with implementation.
  The native consumer clamp is present; this finding does not allege an
  unbounded native draw from this changed path.

Existing connected rule gaps remain required #780 work: `lod.rs:81–126` puts
impls before later structs (`crates/AGENTS.md:90–94`), and `lod.rs:150` retains
`counts.last().unwrap()` (`126–127`). Its nonempty construction proof does not
waive the production unwrap ban. Neither was independently edited here.

## Verified source relationships

| Family | Source-backed contract and remaining limit |
| --- | --- |
| Prefix owner/admission | Existing `BufferByteLength` enters `prefix.rs:36–48`; requested bytes must fit both parent logical size and actual native `Buffer.size`. The result retains the same Arc and narrows only returned metadata. Zero and unaligned logical bytes are valid. No allocation, mapping, copy, submission or wait occurs. Public Buffer fields mean this is not a globally sealed invariant. |
| Byte consumers | `binding.rs:28–46` rejects zero and physical overrun, and binds logical bytes. `read.rs:21–51` checks physical size; private `MappingExtent` alone owns mapping alignment, and `Readback` returns logical bytes only. Native mapping/staging/cleanup cause behavior is unchanged. `write_at` honors logical capacity; `write` is the finding above. |
| Policy ownership/admission | Existing math Ratio/Metres and parameter StrandCount flow into typed counts, ordinals, selection records and Representation payloads. Checked nonempty selection retains typed EmptyBands/MissingFamily/StrandsWithoutLevels; absent geometry selects Empty. Historical policy numerical fallback remains distinct from actual rendered finite-input admission. No invented width-scale upper bound or provider extent policy was added. |
| Temporal state | Scene and target replacements reset last view, frames and History::Empty. Entering accumulation without Current history now resets frames before computing weight one; recording the accumulation pass publishes Current(to). Fixed HistorySlot maps the native pair. Moving simulation retains its intended temporal filter. |
| Draw handoff | Logical u32-representable population is split before per-level resident clamp, preserving finer/coarser prefix identity. `handed.min(count)` prevents rounded f32 products from causing subtraction underflow. Actual renderer draw and draw_gpu clamp to resident values. Physical layouts, producer counts and source membership still require #780 integration. |
| Native causes/provider | Bare renderer retains the actual supplied Device/Queue and documented same-logical-device precondition. Device configured handling remains intentional for render native operations. Callback/readback causes and canonical context ownership are unchanged. Native feature std thread-key evidence and feature-specific Wasm/SendSync limitations retain their exact prior source receipts; no new universal platform assurance is granted. |

`lod.rs:35–38` now exports canonical Representation consumed by the renderer.
The previous 73b review missed the missing export; its specific export/API
assurance was incomplete and is withdrawn. Current source, not that prior
verdict, establishes this repair.

The Sample declaration now precedes Shot's impl. Both complete item bodies and
attached test population are byte-preserved. Dataset randomization, ZIP/NumPy,
serialization and image behavior were not independently repaired.

## Coverage, rules and preservation

All eight changed pinned files and complete diffs were read. The entire current
renderer was freshly read at lines 1–2206, including constructor/native setup,
CPU/GPU upload, dynamics, head/cover, targets, invalidation, policy/convergence,
draw/post/submit, headless provider, helpers and existing fixtures. Full current
LOD/policy, frame state, draw and dataset changed responsibilities were read.
The five unchanged buffer read/map/extent/binding/upload owners and consumers
were re-read. Seventy-seven other prior whole-body or precisely scoped receipts
match current source bytes; they retain their declared granularity, not new
unread whole-body credit. JSON records all exact hashes, coverage and thirteen
semantic relationships.

Root/crates governing rules and Rule Review skill/reference hashes match the
previous read receipts. Prefix is a focused module with named typed error
fields and owning result alias; its pure predicate error has no native source
cause to fabricate. Manual Display/std Error fits the GPU crate's existing
lack of thiserror. Item ordering and native adapters are coherent. New prefix,
frame-state and draw modules remain below 500 lines; changed functions are below
100. The oversized renderer shrinks from 2210 to 2206 lines, render shrinks, and
resize's existing 103-line allowance does not grow. No ceiling is raised.
Required hosted baseline reductions remain pending.

Renderer's existing two tests, validate helper and five assertions, dataset's
existing test module, GPU Buffer's test module, five native POD item bodies and
three shader files are exact comparison bytes. No new case, fixture, test
helper, assertion or doctest was authored or executed. No source changes,
project scanners, checks, builds, tests, generators, project runtime, push or CI
were run. Read-only Git/source/hash accounting and ignored report writing were
the only completed operations.

The #780 physical count/layout/strand/card/simulation/atlas producers, Surface
and head-source/cover association, creator/UI/callers, completion publication,
canonical dependencies and lock integration remain required. The known head
column/index/cover gap and stale cover/source relationship remain actionable
coordinated implementation work. These are incomplete paths, not accepted
silent deferrals. Fresh complete audit waves, all three independent reviews and
required hosted checks must agree on the final pushed revision.
