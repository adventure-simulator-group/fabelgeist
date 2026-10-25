# Incoming material family Code Review

Pin: `c6659f6cfa4610f066c659411ade6636e21bd3b5`.
Tree: `12f2d808ab9d659999aaec627e02e594897c8beb`.
Root reference: `cac5a7c33a83d1c08a5e55d43d912967edf06814`.

This family is not clear. One concrete connected Root progress defect remains.
No additional defect was found in the canonical checked tile/maps/recipe/image/
sampling/count source reviewed here.

## [P2] Let Metal readback yield to browser mapping callbacks

`crates/fabelgeist-armor/src/gpu/metal/mod.rs:117-120` runs
`pollster::block_on(Buffer::read)`. Public MetalGpu::textures reaches this helper
for its nonempty normal/roughness outputs after submission. A valid browser
context can be supplied through MetalGpu::new, so synchronous native open is
not needed to reach the problem.

The lower browser read awaits a WebGPU mapping callback and event-loop turns.
SDK `wgpu-29.0.4/src/backend/webgpu.rs:2795-2809` registers that callback with a
Promise; `1042-1083` shows same-thread then2 delivery. Meanwhile
`pollster-0.4.0/src/lib.rs:111-129` handles Pending with Signal::wait, whose
`50-68` Condvar blocks instead of yielding the browser thread. The callback
cannot progress normally under that blocking stack. Depending on the wasm host,
the wait blocks or fails; this is a source-proven progress gap, not merely an
unexecuted browser test.

Root confirmed there is no accepted native-only disposition. Root owns this
native facade; #780 owns bake and application receivers. They must agree the
whole-function repair after complete connected discovery. The real browser
read chain must await lower progress without blocking it. Preserve actual
errors, native explicit waits and numerical bytes. Do not add a string adapter,
compatibility forwarding API, silent native-only assumption or arbitrary
unsupported-operation workaround.

The same source proof reopens ScalpTextureChain::read's synchronous block_on at
`hair/gpu/bake/readback.rs:15`, reached by public GpuScalpTexture::read; longest
uses the same pattern. Previous Scalp layout/identity/fixture proofs remain
valid, but its bounded receipt is not browser-progress clearance.

## Source contracts and receiving limits

All 22 complete candidate files were read, plus five complete reference files.
The sixth Creator library reference is exactly the earlier full-read source;
its module/export boundary was freshly reread. All 28 independent hashes match
the manifest and export. The JSON records additional actual caller, native and
SDK read boundaries. No source was copied.

The tile admits sides 32 through1024; checked word populations, native stride
and sealed map byte lengths agree. Immutable recipe/image associations prevent
relief relabeling; IO and decoding errors retain their actual causes. Positive
image extents and native word bounds cover wrapped indexing. Four affine
corner bounds use actual tile pixel centers and uploaded sin/cos coefficients,
with explicit f32 projection, admitting floor and floor+1 as signed addresses.
The image shader template is byte-exact to Root after textual placeholder
expansion; no shader generator was executed.

Canonical MetalImages::new returns anyhow::Result and uses PREVIEW plus the
sealed size/into_maps API, supplying the exact missing CCR-D1 receiver. Root
still has its old String-returning material receiver until coordinated receipt.
The native Metal facade reference is already byte-exact Root; preserve it when
composing types, then repair its progress gap under agreed ownership. Root's
GPU module still lacks the canonical stride/count source and exports consumed
by that facade. Receive their agreed owner; do not replace them with primitives.
The existing typed invocation receiver already fits.

Trim, ornament, decoration and catalog numerical behavior is retained. Native
provider limits and actual error policy remain separate from recipe admission.
The universal device-independent arithmetic wording also needs Root/#780
Comment Review qualification to match host_float's documented limits. This
review supplies no calibration, performance or hardware-wide conclusion.

Seven existing fixture owners preserve test populations and assertion counts;
complete source diffs retain their original oracles through necessary API
adapters. No tests, cases, helpers, assertions or doctests were authored.

No source edit, project check, build, compilation, scanner, generator, numerical
or browser runtime, push or hosted CI occurred. Complete pre-receive and
connected thirteen-lens waves, ownership agreement, actual source composition,
final independent reviews and required hosted checks on one final pushed
revision remain required. No whole-graph or PR clearance is implied.
