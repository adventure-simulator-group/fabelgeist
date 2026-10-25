# Generated-program Comment Review discovery

Snapshot: HEAD `99d70e7c3f0bcf8db563c602f39869b91a9f23bf` in
`/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator`.
Only the unrelated nine-source scalp-chain family was dirty. This is source
discovery before repair, not final PR or composed-family clearance.

Applied `/home/brunosegovia/.codex/skills/comment-review/SKILL.md`, repository
`AGENTS.md` and `crates/AGENTS.md`. Used
`/tmp/pr780-generated-program-rule-discovery.md` as a source map; historical
verdicts did not substitute for reading current source. No source edits,
project checks, builds, tests, new cases/helpers/assertions, CI requests or
external publication. Source inspection and file hashing only.

## Actionable findings

### P3: Map claims a type guarantee its parser does not establish

`crates/fabelgeist-compute/src/map/mod.rs:197` says authored code can be pasted
unchanged "because it contains no banned types". `MapDefinition::new` only
parses its signature (46-53); publicly mutable `code` is parsed again at build.
Ordinary and return types pass through `parse_base_type` (112, 118), whose open
fallback returns `ResourceBaseType::Custom` for other spellings
(`fabelgeist-gpu/src/data/gpu/compute/signature.rs:185-186`). The shader body
has not been parsed or validated at the comment's location. Native parsing and
Naga validation happen after wrapper generation at Map:352, inside
`ComputeShader::new` (compute_shader.rs:16-18). Texture projection separately
uses the narrower `TextureValue::try_from` admission.

The no-rewrite behavior is real; the asserted reason is false. Suggested wording:
"Preserve the authored source; validate the generated program below."
Do not introduce a banned-type list or reject valid custom Buffer structures to
make the old comment true. DistanceField wrappers rely on preserving source
spelling and parameter declarations.

### P3: Scan's auxiliary record is submitted work, not observed completion

`crates/fabelgeist-compute/src/scan.rs:31` calls `ScannedAuxiliary` a "completed
auxiliary scan". Instances are created at 230, 263 and 267 after standalone
dispatch. Their buffers can still contain pending queue work.
`ComputePass::dispatch` records and submits without establishing GPU completion
(`fabelgeist-gpu/src/data/gpu/compute/pass/mod.rs:16-34`). Its actual receiving
`WgpuContext::submit` preserves that distinction
(`globals/wgpu_context/submission.rs:4-18, 34-41`). No explicit completion or
readback operation occurs on Scan's execution path. Native validation may poll
in some provider modes; it is not an all-path completion contract.

Suggested wording: "An auxiliary scan buffer and its exact allocated record
population." The record is also used for the next level's emitted totals, so
buffer wording is more precise than a completed-result assertion. Retain queue
ordering; this finding does not request an added wait.

## Contracts checked against complete current source

- Map: reserved named index/size, actual-input-dependent primary value, actual
  secondary resource membership, authored extra/call order, output-controlled
  dispatch, generated load/store projections. Its raw-argument comment matches
  the retained trimmed declarations. Empty Default source remains a deferred
  holder, not an admitted compiled program.
- Gather: resource-like parameters are stripped into globals; its coordinate
  role determines the generated argument; output controls dispatch. Existing
  grid comments match `ResourcePipeline::workgroups(output)`.
- Scatter: input controls dispatch and loads, resource metadata supplies output
  binding type, call arguments use the current parsed signature. Its colon-parts
  comment describes a private call after parser admission; it does not establish
  invariants for publicly mutable signatures. Known unsigned-role defects remain
  implementation work, not Comment Review clearance.
- Stream: buffer launch population comes from inclusive offsets, texture launch
  axes come from input; counts and inclusive offsets must describe that same
  population; authored writers own output bounds. Current execute docs state
  caller preconditions, without falsely claiming runtime proof. Execution submits
  through standalone `ComputePass::dispatch`.
- Stencil: resource/neighborhood paths; current center/val/fallback behavior;
  all four neighborhood dimensions and field aliases; boundary mapping;
  secondary globals, uniform arguments and output-driven dispatch. Its retained
  signature comment matches the resource-mode rewrite. No comment establishes
  admission for its currently broad Neighbors substring parser.
- Advect/Gradient/Divergence: complete builders/execution, descriptors, boundary
  modes, native workgroup arithmetic, uniforms and load/store code read. The
  earlier false Advect textureLoad conversion explanation is absent. Its final
  3D Clamp address uses dimensions minus one, matching BoundaryMode's nearest
  edge. No unsupported units, interpolation, completion or health claim found.
  Physical/Grid/native concerns remain separate open work.
- Scan: reflected native array stride, exact two-stage block-shape admission,
  workgroup totals versus invocation counts, scratch extents, native guards and
  queue order. The intermediate-dispatch comment matches its deliberate
  one-group-per-stored-total launch and live-prefix guard. The completion
  adjective above is inaccurate.
- DistanceField: complete wrappers/direct methods. Raw declarations and call
  order survive when prepending primary `in_val` and appending `smin_k`. In-place
  wrappers copy to a temporary source before separately submitted Map work.
  Their aliasing explanation matches actual separate source/target resources;
  no completion guarantee is made.
- JFA: complete 2D/3D generators, seed sentinels, step uniforms, primary values,
  index/size use and current fixtures. Complete Simplex/Perlin generators retain
  their actual scalar/vector declarations, no-primary Map calls and shader
  bodies. No unsupported provenance/performance comment found.
- Shared signature: open Custom native vocabulary, exact CoordinateShape
  spellings, retained coordinate name/shape, TextureValue numeric lane admission
  and exact-zero store padding, ResourceFetch source/address pairing, distinct
  TextureAddressing last-index/population projections and combination rule.
  Comments match these implementations.
- ResourcePipeline: immutable native pipeline/layout/shape from one source,
  controlling binding global-name protocol, reflected Buffer stride,
  independent texture axes and per-expression scalar bounds. Recipes sort
  secondary identities while generated binding order follows authored order.
  Source cache retains device/source/recipe/validation policy, builds outside
  its lock and never publishes a failed constructor outcome.
- Native receiving boundary: complete ComputeShader, ComputePipeline, entry
  selection, main reflection, ComputePass, submit and completion source read.
  Comments distinguish shader validation, interface reflection, provider policy,
  standalone submit and explicit completion. This is not backend clearance.
- Actual extraction callers: complete MC/DC constructors/execution, all built-in
  count/stream programs, shared indirect/deinterleave, lattice and projection
  source. They consume Gather -> Scan -> Stream and Map with a dummy primary
  plus predeclared global offset/indirect resources. Stream pointer metadata is
  stripped, not a supplied function argument. Current bounds/publication
  comments match the read guards. Geometry and table correctness are not
  independently cleared.
- Existing Map/Gather/Stream/Stencil/JFA fixtures and shared test utility were
  read without running or modifying them. Readbacks remain separate observation
  boundaries. Reading fixtures supplies no hosted CI evidence.

## Limits and remaining work

Two actionable P3 comment findings; no edits made. Parent must inventory before
repair and re-review the final composed family. Public mutable parallel
signature metadata and shaped-parser implementation findings remain open in
Root's all-thirteen work. No Rule or Code Review clearance is claimed here.

Not freshly read: Reduce/Broadcast/Reshape/Transpose/PrefixSum implementations
and fixtures; sorting/BVH/normals/cloth/underlayer families; all texture
constructors/views/bind-group packers; reflection image/uniform submodules;
`parse_naga` internals; native backend/SDK execution; external users; all
extraction tests and table contents. Broader crate-level bullets were read for
orientation, not cleared against unread implementations. Historical verdicts
and searches are not clearance. Searches located calls whose listed bodies were
then read completely.

## Current SHA-256 fingerprints

Every following file was read completely in this wave. Paths are relative to
the isolated repository above.

| Source | SHA-256 |
| --- | --- |
| `crates/fabelgeist-compute/src/map/mod.rs` | `6bac7f1ff95532004d88a383de80e559f550f4e8d3223d5adbbffd50a8907071` |
| `crates/fabelgeist-compute/src/map/recipe.rs` | `2b74f16ffced873fe963fa9e529ae410a73f41140d10334818b26c8663bef420` |
| `crates/fabelgeist-compute/src/map/tests/mod.rs` | `b5a49e81728272b5bb04064a40395b62442969f9e4fcf10dbb0d13e1149d30e9` |
| `crates/fabelgeist-compute/src/map/tests/helper.rs` | `307a22684a265bcbdd41fc0b7b038b9cdec839b1988eed7f66b8390fee2e6a5d` |
| `crates/fabelgeist-compute/src/gather/mod.rs` | `95da5cd7a5e390cb9ea21c4dd61be55d324726a9d1dfef9c9bc769629318bfd9` |
| `crates/fabelgeist-compute/src/gather/recipe.rs` | `f9f1191683c0890a728199fbca3a598436d356ce1857b8ab96ccdafd354bebf8` |
| `crates/fabelgeist-compute/src/gather/tests/mod.rs` | `6a9360ebc741c55a8068c062857298ce9ee455bb10ce99202fd8f03cb71ee160` |
| `crates/fabelgeist-compute/src/scatter.rs` | `20e9e3f15f7e286e98018ec8bba3d8e4d430e24a13d800d2fd8246f8f0c50932` |
| `crates/fabelgeist-compute/src/stencil.rs` | `a5552583fdd75934fc3f6f9623ca45fc7de5f4ae2def11544c19d984dc09cdf6` |
| `crates/fabelgeist-compute/src/stencil/recipe.rs` | `a30f08a0e95afb9c0573bd1633f846099d18d5664fecd4107b98ef3927a1015a` |
| `crates/fabelgeist-compute/src/stream/mod.rs` | `5d8fdfbb20b5107dc061d22f3bae3611904ea4828ec4ee38fd47ce3518e5ba6e` |
| `crates/fabelgeist-compute/src/stream/call.rs` | `a164da4c7d62a4eb24a6a3b0e1d6f0a15851ed90747e5af07dd9f43b08b941b9` |
| `crates/fabelgeist-compute/src/stream/tests/mod.rs` | `16f48fa721c0981cf7799288e7e4e898e1f6e2481b312bce73487c10070da743` |
| `crates/fabelgeist-compute/src/advect/mod.rs` | `55748934ab8638b33ef380aae35e12c0ab61b888158a5e0b5de459c68495ba7a` |
| `crates/fabelgeist-compute/src/gradient/mod.rs` | `bca4af50d86428b57eb0834e28e9879da517e4146b2bd71fa36598f2004f0e97` |
| `crates/fabelgeist-compute/src/divergence/mod.rs` | `d56ea387e7ea0d2c31ce000287ca0932029581a9e794fa0d54207dad76774c38` |
| `crates/fabelgeist-compute/src/scan.rs` | `f8ec75e11bdf6145f32018251f300ccedd7c73067bc549ebd35ace53e5745d79` |
| `crates/fabelgeist-compute/src/scan/wgsl.rs` | `803e777cf527e7c130b233a52edf330827c91ab7de8b2d9206bdd7d5bf56f9ec` |
| `crates/fabelgeist-compute/src/distance_field.rs` | `2e0eea32b384b061a6b1bbb3162b077479bfefde32e196a8978e66d6c3b84b69` |
| `crates/fabelgeist-compute/src/distance_field_jfa.rs` | `64de20033ee6f686c0877938844c0456dc46748bba806d4fcc2139583fb95b42` |
| `crates/fabelgeist-compute/src/simplex_noise.rs` | `9b30738361080478eafcde92beb4b5c2c3e62df3062bd73012ee499a70dc5b4e` |
| `crates/fabelgeist-compute/src/perlin_noise.rs` | `6e86661c9b0bbfec101b4d0ff7ec68167a7c1c35591f6dd4f7abcd0c4102d092` |
| `crates/fabelgeist-compute/src/resource_pipeline.rs` | `d6532f73e5fd6aac8d83d84f4e56ef46191d98982397d0d90a928326bce80eeb` |
| `crates/fabelgeist-compute/src/resource_recipe.rs` | `620d3f2477548bfff9c9c15e5ba64e534a9ee2e2a756d1b1dbeeff64dd792ac2` |
| `crates/fabelgeist-compute/src/boundary.rs` | `31cd64fb15c7bb2edcbc355da4483470f1f28ae2e7c061790aa97341abf137b8` |
| `crates/fabelgeist-compute/src/test_utils.rs` | `d6a110a5317449bd4a0330749af0cb0433582763f40280c8294112203193cf6e` |
| `crates/fabelgeist-compute/src/lib.rs` | `c04feb5e8852212086b588c9946c8b191bb96ff90814d1278192e7b4bce73d07` |
| `crates/fabelgeist-compute/src/prelude.rs` | `68d9ab3add92c4324cd578477317e07ec1d8928dd58d212e8e53f9aeef8926b4` |
| `crates/fabelgeist-compute/src/dual_contouring/mod.rs` | `22625a943934af4f763dc3466fc3da94d759ce5e1e77637076a88633cb48238b` |
| `crates/fabelgeist-compute/src/dual_contouring/execution.rs` | `e54c3de9b89070020b775d89c1e655d2063fc4005dbc8412e1cc9542ab6d46ca` |
| `crates/fabelgeist-compute/src/dual_contouring/wgsl/mod.rs` | `8a6c40eef3069bc4c265b6136c6159abf092dc1181a23ae8c288666deba03af2` |
| `crates/fabelgeist-compute/src/dual_contouring/wgsl/count.rs` | `39456912fbb2c7b6064b95fccdcfaa4aef7313d4658d3a75e9684447170022e5` |
| `crates/fabelgeist-compute/src/dual_contouring/wgsl/vertex.rs` | `ccb22f4966c34d402b4ab7c82abd08bed44cb370bd65b758a6f8b4a05af9bc47` |
| `crates/fabelgeist-compute/src/dual_contouring/wgsl/index.rs` | `99968238e06d234d0f27422093cd5dee4c2cb4bbcf179fdddf7d845369e1b8f8` |
| `crates/fabelgeist-compute/src/marching_cubes/mod.rs` | `20e8ff76b8ffe77cd50150082f3acf9cdfb56166be89945786e94c147151d31b` |
| `crates/fabelgeist-compute/src/marching_cubes/wgsl.rs` | `fc4a7d68f8c8a443bbdc7c1fff67a047ccaa21ed56b315e6ce0805e3da89df93` |
| `crates/fabelgeist-compute/src/extraction/mod.rs` | `f9914138b034fab9e37f6cffcd87510f2dcb38da129a23c4ce8c04c4a0768cce` |
| `crates/fabelgeist-compute/src/extraction/projection.rs` | `1f916775018fe658abd71af9a1ad2c4fe19b62516f5ce5827b960a8dd237f592` |
| `crates/fabelgeist-compute/src/extraction/wgsl.rs` | `f4ab8a5471198e041691d776086f0c51f0ed693d8bbf64b81dd985bc30f2bc04` |
| `crates/fabelgeist-gpu/src/data/gpu/compute/signature.rs` | `991f1eebf5b1a9cf7f8fa7314793b2610a78fa7f5b01236a19c84ea9f73773fd` |
| `crates/fabelgeist-gpu/src/data/gpu/compute/signature/coordinates.rs` | `505a16459267c2280842514d42eb3aded371b6a2a84584ba6faf9ebb11c4ca2f` |
| `crates/fabelgeist-gpu/src/data/gpu/compute/signature/texture_value.rs` | `79124150e675e45b70b38b8287f82d80b7eda48202e0d2fa6e24f880bb153de6` |
| `crates/fabelgeist-gpu/src/data/gpu/compute/resource_addressing.rs` | `3fc5be3aee86a4b639de572e8309ffeea38a34d0157e5533252e9b15919e9503` |
| `crates/fabelgeist-gpu/src/data/gpu/compute/mod.rs` | `7e21b07871659c611dbd1aa3db5fbcc4323a55adc4183f8aa3d1d4ba4cac84c3` |
| `crates/fabelgeist-gpu/src/data/gpu/compute/source_cache.rs` | `686fa8c6510d57714063bb26f287a16c170e690ea25695abd94078041e1d9836` |
| `crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/compute_shader.rs` | `f05e7916ac7e0e1474973913f4628d94cd99a9ca69d900d1306101f3286ce169` |
| `crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/mod.rs` | `13cfda2ca90d3fb38739dd6564b412dfb99af3289242b9e277ee8e840baee8d3` |
| `crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/entry_point.rs` | `2d646038056780254cfafb4d59d6ba66963cf820ca662df217ed58ff7527bde7` |
| `crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/reflection.rs` | `28875e4c667ce2a16c41e3640b8948fcba6f90e399b45d649ddb43f0832f8b90` |
| `crates/fabelgeist-gpu/src/data/gpu/compute/pass/mod.rs` | `b9123559b221216745a20cd4f2881f91601174ffa8a6da1d51f07dc656c4cbe3` |
| `crates/fabelgeist-gpu/src/data/gpu/resource.rs` | `d80964b752b34346f83e75fb30110cc692ac0a3a4a3283c4a09185d85175bda4` |
| `crates/fabelgeist-gpu/src/globals/wgpu_context/submission.rs` | `70b94328d0cfb9388ff86b5cc7960a7bf2420867a0a0ad406002ead4dd1c5a27` |
| `crates/fabelgeist-gpu/src/globals/wgpu_context/completion.rs` | `0bf57e56179c36d0bf533b46daf1971b4037363b98505b14b55cf6c4ff2f4fdf` |
