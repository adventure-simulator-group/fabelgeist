# Independent generated-program Code source discovery

Pinned HEAD: `c6659f6cfa4610f066c659411ade6636e21bd3b5`. Working tree was clean. Generated-program sources had
not been edited. This is independent pre-implementation discovery and acceptance
input, **not clearance of an implementation or the full PR**. The two prior Rule
reports supplied source maps; all conclusions below follow fresh current source
reads. No source edits, local checks, probes, builds, tests, cases, helpers,
assertions, CI, pushes or external posts occurred.

## Findings and acceptance counterexamples

### Known actionable Scatter unsigned-value rejection

`ScatterDefinition::parse_signature`, scatter.rs:124–147, classifies every
eligible unsigned scalar/vector in the index branch. Once an index exists, a
second ordinary unsigned parameter falls through without becoming a value;
`has_value_param` remains false and admission rejects it. A function with
explicit `index: u32`, ordinary `value: u32` and an output `Resource<u32>`
therefore fails before native shader admission. Buffer fetch and call emission
already support unsigned values, and Stream's corresponding remaining-unsigned
branch already treats them as values. This is not evidence of a float-only
Scatter contract. The proposed role separation must repair the classification.
No existing caller exercising this defect was found in the named actual
consumer/fixture graph; this is source-proven admission behavior, not a run.

### Known actionable Stream missing load after later explicit index

`StreamDefinition::parse_signature`, stream/mod.rs:109–117, can first choose an
ordinary unsigned parameter as its alias index, then replace it with a later
explicit `index` without reclassifying the displaced parameter. For an ordinary
`value: u32` preceding `index: u32`, followed by `offset: u32` and writer metadata,
`has_input_val` stays false. `build_pipeline:229–244` omits the load, while
`stream/call.rs` emits `in_val` for the ordinary value. The generated program
therefore references an undefined value. Establish explicit supported index
ownership before assigning the remaining roles; only use the first eligible
unsigned alias when no explicit coordinate exists.

### Preserve ordinary float parameters named index

This is an additional concrete acceptance guard, not a newly requested DSL
feature. In Scatter and Stream, `index: f32` does not match CoordinateShape and
currently becomes an ordinary value. For example, the source protocol
`fn stream(index: f32, offset: u32) { output[offset] = index; }` has a real input
load and call `(in_val, _offset)`. A new unconditional name==index coordinate
admission would reject a previously supported ordinary float parameter. Select
an explicit index only when its native type is a supported unsigned coordinate.
Map and Stencil separately reserve their `index` name and reject unsupported
coordinate types; a shared lexer must not erase this algorithm-specific policy.
Likewise `offset` is reserved exactly-u32 only in Stream, not universally.
These examples were reasoned from source, not added to any test/case/helper.

### Existing neighborhood-plus-Buffer limit

Stencil's neighborhood path at stencil.rs:728–763 emits `_val_weights =
_params.weights` for a user entry whose cleaned type is scalar. If an actual
secondary Buffer selects that entry as a resource, `311–335` omits it from the
uniform block, potentially leaving no `_params` at all. The neighbor-only source
preprocessing also leaves a `Resource<...>` argument unlowered. Thus that
combination currently fails native shader admission. No valid actual caller or
authoritative promise for that combination was found in the complete named
consumer/fixture graph. Parent confirmed the same source trace. Explicitly
retain this existing rejection/unsupported limit; do not claim a new ordered
role record makes it executable. This is not a request to add a new feature.

## Complete parser-to-native contracts

- Each signature must own one private authored-order parameter/declaration
  collection. Names and native types belong to the shared named leaf or actual
  CoordinateParameter; roles stay local to Map/Gather/Scatter/Stream/Stencil.
  Original declaration text is immutable lexical emission data. Derive all
  iteration/call order, coordinate lookup, role-presence predicates and native
  input/output projections from this authority. Mutable source Definitions
  must continue parsing current source at use, with the full source in cache
  identity; no signature cached beside independently mutable code.
- The existing angle-depth comma splitting handles `ptr<storage,array<Vertex>,
  read_write>` and vector generics. Resource membership must be decoded from
  the original outer native spelling before `parse_base_type` cleaning removes
  ptr/Resource/array wrappers. A Custom identifier merely containing Resource,
  ptr or Neighbors is not itself a reserved protocol. This local lexer is not
  a new general WGSL validator; native Naga admission remains authoritative.
- Map's primary input is contextual: only with an actual input descriptor does
  the first ordinary user parameter become the loaded value. With no input,
  every ordinary parameter is an extra uniform/resource. JFA's typed index and
  size and Perlin/Simplex's unsigned octaves remain ordinary source roles in
  this distinction. Secondary Buffer descriptors use the matching declared
  element type; texture descriptors use actual format. Authored parameter
  order determines bindings/calls; SecondaryResources sorting is cache
  identity only. Preserve binding0 input, binding1 output, extras from2.
- Gather has resource/index semantics and calls only its selected index.
  Ordinary extras currently are not forwarded as in_val/uniform arguments;
  do not silently add that feature. Scatter and Stream use the same in_val
  for remaining ordinary value roles; their input type is the last value
  role (or F32 fallback), and output type is the last resource role (or F32).
  Keep those projections derived, not independently retained fields.
- Stream with only index/offset/writer metadata legitimately needs no input
  value. MC/DC use precisely that protocol. Offset remains its own semantic
  u32 role and is computed from inclusive_offsets minus counts. Buffer work
  is controlled by the offset-table prefix; textures by actual input axes.
  Counts/offsets matching the population and writer output bounds remain the
  existing caller preconditions; this role migration does not certify them.
- Stencil resource mode strips actual input/secondary-resource arguments into
  globals. Neighborhood mode has exact closed Neighbors1D through Neighbors4D
  payloads, separately from the unsigned 1D/2D/3D coordinate shape owner.
  Preserve optional center, explicit center/val precedence, then first eligible
  ordinary fallback only if no explicit center exists. Neighbors-only is valid.
  Earlier explicit rejection for duplicate center/neighborhood or unsupported
  reserved family spelling is an intended admission-policy change, not byte-
  identical behavior for malformed programs. Preserve all existing dimension
  field aliases, boundary Clamp/Wrap code, grid/fetch formulas and call order.
  Existing absent-axis64 defaults and i32 extent arithmetic are a separate
  known numerical family, not cleared by this record design.
- Map/Gather/Stencil retain their declared return type. Scatter/Stream are void
  writers; do not invent a return type or require a new output-role parameter.
  Empty Definition Defaults are deferred source holders: execute/build still
  parse and reject missing functions. Deriving equivalent Default must retain
  the cache's no-Parameters/Value-Default generic contract and Stencil Clamp.
  Definition equality remains code-only or code-plus-boundary, not cache state.
  Remove unused context constructor arguments directly and update actual
  call sites; Scan's separate constructor context is not covered by that removal.

## Actual consumers read and acceptance implications

**DistanceField:** complete generate/min/smooth-min methods were read. Both
wrappers rename the user function, prepend a new loaded in_val parameter,
append original declarations verbatim and call original names in authored order;
smooth-min appends its own k uniform. Keep those lexical operations. A cleaned
ResourceBaseType is a payload projection, not a substitute for native declaration
text. The same temporary texture copy, device admission and submit error chain
must remain. DistanceField binary min/smooth-min retain actual secondary texture
fetches and uniform k; changing Map's primary selection breaks these callers.

**JFA:** both complete 2D/3D pipelines and existing fixtures were read. Their
Map init/step/distance stages use explicit typed index, optional typed size,
first ordinary in_val and ordinary step uniform. Preserve no-input/input
selection, lane suffixes, sentinels, ping-pong and step policy.

**Perlin/Simplex:** complete native programs and execute were read. They are
Map generators with no primary input, coordinate type chosen from the actual
output kind, and scale/offset/octaves/lacunarity/gain as ordinary parameters.
The unsigned octaves uniform must not become a coordinate heuristic in Map.

**MC/DC:** complete constructors, execution and native program sources were
read. Count uses Gather index-only; Stream functions use vec3 index, u32 offset,
and `_res: ptr<storage,array<Vertex or u32>,read_write>` only as writer metadata.
They access the authored global output; the pointer is stripped and is not a new
runtime argument. Custom Vertex native storage stride must continue to come
from reflection. Both shared indirect programs are Map(val:u32)->u32 with
already-authored inclusive_offsets/indirect globals and capacity uniform; do not
synthesize function arguments from their globals or change authored bindings.
All actual constructor call sites must adopt the direct final signatures.
This investigation does not re-clear lookup tables, geometry policies or
ExtractionGrid's complete admission family merely by reading these consumers.

**Native boundary:** ResourceDescriptor uses actual texture formats and the
requested Buffer element type; ResourceFetch couples generated source with
TextureAddressing. ResourcePipeline retains the generated pipeline, actual
reflected controlling layout/stride and workgroup shape. Buffer complete-record
counts become canonical invocation/workgroup quantities; texture axes and
LinearIndex versus LinearPopulation bounds stay distinct. Do not substitute
sorting counts or impose a universal scalar-volume cap on coordinate-only work.
ComputeSourceCache keys source/device/validation policy/recipe and publishes only
successful builds. Native ComputeShader/Pipeline/Pass source was read through
parse/reflect/dispatch/submit. Its pending lower composition and validation-scope
work remains open; reading it is not clearance or ownership to modify it.

## Coverage and preservation requirements

Fresh full source reads: all 39 source files in the main fingerprint group below,
including all five complete signatures/parser/build/cache/execute families,
Stream call helper, shared descriptors/cache/native owners, full listed consumers
and existing Map/Gather/Stream/Stencil/JFA fixtures. lib.rs/prelude.rs were
inspected for actual exports only. Existing fixture inputs/oracles/cases/helpers
and assertions must remain unchanged except necessary direct API access/calls.
No new tests/cases/helpers/assertions/doctests are authorized or were created.

Not freshly reviewed as complete families: native binding-group assembly,
reflection internals, Texture constructors, Scan/Sort, extraction tables/grid/
output owners, broad external callers outside the named graph, Advect/Gradient/
Divergence and whole-PR main/head composition. Fingerprints and searches do not
clear those paths. Full implementation, post-repair all-thirteen-lens waves,
independent final Code/Rule/Comment Review and hosted project checks remain
required on the same final pushed revision. No merge.

## Current SHA-256 fingerprints

Paths are relative to the isolated audit-pr-780 worktree. The final two are
export-inventory-only; other 39 were read in full in this investigation.

| Path | SHA-256 |
| --- | --- |
| `crates/fabelgeist-compute/src/map/mod.rs` | `6bac7f1ff95532004d88a383de80e559f550f4e8d3223d5adbbffd50a8907071` |
| `crates/fabelgeist-compute/src/map/recipe.rs` | `2b74f16ffced873fe963fa9e529ae410a73f41140d10334818b26c8663bef420` |
| `crates/fabelgeist-compute/src/map/tests/mod.rs` | `b5a49e81728272b5bb04064a40395b62442969f9e4fcf10dbb0d13e1149d30e9` |
| `crates/fabelgeist-compute/src/map/tests/helper.rs` | `307a22684a265bcbdd41fc0b7b038b9cdec839b1988eed7f66b8390fee2e6a5d` |
| `crates/fabelgeist-compute/src/gather/mod.rs` | `95da5cd7a5e390cb9ea21c4dd61be55d324726a9d1dfef9c9bc769629318bfd9` |
| `crates/fabelgeist-compute/src/gather/recipe.rs` | `f9f1191683c0890a728199fbca3a598436d356ce1857b8ab96ccdafd354bebf8` |
| `crates/fabelgeist-compute/src/gather/tests/mod.rs` | `6a9360ebc741c55a8068c062857298ce9ee455bb10ce99202fd8f03cb71ee160` |
| `crates/fabelgeist-compute/src/scatter.rs` | `20e9e3f15f7e286e98018ec8bba3d8e4d430e24a13d800d2fd8246f8f0c50932` |
| `crates/fabelgeist-compute/src/stream/mod.rs` | `5d8fdfbb20b5107dc061d22f3bae3611904ea4828ec4ee38fd47ce3518e5ba6e` |
| `crates/fabelgeist-compute/src/stream/call.rs` | `a164da4c7d62a4eb24a6a3b0e1d6f0a15851ed90747e5af07dd9f43b08b941b9` |
| `crates/fabelgeist-compute/src/stream/tests/mod.rs` | `16f48fa721c0981cf7799288e7e4e898e1f6e2481b312bce73487c10070da743` |
| `crates/fabelgeist-compute/src/stencil.rs` | `a5552583fdd75934fc3f6f9623ca45fc7de5f4ae2def11544c19d984dc09cdf6` |
| `crates/fabelgeist-compute/src/stencil/recipe.rs` | `a30f08a0e95afb9c0573bd1633f846099d18d5664fecd4107b98ef3927a1015a` |
| `crates/fabelgeist-compute/src/distance_field.rs` | `2e0eea32b384b061a6b1bbb3162b077479bfefde32e196a8978e66d6c3b84b69` |
| `crates/fabelgeist-compute/src/distance_field_jfa.rs` | `64de20033ee6f686c0877938844c0456dc46748bba806d4fcc2139583fb95b42` |
| `crates/fabelgeist-compute/src/simplex_noise.rs` | `9b30738361080478eafcde92beb4b5c2c3e62df3062bd73012ee499a70dc5b4e` |
| `crates/fabelgeist-compute/src/perlin_noise.rs` | `6e86661c9b0bbfec101b4d0ff7ec68167a7c1c35591f6dd4f7abcd0c4102d092` |
| `crates/fabelgeist-compute/src/marching_cubes/mod.rs` | `20e8ff76b8ffe77cd50150082f3acf9cdfb56166be89945786e94c147151d31b` |
| `crates/fabelgeist-compute/src/marching_cubes/wgsl.rs` | `fc4a7d68f8c8a443bbdc7c1fff67a047ccaa21ed56b315e6ce0805e3da89df93` |
| `crates/fabelgeist-compute/src/dual_contouring/mod.rs` | `22625a943934af4f763dc3466fc3da94d759ce5e1e77637076a88633cb48238b` |
| `crates/fabelgeist-compute/src/dual_contouring/execution.rs` | `e54c3de9b89070020b775d89c1e655d2063fc4005dbc8412e1cc9542ab6d46ca` |
| `crates/fabelgeist-compute/src/dual_contouring/wgsl/mod.rs` | `8a6c40eef3069bc4c265b6136c6159abf092dc1181a23ae8c288666deba03af2` |
| `crates/fabelgeist-compute/src/dual_contouring/wgsl/count.rs` | `39456912fbb2c7b6064b95fccdcfaa4aef7313d4658d3a75e9684447170022e5` |
| `crates/fabelgeist-compute/src/dual_contouring/wgsl/vertex.rs` | `ccb22f4966c34d402b4ab7c82abd08bed44cb370bd65b758a6f8b4a05af9bc47` |
| `crates/fabelgeist-compute/src/dual_contouring/wgsl/index.rs` | `99968238e06d234d0f27422093cd5dee4c2cb4bbcf179fdddf7d845369e1b8f8` |
| `crates/fabelgeist-compute/src/extraction/wgsl.rs` | `f4ab8a5471198e041691d776086f0c51f0ed693d8bbf64b81dd985bc30f2bc04` |
| `crates/fabelgeist-compute/src/resource_pipeline.rs` | `d6532f73e5fd6aac8d83d84f4e56ef46191d98982397d0d90a928326bce80eeb` |
| `crates/fabelgeist-compute/src/resource_recipe.rs` | `620d3f2477548bfff9c9c15e5ba64e534a9ee2e2a756d1b1dbeeff64dd792ac2` |
| `crates/fabelgeist-compute/src/test_utils.rs` | `d6a110a5317449bd4a0330749af0cb0433582763f40280c8294112203193cf6e` |
| `crates/fabelgeist-compute/src/boundary.rs` | `31cd64fb15c7bb2edcbc355da4483470f1f28ae2e7c061790aa97341abf137b8` |
| `crates/fabelgeist-gpu/src/data/gpu/compute/mod.rs` | `7e21b07871659c611dbd1aa3db5fbcc4323a55adc4183f8aa3d1d4ba4cac84c3` |
| `crates/fabelgeist-gpu/src/data/gpu/compute/signature.rs` | `991f1eebf5b1a9cf7f8fa7314793b2610a78fa7f5b01236a19c84ea9f73773fd` |
| `crates/fabelgeist-gpu/src/data/gpu/compute/signature/coordinates.rs` | `505a16459267c2280842514d42eb3aded371b6a2a84584ba6faf9ebb11c4ca2f` |
| `crates/fabelgeist-gpu/src/data/gpu/compute/signature/texture_value.rs` | `79124150e675e45b70b38b8287f82d80b7eda48202e0d2fa6e24f880bb153de6` |
| `crates/fabelgeist-gpu/src/data/gpu/compute/resource_addressing.rs` | `3fc5be3aee86a4b639de572e8309ffeea38a34d0157e5533252e9b15919e9503` |
| `crates/fabelgeist-gpu/src/data/gpu/compute/source_cache.rs` | `686fa8c6510d57714063bb26f287a16c170e690ea25695abd94078041e1d9836` |
| `crates/fabelgeist-gpu/src/data/gpu/compute/pass/mod.rs` | `b9123559b221216745a20cd4f2881f91601174ffa8a6da1d51f07dc656c4cbe3` |
| `crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/mod.rs` | `13cfda2ca90d3fb38739dd6564b412dfb99af3289242b9e277ee8e840baee8d3` |
| `crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/compute_shader.rs` | `f05e7916ac7e0e1474973913f4628d94cd99a9ca69d900d1306101f3286ce169` |
| `crates/fabelgeist-compute/src/lib.rs` | `c04feb5e8852212086b588c9946c8b191bb96ff90814d1278192e7b4bce73d07` |
| `crates/fabelgeist-compute/src/prelude.rs` | `68d9ab3add92c4324cd578477317e07ec1d8928dd58d212e8e53f9aeef8926b4` |
