# Independent Advect decode/clamp Rule Review

Snapshot: Root worktree
`/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator`,
HEAD `fff01705f4eaae22f98242ce64e538e02ac76e86`, with only
crates/fabelgeist-compute/src/advect/mod.rs dirty. Reviewed current hash
`55748934ab8638b33ef380aae35e12c0ab61b888158a5e0b5de459c68495ba7a`.

No actionable finding remains in the two authorized changes. This is a narrow
source verdict, not an Advect/module/generated-family or final PR clearance.

Read the complete current Advect file and diff, all thirteen entries of
advect_decode_clamp_preinventory, existing BoundaryMode, ResourceDescriptor
binding/type code, texture selection and both view modules, actual native
constructor functions, image-binding reflection and bind-group assembly.
Checked the actual pinned local wgpu-types29.0.4 format contract. Source reads
and changed-region comparison establish this verdict; earlier review reports
and ledger assertions do not substitute for those reads.

## Causal owners and resulting behavior

ResourceDescriptor::from_resource retains the whole Texture2d/3d nominal format.
Generated input bindings are sampled texture types with that format's scalar
kind. Naga image reflection sets sampled TextureBinding.format=None. Binding
assembly calls canonical PassParameter::texture_binding, which selects the
whole resource's nominal view; same-format selection uses the constructor's
cached native view and checks selected device/dimension. Constructors create
that view with requested nominal format and exclude STORAGE_BINDING for sRGB.
Therefore an admitted sRGB sampled input is interpreted by the native sRGB
view. Local wgpu-types29.0.4 texture/format.rs:104-117 explicitly assigns loading
conversion to the native format and says UnormSrgb gives linear shader values;
Float gives no conversion. Detached TextureView alternatives are not input
variants of Advect's GpuResource API.

Deleting the Rust quantity-format Boolean, emitted IS_QUANTITY_SRGB and manual
shader transfer branch removes a duplicate authority. sample_bilinear now
interpolates the already-interpreted load results. It also ceases to apply
quantity's encoding to unrelated velocity samples through their shared helper.
No alternate view, effective-format getter, second decoder, new type or
compatibility adapter is added. Output retains canonical exact storage-format
admission; this repair does not make nominal sRGB textures valid storage output.

BoundaryMode::Clamp owns nearest-edge sample semantics. Current3D Clamp code
at advect/mod.rs:158-160 obtains actual quantity textureDimensions and applies
component-wise min to the final converted unsigned coordinate and dimensions
minus one. The original normalized clamp/conversion and interior nearest
sampling remain intact. At a normalized upper endpoint of1, the original
conversion yielded dimension; it now yields dimension-1, the last valid texel.
The bound comes from the quantity resource being addressed, not output or
velocity extents. Successfully initialized Texture3d dimensions are positive;
uninitialized defaults have no selected native view. Thus subtraction cannot
underflow for an admitted native quantity texture. Literal0/1 endpoints and
one-less last index are intrinsic numerical-kernel math, not new tuning policy.
WGSL local names are actual generated program tokens, not semantic state strings.

Recipe descriptors, BoundaryMode key, actual device/validation policy cache,
uniform ABI, execute/caller API, workgroup math, Wrap path,2D integer clamp,
velocity loads,3D interior conversion and fallible native dispatch are unchanged.
Quantity native format and native dimensions remain their canonical owners.

## Source confinement and limits

Independent direct source comparison found exactly three changed regions:
Rust classifier/emission deletion; shader decoder/comment deletion;3D Clamp
address replacement. All other complete-file bytes are unchanged from HEAD.
No fixture, case, helper, assertion, dependency or API was added. The modified
file shrank; there is no new production-growth responsibility in this delta.

Known broader findings remain open in generated_program_receiving_current_discovery:
private ordered shaped signatures/role admission, unused public AdvectSignature,
manual Default derivability, existing declaration order, and canonical launch
producer/source-workgroup responsibilities. They are not waived by this narrow
verdict, and retained typeorder is not called globally compliant. General mixed
dimension/integer descriptor combinations remain subject to actual native
program admission; this repair does not invent support. NaN/infinite motion
input policy is unchanged and not newly certified. Constructor/selection peers'
full receiving audit is outside this two-change review.

No source edits, project checks, tests, builds, lints, scanners, generators,
runtime or hosted checks were performed. No new tests/cases/helpers/assertions
or external posts were authored. Required final complete thirteen-lens waves,
three independent reviews and hosted checks on the same final pushed revision
remain outstanding. No runtime result or production acceptance is claimed.

## Source SHA-256

- `crates/fabelgeist-compute/src/advect/mod.rs`: `55748934ab8638b33ef380aae35e12c0ab61b888158a5e0b5de459c68495ba7a`

- `crates/fabelgeist-compute/src/boundary.rs`: `31cd64fb15c7bb2edcbc355da4483470f1f28ae2e7c061790aa97341abf137b8`

- `crates/fabelgeist-gpu/src/data/gpu/compute/mod.rs`: `7e21b07871659c611dbd1aa3db5fbcc4323a55adc4183f8aa3d1d4ba4cac84c3`

- `crates/fabelgeist-gpu/src/data/gpu/compute/source_cache.rs`: `686fa8c6510d57714063bb26f287a16c170e690ea25695abd94078041e1d9836`

- `crates/fabelgeist-gpu/src/data/gpu/texture/selection.rs`: `8a600e85d0724eb9f9cfb46e9c6d781eec1186c336335f266c7cff2a7e5cc661`

- `crates/fabelgeist-gpu/src/data/gpu/texture/texture_2d/mod.rs`: `537cbf39f00bc36c4f46ff60a6b550037b283eedafcf41d3975274e5f3e61184`

- `crates/fabelgeist-gpu/src/data/gpu/texture/texture_2d/views.rs`: `a084bd9266444ce4ed50458bfac88cc19e31ba899933eea8ad1ec20f176934ac`

- `crates/fabelgeist-gpu/src/data/gpu/texture/texture_3d/mod.rs`: `31fcc315c4314f2d63153aac4b2db696eae5a02d5ce7ae97881460e06a251295`

- `crates/fabelgeist-gpu/src/data/gpu/texture/texture_3d/views.rs`: `2a09fff5d2d38ccd60190e1d41cd806f8298d3c40cb27bccb2a7844febdc7952`

- `crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/reflection/images.rs`: `3130b402b58ceb84fa372a2d3b4108517795df791998c26dded8b5bed8d5d145`

- `crates/fabelgeist-gpu/src/data/gpu/bind_group/assembly.rs`: `955f5bb0e7dc346933dab81076091f23ba15158a268849dcc0121858fc648658`

- `crates/fabelgeist-gpu/src/data/gpu/texture/format/storage.rs`: `5c67a62164329d7fc2a0d7b96fb4eac6e7cc7317bb6c122ba671ddc95c40335b`

- `/home/brunosegovia/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-types-29.0.4/src/texture/format.rs`: `0a8f0cf2506ae25d2e185e9dbc22edfe292ef82ad392f58b99a7e82ede45e5da`
