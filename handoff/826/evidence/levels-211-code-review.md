# Independent native-level checkpoint Code Review

Revision: `21118b983b27e00d2cc70f45ed345686e0262cc2`.
Tree: `5a4c0caac1b7a60f4bb39c12fe1b557c87f0ceb6`.
Parent: `fc23cece6cf1acb2f0f8b873a7e01f4c103f9b75`.
Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.
Canonical layout: `95f9d31b7d9fbb56aa8330c72fffb46fb8e0c0be`.

No new owned actionable correctness finding in this bounded receiving review.
All five changed files were freshly read completely, including the full
1,565-line GPU module and 2,029-line renderer. The canonical layout file is
byte-identical to the pinned peer source. JSON records hashes and exact
fresh/full/partial/carried coverage of changed and connected sources.

Host layout admission checks the complete position collection with checked
host arithmetic before choosing a resident prefix. Both position and info
strides constrain the prefix; a level unable to fit one strand stays truly
empty. Full host collections need not fit native addresses when their chosen
resident prefix does. Exact input order and native payload bytes are retained.

Empty logical data remains zero bytes and zero instances. CPU uploads provide
one positive native record for each storage binding. GPU receipt separately
checks the logical prefix, physical extent, STORAGE usage and the actual
device's storage-binding and allocation limits. Explicit binding sizes expose
the resident bytes rather than the whole allocation. SDK binding source
confirms positive size, physical range, usage and per-binding limits matter.

The canonical layout proves nonempty point/sample/draw bounds. Empty draw
vertices return zero before multiplying a potentially huge point count.
Main, shadow and opacity draw vertex ranges consume that projection;
instance ranges and primitive counts clamp to each resident population.
Lower layout/prefix errors retain their concrete sources through the renderer
and facade errors. Native creation still follows the agreed device-handled
policy, without a returned validation or GPU-completion claim.

Both groom setters construct all fallible level candidates before replacing
the logical scene. Prior scene fields survive admission failure. Earlier
native uploads are possible, so this is logical publication ordering rather
than a GPU rollback guarantee. Frame/deformation publication remains after
normal queue submission, under its previously documented limits.

The existing GPU and renderer fixture modules and every render/native-groom
shader source are exact parent bytes. No test population was added. The
fc23 dispatch documentation accurately permits the scan's independently
proven workgroup-uniform block guard before lane products and barriers.

Required integration remains explicit. GpuLevel layout/position/info getters
and sealed GpuLods device/source methods are absent from the current producer,
so this snapshot cannot compile independently. Canonical producer/caller and
lock integration is required. Simulation rest copies and bindings still use
whole physical buffers; source-associated host/GPU simulation admission,
manual deformation folding and native card count/physical admission also
remain required connected repairs, with no acceptance or silent deferral.

No source was edited and no project execution, build, check, test, scanner,
generator, runtime, push or CI ran. This report establishes no compilation,
full renderer/native-family, all13-lens, whole-PR or final clearance. Final
reviews and hosted gates remain required on the same final pushed revision.
