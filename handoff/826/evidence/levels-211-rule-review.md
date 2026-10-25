# Independent Rule Review: resident strand layouts

Pinned revision `21118b983b27e00d2cc70f45ed345686e0262cc2`, tree
`5a4c0caac1b7a60f4bb39c12fe1b557c87f0ceb6`, compared with parent
`fc23cece6cf1acb2f0f8b873a7e01f4c103f9b75`. Main remains pinned at
`990430b77a9f63548ffcb642074185f61ddb3322`. Parent confirmed the exact pin after
the adjacent five-path count initially appeared as part of the abbreviated SHA.

Two owned rule blockers remain at this reviewed pin. Strand receiving, bounded
storage bindings and draw projections otherwise have positive source proofs.
Producer, rest/simulation/card/native integration is incomplete; this is not
compilation, full family or final PR clearance. Later repairs are not credited
to the pinned211 source.

## Findings

- **P2 211-NOGROWTH-01 — the GPU module exceeds its unchanged ceiling.**
  `gpu/mod.rs:89,95` adds the layout module and re-export. Source accounting using
  the checker's documented meaningful-line definition gives parent520 and
  current522 production lines; `rust-quality-baseline.toml:714–715` retains a
  ceiling of520. The checker's source at scan.rs:425–451 was read, not executed.
  Under `crates/AGENTS.md:166–178`, resolve growth through coordinated
  compositional import cleanup or canonical module integration, preserving the
  ceiling and peer function bodies. Do not conceal code or raise the allowance.
- **P3 211-ORDER-01 — the new re-export follows module declarations.**
  `gpu/mod.rs:95` inserts an import/re-export after declarations80–93, contrary
  to `crates/AGENTS.md:90–94`. Put the owned export before the first mod without
  independently reorganizing peer producer functions. Existing broader module
  ordering debt remains separately required peer integration work.

## Contract traces

| Family | Verified source and limits |
| --- | --- |
| Canonical layout | The entire308-line gpu/layout.rs is byte-identical to immutable95f9d31b7d9fbb56aa8330c72fffb46fb8e0c0be. Private canonical StrandCount/PointsPerStrand feed checked u32 conversions, nonempty root/tip admission, checked sample multiplication and nonempty ribbon vertex multiplication. Host full position length is checked before resident fitting; a large host population can have a representable smaller resident prefix. Attribute admission pairs root UV length with normals/IDs/groups/radii. No geometry-value or actual-device guarantee is invented. |
| Binding admission | levels.rs:53–109 checks actual receiving max_storage_buffer_binding_size and max_buffer_size, actual physical extent and STORAGE usage for positions/info. Bindings use explicit canonical positive byte ranges rather than the whole allocation. Private resident GpuLevel retains layout and the actual Arc buffers/groups; it replaces independently stored raw points/count. Its renderer-specific bind groups distinguish this record from the source GPU producer record. |
| Host upload and zero | upload_level uses canonical full-host shape admission and actual device fitting. Minimum physical record limits are checked before allocation. Nonempty slices are bounded by the admitted layout; empty logical positions/info use one native record, zero strands and zero drawn vertices. Source count/order and numerical geometry are preserved. A strand that cannot fit may produce a zero resident population rather than force an invalid one-strand binding. |
| GPU upload | upload_gpu_level fits the canonical layout and uses Buffer::prefix for logical/physical data admission before padded native binding admission. Exact source/layout/device association depends on the required sealed GpuLevel/GpuLods producer, whose readonly methods are not imported yet. Current root still has old raw fields; no compile or complete source-provenance claim is made. |
| Publication and draw | Both groom setters collect fallible level candidates before scene publication. DrawGpu, main/transition clamps, shadow vertex ranges and FrameInfo populations use the retained resident layout. native_draw_vertices returns zero for empty populations, avoiding unchecked multiplication of an unused large point count. Deformation sample multiplication now uses canonical native_samples; its launch/address/source family remains open. |
| Errors | RenderLevelError retains actual StrandLayoutError and BufferPrefixError causes. Named buffer role/extent/limit/usage context preserves units and classification; HairGpuError's anyhow carrier retains those causes. Native allocation/bind-group errors continue through the documented device policy. Successful predicate admission does not establish global native health. |

The raw rest/simulation paths are still connected obligations. bind_dynamics
uses positions.size for a whole physical rest copy/allocation and entire-buffer
bindings at1201–1239; clear_dynamics copies the whole positions allocation at1301.
These can differ from the fitted resident prefix. COPY_SRC/DST, rest/ID/weight
extents, actual device and simulation source association are not established by
the new drawing STORAGE check. Manual64/65,535 deformation launch and the shader's
padded flattened address also require their agreed canonical family repair.
The ledger explicitly records these requirements; they are not accepted silent
deferrals or newly cleared paths.

The fc23 guarded-scan documentation repair is verified: the geometry API now
allows checked lane projection or a block guard before lane addresses, with
actual receiving device admission for either. This resolves51B-GUARD-DOC-01.

## Coverage, preservation and limits

All five changed production responsibilities and complete diffs were examined.
New layout/levels and full draw bodies were read; GPU module production1–654
was read completely, and current groom setters/draw/deform/rest/clear consumers
were re-read. GPU test-body slices have only their recorded coverage; the entire
fixture module is verified byte-identical, not claimed as a new numerical test
audit. Other renderer regions retain exact declared prior full/slice reads
through source deltas and identity receipts, not unread whole-body credit.

JSON records110 prior source receipts:109 match; the only differing receipt is
the independently reviewed fc23 documentation change. It also records canonical
leaf/host collection/Buffer/native ABI sources, all thirteen semantic
relationships, source counts and governing rule hashes. The pre-repair ledger
was checked after source.

New modules have308 and224 lines; added methods have15–57 raw lines and remain
under100. The large renderer shrinks. No ceiling was raised. The existing GPU,
renderer/card/dataset/Buffer fixture modules, five renderer POD bodies and all
three renderer shaders are exact parent bytes. No test, case, fixture, test
helper, assertion or doctest was authored or executed; no production unwrap,
expect, unsafe or suppression was added. Native shader source and valid
geometry formulas remain preserved.

No source edits, project execution/scanners/checks/builds/tests/runtime/generators,
push or CI were performed. Read-only source/Git/hash accounting and ignored
report writing were the only review operations. Required header repairs,
canonical GPU producer/getter/device/source integration, physical Sim/rest/card
families, caller/creator/lock migrations and hosted baseline/project gates remain
open. Final clearance requires complete audit waves, all three independent
reviews and CI on the same final pushed revision.
