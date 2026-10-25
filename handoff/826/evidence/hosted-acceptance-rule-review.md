# Hosted acceptance source discovery

Pinned Root `ccb1b6944d674c0bf29d10e6820c0ac4c55092b7` / tree
`4cc54cebccefd4eb249900fb22d160cdaba53485`, compared with main
`990430b77a9f63548ffcb642074185f61ddb3322`. Peer context is exact `d000e848f`;
no receipt or compilation is inferred. This report inventories acceptance before
workflow edits. All thirteen focused lens records, individual source hashes/read
ranges, canonical carries and existing target records are in the adjacent JSON.

The current workflow does not execute existing tests for several connected
Compute, Hair and Armor migration families. Clippy compiles test targets; it
does not execute their numerical/native/readback behavior. HA-R1 is an
actionable acceptance gap, not a newly demonstrated production runtime failure.

Hair adds a concrete evidence trap (HA-R2): ten existing GPU tests print `no
GPU: skipped` and return success if headless setup fails. The harness can report
passed with zero ignored; the current nonempty verifier would accept it. Use
existing cases with captured output and extend the existing verifier or explicit
blocking acceptance accounting. Do not change tests or add a provider probe,
checker or helper. Unsupported provider remains a visible limit.

 Each listed filter is a separate Cargo invocation; comma lists below are
alternatives, not one executable command. `--lib`, `--test` and `--doc` are
separate target selections. Preserve the existing full GPU texture fixture/docs
steps; full GPU `--lib` is also an existing broader native-owner option. Armor
`--lib engraving::tests::` adds three already existing decode/admission cases.

| Family | Existing selection | Boundary/limit |
| --- | --- | --- |
| Generated Compute | `--package fabelgeist-compute --lib map::, gather::, stream::, stencil::, reduce::tests::` | Actual required-GPU contexts, no self-skip in these selected sources. Macro cases cross compatible resource shapes; does not provide direct Scatter/fluid/projected-z/4D-specific coverage. |
| Compute lower/shared receiver | `--package fabelgeist-compute --lib kernel::dispatch_tests::, kernel::context_tests::, readback::tests::, prefix_sum::tests::, sort::tests::, mesh_query::tests::, vertex_normals::tests::` | Existing registrations located; final canonical peer names/API adaptations pending. This focused report does not fresh-review every test body in these groups; Code discovery separately inventories them. Full package executes them without choosing individual filters. |
| Compute concurrency | `--package fabelgeist-compute --test concurrent_kernels ` | Real automatic Cargo integration target; tests/common-style helper not a target; canonical submit propagation pending. |
| Host arithmetic | `--package fabelgeist-compute --lib host_float:: ` | Historical three software-provider numerical failures must remain documented. Fixed sample count/oracles/tolerances unchanged; no current run. Full Compute includes this group; source-focused selection is not full package pass. |
| Hair | `--package fabelgeist-hair --features render --lib gpu::tests::, render::tests::` | render (default) enables gpu. gpu-only no-default selection omits renderer tests. Ten GPU cases can self-skip with success; one is shader-only. Render cases only validate shader/POD; no rendered-frame/readback oracle. |
| Armor | `--package fabelgeist-armor --lib gpu::metal::tests::, --test gpu_concurrency` | Eight top-level target names located; only metal/concurrency/common bodies fresh read for this focused report. Full package is broader honest execution option. Current old fixture operands require canonical API-only propagation before CI. |
| BVH | `--package fabelgeist-bvh --features gpu --lib gpu::tests::, cpu::tests::` | Fresh existing host/native source; GPU contexts fail rather than self-skip. Final canonical producer/count/caller adaptations pending. |
| Mesh | `--package fabelgeist-mesh --features gpu --lib data::tests:: ` | One GPU readback roundtrip, two CPU validation/topology cases. GPU feature needed for native case; canonical counts/strip body/test adapters pending. |
| Cloth | `--package fabelgeist-cloth --lib cloth::tests:: ` | Existing required-GPU Harness, active self-contact and crossing/submission cases; bend_scaling_grid already ignored calibration sweep, never claim it ran under default selection. No new tests or ignore policy. |
| Creator lower/material recipients | `--manifest-path crates/adventuresim-character-creator/Cargo.toml --lib device_underlayer::, armor_metal::tests::, garment_material::tests::` | Separate workspace/manifest. Existing workflow underlayer::invalid... is a different CPU admission case. Default offline-creator enables material tests; full lib historical SIGSEGV not a current result. Canonical material/density API/test adapters required. |
| Creator retained binary recipients | `--manifest-path crates/adventuresim-character-creator/Cargo.toml --bin adventuresim-character-creator underlayer_equipment::` | Code discovery identifies binary registration; this reviewer has not fresh-read its body, so final canonical owner/name confirmation remains required. Existing character_morphs/animation cells retained. parametric_equipment asset/GPU test is already ignored, requires MHR_ASSETS and cannot be counted as ordinary native coverage. |

Full existing Compute, Hair (render), Armor, BVH (gpu), Mesh (gpu) and Cloth
package suites are broader honest options. Focused contract filters are already
a workflow pattern and can provide bounded evidence; they must not be chosen
merely to hide a failure. Full Compute includes the three historically failing
software-provider arithmetic cases. Full Creator `--lib` has a historical
SIGSEGV limit. Neither history is a current-revision or current-main result. Do
not add skips, ignored attributes, allow-failure/continue-on-error, weakened
assertions or tolerances. Every newly selected required failure stays a blocker.

Generic Scan, Scatter, fluid operators and extraction/MC/DC have no dedicated
registered test cases in the inspected owning modules. A zero-match filter is
not coverage. Renderer cases validate shader/POD contracts, not rendered pixels
or browser progress. Existing ignored calibration/asset fixtures remain visible.

Preserve every current gate. Add selected existing execution, command/feature/
filter logs and exact final PR-head evidence to the existing verifier and
uploads; preserve actual Cargo failures through pipes. Complete canonical
source, callers, test API adaptations and locks before requesting hosted checks.
Apply only relevant reductions from the final hosted baseline export. Require
the three reviews and required CI on the same final pushed revision.

No source, test, workflow or assertion was edited. No project check, scanner,
build, test, generator, runtime/probe, push or CI request ran. This is discovery
and bounded Rule Review, not runtime, compilation or final graph clearance.
