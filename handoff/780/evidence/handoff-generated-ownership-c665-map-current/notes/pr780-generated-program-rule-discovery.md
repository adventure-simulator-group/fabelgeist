# Independent generated-program Rule source discovery

Root worktree:
`/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator`.
HEAD `b11d6697c5b2fc05ca59cf7497f3d6ab626580b5`;
main `990430b77a9f63548ffcb642074185f61ddb3322`.
The final source snapshot had a clean working tree. This is discovery before
Root's complete thirteen-lens preinventory and repairs, not final clearance.

## Governing rules and coverage

Applied root AGENTS.md, crates/AGENTS.md, rule-review/SKILL.md and its
bespoke-type-review reference. Mandatory rules require named semantic records,
closed role/state enums, invariant-bearing aggregates, canonical shared owners,
producer-to-consumer propagation, file item order and cohesive same-error Result
aliases. The user's no-new-tests/cases/helpers/assertions and hosted-check
constraints control execution. No source changes, project checks, builds, tests,
lints, scanners, generators, runtime, CI, push or external posts occurred.

Read the peer storage-projection canonical report/findings, then independently
read the full Map and Stencil files including existing fixtures, their recipes,
shared signature/coordinates/addressing, ResourcePipeline/resource recipe/cache,
and native ComputeShader/ComputePipeline/ComputePass. Complete caller reads:
DistanceField, DistanceFieldJfa, Simplex, Perlin, Map tests/helper. Extraction
coverage is the complete owning constructors, shared indirect-WGSL producer and
complete Map sync-indirect blocks in both extractors; other extraction behavior
is not freshly cleared. Advect/Gradient/Divergence were read whole. For Gather,
Scatter and Stream this investigation confirms exact declaration-order locations
and composition, not complete fresh semantic clearance of their programs.
Searches identified usages; receiving source bodies establish the contracts.

## P2: named parameters and one signature authority

MapSignature at map/mod.rs:15-21 independently exposes raw declarations, ordered
names, coordinate options and positional user records. StencilSignature at
stencil.rs:16-26 similarly exposes names/coordinates/user tuples and mode fields.
Map parser:55-130 and Stencil parser:71-207 populate the projections consistently
on ordinary inputs, but every public field can be independently changed. A
caller can replace param_names without changing typed parameters or declarations.
The resulting public value has no coherent argument-list guarantee. This is a
representable API/aggregate and incomplete-migration gap, not proof of a current
pipeline failure: build_pipeline reparses Definition.code and does not consume a
fabricated external signature.

Map:147-177 and Stencil:311-321 reproduce semantic name/type tuples when
partitioning uniforms/resources. These directly violate the named-record rule.
Map:320 and Stencil:739 derive calls from separately retained name lists.

Named user records plus a tagged StencilInput alone are insufficient while the
other independent mutable authorities remain. Recommended final design:

- Shared native ShaderParameter with name:String and existing ResourceBaseType
  belongs in the GPU signature owner; no Compute-local competing equivalent.
- Each algorithm signature owns one private ordered parameter collection with
  typed role payloads. Existing CoordinateParameter owns index/size name+shape;
  ShaderParameter owns ordinary name+type. Stencil roles distinguish resource,
  center and neighborhood as appropriate. Do not duplicate an outer name beside
  a role payload that already owns it.
- Derive user iteration, coordinates, primary Map input selection, secondary
  partitions and call order from this collection. Borrow/filter named entries
  instead of copying positional pairs. No second mutable name/user/index array.
- Preserve each original declaration as immutable constructor-associated text
  in its ordered entry, documenting its lexical purpose. DistanceField
  generate_min:31 and generate_smooth_min:114 extend sig.map_args:44/:128 and
  call original ordered sig.param_names:46/:131. ResourceBaseType's cleaning
  erases some Resource/ptr/array/alias spelling. Rebuilding declarations from
  as_str() alone can change the native source protocol. Immutable verbatim text
  is a justified native projection, not independent mutable semantic authority.
- Definition.code is currently public and mutable. Continue parsing current
  source at use; do not cache a signature beside mutable code. An immutable
  source+signature owner is a larger coherent alternative, not extra state.

Update public callers to the final API directly. No tuple forwarding accessors,
compatibility aliases, temporary primitive adapters or dual implementations.
Native identifiers/custom types remain open strings; no identifier catalog.

## P2: admitted Stencil neighborhood schema and exact decoder

stencil.rs:17-24 carries closed Resource/Neighborhood mode through a bool, raw
u32 dimension, input type and independently optional center/neighbor names. The
public type admits contradictory modes/payloads and any dimension. Parser
:112-120 starts at2 and matches any type containing Neighbors; :138-155 selects
1..4 by substring and leaves unknown forms as2D. Codegen's native replacement
:222 recognizes Neighbors([1-4])D, then generation branches independently at
:271/:275, :441/:445, :535/:542/:549, :565 and :744. There is no single admitted
mode/dimension authority. This is a schema/native-classification gap, not a
claim that existing ordinary valid programs demonstrably fail.

Use Resource versus Neighborhood as a tagged schema. Resource owns only its
actual input metadata; Neighborhood owns its admitted dimension and element
type. In the ordered collection, its neighborhood-role entry owns the native
name and typed payload. Derive the aggregate mode or admit an immutable private
aggregate once; do not store the same dimension/name/type as separately mutable
facts. A center is optional: neighbors-only functions legitimately have none.
Preserve current center/val and first-other-parameter fallback unless an explicit
separate API policy changes it; do not invent a mandatory center.

Neighborhood dimension is a closed1D/2D/3D/4D enum. CoordinateShape is a different
owner for unsigned scalar/plane/volume index/size and cannot encode4D. Decode
exact Neighbors1D<T> through Neighbors4D<T> at the source boundary. Do not default
an unknown family spelling to2D or classify an arbitrary custom identifier just
because it contains Neighbors. Keep actual Custom WGSL identifiers open; make
unsupported reserved-family behavior explicit. Exhaustive typed matches replace
raw-dimension fallbacks without changing native numerical policy.

Keep actual WGSL keywords/operators/type tokens/declaration text as source
strings. Reuse BoundaryMode. A source grammar token is not mutable display prose.

## P3: exact declaration-order locations

crates/AGENTS.md mandates imports/re-exports, mods, aliases, structs, enums,
traits, impls, public-before-private functions, then cfg(test) module last.
Move these existing unit structs before impls, keeping their docs/attributes:

| File under crates/fabelgeist-compute/src | Declaration |
| --- | --- |
| map/mod.rs | Map:383 |
| gather/mod.rs | Gather:302 |
| scatter.rs | Scatter:280 |
| stencil.rs | Stencil:821 |
| stream/mod.rs | Stream:277 |
| advect/mod.rs | Advect:213 |
| gradient/mod.rs | Gradient:167 |
| divergence/mod.rs | Divergence:156 |

AdvectPipelineCache at advect/mod.rs:15 follows AdvectSignature; move the alias
before structs. GPU compute/signature.rs:137 private split_wgsl_template_args
precedes public parse_base_type:159, parse_binary_op_signature:191 and
parse_compare_signature:211; put it after public functions. Compute lib.rs
re-exports:66-101 follow mods:28-61 and cfg(test) test_utils:63-64. Re-exports go
before mods and test_utils last. Preserve all cfgs, visibility and exports. This
is item movement, not justification for native/fixture body changes.

## P3: exact cohesive Result-alias locations

GPU compute/signature/coordinates.rs:54 from_native explicitly returns
Result<Self,CoordinateParameterError>. This focused module has one fallible
error family. Add an owning descriptive CoordinateParameterResult<T> alias
before structs and export with the owner if public. GPU compute/
resource_addressing.rs:83 admit_extent explicitly returns
Result<(),TextureAddressError>; similarly use an owning TextureAddressResult<T>
and the appropriate re-export. Names are implementation choices; don't shadow a
wider prelude Result accidentally.

Do not force narrow same-error aliases onto mixed-error APIs. ResourcePipeline
crosses Naga, entry/workgroup/stride and resource errors. ComputePass crosses
binding/grid/pipeline/native-submission errors. Their wider Result boundaries
do not establish a single new narrow semantic error owner. Preserve distinctions.

## Source-backed connected contracts

Map: new validates source; build/execute reparse current source. Reserved index
and size use CoordinateParameter. With actual input, the first ordinary user
parameter is primary input; without input all are extras. Actual PassParameters
resource membership selects secondary resources; other values become uniforms.
Input binding0, output1 and secondary bindings from2 retain authored order.
Call order follows authored parameters. Input Buffer fetched using output texture
coordinates reports LinearIndex; scalar texture size reports LinearPopulation.
ResourceFetch pairs generated source with address requirement; TextureValue owns
native lanes. Execute derives Buffer element type from the signature and texture
descriptors from actual formats, inserts resources, gets output workgroups and
propagates fallible ComputePass dispatch.

Stencil: Resource mode removes input/secondary function arguments into globals;
coords/uniform arguments remain. Neighborhood mode rewrites native generic type,
emits matching structure and boundary lookups, and uses center/neighborhood/user
roles in authored order. Keep1D left/right/prev/next,2D vertical aliases,3D
front/back and4D ana/kata/w/past/future fields. Center is optional. Coordinates
remain distinct from neighborhood dimension. Actual input/output descriptors
flow to output-controlled dispatch. Missing multidimensional Buffer extents
currently default to64 axes; native i32 grid arithmetic is a separate pending
extent-policy family. This record repair does not authorize removing defaults,
changing supported kernels or speculative numeric rewrites.

ResourcePipeline retains immutable native pipeline, controlling layout and
reflected workgroup shape from the same generated source/named entry. Actual
array stride determines complete Buffer records. Texture kind/projection and
actual axes control workgroups. TextureAddressing distinguishes last flat index
(population minus1), full population and independent coordinates. Do not impose
a universal flat-volume cap or substitute Sort counts for launch quantities.

Named recipes retain actual descriptors and sorted SecondaryResources cache
identity. Binding order remains authored order, not sorted cache-key order.
ComputeSourceCache keys include complete source, actual Device, validation
observation policy and recipe; failures never publish values. ComputeShader
parses/validates Naga and creates native module. ComputePipeline enforces device,
entry/reflection and native outcome/cache contracts. ComputePass validates
Grid/device, compiles/binds/records and submits fallibly; success is not proof of
GPU completion. Preserve those existing owners and error propagation.

DistanceField wrappers require original declarations/order. JFA2D/3D preserve
index/size, primary value and step uniform. Simplex/Perlin generator calls have
no primary input and keep authored native declarations. Both extractors receive
shared EmissionKind-driven indirect source, passing dummy primary input plus
actual inclusive_offsets/indirect resources and typed capacity through Map.
Those globals are authored outside fn map; do not turn them into new function
parameters or rewrite native side effects.

## Connected acceptance and limits

Before expanding Stencil (1082 total lines, production through922) or the giant
build_pipeline (:210; cache method starts:794), extract real responsibilities:
admitted ordered signature/parser, neighborhood schema/emission, and cohesive
binding/coordinate/call emission. Map has495 total lines and build_pipeline
starts:133. No include fragments or pass-through splits to game limits. Preserve
native programs, policy, errors and fixtures. Apply relevant existing baseline
reductions through authorized hosted checks; never raise ceilings. No scanner
ran here; these source observations aren't authoritative checker output.

Acceptance: shared named native parameter owner; one immutable ordered record;
exact closed neighborhood decoder/schema; no independently mutable duplicate
name/user/coordinate/mode arrays; all parser/codegen/descriptor/execute/wrapper
consumers migrated; item-order/aliases repaired. Existing Map helper
map/tests/helper.rs:18-21 and Stencil helper stencil.rs:952-958 need API access
adaptations only. Preserve all existing cases, inputs, oracles and assertions;
add no tests, cases, helpers, assertions or doctests. No compatibility shims.
Re-review complete resulting source, not searches or historical verdicts.

This report establishes API/schema/rule gaps and does not assert a new runtime
failure. Native extent/Grid and other Root migration families stay open. Creator
strip's earlier b11d scope isn't freshly reviewed here and grants no global
clearance. Final thirteen-lens waves, independent three reviews and hosted checks
must all pass on the same final pushed SHA.

## Current source fingerprints

SHA-256; paths relative to Root above. Whole-file hashes also identify files
where only a named owning region was in this investigation's source scope.

- `crates/fabelgeist-compute/src/map/mod.rs`: `6bac7f1ff95532004d88a383de80e559f550f4e8d3223d5adbbffd50a8907071`

- `crates/fabelgeist-compute/src/map/recipe.rs`: `2b74f16ffced873fe963fa9e529ae410a73f41140d10334818b26c8663bef420`

- `crates/fabelgeist-compute/src/map/tests/mod.rs`: `b5a49e81728272b5bb04064a40395b62442969f9e4fcf10dbb0d13e1149d30e9`

- `crates/fabelgeist-compute/src/map/tests/helper.rs`: `307a22684a265bcbdd41fc0b7b038b9cdec839b1988eed7f66b8390fee2e6a5d`

- `crates/fabelgeist-compute/src/stencil.rs`: `a5552583fdd75934fc3f6f9623ca45fc7de5f4ae2def11544c19d984dc09cdf6`

- `crates/fabelgeist-compute/src/stencil/recipe.rs`: `a30f08a0e95afb9c0573bd1633f846099d18d5664fecd4107b98ef3927a1015a`

- `crates/fabelgeist-compute/src/distance_field.rs`: `2e0eea32b384b061a6b1bbb3162b077479bfefde32e196a8978e66d6c3b84b69`

- `crates/fabelgeist-compute/src/distance_field_jfa.rs`: `64de20033ee6f686c0877938844c0456dc46748bba806d4fcc2139583fb95b42`

- `crates/fabelgeist-compute/src/simplex_noise.rs`: `9b30738361080478eafcde92beb4b5c2c3e62df3062bd73012ee499a70dc5b4e`

- `crates/fabelgeist-compute/src/perlin_noise.rs`: `6e86661c9b0bbfec101b4d0ff7ec68167a7c1c35591f6dd4f7abcd0c4102d092`

- `crates/fabelgeist-compute/src/dual_contouring/mod.rs`: `22625a943934af4f763dc3466fc3da94d759ce5e1e77637076a88633cb48238b`

- `crates/fabelgeist-compute/src/dual_contouring/execution.rs`: `e54c3de9b89070020b775d89c1e655d2063fc4005dbc8412e1cc9542ab6d46ca`

- `crates/fabelgeist-compute/src/marching_cubes/mod.rs`: `20e8ff76b8ffe77cd50150082f3acf9cdfb56166be89945786e94c147151d31b`

- `crates/fabelgeist-compute/src/extraction/wgsl.rs`: `f4ab8a5471198e041691d776086f0c51f0ed693d8bbf64b81dd985bc30f2bc04`

- `crates/fabelgeist-compute/src/resource_pipeline.rs`: `d6532f73e5fd6aac8d83d84f4e56ef46191d98982397d0d90a928326bce80eeb`

- `crates/fabelgeist-compute/src/resource_recipe.rs`: `620d3f2477548bfff9c9c15e5ba64e534a9ee2e2a756d1b1dbeeff64dd792ac2`

- `crates/fabelgeist-compute/src/gather/mod.rs`: `95da5cd7a5e390cb9ea21c4dd61be55d324726a9d1dfef9c9bc769629318bfd9`

- `crates/fabelgeist-compute/src/scatter.rs`: `20e9e3f15f7e286e98018ec8bba3d8e4d430e24a13d800d2fd8246f8f0c50932`

- `crates/fabelgeist-compute/src/stream/mod.rs`: `5d8fdfbb20b5107dc061d22f3bae3611904ea4828ec4ee38fd47ce3518e5ba6e`

- `crates/fabelgeist-compute/src/advect/mod.rs`: `4b94a0b3a1a1e51b108608d28ca28fda6178e366ab2c8f49980a4968e2aff721`

- `crates/fabelgeist-compute/src/gradient/mod.rs`: `bca4af50d86428b57eb0834e28e9879da517e4146b2bd71fa36598f2004f0e97`

- `crates/fabelgeist-compute/src/divergence/mod.rs`: `d56ea387e7ea0d2c31ce000287ca0932029581a9e794fa0d54207dad76774c38`

- `crates/fabelgeist-compute/src/lib.rs`: `c04feb5e8852212086b588c9946c8b191bb96ff90814d1278192e7b4bce73d07`

- `crates/fabelgeist-compute/src/prelude.rs`: `68d9ab3add92c4324cd578477317e07ec1d8928dd58d212e8e53f9aeef8926b4`

- `crates/fabelgeist-gpu/src/data/gpu/compute/mod.rs`: `7e21b07871659c611dbd1aa3db5fbcc4323a55adc4183f8aa3d1d4ba4cac84c3`

- `crates/fabelgeist-gpu/src/data/gpu/compute/signature.rs`: `991f1eebf5b1a9cf7f8fa7314793b2610a78fa7f5b01236a19c84ea9f73773fd`

- `crates/fabelgeist-gpu/src/data/gpu/compute/signature/coordinates.rs`: `505a16459267c2280842514d42eb3aded371b6a2a84584ba6faf9ebb11c4ca2f`

- `crates/fabelgeist-gpu/src/data/gpu/compute/resource_addressing.rs`: `3fc5be3aee86a4b639de572e8309ffeea38a34d0157e5533252e9b15919e9503`

- `crates/fabelgeist-gpu/src/data/gpu/compute/source_cache.rs`: `686fa8c6510d57714063bb26f287a16c170e690ea25695abd94078041e1d9836`

- `crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/compute_shader.rs`: `f05e7916ac7e0e1474973913f4628d94cd99a9ca69d900d1306101f3286ce169`

- `crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/mod.rs`: `13cfda2ca90d3fb38739dd6564b412dfb99af3289242b9e277ee8e840baee8d3`

- `crates/fabelgeist-gpu/src/data/gpu/compute/pass/mod.rs`: `b9123559b221216745a20cd4f2881f91601174ffa8a6da1d51f07dc656c4cbe3`

## Supplemental full shaped-parser source investigation

After the initial report, read the complete current Gather, Scatter and Stream
files, Stream call emitter and existing Stream fixture. This supersedes the
initial declaration-only limitation for those three source files; it does not
claim complete caller-family runtime coverage or final clearance. Sources remain
b11d; the separate dirty Mesh error-file repair does not affect these hashes.

The single private ordered parameter/role authority repair belongs to all five
shaped parser families, not only Map/Stencil. GatherSignature at gather/mod.rs
:14-20 exposes declarations, names, index and type projections independently.
ScatterSignature at scatter.rs:10-16 does likewise. Scatter call_arguments:31-47
zips declarations and names and reparses colon-split type text. StreamSignature
at stream/mod.rs:10-18 additionally retains has_offset and has_input_val bools
beside its declarations; build_pipeline:148 and stream/call.rs:8 independently
pair names with raw argument entries and reclassify their native type text.
Those stored flags and order/type/name projections can disagree via public
mutation. Derive them from one admitted ordered role collection; boolean query
methods over that collection are ordinary predicates, not parallel state.
Actual offset remains a specific semantic role of native u32, distinct from an
index or input value despite equal underlying type. Preserve original native
resource declaration strings while assigning typed roles once at admission.

### P2: Scatter drops unsigned value roles after selecting an index

At scatter.rs:124-140, every non-resource u32/vec2<u32>/vec3<u32> enters the
index-shape branch. If an index already exists and this parameter is not named
index, the branch does nothing. It never sets has_value_param or the input type.
Then :143-147 rejects the program for having no input value. Static example:
a normal scatter function with explicit `index: u32`, followed by `value: u32`
and a `Resource<u32>` output. The output resource supplies the output type, but
the value is discarded by role classification and new()/parse_signature fail
before native WGSL. The same holds for vec2/vec3 unsigned values. No program or
new test/case/helper was created or run to establish this control-flow fact.

This is a demonstrated parser classification defect, not merely an unsupported
feature proposal: ResourceBaseType explicitly includes U32 and unsigned vector
forms, Scatter's Buffer fetch preserves the supplied element type, and its
call_arguments maps non-index/non-resource parameters to in_val without a
float-only restriction. Stream's equivalent branch at stream/mod.rs:109-117
already classifies the remaining unsigned-shaped parameter as input value.
There is no source-declared Scatter float-only policy; the rejection is caused
by conflating coordinate shape with semantic role. No existing repository caller
or fixture was found exercising unsigned Scatter, so no current application
runtime regression is claimed. The defect is source-proven admission/API
behavior, not an executed GPU validation result.

Repair role selection in the common ordered-schema family: explicit coordinate
role is determined by its protocol name and type; a remaining unsigned ordinary
parameter is a value, using existing ResourceBaseType. Retain the supported old
alias-index heuristic when no explicit index is present rather than making all
index aliases unsupported. If an explicit index appears later, do not lose the
earlier unsigned value by selecting it permanently as a provisional index;
admission can first establish explicit role ownership and then apply the alias
fallback only when absent. Do not broaden unrelated parser grammar or invent
new scalar/vector features. No extra tests/cases/helpers/assertions are
authorized; preserve and adapt existing fixtures only. Record this additional
classification repair in the complete pre-edit inventory, then re-read the
whole resulting parser/codegen/execute family and existing caller sources.

- `crates/fabelgeist-compute/src/stream/call.rs`: `a164da4c7d62a4eb24a6a3b0e1d6f0a15851ed90747e5af07dd9f43b08b941b9`

- `crates/fabelgeist-compute/src/stream/tests/mod.rs`: `16f48fa721c0981cf7799288e7e4e898e1f6e2481b312bce73487c10070da743`

# Complete shaped-parameter discovery inventory at12a14b478

This section supersedes earlier coverage limitations for the named current
Map/Gather/Scatter/Stencil/Stream signature family. Source snapshot is clean
`12a14b478506044f85a062f07aa05855cae0c8c7`. This completes source discovery and
candidate disposition for that named parser/record/emission/caller family; it
is not repair verification, runtime acceptance or global PR clearance. Earlier
sections remain historical discovery, not a current clearance claim.

Full reads now cover all five parser/build/execute files, Stream call emitter,
all existing Map/Gather/Stream and inline Stencil fixtures/helpers, test_utils,
DistanceField/JFA/Simplex/Perlin, both extraction constructors and full MC/DC
execution modules, shared indirect source, entire marching-cubes WGSL and all
four dual-contouring WGSL files. Shared signature/coordinates/TextureValue,
ResourceDescriptor fetch/binding, ResourcePipeline/recipes/source cache and
current native entry/reflection/uniform/image/pipeline/pass boundaries were read.
Additional source reads below establish real protocols, not search verdicts.
Existing direct fixtures were read, not executed or expanded. No source edit,
project check, build, lint, scanner, probe, test, generator or external post.

## Current contracts and native protocols

| Family | Source admission and ordered roles | Actual receiving control |
| --- | --- | --- |
| Map | map/mod.rs:55 parses a required-return fn map; index/size are reserved CoordinateParameter; all other named typed values retain authored order | Optional actual input selects the first ordinary value as primary. Without input it remains an extra. Actual PassParameters resource membership partitions other extras; output controls dispatch |
| Gather | gather/mod.rs:52 parses required-return fn gather; explicit named index wins, otherwise supported unsigned-shaped alias fallback; Resource/ptr/texture declarations supply input element metadata and are stripped into native globals | Actual input descriptor is retained; authored global input supplies data. Actual output determines workgroups and texture store lanes |
| Scatter | scatter.rs:74 parses void fn scatter; index and ordinary input value are distinct even when both native u32/vector-u32; output resource declaration supplies element metadata and is stripped | Actual input drives workgroups and in_val load, output is actual target. Existing unsigned-role drop is actionable |
| Stream | stream/mod.rs:50 parses void fn stream; index, native-u32 offset, optional ordinary input value, and stripped output resource are distinct | Buffer inclusive_offsets controls prefix dispatch; textures use actual input axes. counts and inclusive offsets describe same population; authored writer owns output bounds |
| Stencil | stencil.rs:71 parses required-return fn stencil; exact index/size; Resource mode or exact Neighbors1D..4D neighborhood with optional center and extra typed values | Actual input/output descriptors and secondary membership control global bindings; output controls dispatch. BoundaryMode owns Clamp/Wrap, not a raw word |

This table names supported roles, not a promise that every parsed combination
compiles. Definitions are mutable source holders: new parses a signature, build
and execute parse current code again, Naga validates the generated program, and
native pipeline/binding admission checks actual resources. Empty Default source
is an existing deferred source-holder state; it is not an admitted compiled
program. Do not accidentally turn Default into proof that empty source executes.

ResourceBaseType is the existing native value vocabulary, including numeric
lanes, texture handles, sampler and open Custom names. clean_type_name retains
some native forms and projects Resource/array/ptr wrappers to their element,
including `ptr<storage,array<Vertex>,read_write>` -> Custom(Vertex). It is not a
full WGSL type checker. Actual alias/custom declarations remain in source and
can be valid Buffer programs while TextureValue correctly rejects unsupported
numeric texture projection. Do not remove CustomBuffer support or claim all
Custom types are numeric texels. Do not add a new language or unrelated grammar
restrictions while replacing stored role records.

Real MC/DC code uses gather(index:u32)->u32 over native global input plus
already-authored uniform/storage globals. Its stream signatures use
index:vec3<u32>, offset:u32, and `_res:ptr<storage,array<Vertex>,read_write>` or
array<u32>. The pointer is output-type metadata and is stripped; functions write
the canonical native global output, not a renamed local `_res`. Their native
uniform globals must not be reinterpreted as newly added function parameters.
The actual host execution supplies counts/offsets and extra table/cell/vertex
resources and typed capacities before each call. Preserve their complete built-in
program source and emission behavior during the AST migration.

DistanceField generate_min/smooth_min preserves original raw declaration text
and original call order when adding its primary in_val and optional smin_k.
Do not canonicalize its wrapper declarations from ResourceBaseType.as_str:
projection can erase aliases/wrapper spelling or spacing. Keeping immutable
verbatim declaration text associated with the admitted role record is justified.
This lexical preservation is not proof that every Resource wrapper passed through
Map is a valid final WGSL function; native validation remains authoritative.
JFA's2D/3D programs use index/size, actual primary values and a step uniform.
Simplex/Perlin use Map as generators with no primary resource and preserve their
actual scalar/vector declarations and authored shader bodies.

Current parser limitations remain visible: parameters are angle-depth comma split
and require two colon-separated parts; function extraction is regex-based.
CoordinateShape supports exact u32/vec2<u32>/vec3<u32> forms, not arbitrary alias
resolution. Stream offset is exactly native u32. Missing required returns and
Scatter missing value already fail early. Unsupported malformed grammar, duplicate
identifiers, unsupported resource/scalar combinations and shader body errors are
also subject to Naga/native failure. The repair should deliberately classify
reserved protocol roles once, preserve successful old protocols and make any
earlier validation/error-order change explicit. It should not invent support
for arbitrary additional arguments or require newly fabricated bounds.

## Additional demonstrated same-family precedence defect

Stream parser:101-124 initially treats the first unsigned-shaped parameter as
an alias index if none exists. A later explicit index replaces it without marking
the earlier unsigned parameter as a value. E.g. ordinary value:u32 before
index:u32 and offset:u32 leaves has_input_val false, while call.rs:8-37 still
passes in_val for that first parameter. build_pipeline:226-241 emits no in_val
load. This is a source-proven parser-to-codegen inconsistency, not an executed
GPU result. No existing application caller/fixture was found exercising that
ordering, so no current runtime regression is claimed. It joins the already
source-proven unsigned Scatter rejection finding.

For Scatter and Stream, establish any explicit coordinate role first; apply the
old supported alias-index fallback only if no explicit index exists. Remaining
ordinary unsigned parameters then retain value roles. Preserve no-index float
value and explicit index/offset/resource-only Stream programs; optional input
must not become mandatory. Gather has no ordinary input-value role: do not turn
its otherwise unsupported extra parameter into a new Gather feature by analogy.
Map index/size are reserved by name and do not use the alias fallback. Stencil
uses its own reserved coordinate/center/neighborhood classification.

## Connected repair units before implementation

R1: one private ordered role/declaration vector per signature; shared native
ShaderParameter{name,value_type:ResourceBaseType} at GPU signature owner where
applicable; reuse CoordinateParameter without a second stored name. Derive
coordinate/value/resource/name/call-order queries and Stream presence predicates.
No public mutable duplicate vectors, independently retained flags or new stored
signature beside mutable Definition.code. Keep raw declaration text private,
immutable and associated atomically with each admitted semantic entry.

R2: own Stencil's Resource/Neighborhood schema and exact1D..4D dimension in typed
role payloads; center remains optional. Classify exact public native spellings,
not any Custom identifier containing Neighbors. Unknown family spellings cannot
silently become2D. Do not duplicate neighbor dimension/type/name as separately
mutable aggregate fields; no compulsory center. Preserve existing supported
center/val/fallback behavior and neighbor field aliases. Decide ambiguous duplicate
role errors explicitly rather than accidentally generating a new multi-center
or multi-neighborhood feature.

R3: close unsigned Scatter/Stream value and index precedence in admission and
call emission as one family, preserving valid alias-index protocol. Derive all
call arguments and value-load decisions from the same ordered roles; do not
reparse declaration strings at each consumer or reconstruct positional pairs.
Map primary input selection must remain dependent on actual input presence,
not a permanently assigned parse-time primary role. Stencil and Map secondary
binding classification must continue consuming actual parameter-resource
membership; it is not determined solely by a native type-name substring.

R4: migrate every actual wrapper, descriptor and existing fixture access to the
new direct API. Definition constructors that only parse source currently take
unused `_context` in Gather:44, Scatter:66, Stream:42 and Stencil:53. Removing
that unused device parameter directly is a proportionate API cleanup candidate,
not a requirement to change native admission or a new device abstraction. If
chosen, update current extraction constructors/fixtures directly, no old forwarding
constructor. Keep heterogeneous anyhow boundaries and exact cause propagation.

R5: correct prescribed item order/Result aliases already located above, derive
the five mechanical Definition Defaults, and extract real parser/neighborhood/
binding-emission responsibilities before expanding existing giant builders.
Apply relevant hosted size-baseline reductions without raising ceilings. No
pass-through/include splits. Existing Definition PartialEq intentionally ignores
cache identity and remains manual. Native generic ComputeSourceCache Default
must remain unconstrained rather than deriving new recipe/value bounds.

R6: retain actual native owners and emission behavior: ResourceBaseType,
CoordinateParameter/Shape, TextureValue, ResourceFetch/TextureAddressing,
ResourcePipeline reflected stride/shape, DispatchResource, complete descriptor
recipes, native Device/source/validation-policy cache keys, actual fallible
ComputePass submit. Sort populations never become launch populations. Preserve
all built-in program source/fixtures except direct API access adaptations.

## All thirteen source-backed lens dispositions

### 1. Producer propagation — incomplete, actionable R1/R3/R4/R6

All five parse producers create correlated native-name/type/order projections;
Map/Stencil additionally carry primitive tuples into binding partitions. Stream
call helper and Scatter call method parse raw declaration types again. New shared
named native leaves must be produced at the signature boundary and retained
through parse, build, execute/descriptors and DistanceField wrappers. Real MC/DC,
JFA/noise and fixture consumers above complete the actual receiving inventory.
Acceptance: one admitted record reaches every consumer; extraction globals and
actual resource membership retain current behavior; no primitive extraction and
readmission or legacy adapter. Depends on R2/R3 role admission and R6 native owners.

### 2. Existing type owners — reuse, actionable R1/R2/R6

CoordinateParameter already owns native identifier plus admitted CoordinateShape;
ResourceBaseType owns native value forms; TextureValue owns numeric lane projection;
ResourceFetch pairs source and addressing; DispatchResource owns controlling
binding role. A shared ShaderParameter is justified only for native name+value
records missing today, not an equivalent coordinate owner. Neighborhood4D cannot
be squeezed into CoordinateShape's unsigned3D vocabulary. Acceptance: consumers
import canonical leaves, no local competing wrappers, names remain open native
text and roles stay local to actual algorithms. Depends on shared-owner visibility
and exact role contracts, not equal underlying numbers.

### 3. Admission contracts — incomplete, actionable R1/R2/R3/R4

Every constructor/new parse, build/execute reparse, public-field mutation and
Default route was inventoried. There is no signature Deserialize/reflection
route. Current public fields admit contradictory records; defaults admit empty
source holders, not compiled programs. Unsigned Scatter and Stream ordering
control-flow defects are concrete. Exact neighborhood protocol needs typed
admission; optional center and generic native text remain legitimate. Acceptance:
private aggregate construction, no mutable bypass, direct API users adapted,
unsupported/duplicate grammar policy and intentional early-error differences
recorded. No invented finiteness/extent bounds or support expansion. Depends on
R3 precedence and native Naga validation remaining the final shader admission.

### 4. Invariant operations — incomplete role operations, preserve R3/R6

Derived iteration/split/partition/call generation must preserve the same admitted
order and role, avoiding arrays[i] relationships across independently mutable
columns. Current indices backed by nonempty checks or immutable HashMap membership
are proven; do not invent missing-element errors for them. Existing scalar
flattening, texture size population and reflected Buffer extent arithmetic have
their own checked owners. Acceptance: unsigned input loading and call role agree,
no stale signature cache, no changed numerical kernel/boundary formula. Stencil's
64-axis defaults/i32 extent arithmetic stay a separate pending extent-policy
family, not cleared or rewritten by R1. Depends on R1/R2 and actual resource kind.

### 5. Aggregate states — incomplete, actionable R1/R2/R3

Map/Gather/Scatter/Stream declaration/name/coordinate/type arrays and Stream flags
can disagree under public mutation; Stencil adds bool/dim/type/optional-payload
combinations. One private ordered record closes those relationships. Immutable
verbatim lexical projection can coexist with the typed role only when both are
admitted together and no independent setter is exposed. Recipe/resource pipeline
records are already coherent immutable source-associated state. Acceptance: no
independently mutable role/order/type projection and no newly cached signature
beside public mutable code. Constructor/query APIs enforce single actual role
selection; ambiguous duplicate roles receive an explicit policy. Depends on R2/R3.

### 6. Type ownership — actionable R1/R2/R4/R6, native dependencies respected

GPU owns reusable native type/coordinate/fetch vocabulary; Compute owns algorithm
roles and authored source transformations. Actual native pointer/resource strings
are boundary syntax, not replacement launch/count owners. Existing native
pipeline/pass functions remain peer-owned; this investigation authorizes no edit
or competing implementation there. Parent states signature/coordinates/
TextureValue/resource-addressing owners are under explicit Root agreement.
Acceptance: lowest shared owner, direct constructor APIs, no pass-through parser
module or shared manifest overwrite. Native receiving integration stays pending.

### 7. Redundant authority — incomplete, actionable R1/R3/R6

Stored param_names, rawargs semantic reparsing, index/size projections,
Stencil mode/dim/name copies and Stream has_input_val/has_offset are competing
facts. Raw declaration text is needed only for exact lexical emission; source
code remains the editable Definition authority and is reparsed. SecondaryResources
sorting is legitimate canonical cache identity, not authored binding order.
Acceptance: no stored role mirrors; query predicates derive from one ordered
collection; source/device/policy cache keys remain complete and failures do not
publish a successful ResourcePipeline. Depends on current build/use flow staying
source-associated, not adding a second cache for parsed state.

### 8. Behavior preservation — preserve supported paths, R2/R3 changes explicit

Preserve Map optional-primary/generator/extra resources/uniforms, all existing
coordinate shape handling, Gather globals, Scatter value loading, Stream
optional input/index/offset/output pointer and prefix behavior, Stencil Resource
and neighborhood aliases/optional center/Clamp/Wrap. Exact wrappers and pointer
spellings matter to real consumers. The two unsigned-role corrections and unknown
neighbor-default elimination intentionally change erroneous admission; do not
claim exact error timing for unsupported inputs. Acceptance: source equivalence
of native templates/built-in caller bodies and all existing fixture case/oracle
bytes, allowing only direct API use adaptations. No new tests, cases, helpers,
assertions or doctests. No runtime result is claimed here. Depends on all units.

### 9. Enum shapes — actionable R1/R2/R3

Index/size/value/resource/offset/center/neighborhood are named closed semantic
roles, not primitive indices into a second table. Stream presence booleans are
replaced by derived predicates; Stencil bool/dim/options become payload-bearing
schema. A boolean returned by has_input() over actual roles is an ordinary
predicate, not a new carried state. TextureAddressing and BoundaryMode already
fit. Acceptance: no two-state Boolean or raw dimension travels between parser
and emitter; exhaustive typed dimension dispatch retains actual native aliases
and dimension policy. Depends on R1 and exact Stencil admission; do not introduce
new enums for arbitrary identifiers, custom types or source bodies.

### 10. Derivability — Defaults actionable; GPU errors dependency decision

Five Definition Defaults are mechanical: code:String empty, cache's existing
Default, plus Stencil BoundaryMode::default Clamp. Derive built-in Default and
retain exact state. Advect/Gradient/Divergence Defaults have the same adjacent
candidate; Advect decode/clamp is already separately repaired, not undone.
Definition PartialEq is intentionally semantic code(+Stencil boundary) equality,
excluding cache, so manual implementations remain. ComputeSourceCache generic
Default is necessary to avoid new parameter/value Default bounds. Existing
TryFrom/From/native classification is semantic, not mechanical derive material.
Existing private recipes/native type records already derive appropriate traits.

CoordinateParameterError:74/:84, TextureAddressError:32/:47 and TextureValueError
:87/:97 have mechanical Display/Error implementations that thiserror could
express, retaining actual strum::ParseError (coordinate), TryFromIntError
(address projection), and exact messages/source behavior. However actual
fabelgeist-gpu/Cargo.toml and both lock package dependency lists have NO direct
thiserror edge. This is a concrete optional dependency/ownership decision, not
an unconditional available-local-derive rule violation. Keeping the small manual
errors without a new dependency is a valid recorded disposition. If Root chooses
the derive, first inventory/coordinate the shared GPU manifest and both lock-edge
changes; reuse already-resolved thiserror2.0.20 without version/package churn.
Do not silently assume transitive availability, lose the coordinate parser source
or overwrite peer native error owners. Acceptance includes explicit decision and
unchanged Display/cause chain. No serialization is introduced merely for derive.

### 11. Type granularity — R1/R2, avoid needless distinctions

Shared ShaderParameter represents an actual native name+type pair. Local role
records carry algorithm-specific meaning; do not create one generic all-algorithm
role enum with invalid variants for consumers or five equivalent native leaves.
Do not add identifier branding/bounds without an actual invariant. Dimensions
and coordinate shapes differ; native output element, resource kind and launch
shape also differ. AdvectSignature remains unused public metadata and is not
actual pipeline admission; remove directly only within its inventoried adjacent
family. Acceptance: smallest useful owners, no fake checked provenance for native
text or index, no compatibility synonyms. Depends on R1/R2/R6 and direct callers.

### 12. Magic strings — classify once, preserve actual text R1/R2/R3/R6

Function names map/gather/scatter/stencil/stream and reserved index/size/offset/
center/val/Neighbors spellings are source protocol decoded at admission into
closed roles. Resource<...>, ptr<...>, texture_* and concrete generated neighbor
spellings are native grammar, not human prose; do not reinterpret arbitrary
Custom names containing those fragments. Native variable/binding/entry names,
shader bodies, declarations, format templates, authored globals and regex tokens
remain actual source strings. Errors are presentation only. Acceptance: emitter
consumes admitted roles without branching on mutable diagnostic/Debug text or
reparsing competing declaration columns; lexical emission preserves supplied
source and native names. Any grammar tightening beyond exact role ownership
requires explicit disposition, not a new speculative language.

### 13. Magic numbers — preserve native math, separate extent family R2/R6

1..4 neighborhood dimensions own a typed closed protocol; coordinate lane
shapes already own their allowed forms. Vector arities/texel lane padding,
zero/one initialization, neighbor +/-1 offsets and native binding numbers are
intrinsic native emission. Current Map/Stencil workgroup shapes64x1x1,
16x16x1,8x8x4 and Gather/Scatter/Stream64x1x1,16x16x1,4x4x4 remain source-native
declarations; ResourcePipeline reads actual Naga shape and actual reflected
stride before deriving launch quantities. Preserve those declarations, padding,
loop/formula/constants and actual fixture values; no policy bounds or tuning
changes. Stencil64 missing-axis Buffer defaults and related i32 addressing are
separately open. Acceptance: no numeric change beyond typed dimension matches,
no Sort count substituted for WorkgroupGrid/InvocationCount, no raised baseline,
and no claim that role AST repair certifies native extent arithmetic.

## Dependencies, unread limits and final connected acceptance

Discovery here covers the named shaped parameter roles and receiving source
protocols. It does NOT freshly audit MC/DC geometric correctness/table contents,
all extraction tests, general shape/Grid/physical policies, all texture/native
creation methods, host uniform-packing implementation, all native shader backend
execution, or future incoming peer pipeline changes. Current reflection/native
boundaries establish handoff responsibilities, not whole-native-family clearance.
No external application using these public APIs was inspected; breaking direct
API updates are intentional per repository policy. No compile/runtime result or
new test is substituted for missing source acceptance.

Before implementing R1-R6, Root must merge this candidate inventory with its
all13 ledger, including any shared manifest/derive dependency decision and exact
native owner agreement. All source holders, wrappers, current descriptors,
concrete call sites and existing fixture accesses must finish together. Remove
competing raw roles and compatibility paths; preserve native code/values and
supported boundaries. Then independently re-read complete resulting sources,
repeat all13 waves, run the three reviews and required hosted checks on one final
pushed revision. Existing isolated approvals do not clear that future composed
family. No partial-family push/CI or merge is authorized by this discovery.

## Current12a14b478 SHA-256 source identities

- `crates/fabelgeist-compute/src/map/mod.rs`: `6bac7f1ff95532004d88a383de80e559f550f4e8d3223d5adbbffd50a8907071`

- `crates/fabelgeist-compute/src/map/recipe.rs`: `2b74f16ffced873fe963fa9e529ae410a73f41140d10334818b26c8663bef420`

- `crates/fabelgeist-compute/src/map/tests/mod.rs`: `b5a49e81728272b5bb04064a40395b62442969f9e4fcf10dbb0d13e1149d30e9`

- `crates/fabelgeist-compute/src/map/tests/helper.rs`: `307a22684a265bcbdd41fc0b7b038b9cdec839b1988eed7f66b8390fee2e6a5d`

- `crates/fabelgeist-compute/src/stencil.rs`: `a5552583fdd75934fc3f6f9623ca45fc7de5f4ae2def11544c19d984dc09cdf6`

- `crates/fabelgeist-compute/src/stencil/recipe.rs`: `a30f08a0e95afb9c0573bd1633f846099d18d5664fecd4107b98ef3927a1015a`

- `crates/fabelgeist-compute/src/distance_field.rs`: `2e0eea32b384b061a6b1bbb3162b077479bfefde32e196a8978e66d6c3b84b69`

- `crates/fabelgeist-compute/src/distance_field_jfa.rs`: `64de20033ee6f686c0877938844c0456dc46748bba806d4fcc2139583fb95b42`

- `crates/fabelgeist-compute/src/simplex_noise.rs`: `9b30738361080478eafcde92beb4b5c2c3e62df3062bd73012ee499a70dc5b4e`

- `crates/fabelgeist-compute/src/perlin_noise.rs`: `6e86661c9b0bbfec101b4d0ff7ec68167a7c1c35591f6dd4f7abcd0c4102d092`

- `crates/fabelgeist-compute/src/dual_contouring/mod.rs`: `22625a943934af4f763dc3466fc3da94d759ce5e1e77637076a88633cb48238b`

- `crates/fabelgeist-compute/src/dual_contouring/execution.rs`: `e54c3de9b89070020b775d89c1e655d2063fc4005dbc8412e1cc9542ab6d46ca`

- `crates/fabelgeist-compute/src/marching_cubes/mod.rs`: `20e8ff76b8ffe77cd50150082f3acf9cdfb56166be89945786e94c147151d31b`

- `crates/fabelgeist-compute/src/extraction/wgsl.rs`: `f4ab8a5471198e041691d776086f0c51f0ed693d8bbf64b81dd985bc30f2bc04`

- `crates/fabelgeist-compute/src/resource_pipeline.rs`: `d6532f73e5fd6aac8d83d84f4e56ef46191d98982397d0d90a928326bce80eeb`

- `crates/fabelgeist-compute/src/resource_recipe.rs`: `620d3f2477548bfff9c9c15e5ba64e534a9ee2e2a756d1b1dbeeff64dd792ac2`

- `crates/fabelgeist-compute/src/gather/mod.rs`: `95da5cd7a5e390cb9ea21c4dd61be55d324726a9d1dfef9c9bc769629318bfd9`

- `crates/fabelgeist-compute/src/scatter.rs`: `20e9e3f15f7e286e98018ec8bba3d8e4d430e24a13d800d2fd8246f8f0c50932`

- `crates/fabelgeist-compute/src/stream/mod.rs`: `5d8fdfbb20b5107dc061d22f3bae3611904ea4828ec4ee38fd47ce3518e5ba6e`

- `crates/fabelgeist-compute/src/advect/mod.rs`: `55748934ab8638b33ef380aae35e12c0ab61b888158a5e0b5de459c68495ba7a`

- `crates/fabelgeist-compute/src/gradient/mod.rs`: `bca4af50d86428b57eb0834e28e9879da517e4146b2bd71fa36598f2004f0e97`

- `crates/fabelgeist-compute/src/divergence/mod.rs`: `d56ea387e7ea0d2c31ce000287ca0932029581a9e794fa0d54207dad76774c38`

- `crates/fabelgeist-compute/src/lib.rs`: `c04feb5e8852212086b588c9946c8b191bb96ff90814d1278192e7b4bce73d07`

- `crates/fabelgeist-compute/src/prelude.rs`: `68d9ab3add92c4324cd578477317e07ec1d8928dd58d212e8e53f9aeef8926b4`

- `crates/fabelgeist-gpu/src/data/gpu/compute/mod.rs`: `7e21b07871659c611dbd1aa3db5fbcc4323a55adc4183f8aa3d1d4ba4cac84c3`

- `crates/fabelgeist-gpu/src/data/gpu/compute/signature.rs`: `991f1eebf5b1a9cf7f8fa7314793b2610a78fa7f5b01236a19c84ea9f73773fd`

- `crates/fabelgeist-gpu/src/data/gpu/compute/signature/coordinates.rs`: `505a16459267c2280842514d42eb3aded371b6a2a84584ba6faf9ebb11c4ca2f`

- `crates/fabelgeist-gpu/src/data/gpu/compute/resource_addressing.rs`: `3fc5be3aee86a4b639de572e8309ffeea38a34d0157e5533252e9b15919e9503`

- `crates/fabelgeist-gpu/src/data/gpu/compute/source_cache.rs`: `686fa8c6510d57714063bb26f287a16c170e690ea25695abd94078041e1d9836`

- `crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/compute_shader.rs`: `f05e7916ac7e0e1474973913f4628d94cd99a9ca69d900d1306101f3286ce169`

- `crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/mod.rs`: `13cfda2ca90d3fb38739dd6564b412dfb99af3289242b9e277ee8e840baee8d3`

- `crates/fabelgeist-gpu/src/data/gpu/compute/pass/mod.rs`: `b9123559b221216745a20cd4f2881f91601174ffa8a6da1d51f07dc656c4cbe3`

- `crates/fabelgeist-compute/src/stream/call.rs`: `a164da4c7d62a4eb24a6a3b0e1d6f0a15851ed90747e5af07dd9f43b08b941b9`

- `crates/fabelgeist-compute/src/stream/tests/mod.rs`: `16f48fa721c0981cf7799288e7e4e898e1f6e2481b312bce73487c10070da743`

- `crates/fabelgeist-compute/src/gather/tests/mod.rs`: `6a9360ebc741c55a8068c062857298ce9ee455bb10ce99202fd8f03cb71ee160`

- `crates/fabelgeist-compute/src/gather/recipe.rs`: `f9f1191683c0890a728199fbca3a598436d356ce1857b8ab96ccdafd354bebf8`

- `crates/fabelgeist-compute/src/test_utils.rs`: `d6a110a5317449bd4a0330749af0cb0433582763f40280c8294112203193cf6e`

- `crates/fabelgeist-compute/src/marching_cubes/wgsl.rs`: `fc4a7d68f8c8a443bbdc7c1fff67a047ccaa21ed56b315e6ce0805e3da89df93`

- `crates/fabelgeist-compute/src/dual_contouring/wgsl/mod.rs`: `8a6c40eef3069bc4c265b6136c6159abf092dc1181a23ae8c288666deba03af2`

- `crates/fabelgeist-compute/src/dual_contouring/wgsl/count.rs`: `39456912fbb2c7b6064b95fccdcfaa4aef7313d4658d3a75e9684447170022e5`

- `crates/fabelgeist-compute/src/dual_contouring/wgsl/vertex.rs`: `ccb22f4966c34d402b4ab7c82abd08bed44cb370bd65b758a6f8b4a05af9bc47`

- `crates/fabelgeist-compute/src/dual_contouring/wgsl/index.rs`: `99968238e06d234d0f27422093cd5dee4c2cb4bbcf179fdddf7d845369e1b8f8`

- `crates/fabelgeist-gpu/src/data/gpu/compute/signature/texture_value.rs`: `79124150e675e45b70b38b8287f82d80b7eda48202e0d2fa6e24f880bb153de6`

- `crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/entry_point.rs`: `2d646038056780254cfafb4d59d6ba66963cf820ca662df217ed58ff7527bde7`

- `crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/reflection.rs`: `28875e4c667ce2a16c41e3640b8948fcba6f90e399b45d649ddb43f0832f8b90`

- `crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/reflection/uniforms.rs`: `7c79e1c26e7c308c8e8a8e041db1764eeb135a9b2ac7a034c72cc979543d536c`

- `crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/reflection/images.rs`: `3130b402b58ceb84fa372a2d3b4108517795df791998c26dded8b5bed8d5d145`

- `crates/fabelgeist-gpu/Cargo.toml`: `707cc7a7a86fd855e779b3233b4dd97d3fd2be4496aa550a4388a1dec80d8a58`

- `Cargo.lock`: `e76fa8ed410f3a4c8609d2b7528761832a8103ba075659ecfb3d1cc85a268d69`

- `crates/adventuresim-character-creator/Cargo.lock`: `ca7c957fbd5d63fc3b507a0023e05ba2e74a3aef48c3dba54276013f7de6769a`

- `crates/fabelgeist-compute/src/boundary.rs`: `31cd64fb15c7bb2edcbc355da4483470f1f28ae2e7c061790aa97341abf137b8`
