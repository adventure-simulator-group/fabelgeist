# Hosted acceptance coverage: independent Code discovery

The pinned workflow omits execution of existing tests for several connected
native migration families. Clippy compiles targets; it does not execute their
unit tests. This is actionable coverage finding HAC-C1, not a runtime failure.

Reviewed complete `.github/workflows/rust-quality.yml` at
`ccb1b6944d674c0bf29d10e6820c0ac4c55092b7`, plus actual package, feature,
module and target registrations. Exact hashes and read limits are in the JSON.
Root owns the workflow. No workflow or production source was edited.

## Existing targets to select after source integration

| Family | Existing target/filter | Feature or boundary |
| --- | --- | --- |
| Generated Compute | `--lib map::`, `gather::`, `stencil::`, `stream::` | Existing generated resource oracles |
| Compute native receiver | `kernel::dispatch_tests::`, `kernel::context_tests::`, `readback::tests::` | Dispatch/copy/counters/cause and grouped readback |
| Compute shared consumers | `reduce::tests::`, `prefix_sum::tests::`, `sort::tests::`, `mesh_query::tests::`, `vertex_normals::tests::` | Existing native numerical/record-layout consumers |
| Compute concurrency | `--test concurrent_kernels` | Real existing integration target |
| Host arithmetic | `--lib host_float::` | Preserve numerical/provider limits; no hidden skip |
| BVH | `--lib gpu::tests::`, `cpu::tests::` | Explicit `--features gpu` |
| Hair | `--lib gpu::tests::`, `render::tests::` | `--features render` also enables GPU |
| Armor material | `--lib engraving::tests::`, `gpu::metal::tests::` | Real decode and native bake/readback fixtures |
| Armor fits/native | Eight existing integration targets listed below | Existing cases only |
| Mesh | `--lib data::tests::` | Explicit `--features gpu` |
| Cloth | `--lib cloth::tests::` | Includes active self-contact and crossing cases |
| Creator underlayer/material | `--lib device_underlayer::`, `armor_metal::tests::`, `garment_material::tests::` | Creator manifest and default offline feature |
| Creator retained underlayer state | `--bin adventuresim-character-creator underlayer_equipment::` | Binary module; `--lib` would miss it |

Armor's eight top-level targets are `breastplate`, `close_helmet`,
`garment_armor`, `generation`, `gpu_concurrency`, `helmets`, `limb_armor` and
`plate_relief`. `tests/common/mod.rs` is a shared module, not another target.
Creator is excluded from the root workspace and requires its own manifest.
Existing Creator export/garment/rig and bracer/breastplate gates remain relevant.

Full existing package suites are useful broader options and are recorded in the
JSON. Focused filters are legitimate selections for actual migrated contracts;
they are bounded evidence rather than a full package pass. Final canonical
names and affected caller selections still require peer confirmation after all
source/API/manifest/lock receipts are composed. Do not select a filter solely to
make a newly failing run green.

## Coverage and validation limits

The existing Creator underlayer cell checks only invalid design input. The
registered `device_underlayer::` group reaches two actual GPU cases and an
incidence-plan fixture. Cloth has active coincident-contact and triangle-crossing
cases; `bend_scaling_grid` is an existing ignored calibration sweep. Hair's
renderer tests validate WGSL and POD sizes, not rendered pixels. No direct
existing Scatter or fluid numerical cases are registered here, so those programs
retain explicit source-only coverage limits. Do not invent cases to fill them.

Historical three software-provider Compute numerical failures and Creator
SIGSEGV are not current results. They must remain documented limits, without
`--skip`, new ignored attributes, allow-failure, weaker assertions or
`continue-on-error`. A failure observed in a newly selected required cell remains
unresolved until honestly diagnosed and dispositioned. No current broad-suite
success is inferred from earlier source review.

Retain every existing gate, including format/lint, semantic/growth checks,
texture integration/docs, rig, Creator, XPBD/Shell, and hosted baseline export.
Use the existing Mesa/Vulkan and XDG setup, `--locked`, serial native harness
threads, exact PR-head evidence, command/features/filter logs and actual exit
statuses. Required selected cells must execute nonempty cases. Preserve actual
provider/cause evidence and ignored counts rather than claiming device-wide or
browser coverage.

No project command, Cargo metadata/list, build, check, lint, test, scanner,
generator, runtime/probe, push or CI request occurred. This report is source
acceptance discovery, not a CI result or final coverage approval. Finish all
connected migration families before requesting hosted checks, then repeat final
reviews and required gates on the same pushed revision.
