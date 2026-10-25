# Independent Rule Review: head source receiving checkpoint

Pinned revision `04a68c493d152c5250e17bedd09d33497ab67bf3`, tree
`a3084113b320d4452229aee325c5f057c20e9f4b`, compared with
`d9c3521f30f52d3437cfef2850203c9df3192b0e`. Main remains pinned at
`990430b77a9f63548ffcb642074185f61ddb3322`.

No new actionable owned receiving rule finding. Both D9 findings are repaired.
Completeness remains open: canonical source/API/device production and the
remaining physical producer, caller, lock and hosted gates are required.
This is neither compilation evidence nor combined family/final PR clearance.

## Source-backed contracts

| Family | Verified source trace and limit |
| --- | --- |
| Prefix write | `buffer/mod.rs:53–54` delegates to the existing `write_at(START)` operation; logical end and overflow admission at 67–77 precede actual native upload. This repairs D9-PREFIX-WRITE-01 without another backend, compatibility path, alignment policy or error erasure. Prefix/read/map/binding sources remain exact prior bytes. |
| Draw range prose | `render/draws.rs:16–17` now describes the logical u32 interval and recording's resident clamp. This repairs D9-DRAW-DOC-01; split and clamp behavior is unchanged. |
| Head topology | `head.rs:47–49` consumes canonical SurfaceRenderLayout before upload. At immutable peer `bcbc1129b76d378917d8134fbe93a5687606f129`, that owner requires equal positions/normals, every triangle index below vertex count, u32 mesh counts and checked three-corner multiplication. Root reuses MeshVertex/Corner owners and layout rather than defining another validation type. This admission does not claim UV, sampling CDF, BVH or physical geometry validity. |
| Retained head identity | One optional HeadDraw owns actual Arc<Surface>, immutable layout witness, mesh and coverage. It replaces independently optional head/cover and cloned HeadSurface vectors. Native head depth and view passes both consume that same aggregate. Arc sharing prevents safe external mutable access while the capture is retained; equality of counts is never substituted for source identity. |
| Empty and CPU coverage | Empty admitted geometry retains zero native draw corners while vertex/index/coverage buffers have nonempty physical placeholders. ScalpCover::at zips the now proven paired columns, producing one cover entry per vertex. Its existing three smoothing passes use the same admitted triangles, so their direct indexes have a construction proof under `crates/AGENTS.md:80–84`. Empty CPU coverage supplies one zero native pair. Numerical smoothing is preserved. |
| GPU coverage | `with_gpu_cover:90–125` checks actual cover Device, exact Arc::ptr_eq surface capture, exact logical vertex-count-times-eight bytes and physical capacity at least max(logical, eight). The private peer GpuHeadCover constructor captures the actual context device and Vertex buffer usage. Cover None creates fresh bare coverage, resolving old Some-to-None retention. GpuLods::device itself is an accepted pending producer contract, not present source proof yet. |
| Publication and causes | `set_head:177–182` constructs before replacing; malformed layout leaves previous head. `set_gpu_groom:1038–1111` performs typed source/device/extent admission and candidate construction before publishing scene state. RenderSourceError::Layout preserves actual SurfaceRenderError through derive-from/source and HairGpuError's anyhow carrier. Native allocation and pipeline errors retain the documented device-handled policy; successful candidate construction does not certify global device health. |
| Builder integration | The whole TextureBinaryOp builder now branches on canonical ResourceBaseType variants and propagates the agreed fallible storage-binding owner result before shader/pipeline/cache publication. Root adds no duplicate descriptor/type mapping. Both root04 and inspected peer bcbc still have old ResourceDescriptor signatures: typed base_type/fallible output-binding source must be consumed before this can compile or the actual combined cause behavior can be verified. |

The receiving part of 197-BOUNDS-01 is addressed by the canonical paired layout
and immutable capture. Full closure requires importing that exact producer and
finishing the remaining source/count/native graph. CPU set_groom's complete
fallible atomic publication remains required; its current signature was not
silently accepted as the final migration.

Bare HairRenderer and Bevy continue using actual supplied Device/Queue and the
agreed configured provider handling. No fake context or requirement to impose
owned-context synchronous validation was introduced. Existing temporal
History::Empty/Current, finite view admission, complete policy cache identity,
logical transition split and resident draw clamp remain intact.

## Rules, scope and preservation

All five complete diffs and their receiving responsibilities were inspected.
Four smaller current changed bodies were read in full. The entire renderer
responsibility is carried from the immediately preceding full D9 read plus the
complete exact current delta; current set_groom, set_gpu_groom, head passes,
frame admission, convergence, draw, post and submit consumers were re-read.
JSON explicitly records preserved equal regions rather than claiming fresh
unread whole-body credit. Eighty-one other declared source receipts match exact
current bytes. Whole canonical peer layout/head/count owners and relevant
Surface/GpuLods/descriptor slices were read, with exact hashes and scope.

The root and crates rule/skill/reference hashes match prior read receipts.
`head.rs` is a coherent receiver aggregate module, has proper item order,
canonical typed byte/error fields, an owning result alias and actual source
causes (`crates/AGENTS.md:28–71,90–122`). Arc aggregate/mesh Clone derives
preserve capture identity. No compatibility API, parallel source brand,
production unwrap/expect, new unsafe or suppression is introduced. Existing
Buffer unsafe impls are byte-preserved.

The oversized renderer shrinks from 2206 to 2135 lines. The focused new head
module has 189 lines; its largest method has 42 lines. Expanded set_gpu_groom
has 74 lines and builder has 64. Existing resize's 103-line allowance is
unchanged. No ceiling is raised; hosted baseline reductions remain pending
under `crates/AGENTS.md:166–178`.

Existing renderer/dataset/Buffer test module bytes, all five renderer POD item
bodies and all three shaders are preserved exactly. No new test, case, fixture,
test helper, assertion or doctest was authored or executed. No source edits,
project scanners/checks/builds/tests/runtime/generators, push or CI were run.
Only read-only source/Git/hash accounting and ignored report writing occurred.

## Required dependencies

- Import canonical SurfaceRenderLayout/GpuHeadCover/count definitions and exact
  owning features; root currently references sources it has not consumed.
- Implement and consume GpuLods' agreed actual device capture and complete
  physical resource/layout admission. Its accessor alone must not certify
  publicly mixable foreign resources.
- Consume agreed canonical typed ResourceDescriptor base and fallible storage
  generation APIs. Their source and actual combined cause propagation remain
  unverified until available.
- Complete #780 strand/card/atlas/simulation/head source, physical counts/layout,
  creator/callers/completion publication/fixture and lock integrations, together
  with root's remaining CPU/native receiving contracts.
- Retain required #780 lod item ordering and production unwrap repairs. These
  inherited affected gaps were not independently edited.
- Complete hosted baseline reductions and required existing gates, then repeat
  all thirteen lenses and three independent reviews on the same final pushed
  revision. No incomplete path is granted a silent deferral here.
