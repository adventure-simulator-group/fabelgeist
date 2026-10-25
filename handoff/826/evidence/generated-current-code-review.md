# Independent generated-family current Code discovery

Frozen snapshot: `generated-transfer-c665-map-snapshot/manifest.json`, based on
`c6659f6cfa4610f066c659411ade6636e21bd3b5`. Actual Root native contracts are
pinned at `cac5a7c33a83d1c08a5e55d43d912967edf06814`. All65 exported identities
match the manifest. All27 owned files were freshly read completely, including
the newly started Map tranche (4995 lines). This is pre-repair source discovery,
not source compilation, whole-family acceptance or final clearance.

## Actionable findings

### GC-C1 — P2 — crates/fabelgeist-compute/src/scatter.rs:124

Known, confirmed still present in frozen snapshot. Explicit index:u32, ordinary
value:u32, output Resource<u32>.

Every unsigned scalar/vector enters coordinate branch; once index exists, later
unsigned value is discarded without has_value_param, so parse rejects a
supported input value.

Minimum connected repair: Select explicit supported unsigned index before alias
fallback; classify every remaining ordinary parameter as value. Preserve
index:f32 ordinary value behavior, last-value type and authored call order.

Acceptance: Explicit index plus unsigned value reaches native admission and
loads/calls that value; no new cases/helpers/assertions.

### GC-C2 — P2 — crates/fabelgeist-compute/src/stream/mod.rs:111

Known, confirmed still present in frozen snapshot. Ordinary value:u32 before
later explicit index:u32 and offset:u32.

The first value becomes alias index, then explicit index replaces it without
reclassifying earlier value. has_input_val stays false, build omits load,
call_arguments still emits in_val for displaced value.

Minimum connected repair: Resolve explicit supported coordinate before
classifying ordinary roles; derive load presence/type from final value roles.
Preserve no-value MC/DC writer metadata protocol and float parameter named
index.

Acceptance: Authored value/index order cannot remove the required load;
index/offset-only writers still emit no load.

### GC-C3 — P2 — crates/fabelgeist-gpu/src/data/gpu/compute/signature.rs:168

New connected source-confirmed admission failure. MapDefinition::new with
ordinary parameter native type vec2<; shared parse_base_type can also receive
texture_2d< or a truncated multibyte suffix. Stencil native type containing
Neighbors>bad< reaches a separately inverted slice.

Prefix-only guards then [prefix_length..len-1] can invert range or split UTF-8.
Stencil takes first less-than and last greater-than without proving order.
Public fallible source admission panics before Naga can reject malformed native
source.

Minimum connected repair: Use complete checked outer-envelope extraction before
recursion. Preserve open Custom native identifiers and malformed ordinary text
for normal error/native admission; exact reserved neighborhood admission may
return its normal typed error. Do not implement a second general WGSL parser.

Acceptance: All incomplete/reversed/non-ASCII outer forms follow normal
fallible/native error behavior without indexing panic; valid
nested/pointer/Resource/custom native spellings and successful shaders remain
unchanged.

### GC-C4 — P2 — crates/fabelgeist-compute/src/stencil.rs:586

New connected source-confirmed numerical failure. Neighbors2D<f32> Buffer
input/output of4096 records, no size, Clamp. Existing branch declares
default64x64. At output64 (row1,x0), emitted coords=(64,0), so left resolves
input63 instead of input64.

Buffer workgroups are X-only, but 2D/3D neighborhood coordinates directly use
global_id.xy/xyz. Bounds clamp the wrong coordinate after the first row; 3D
similarly never reconstructs y/z.

Minimum connected repair: Project actual flat output index into the existing
admitted/default multidimensional grid before center/neighborhood fetches. Keep
successful texture independent-axis behavior and existing64 policy until
separately justified; admit physical input/grid/count/signed arithmetic
relationships before native dispatch.

Acceptance: Complete supported Buffer row/plane crossings use the same layout
for center and neighbors; both boundary modes, logical prefixes and existing
texture paths remain consistent, without adding fixtures.

## Current Map and affected consumers

Fresh full current Map tranche source reasoning found no additional
implementation defect beyond shared GC-C3; this is not source compilation or
complete generated-family clearance.

Map admission, primary selection, authored binding/call order, resource
renaming, uniform unpacking, result padding and source/device/policy/recipe
caching retain the prior successful emission protocol. The shared lexer owns
immutable original declarations and local roles. Native wrappers/custom names
remain native validation concerns; a lexical record does not promise every
parsed DSL form is executable.

DistanceField now consumes MapSignature declarations/names directly, preserving
original lexical argument spelling and source order. JFA keeps typed
coordinate/size roles, first loaded value, step uniform, ping-pong/sentinels and
result conversion. Perlin/Simplex keep no primary input and ordinary unsigned
octaves. MC/DC Gather count and Stream metadata-only pointer protocols retain
authored globals and reflected custom native record stride; their complete
numerical geometry policies remain held by #780.

## Native timing and stale reference limits

wgpu-core29 command/transfer.rs925-958 enqueues
ArcCommand::CopyTextureToTexture. Actual try_raw/usage admission at1363-1437
occurs while encoding/finalizing, so Root context.submit finish scope can
observe it. wgpu backend immediate handle_error at2520-2538 alone does not prove
destroyed native resource fails at record time.

DistanceField raw encoder ownership/native error-policy closure still belongs
#780; no new proven immediate Validation counterexample is asserted here. Root
observe_validation is pub(crate), so adding an external call requires agreed API
ownership, not blindly copying a helper call.

Root native scope/error/pipeline helpers must survive composition. Several
exported native bodies or their policy documentation differ from Root; exact
comparisons are in generated-current-code-source-proof.json. Their stale
ignored-poll or policy text is not authority to regress Root. Actual source
cache keys source, exact native device, validation policy and canonical recipe;
failures do not publish new successful values.

## Contracts and required closure

**Producer propagation.** New Map owner is consumed by execute, bindings/program
and DistanceField lexical wrappers. Other four role families remain old mutable
projections and must migrate together with all current
constructor/fixture/caller APIs.

**Existing type owners.** Reuse ShaderParameter, CoordinateParameter/Shape,
ResourceBaseType, TextureValue, ResourceFetch, ResourcePipeline, canonical
BufferElementCount/Stride and WorkgroupShape/Grid; no competing launch/storage
type.

**Admission contracts.** GC-C3 must close panic paths. Native source validation
remains authoritative; private ordered Map parameters prevent independent role
mutation. Duplicate Map names now fail earlier intentionally.

**Invariant operations.** GC-C1/C2/C4 remain actionable. Scalar texture
LinearIndex uses population-1; LinearPopulation uses population, with widened
checked arithmetic. Coordinate-only programs retain independent axes.

**Aggregate states.** Map contextual primary derives from actual input presence;
resources/uniforms partition authored ordinary order. Other signatures still
carry independent vectors/flags and are incomplete.

**Type ownership.** GPU owns shared native leaf/coordinate/value/address
projections. Compute owns lexer, immutable declaration association and algorithm
roles. #780 retains extraction, Scan and actual caller functions.

**Redundant authority.** Definition source stays mutable and re-parsed;
source/device/validation-policy/recipe cache identity observes mutations. Map
stores no independently cached signature. ResourcePipeline pairs source-selected
shape/reflected controlling stride.

**Behavior preservation.** Preserve exact Advect native math/source, Map
generators, noise unsigned octaves, JFA ping-pong, DistanceField raw
declarations, MC/DC metadata-only pointers and pre-authored globals. Retain
existing unsupported neighborhood+secondaryBuffer limit; no new DSL feature
promise.

**Enum shapes.** Map has private Index/Size/Value payload roles.
Gather/Scatter/Stream/Stencil local role migrations, exact neighborhood
dimension and optional center/fallback remain required, with explicit earlier
duplicate/reserved-family errors.

**Derivability.** Map Default derive is equivalent because its concrete
ComputeSourceCache has unconditional manual Default. Equality remains code-only;
Stencil code+boundary. Do not presume unavailable native derive dependency.

**Type granularity.** Keep local roles separate; do not brand every native
identifier or implement aliases/primitive forwarding APIs. Original immutable
declaration text is lexical emission data, not a second semantic authority.

**Magic strings.** Native DSL names/main/bindings remain source ABI. Classify
exact outer resource/neighborhood protocols before wrapper cleaning; substring
Custom names must not silently select reserved roles. Preserve original
DistanceField declarations.

**Magic numbers.** Preserve native workgroup declarations and Advect
coefficients. Stencil64 defaults and i32 add/multiply/wrap must receive a
complete bound/policy disposition; they are not silently cleared by role
migration or by new unsigned texture addressing.

## Validation and ownership limits

No local project execution, builds, checks, lint, tests, probes, scanners,
generators, runtime, source changes, pushes or CI requests.

All27 owned files freshly read in full (4995 lines);65 exported identities
checked against manifest. Nine source/reference paths differ from c665 exactly
as listed in source proof, including newly started Map files.

Fresh complete named consumer and native program reads are identified per source
account. Remaining native references carry provenance only; actual Root native
implementation source coverage comes explicitly from
gpu-complete-cac5a7c33-code-review plus fresh validation/submission reads.
Differing stale native reference bodies are not cleared.

Two locks are hash/provenance context only. No lock composition acceptance.

Git symbol search accounts named direct generated-family constructors/execute
callers at c665; live moving peer source was not used. Broader numerical
geometry/application graph is not independently cleared.

Complete remaining four algorithm signature/parser/build/call/execute families,
reserved-role policy and direct caller constructor migrations; no partial-family
publication.

Resolve Stencil declared/default grid, physical input/output prefixes, signed
conversion/neighbor arithmetic and Buffer dimensional coordinates. Existing64
defaults are source-established policy; changing them is a separate documented
behavior decision.

Receive agreed canonical stride/count/ResourcePipeline/recipe/export closure
without replacing Root native core with stale peer references.

Preserve actual Root ValidationScope progress errors and cache retry semantics,
direct ordinary-native identity, borrowed/browser policy, immediate observers
and separate finish/queue submit.

Keep MC/DC/extraction/Scan locks and broader applications held by #780;
coordinate exact immutable final caller receipt.

Repeat complete post-repair thirteen lenses, independent Code/Rule/Comment
reviews and hosted checks on final pushed revision.
