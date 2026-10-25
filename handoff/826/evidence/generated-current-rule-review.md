# Independent generated-family Rule Review

Scope: frozen generated-transfer-c665-map-snapshot/manifest.json; base
c6659f6cfa4610f066c659411ade6636e21bd3b5 plus nine started-Map source
deviations. Manifest SHA-256
1da74380dd8a3c6a54c24803aea6a48602f4baa12e69adc7a51fee869d5b3373. All65
identities independently match. Full source coverage and per-file contracts are
in generated-current-pre-repair-all13.json:27 owned full bodies,36 reference
full bodies, two lock bookkeeping accounts.

Correctness: not clear. Four concrete defects below remain. Completeness:
current all13 discovery recorded, but transferred sibling producers, Stencil
physical policy, canonical receiving/callers/exports/locks and final hosted
checks remain incomplete. No final clearance or compilation claim.

## Findings

### GCR-P1 — P2

crates/fabelgeist-gpu/src/data/gpu/compute/signature.rs:168-186;
crates/fabelgeist-compute/src/stencil.rs:151-154

existing connected correctness defect; independently confirmed, deduplicates
GC-C3. parse_base_type slices a vector/texture inner value after only a
starts_with guard. vec2< gives [5..4]; vec2<é can split a UTF-8 codepoint. The
public Map source constructor reaches this before Naga. Stencil separately
slices first < through last > without proving their order; Neighbors>bad<
reverses them.

Rule: crates/AGENTS.md:80-84; fallible source admission must not bypass native
error boundary.

Connected correction: Check the complete outer envelope and
character-boundary-safe extraction before recursion. Preserve open Custom names
and existing valid wrappers. Malformed ordinary native text may remain
native-rejected Custom text; closed reserved neighborhood syntax may return its
typed admission error. Do not create a second WGSL parser.

Acceptance: All incomplete/reversed/non-ASCII envelopes return the existing
fallible/native outcome without panicking; every shared
parse_base_type/clean_type_name producer and Stencil neighborhood route uses the
final extraction contract.

### GCR-P2 — P2

crates/fabelgeist-compute/src/scatter.rs:124-144

known existing correctness defect; confirmed still present, deduplicates GC-C1.
With index:u32, value:u32, out:Resource<u32>, every ordinary unsigned parameter
enters the coordinate branch. Once index exists, value is discarded instead of
becoming an input value; has_value_param remains false and parsing rejects it.

Rule: crates/AGENTS.md:30-70; role admission/operation agreement.

Connected correction: Select explicit supported unsigned index before alias
fallback, then classify all remaining ordinary parameters as values in authored
order. Preserve an ordinary floating value named index, last-value type
behavior, resource metadata/global emission and existing unsupported forms.

Acceptance: Unsigned input value is loaded and passed after a selected index; no
new test cases/helpers/assertions, and actual constructor/call/execute consumers
migrate together.

### GCR-P3 — P2

crates/fabelgeist-compute/src/stream/mod.rs:106-124, 203-227;
crates/fabelgeist-compute/src/stream/call.rs:7-42

known existing correctness defect; confirmed still present, deduplicates GC-C2.
A value:u32 before a later explicit index:u32 initially becomes the alias index.
The later explicit index replaces it, but has_input_val/input type are not
recomputed. The builder can omit in_val while call_arguments still emits in_val
for the displaced value.

Rule: crates/AGENTS.md:30-70; single role authority.

Connected correction: Resolve final explicit index/fallback before classifying
value roles and derive input-load presence/type from those final roles. Retain
offset:u32 and resource-pointer-only/no-value MC/DC writers.

Acceptance: Parameter ordering cannot remove a required load, and
index/offset/resource-only writers remain valid without a synthesized load or
new DSL arguments.

### GCR-P4 — P2

crates/fabelgeist-compute/src/stencil.rs:483,510,535-539,586-617;
crates/fabelgeist-compute/src/resource_pipeline.rs:89-94

existing connected correctness defect; independently confirmed, deduplicates
GC-C4. Buffer workgroups cover X only. Neighbors2D and Neighbors3D construct
coords from global_id.xy/xyz rather than decomposing the flat record ordinal. In
the established 64x64 Buffer default with 4096 records, output64 is row1,x0;
left should clamp to input64, but emitted coords=(64,0) fetch input63. 3D
likewise never reconstructs y/z.

Rule: crates/AGENTS.md:30-55,80-84; coordinate/layout operations preserve frame
and logical population.

Connected correction: Admit the actual/default physical grid against
input/output logical records and project each Buffer ordinal through that
admitted grid before center and neighbor fetches. Keep texture independent-axis
behavior. Settle 64-axis/signed-arithmetic policy in the same connected family.

Acceptance: Buffer row/plane crossings, Clamp/Wrap and center/neighbors use one
layout. Logical prefixes, grid products, signed conversion and neighbor
additions are admitted before launch; no new fixture is authored.

### GCR-R1 — P2

crates/fabelgeist-compute/src/gather/mod.rs:13-20;
crates/fabelgeist-compute/src/scatter.rs:9-16;
crates/fabelgeist-compute/src/stream/mod.rs:9-18;
crates/fabelgeist-compute/src/stencil.rs:15-27

existing migration gap in transferred owners; not a new started-Map regression.
Four signatures still expose independent ordered declarations/names, selected
coordinate/type projections and optional flags; Stencil additionally has mode
bool, raw dimension, independent center/neighborhood names and positional user
tuples. Public mutation can represent contradictions. Build reparses code, so
this is not a claim that mutating a returned signature currently reaches the
private builder unchecked.

Rule: crates/AGENTS.md:30-70.

Connected correction: Complete private ordered local role records with canonical
ShaderParameter/CoordinateParameter leaves, retain immutable authored
declarations only for lexical emission, and derive partitions/presence/query
projections. Migrate parse/build/execute/call helper/default/direct callers
together; no raw-field compatibility API.

Acceptance: No independently mutable semantic columns or retained presence/mode
flags remain. Actual resource membership and contextual primary selection are
preserved, and no signature is separately cached beside mutable Definition.code.

### GCR-R2 — P2

crates/fabelgeist-compute/src/stencil.rs:113-149,177-185,445-527,565-713

existing protocol/migration gap plus required physical-policy decision.
contains(Neighbors) selects neighborhood mode; unknown names silently retain
dim=2, while contains(Neighbors1D..4D) classifies dimensions. This is not an
exact closed protocol. Its physical Buffer 64 defaults, optional size
projection, signed i32 add/multiply/wrap and logical records are not admitted by
unsigned texture flattening. A closed role enum alone would not fix GCR-P4 or
certify these ranges.

Rule: crates/AGENTS.md:21-70,80-84.

Connected correction: Define exact outer neighborhood protocol and a
payload-bearing resource/neighborhood schema, preserving optional center and
center/val/fallback precedence. Separately but before family clearance,
decide/admit the established grid defaults, prefix populations and signed
neighborhood address calculation. No invented epsilon, arbitrary cap or silent
default removal.

Acceptance: Unknown/custom substring names do not silently select a reserved
mode; all four actual neighborhood dimensions/aliases are represented, and
actual Buffer/texture grid/size/stride bounds resolve as an implemented contract
rather than a deferred note.

### GCR-R3 — P3

crates/fabelgeist-compute/src/gather/mod.rs:302;
crates/fabelgeist-compute/src/scatter.rs:280;
crates/fabelgeist-compute/src/stream/mod.rs:277;
crates/fabelgeist-compute/src/stencil.rs:821;
crates/fabelgeist-compute/src/advect/mod.rs:8-15,191;
crates/fabelgeist-compute/src/gradient/mod.rs:167;
crates/fabelgeist-compute/src/divergence/mod.rs:156;
crates/fabelgeist-gpu/src/data/gpu/compute/signature/coordinates.rs:50-54;
crates/fabelgeist-gpu/src/data/gpu/compute/resource_addressing.rs:83

existing changed-owner rule debt; started Map already repairs its unit-struct
order. Seven executor unit structs follow impls; Advect cache alias follows a
struct. Cohesive coordinate/address admission modules repeat one explicit error
Result instead of their owning aliases. Reference Compute lib source also
interleaves exports/modules; composition remains peer-owned.

Rule: crates/AGENTS.md:90-94,118-122.

Connected correction: Move whole attached items into required order and
introduce/reuse owning error-specific aliases for those modules; preserve mixed
ResourcePipeline anyhow results and trait signatures. Coordinate public export
changes with peer lib/prelude owner.

Acceptance: Final owned files follow item order, tests remain last, aliases
retain exact errors/source chains, and peer compositional exports satisfy the
same rule before integration.

## Source-backed traces

Map: mutable source -> NativeParameter lexical declarations -> private ordered
SourceParameter<MapParameterRole> -> transient actual primary/resource/uniform
partition -> original call order/native program -> complete
source/device/policy/recipe key -> same-source reflected ResourcePipeline
layout/shape -> actual controlling output dispatch. Build reparses current
source; no separately retained stale signature. Primary is contextual input
presence. Immutable native lexical spelling serves actual DistanceField wrapper
emission.

Sibling algorithms: public parallel raw signature columns -> builder
reparsing/classification -> generated source -> cached pipeline -> actual
resource dispatch. Gather/Scatter input/resource metadata, Stream
index/offset/writer metadata, Stencil resource/neighborhood1..4D and optional
center were read fully. Correctness defects and represented incompatible state
above prevent family clearance.

Native quantities: actual Naga Array.stride -> BufferElementStride -> Buffer
logical ByteLength -> BufferElementCount -> invocation population -> reflected
WorkgroupShape -> WorkgroupGrid. Texture independent axes retain full dimensions
unless generated scalar ordinal/population requires the explicit checked
TextureAddressing projection. These do not prove Stencil signed physical
topology.

Actual receivers: full noise/JFA Map generator paths, DistanceField
declaration/name wrapper accesses, MC/DC Gather and Stream metadata-only pointer
writers, Scan/extraction count orchestration and existing fixtures read. No new
args synthesized for authored native globals; actual custom vertex stride
reflects native storage. Full geometry tables/output/all applications are not
cleared.

## Rules, no-growth and validation limits

rust-quality-baseline.toml stencil file924/build579/parse133; Gather build174;
Scatter build160; Stream build125; Advect build127; old Map build233. These are
retained records, not independently recomputed checker findings.

Stencil1082 whole, cfg(test) begins923; build_pipeline210-792. Existing
oversized responsibilities require real extraction before expansion, never a
higher ceiling. New Map files216/116/113/110 and common lexer78 are below500;
Map build42-85/execute123-212 and other new whole functions are below100 raw
lines. Raw spans are conservative source bounds, not clone checker results.

Relevant baseline reductions required from hosted authoritative export after
final coherent source. No local scanner/project check or ceiling update by
reviewer.

Derive equivalent sibling Default implementations if desired without changing
code-only equality or generic bounds.

Manual native Error/Display may validly remain without adding GPU thiserror
dependency; coordinated manifest/both locks required if choosing derive.

Discovery input before Root import/repair; not postrepair/all-PR/all13
implementation clearance. Every lens has a recorded
contract/disposition/repair/dependency/acceptance, but material
policy/implementation gaps remain required.

All27 owned and36 non-lock reference files freshly read completely; both locks
handled by complete TOML package/dependency bookkeeping only. Historical packet
notes not used as current source-body coverage. One shared exact source table
below; no sample/hash body credit.

Root native reference anchor cac5a7c33a83d1c08a5e55d43d912967edf06814 preserves
prior complete GPU body receipt; frozen packet reference hashes independently
compared to that pin. Moving Root HEAD
observed6e56adeab1101743941c6d50eef4340008b910ea is bookkeeping only, not
reviewed implementation scope.

No fresh full body coverage for extraction/output.rs, marching-cubes tables, all
extraction fixture files, complete external application graph, full
physical/hair/body/material domains or general numerical geometry correctness.
Those remain peer-owned required integration/audit dependencies, not clear from
orchestration reads.

Native SDK std/thread-key/Error SendSync and browser policy carry previously
source-read29.0.4 evidence; no new platform execution or atomic-Wasm
verification. Do not overwrite Root progress/error policy using differing peer
reference bodies.

DistanceField copy record-time immediate native failure not asserted: SDK29
defers actual native encoding/finalization; Root submit covers finish/submit.
Peer owns final raw-encoder policy/consumer closure; no speculative observer
addition.

No source edits, project execution, build, lint, tests, probes, quality scanner,
generator, runtime, push/CI or external posts. Only source/Git reads, exact
identity/TOML bookkeeping and ignored report writes. No new
fixture/case/helper/assertion/doctest.

Complete connected owned repairs and peer caller/export/lock integration before
source push/requested hosted CI; final fresh all13, independent
Code/Rule/Comment and required hosted checks must pass one final pushed SHA. No
merge.
