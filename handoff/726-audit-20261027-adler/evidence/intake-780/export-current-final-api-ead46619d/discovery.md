# Current export admission and GLB final API preinventory

Root HEAD `ead46619dec5ef45993f63202cb289bd3986b018` plus unrelated generated
receiving WIP.
Current source accounts: 43; original archived accounts:25.
No repairs or clearance are asserted. All thirteen named receiving lenses are
freshly inventoried before repair. Root is sole implementer; native826 fixes
and this independent family can proceed separately.

## Current outcome and source drift

The old UV-R1/UV-R2 remain actionable in the current PBR tree. Every current
export child source, complete export.rs including all fixtures, character and
equipment direct callers and external textile export fixture were freshly read.
Their exact hashes/read limits accompany the JSON. No search or old verdict
establishes current clearance.

validation: three primitive scalar fields replaced by SurfaceMaterial.is_finite;
shell: effective-white canonical checked LinearSurfaceMaterial projection and
Result propagation; textures: same white policy moved to sole scalar_appearance
owner plus full item moves. Four prior WIP blobs not recovered exactly; complete
current source reread except explicitly bounded main; no old clearance carried.

The remaining unrecovered old WIP blobs are:

- `crates/adventuresim-character-creator/src/export.rs`.
- `crates/adventuresim-character-creator/src/character_export.rs`.
- `crates/adventuresim-character-creator/src/main.rs`.
- `crates/adventuresim-character-creator/src/surface_projection.rs`.

## Concrete final API recommendation

### public export

Keep export_rigged_glb(output,...,mesh,shells,sockets)->anyhow::Result<()>;
existing callers and contexts naturally retain concrete typed roots through ?.
Do not widen/duplicate a global export error merely to shorten this mixed
operation.

### source admission

New focused export/admission_error.rs: public ExportAdmissionError,
ExportAdmissionResult<T>, ExportSubject::Body | Shell { index:usize, name:String
}; closed column/morph/socket violation roles only where payload contract shares
actual meaning. Error variants carry exact host counts and canonical
RigJointOrdinal/RigJointCount where available. Own diagnostic strings only at
error production; no string-driven branching.

### lod

lod.rs owns CharacterLodError with requested u8 and canonical MIN/MAX Display;
TryFrom keeps Result<Self,Self::Error>, no alias-only trait signature. Do not
copy range into export error or change stored u8 public input as incidental
compatibility bridge. main loader ? retains source; formatter handles Err
without expect.

### validation

validate/validate_shell/morphs::validate/validate_proportions use actual
ExportAdmissionResult. Preserve original guard order where practical,
signed/out-of-range UV acceptance, same skin tolerance and joint cap. If
proportions move into initial admission to establish one phase, remove late
duplicate and inventory changed error priority explicitly.

### native assembly

One focused ExportAssemblyError/Result (or named equivalent) for current missing
primitives/native object/children/root shape errors; canonical
RigJointLookupError can remain a source variant. Either pass actual Vec<Value>
rather than extracting array from Value, or use checked extraction. Do not
return fabricated empty primitives; public top retains anyhow conversion. Typed
native projection failures belong here or a focused sibling projection owner,
not source admission.

### glb error

glb/error.rs (or small glb.rs owner) exports GlbError and cohesive GlbResult<T>.
Actual families Serialize(serde_json::Error), Layout(GlbLayoutError),
Reserve(TryReserveError), CreateDirectory{path:PathBuf,source:io::Error},
Write{path:PathBuf,source:io::Error}. Derive Debug/Error with #[from]/#[source]
as appropriate; no String cause or manual flattening. Publicly reexport errors
needed for downcast/type classification; only genuinely public payload role
types exposed.

### layout

Private glb/layout.rs GlbLayout constructor takes actual JSON Vec len and BIN
Vec len; no Default/serde/public fields. Named ChunkLayout for json/bin and
total32 certificate; store only private coherent derived fields needed by
encode. GlbLengthRole::{Json,Bin,Container} names native field. Checked padding
add and complete total add; Width{role,bytes,source:TryFromIntError} distinct
from Overflow{role} which has no fabricated std cause.

### packet

Prepare from actual serde_json::to_vec(document)? and consumed BIN Vec; admit
layout BEFORE any padding mutation/reserve/header/filesystem. Fallibly reserve
exact final frame capacity once and write original payloads plus pad bytes into
final Vec; avoid separate padded JSON/BIN reallocations. Prepared packet retains
only complete bytes (layout can be consumed), so no competing mutable metadata.
This is production format ownership, not a metadata-only testing adapter.

### writer

glb::write(path,document,bin)->GlbResult<()> constructs complete packet first,
then existing mkdir and fs::write with exact paths/sources. Top uses
glb::write(...)?;Ok(()) to convert into mixed anyhow. No GLB
create/truncate/write on invalid layout or failed preparation; later IO may
partially write, no transaction/rename framework.

### publication

TextureImages.finish still occurs first and may publish sibling PNGs / create
parent directory. No all-output-atomicity claim. Keep
hashes/cache/corrupt-shared-file rejection and surface alpha/sampler/image
precedence exact.

### pbr

Retain validation of original SurfaceMaterial.is_finite. Writer applies
SurfaceTextures::scalar_appearance then canonical
LinearSurfaceMaterial::try_from; white selection precedes decode when base color
map exists. Projection error remains SurfaceProjectionError source, not recoded
as UV/missing-map/admission error. Finite out-of-unit source factors remain
accepted where representable, no recipe bounds/clamp added.

### fixtures

No new tests/cases/helpers/assertions/doctests/probes. Existing whole fixture
source read; only necessary API/import/? adaptations allowed. Preserve oracles
skin weights sum, ordered morph rejection/matching/NaN,4..=6 u8 loop, GLB native
parse/image/skin/UV/morph/hinge/socket and shared texture corruption.

## Actionable current source findings

### UV-R1-CURRENT [P2]

Location: `export/validation.rs;export/morphs.rs;export/skeleton.rs`.

Contract: Stable owned input-admission errors and canonical lower source
identities.

Source: All existing guard classifications remain anonymous anyhow bail/ensure;
PBR migrated final finite guard to SurfaceMaterial but not its typed failure
ownership.

Connected repair: One ExportAdmissionError + descriptive ExportAdmissionResult;
typed body/shell subject and relevant payloads; canonical CharacterLodError at
lod; no duplicate validator.

### UV-R2-CURRENT [P2]

Location: `export/glb.rs:13-26`.

Contract: Native frame padding, complete additions and all u32 headers checked
on actual payload lengths before any GLB path operations.

Source: next_multiple_of(4), usize total sum, as-u32 JSON/BIN/total and
infallible Vec::with_capacity remain unchanged from old discovery.

Connected repair: Private checked layout and one prepared packet; actual
TryFromIntError and TryReserveError retained; GlbError separates
serde/layout/allocation/directory/write causes.

### EXPORT-FINITE-BODY [P2]

Location: `export/validation.rs:13-16;export.rs:305-313,414`.

Contract: Actual exported body position/normal native words must be finite.

Source: Public body arrays only population checked, unlike shell arrays; NaN/Inf
reaches BIN and bounds JSON.

Connected repair: Add finite positions/normals admission at same source boundary
before compaction/native buffer construction; no norm-length/range or topology
policy.

### EXPORT-FINITE-JOINT-SOURCE [P2]

Location: `export/validation.rs:25-36;export/skeleton.rs:20-33`.

Contract: Retained raw joint states are actual finite source input to native
transforms.

Source: Public globals table checked only for length; shell-only finite geometry
plus NaN joint translation passes and reaches null node JSON/nonfinite inverse
bind bytes.

Connected repair: Separate typed nonfinite source-state guard with actual
ordinal before native construction. Negative/zero/denormal scale policy is not
invented by this check.

### EXPORT-FINITE-HINGE [P2]

Location: `export/validation.rs:146-158;export/assembly.rs:38-42`.

Contract: Optional native hinge metadata components must be finite.

Source: Public optional hinge origin/axis bypass current geometry/PBR finite
guard and serialize directly; nonfinite source becomes null JSON metadata.

Connected repair: Typed finite-only optional hinge admission; no normalization,
length threshold, sign or physical solve policy.

### EXPORT-NATIVE-PROJECTION [P2]

Location:
`export.rs:400-407,553-576;export/skeleton.rs:20-33;fabelgeist-mhr/src/math.rs:152-220`.

Contract: Finite source alone does not certify representable finite native
node/matrix output.

Source: Transform inverse divides by scale and compose/narrowing can overflow;
zero or sufficiently tiny finite scale may produce nonfinite inverse-bind words,
even after source finite guard.

Connected repair: Guard actual derived inverse-bind matrix and node
to_skel_state f32 components at receiving projection; typed native projection
error distinct from malformed source; retain exact math and all finite outputs.
No positive-scale/clamp policy.

### EXPORT-NATIVE-UNWRAP [P3]

Location: `export/morphs.rs:74-78;export/assembly.rs:22`.

Contract: No production unwrap; typed native assembly is distinct from authored
input.

Source: Known producer uses JSON primitives arrays but both private native
transformations call unwrap.

Connected repair: Prefer direct actual Vec ownership if chosen before repair;
otherwise fallible shape extraction with one focused ExportAssemblyError and
descriptive alias. Propagate through existing top anyhow, no compatibility path.

### EXPORT-LOD-EXPECT [P3]

Location: `model_controls.rs:14-16`.

Contract: No production expect; LOD accepted range stays at canonical owner.

Source: Formatter unconditionally expects TryFrom despite actual fallible API
and explicit no-expect rule.

Connected repair: Handle formatting failure at actual UI boundary while
retaining same valid display and no clamp; main load_body_model propagates
canonical source via ?.

### EXPORT-PROFILING-EXPECT [P3]

Location: `profiling.rs:36-43`.

Contract: No production expect; optional profiling must not alter operation
return/cause.

Source: GLB profiling wrappers invoke record, which expects serde encoding of
Timing.

Connected repair: Handle diagnostic serialization result explicitly; keep
generic measure<T> operation result intact and profiling optional. No new export
error for telemetry or swallowed operation failure.

### EXPORT-RULE-ORDER-RECORDS [P3]

Location: `export.rs module
declarations;export/morphs.rs:5-18;export/assembly.rs:13-22;export/skeleton.rs:5-9;export/shell.rs:3;export/skinning.rs:8`.

Contract: Public before private free functions; named semantic records and
existing owner reuse; no-growth responsibilities.

Source: morphs private channels precedes public validate/write; channels carries
semantic target/population tuple. Native parts use name/hinge/shell triple;
native skeleton tuple and shell/skin view tuples encode semantic roles. These
are source-backed rule debt, not demonstrated runtime failures.

Connected repair: Before editing each responsibility, choose named MorphChannel,
native Parts/Nodes/SkinViews/ShellViews where touched; direct typed collection
approach may close shape/tuple roles together. Move complete docs/attrs with
items; keep existing production export oversized math outside expansion or
extract only actual new responsibility.

## Exact source admission and native guard inventory

Actual body/shell counts in error diagnostics remain native host counts; they
must retain rejected values that may not fit a checked mesh count. Reuse
RigJointOrdinal/RigJointCount when the source already owns those quantities.
Do not admit a new native bound just to construct an error payload.

- **CharacterLod::try_from** (`lod.rs:8-18`): requested u8, actual MIN/MAX
  owners; same4..=6 accepted; source-preserving CharacterLodError. Role:
  `CanonicalLod`.
- **body population** (`export/validation.rs:13-16`): positions nonempty;
  normals same length; preserve actual host usize counts. Role:
  `GeometryPopulation`.
- **body geometry finiteness** (`export/validation.rs:13-16 (missing)`):
  positions and normals finite before buffer construction; newly reject NaN/Inf
  source only. Role: `NonFiniteGeometry`.
- **body skin populations** (`export/validation.rs:17-19`): indices/weights
  exactly positions; both native host counts retained. Role: `SkinPopulation`.
- **body rig bound** (`export/validation.rs:21-24`): reuse actual RigJointCount
  from skin/table; equal bound does not prove rig instance identity. Role:
  `SkinRigBound`.
- **skeleton populations** (`export/validation.rs:25-28`): joint names nonempty,
  parents/globals equal lengths. Role: `SkeletonPopulation`.
- **native joint width policy** (`export/validation.rs:29-31`): retain
  exactly65535 current cap, no65536 widening and no GPU ceiling. Role:
  `JointLimit`.
- **parent order** (`export/validation.rs:32-36`): reuse RigJointOrdinal; Some
  parent < current; first parent therefore None. Role: `InvalidParent`.
- **retained global source finiteness** (`export/validation.rs:25-36
  (missing)`): actual raw [f32;8] components finite; no normalization or
  sign/range addition. Role: `NonFiniteJointState`.
- **body faces** (`export/validation.rs:37-44`): every body u32 corner within
  body positions, even shell-only body topology used for sockets. Role:
  `MissingVertex`.
- **body weight sums** (`export/validation.rs:45-50`): same finite sum and
  absolute budget1e-4; preserve skin weights sum Display oracle; do not add
  nonnegative weights policy. Role: `WeightsSum`.
- **baked plate maps** (`export/validation.rs:52-58`): normal and metalroughness
  Vec nonempty; no PNG decoding policy. Role: `EmptyMap`.
- **UV presence** (`export/validation.rs:59-62`): either map family requires
  sole texcoords; no per-map alternative UV owner. Role: `MissingUvs`.
- **UV population/finiteness** (`export/validation.rs:63-68`): rows equal shell
  positions and finite; signed/outside-unit UVs remain accepted. Role:
  `UvPopulation/NonFiniteUvs`.
- **shell name** (`export/validation.rs:101-103`): trim only for emptiness;
  retain original authored text in errors/output. Role: `BlankShellName`.
- **shell geometry shape** (`export/validation.rs:104-110`): positions and faces
  nonempty; normals equal positions. Role: `GeometryPopulation`.
- **shell skin availability** (`export/validation.rs:111-122`): both supplied or
  neither with body vertex-count topology; preserve same count contract, no
  newly inferred identity. Role: `IndependentSkin`.
- **shell rig bound/populations** (`export/validation.rs:123-129`): same rig
  bound; exact rows; reuse existing Rig owners. Role:
  `SkinRigBound/SkinPopulation`.
- **shell weight sums** (`export/validation.rs:130-135`): same finite sum and
  tolerance, native weight behavior unchanged. Role: `WeightsSum`.
- **shell faces** (`export/validation.rs:136-145`): every shell face within
  shell positions; plate-edge old filter policy unchanged. Role:
  `MissingVertex`.
- **shell geometry/PBR finiteness** (`export/validation.rs:146-158`): source
  positions/normals and ORIGINAL SurfaceMaterial.is_finite; retain source finite
  even if texture later shadows color. Role:
  `NonFiniteGeometry/NonFiniteAppearance`.
- **hinge metadata finiteness** (`export/validation.rs +
  export/assembly.rs:38-42 (missing)`): raw optional origin/axis currently
  emitted directly; finite check only, no axis-length/sign/normalization policy.
  Role: `NonFiniteHinge`.
- **morph channel order** (`export/morphs.rs:20-32`): body participates only
  export_body; all selected primitives same ordered names; use named
  MorphChannel record. Role: `MorphChannels`.
- **morph target admission** (`export/morphs.rs:34-49`): nonblank unique name,
  position/normal delta rows equal vertices, all finite; retain original names.
  Role: `MorphTarget`.
- **proportion basis admission** (`export/skeleton.rs:45-55`): empty or same
  joint length; actual JointProportionBasis::is_finite; move admission earlier
  only if documented error order change. Role: `Proportions`.
- **socket identity** (`export/validation.rs:73-80`): nonblank unique open
  attachment IDs; preserve actual index/name context. Role: `SocketIdentity`.
- **socket native state** (`export/validation.rs:74,81-86`): to_skel_state
  actual [f32;8] finite, including native narrowing; no new source quaternion
  policy. Role: `SocketTransform`.
- **socket UV/domain** (`export/validation.rs:87-94`): domain nonblank and UV
  finite, no domain enum/range restriction added. Role: `SocketUv`.
- **native skeleton root** (`export.rs:486-490`): current local missing-root
  Context root; preserve or replace with actual roots output, not human prose.
  Role: `SkeletonRoot`.
- **native primitives shape** (`export/morphs.rs:74-78;export/assembly.rs:22`):
  two production unwraps: consume actual typed Vec or return stable native
  assembly error; no dummy/default geometry. Role: `Primitives`.
- **socket native node shape** (`export/sockets.rs:47-54`): existing
  object/children array guards become native assembly errors; canonical ROOT
  require_in keeps RigJointLookupError source. Role: `NodeObject/NodeChildren`.
- **derived native joint projection**
  (`export.rs:400-407,553-576;export/skeleton.rs:20-33`): finite raw source may
  still produce nonfinite inverse-bind f32/node f32 output; check actual
  projection before publishing, no physical math rewrite. Role:
  `NonFiniteNativeJoint`.
- **profiling serialization** (`profiling.rs:36-43`): production expect reached
  by glb_serialize/glb_write when enabled; handle diagnostic encode error while
  preserving generic operation result. Role: `DiagnosticEncoding`.

## Native byte and publication contract

The frame is12-byte header,8-byte JSON chunk header, padded JSON,8-byte BIN
chunk header, padded BIN. Four-byte alignment uses ASCII spaces for JSON and
zeros for BIN. Preserve the empty BIN chunk. Each padded chunk and the complete
28+payload total must fit native u32; the largest aligned total is u32::MAX-3,
derived from format width, never a new tunable cap. Check host padding/addition
overflow before u32 narrowing and fallible final reservation. Actual conversion
and allocation errors retain std sources; arithmetic Option::None has no std
cause to fabricate.

Serialize actual JSON, inspect actual BIN length, construct/check layout,
fallibly reserve/build complete packet, then mkdir/write GLB. Invalid layout
or preparation must not create/truncate/write the GLB path. Existing shared PNG
publication still precedes this writer. Later IO failure is not transactional.

Source joint state finite checks alone do not prove native projection: current
inverse divides by scale, local compose can overflow and f64-to-f32 can narrow
to Inf. Check actual emitted inverse-bind/node words as a receiving guard if
closing native projection; preserve math and every finite outcome. This does
not require positive scale, a normalized-source-quaternion rule or new
tolerance.

## Existing fixture closure

All existing export.rs fixtures were read, including normal skin/zero animation,
weight rejection, clothing and independent topology, shell-only sockets, named
morphs and UV socket/bootstrap geometry. Full
assembly/morph/skinning/LOD/texture
fixtures and external textile fixture were read. Keep all scenarios, helpers,
assertions and oracles exact. The weight rejection checks `skin weights sum`;
the morph scenario rejects mismatched names, accepts matching, then rejects its
NaN delta. LOD loop accepts exactly4..=6. Shared-texture corruption still checks
`unexpected contents`. No large-frame edge-case runtime is present and none is
fabricated. API/import/? adaptation is allowed only where required; add no test
case/helper/assertion or source-only boundary probe.

## Sequential all-thirteen current dispositions

### 1. producer-propagation

Contract/source: Public raw RiggedMesh/Shell/Socket -> single source admission
-> compact remap -> skin/native accessor/texture/PBR ->
skeleton/morph/socket/assembly -> actual JSON/BIN -> checked packet -> file. All
actual direct application and existing fixture callers read.

Disposition: ACTIONABLE typed-root migration and finite bypasses; no repair
performed.

Connected repair: admission+native errors propagate by ? through current mixed
anyhow without prose conversion; fresh review of all touched receivers after
implementation.

Dependencies: canonical LOD/Rig/PBR and actual direct caller APIs; no native
peer edits needed.

Acceptance: No compatibility fields, obsolete infallible get/forwarder, second
validator or swallowed native failure.

### 2. existing-type-owners

Contract/source: Rig owns ordinals/count/bounds and lookup source; LOD
owns4..=6; Color/Armor/Creator own source appearance/native projection. GLB owns
container bytes. Mesh counts restrict u32 and MeshData adds unrelated
cap/optional attributes/skin policy.

Disposition: Reuse available exact owners; raw usize source counts remain honest
diagnostics for possibly invalid values.

Connected repair: Export-owned local input/native errors; glb layout/error
ownership; no GPU byte/count/launch or HairDataError substitution.

Dependencies: Current canonical count.rs, Rig, MHR constants and PBR sources
read.

Acceptance: No duplicate LOD range, universal mesh certificate, native byte
wrapper identical to foreign GPU meaning or widened rejection.

### 3. admission-contracts

Contract/source: Every current guard and missing geometry/joint/hinge finite
source path enumerated separately; actual original PBR input and effective
native appearance both retain their distinct checks.

Disposition: Full guard inventory fresh; new finite source corrections explicit
before repair; source API final naming pending Root choice.

Connected repair: Typed focused guards and canonicalLOD; packet layout sole
constructor.

Dependencies: Source-finite is not derived-native-finite; projection admission
separate.

Acceptance: Same valid shapes, optional maps, UV semantics, cap/tolerance; no
new normalization/positive-scale/negative-weight policy.

### 4. invariant-operations

Contract/source: Alignment, host total additions, native width and reserve
checked before frame allocation/output; actual compose/inverse/f32 narrowing and
strongest_four operations inspected.

Disposition: UV-R2 confirmed; finite source alone insufficient for native
transform output.

Connected repair: Checked actual layout; receiving finite guards for actual
emitted joint matrices/states; same math unchanged.

Dependencies: Existing Vec/serde allocation earlier in buffer pipeline not made
globally fallible by final frame reserve.

Acceptance: No overflow/truncation; no fabricated TryFrom cause; all
representable regular output bytes exact.

### 5. aggregate-states

Contract/source: One private native packet binds actual payloads, pads, chunk
lengths and total. Named subject ties actual body/shell/target/socket context.
Compact retains sole UV/remap across all columns and same target order.

Disposition: No duplicate schema/certificate selected; native primitives JSON
shape currently assumed.

Connected repair: Immutable layout/packet, named MorphChannel and native
record/shape errors; retain source data as authority.

Dependencies: Source borrowed public records stay immutable during synchronous
export; no need branded public mesh wrapper.

Acceptance: Headers equal exact bytes, no independently mutable layout facts; no
fake empty geometry/default success.

### 6. type-ownership

Contract/source: Source admission, native assembly/projection, container
framing, lowerRig/LOD and PBR each have lowest shared real owner.

Disposition: Root sole implementation; no826nativeownership blocker for this
family.

Connected repair: Focused new modules before growing old oversized export.rs;
descriptive results; types before impl/functions/tests.

Dependencies: Public reexports/cfg offline-creator; thiserror/serde_json already
direct deps.

Acceptance: No passthrough framework, parallel public legacy writer, aliases for
compatibility or misplaced Hair/GPU error owner.

### 7. redundant-authority

Contract/source: One validator and one current typed PBR projection; effective
white policy is SurfaceTextures owner; source does not store native factors.
Morph target order derived from same channel vector.

Disposition: Current message roots and ad hoc casts are not stable authority;
replacement must remove old paths directly.

Connected repair: One checked layout and named ordered channels, no separate
mutable cached target-name list.

Dependencies: Keep pointer-based shell texture reuse and content hash identity
unchanged.

Acceptance: No dual validator/writer/projection; no error prose status or
re-admission loops.

### 8. behavior-preservation

Contract/source: Exact current PNG-before-GLB order, GLB
header/magic/chunks/padding, existing tests and all literal oracles read; finite
invalid rejection deliberately added.

Disposition: No runtime evidence or blanket old verdict; source-ready acceptance
only.

Connected repair: Source inverse for unchanged bodies/fixtures; document error
priority if proportions moved; no serialization-order drift.

Dependencies: Existing hosted gates later; no new tests available for huge
layout edge cases.

Acceptance: No GLB file operations before successful packet; sibling PNGs may
already exist; no atomic later-IO promise.

### 9. enum-shapes

Contract/source: Body/shell, input column/violation, JSON/BIN/container and
local native shape roles are closed; authored names/paths remain open. Existing
export_body is a meaningful named emission predicate.

Disposition: Closed prose classification requires typed payload; no arbitrary
enum of numeric indices.

Connected repair: Error subjects/roles with actual relevant payload; named or
tagged Parts for body vs shell if touched.

Dependencies: Canonical Rig names remain authored strings; dimensions/nativeGLTF
codes true external tokens.

Acceptance: No optional-payload bag, boolean error category, dual role/name
authority or unnecessary new enum for ordinary predicate.

### 10. derivability

Contract/source: Creator direct thiserror2.0.20 permits source-preserving
derive; serde_json1.0.151/anyhow1.0.104 both locks inspected. Layout readonly no
uncheckedDefault/Deserialize needed.

Disposition: Typed errors can derive; actual std integer/IO/reserve/serde source
types retained.

Connected repair: Derive Debug/Error; explicit traitResult; ordinary cohesive
aliases; no error Clone requirement for IO.

Dependencies: No Cargo/lock changes anticipated unless Root chooses unnecessary
new dependency; native source chain reviewed after repair.

Acceptance: No handcoded Error boilerplate/cause stringification or fake source
for checked_add None.

### 11. type-granularity

Contract/source: Native frame lengths are format u32 bytes, not mesh population.
Morph channels and native view/part/skeleton record tuples carry distinct roles,
currently unnamed.

Disposition: Source-backed named-record debt in touched responsibilities
explicit; broad geometry tuple/math rewrite outside selected repair.

Connected repair: Small checked layout + focused error owners + named actual
receiving records; avoid per-field wrapper explosion.

Dependencies: Source counts may exceed u32 when diagnosing malformed input; do
not force MeshVertexCount prematurely.

Acceptance: Real responsibility extraction, no arbitrary extra API layer/one-use
primitive adapter; no file/function ceiling growth.

### 12. magic-strings

Contract/source: Existing source-error roots closed; glTF schema keys/type
names, paths, open authored IDs and morph names legitimate native/source
boundary strings.

Disposition: Typed local roots required; diagnostics remain terminal only;
nativeJSON unwraps not silently waived.

Connected repair: Stable variant roles; preserve required skin weights sum
oracle; no Display/Debug branching.

Dependencies: Profiling expect separate diagnostic boundary; PBR native
projection preserves actual source.

Acceptance: No string source fabrication, human prose classifier, duplicated
JSON parser or protocol fallback.

### 13. magic-numbers

Contract/source: GLB12/8/4/version2/chunk words native format, finite guard no
tolerance; existing skin1e-4 and joint65535 exact existing policy. Metres are
source/native coordinate contract, not altered here.

Disposition: No new caps or epsilons; actual byte arithmetic currently
unchecked.

Connected repair: Owning framing constants/checked native lengths; preserve all
original calibration, UV/skin/physical formulas.

Dependencies: Mesh/GPU count and native dispatch owners distinct; no
sourceextentpolicy migration hidden in frame fix.

Acceptance: Same finite normal data/native bytepacking; invalid size rejects
without truncation; exact pad spaces/zeros incl empty BIN chunk.

## Current exact source accounts

| Path | SHA-256 | Fresh read extent |
|---|---|---|
| `crates/adventuresim-character-creator/src/export/glb.rs` | `99b359ccf03e94e704e08e8f66f7b23c4dcf00a358ac7b1fabd98993530d26f8` | whole current source, all production and existing inline fixtures; all changed PBR admission/receiving responsibilities freshly read |
| `crates/adventuresim-character-creator/src/export/validation.rs` | `dc8418b20e33ff08fc66ccfe4c1f9aee75d43c450ade95fd7e86ebf36030dfb4` | whole current source, all production and existing inline fixtures; all changed PBR admission/receiving responsibilities freshly read |
| `crates/adventuresim-character-creator/src/export/morphs.rs` | `7eede3013d854f1905e7e4806f7914bd978b9a54a0d9201b001578bfa3e63013` | whole current source, all production and existing inline fixtures; all changed PBR admission/receiving responsibilities freshly read |
| `crates/adventuresim-character-creator/src/export/sockets.rs` | `d51e531f136697d43d673cc1e6d16764b2c6fd35661d83d9a46b45a4eb5facdf` | whole current source, all production and existing inline fixtures; all changed PBR admission/receiving responsibilities freshly read |
| `crates/adventuresim-character-creator/src/export/skeleton.rs` | `ac761936db58c742f2c58ab7381b8e26a8b6f819bfb91c0b29c54170fe96abff` | whole current source, all production and existing inline fixtures; all changed PBR admission/receiving responsibilities freshly read |
| `crates/adventuresim-character-creator/src/export/assembly.rs` | `92f6f5351a361dc257d646c8b9b411e53a7af03ea6aef9043536eb2bdb707c72` | whole current source, all production and existing inline fixtures; all changed PBR admission/receiving responsibilities freshly read |
| `crates/adventuresim-character-creator/src/export/skinning.rs` | `7ddf91c41655775cc2ac55844fa884d4f06dd8ba6951ee0d45b4550bbde5b199` | whole current source, all production and existing inline fixtures; all changed PBR admission/receiving responsibilities freshly read |
| `crates/adventuresim-character-creator/src/export/buffer.rs` | `c92b80be46c77268cf276c3686aaedbd70bba5be1e4f7d5eaef7041f865be104` | whole current source, all production and existing inline fixtures; all changed PBR admission/receiving responsibilities freshly read |
| `crates/adventuresim-character-creator/src/export/compact.rs` | `4caf4186ff5bdf5bbeef68f6dc21fd1efe09273d6a51a778862c5c3699db34ef` | whole current source, all production and existing inline fixtures; all changed PBR admission/receiving responsibilities freshly read |
| `crates/adventuresim-character-creator/src/export/rigged.rs` | `73b6b6c79dad123790c5136a530112273cc766f8602f07360d36ffe5b1bb8cd6` | whole current source, all production and existing inline fixtures; all changed PBR admission/receiving responsibilities freshly read |
| `crates/adventuresim-character-creator/src/export/shell.rs` | `e430c45bc2b8f6b87ab984c3b1684a287e124a4275eb29f8ab86906db79093e2` | whole current source, all production and existing inline fixtures; all changed PBR admission/receiving responsibilities freshly read |
| `crates/adventuresim-character-creator/src/export/textures.rs` | `5d23e1c4decd680acec00f3d2bf5f5a66beac501291d73165cafe7f9ccd26322` | whole current source, all production and existing inline fixtures; all changed PBR admission/receiving responsibilities freshly read |
| `crates/adventuresim-character-creator/src/export.rs` | `117a80aebc7a1de9da1a89065490b5407d182b9fbbdae2b70735945084fc353c` | whole current source, all production and existing inline fixtures; all changed PBR admission/receiving responsibilities freshly read |
| `crates/adventuresim-character-creator/src/character_export.rs` | `6210517f4a5b5ff82c940bfaddb1581c206b3c92630e45a1e733c2ccf5f23bdd` | whole current source, all production and existing inline fixtures; all changed PBR admission/receiving responsibilities freshly read |
| `crates/adventuresim-character-creator/src/equipment_export.rs` | `dbbb086bd77ecd0153686025543f523cc7787e9a909a030625275c6d124a9b59` | whole current source, all production and existing inline fixtures; all changed PBR admission/receiving responsibilities freshly read |
| `crates/adventuresim-character-creator/src/lod.rs` | `73ad82b91f4b15146231c922544a72466849407fc7fca6ebe2268d78f45d7fa3` | whole current source, all production and existing inline fixtures; all changed PBR admission/receiving responsibilities freshly read |
| `crates/adventuresim-character-creator/src/model_controls.rs` | `f52b873a70ee6fd3ac34fc78e3acb63fac9b6466d973114ca313fe066105b5ae` | whole current source, all production and existing inline fixtures; all changed PBR admission/receiving responsibilities freshly read |
| `crates/adventuresim-character-creator/src/main.rs` | `366087f0e3753b4917fc7fb7fe1f5aa1c6bc654dd4fe3ed03d97c7df0f3395e8` | 402-438: whole load_body_model and surrounding actual LOD/reload receiver boundary; rest of binary not re-reviewed |
| `crates/fabelgeist-mesh/src/data.rs` | `41c300bd088e2a69594225a867cee9f1bc3fe5f19410bf77d3c8de73a383f1b4` | 1-200: exact MeshData admission/reuse comparison, not whole GPU/data family |
| `crates/fabelgeist-hair/src/export/error.rs` | `9a6b4ec4caa2f2942c4ed56febc4e4ff025a73319b24a4efc69000a03b0a6d41` | whole current source, all production and existing inline fixtures; all changed PBR admission/receiving responsibilities freshly read |
| `crates/fabelgeist-rig/src/skin.rs` | `c1c856dd707201d43249ad4e61576728edf4ff9a2a2d1dbf68f6c980a1c2d206` | whole current source, all production and existing inline fixtures; all changed PBR admission/receiving responsibilities freshly read |
| `crates/adventuresim-character-creator/src/export/textures/tests.rs` | `461ae69b0d57a3e9e0c4f412dfbf39ad34864bdc72fe6e5c964d292e50391c89` | whole current source, all production and existing inline fixtures; all changed PBR admission/receiving responsibilities freshly read |
| `crates/adventuresim-character-creator/src/surface_projection.rs` | `d0cb73bb8dee9cf7def0509cd17744d40598b5a9cb99dc0a5fceec0ff16bc24d` | whole current source, all production and existing inline fixtures; all changed PBR admission/receiving responsibilities freshly read |
| `crates/adventuresim-character-creator/Cargo.toml` | `89cf06ca88fa0b2863f103f8cf6961345e3c5cfc2c52ad8802bbe3f452d70361` | whole current source, all production and existing inline fixtures; all changed PBR admission/receiving responsibilities freshly read |
| `crates/AGENTS.md` | `2d9c7c48f7c9d02c2996ee71cd64c8dc97ef71ee260f99b16ff388a782f63f23` | whole governing rule source |
| `crates/adventuresim-character-creator/src/lib.rs` | `3014f331d32d092248326e49b6966aca09036b86b68220167f56924abf160d84` | 1-175: actual cfg-gated export/lod and native projection exports; unrelated later recipes not re-reviewed |
| `crates/fabelgeist-rig/src/lib.rs` | `7e73efb0a5ce824e111c2d28e54a5422b75bdca12e0c7553423dc0dfab8f563b` | whole identity/ordinal/lookup/error owner (excluding external test module) |
| `crates/fabelgeist-rig/src/table.rs` | `491cfcace087aef015067a75e218e3bee9dc3c1e8bf681b40afa840f806da282` | whole rig-column ownership/admission/indexing source |
| `crates/fabelgeist-rig/src/names.rs` | `94146b1b91f11d9711a5439e607b357cd8bb2fe828de4763cb432f58b0225554` | 1-91: whole authored name constants source |
| `crates/fabelgeist-mhr/src/math.rs` | `20b8033a4cd2771f494d9a73e0820bd9c27c5cdce23c097687360719bfc46b0a` | 1-235: quaternion math, Transform and exact compose/inverse/to_skel_state paths; remaining affine routines not reviewed |
| `crates/fabelgeist-mhr/src/model.rs` | `40ffb38f7e31078dc362350d86467b2a82568d14153ad3e7eef940961b828f84` | 1-70: canonical MIN/MAX_LOD and native MhrOutput contract only |
| `crates/fabelgeist-mhr/src/lib.rs` | `9d99fd95ed9ec73c741b282a32784af8c92b4a8d3abcbc8ae15914f94a50f06b` | 37-43: actual canonical LOD/model reexports only |
| `crates/adventuresim-core/src/character_proportions.rs` | `7a3b0a5797dbb6d8c811aa2e2df9d831e7424367faf501cf6b3b9bffe1683433` | 130-175: whole JointProportionBasis data/translation/is_finite owner and fixture opening; other proportions algorithms not re-reviewed |
| `crates/fabelgeist-mesh/src/count.rs` | `e609778b6bd24b4ee231f3d3281d463764a55d5b787f02f4015f4ff7c4b4072b` | whole canonical mesh count admission/arithmetic (no test module) |
| `crates/fabelgeist-armor/src/surface_material.rs` | `ea2e7125a8a3e4191aac2a1ab33237df671f8cf0aac607ccbef869fd2f7e691e` | whole typed source appearance/finiteness owner |
| `crates/adventuresim-character-creator/src/profiling.rs` | `73eb7657f5e30926f9ad6d5f5f3603b03284b29f3c1fe7c39e895b723729b950` | whole timing wrapper/recording production source |
| `crates/adventuresim-character-creator/src/character_morphs.rs` | `c2ccb364a4ac459ff340265962f4d5dd1a171be1cf9a8554a5de11a82e558865` | 175-310: armor targets tail, complete component_materials/rigged_clothing/rigged_armor constructors, remove_skeletal_translation and fixture opening; other generation not re-reviewed |
| `crates/adventuresim-character-creator/src/character_export/hair.rs` | `c5e9761baffad2f4392216dddc34d97d391a670f3526dc7ea5a1d28bdc29c415` | whole export hair producer/retained shell/native image/source cache paths |
| `crates/adventuresim-character-creator/src/garment/export.rs` | `d4d9bf1aea7c1a69e348644cf11a09056260ce34f97b340357aed60c646c4a5a` | whole actual draped/mail RiggedShell constructor |
| `crates/adventuresim-character-creator/tests/textile_export.rs` | `2af2db51fbec555697e97ac7e653f95ae09e381dd8acbc9bb110f66b6346c314` | whole existing external export fixture and all oracles |
| `Cargo.lock` | `ba4082d74f3bdcafdb689588768e1ef4b37041e55493745dcc15e5c541fae33c` | full TOML parse of package records for actual thiserror/serde_json/anyhow versions only |
| `crates/adventuresim-character-creator/Cargo.lock` | `a068c4409c2ddee8160dba37c314676eebaafb33c324984a872a9478ebd3e354` | full TOML parse of package records for actual thiserror/serde_json/anyhow versions only |

| `crates/fabelgeist-armor/src/components.rs` | `39076111488cb7bfbc945fbbb14171173b6db894259f1c877af261ddde6109fe` | 1-25: actual ArmorHinge metre-space public origin/axis fields, serde construction and retained ArmorComponent optional hinge; rest of components not re-reviewed |

Existing binary fixture dependencies (hash only):

- `assets_src/equipment/materials/mail-base-color.png`:
  `da1a5fbb246602080895023758024b4072838b3fd72ee20e074bf16cd4ffac64`.
- `assets_src/equipment/materials/mail-normal.png`:
  `f34896f80e1b01a61a9e4952217c8400e5197b749535e7725754cd83726e6d5e`.
- `assets_src/equipment/materials/mail-occlusion.png`:
  `a6f24a4a8570d8a269530f58fa266a41e72e7b93d69681085ed22e94bd5bccbc`.

## Limits and pre-edit disposition

All thirteen CURRENT named export-admission/native frame responsibilities
inventoried before repair. Exact final API names and selection of newly found
joint/hinge/native projection/diagnostic corrections remain Root decisions to
record before edits. Broader numeric algorithms are not cleared.

- No production edits, checks, builds, tests, scanners, probes, generators, new
  tests/cases/helpers/assertions/doctests, push, CI or publication.
- Fresh whole current export.rs/all export child files/all existing export
  fixtures, complete character/equipment direct callers and textile external
  fixture. Actual import/search enumerations locate callers but source reads—not
  searches—establish the listed coverage.
- Main only402-438, character_morphs175-310, MHR math1-235/model1-70 and other
  source ranges explicit; no claim entire binary, runtime model/procedural
  generation, rig universe or physical socket solver policy audited.
- Public external downstream Rust consumers beyond this repository not
  enumerated. Current actual in-repository direct export_rigged_glb call sites
  all read.
- No execution covers huge native frame/overflow/allocation failure; checked
  source reasoning and existing later hosted checks are required.
- Preparing a final frame does not make earlier BufferBuilder/serde/texture
  allocations fallible or all output transactional. Shared PNGs may publish
  first; later IO may partially write.
- No global/final same-pushed-SHA review or CI pass; generated/native GPU
  receiving and broader physical/count/seed/engraving/trim/cause families remain
  open.
