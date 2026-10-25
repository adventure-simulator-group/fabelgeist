# Receiving result ownership review

Pinned Root `ccb1b6944d674c0bf29d10e6820c0ac4c55092b7`; tree
`4cc54cebccefd4eb249900fb22d160cdaba53485`. The complete focused source account and
all13 contracts are in the JSON. No source edits or project execution occurred.

RS-R1 (P3): `TextureValue::try_from` at `texture_value.rs:32` hides the trait
associated error behind `TextureValueResult<Self>`. The explicit exception at
`crates/AGENTS.md:118–122` requires trait error parameters to remain visible.
Use `std::result::Result<Self, Self::Error>` there. Retain the owning alias for
nontrait fetch/store consumers and preserve its actual error and method body.

Root selected RS-S1 as an accepted focused cleanup: use one `StagingResult<T>`
for the two `StagedResults` decoding methods. The same rule explicitly permits standard
results where errors differ, and this module also contains facade APIs returning
`GenerateError`; therefore repeated explicit results do not independently prove
a must-fix violation. Keep the facade family separate. Never widen errors or
replace source causes merely to shorten signatures.

Texture admission retains its private scalar/lane relationship and typed wrong-
type error. Staging retains exact session/ticket membership, checked byte extent,
zero-sized rejection, fallible allocation, zero-prefix behavior and native
readback/source chains. Those bodies and native shader tokens need no change for
the proposed result spelling.

Current old body/part/bracer/breastplate/Creator callers still require the agreed
borrowed fallible ticket API and canonical grouped Readback receipt. The Root
Readback API currently differs from the facade's expected canonical API. These
are required #780 dependencies, not repaired or cleared by alias spelling.

No tests, cases, helpers, assertions, doctests, native loops, waits, traits,
error variants or compatibility APIs were authored. Both focused owners are
small; alias belongs before structs. Relevant final hosted baseline reductions,
complete connected integration and final same-head reviews/CI remain required.

| Lens | Contract/disposition | Acceptance |
| --- | --- | --- |
| magic-numbers | WGSL lane arities2/3/4 and padding zero are native grammar; status nativeu32 and prefix one/first index0 are inherent host/native decoding. Host usize count/size_of is actual allocation/index boundary, checked before arithmetic use. Intentional boundaries; no new count/byte/ordinal owner for signature spelling. | Keep byte conversions in BufferByteLength errors and exact existing arithmetic; no literal replacement or new policy. |
| magic-strings | WGSL type names, lane suffixes and native constructor expressions are shader format tokens. Error Display is presentation, never compared. Staging readback label is diagnostic. Preserve native token/diagnostic bytes; no protocol classification from prose. | RS-R1 and optional RS-S1 body/Display/native literals exact; no error-string branches. |
| existing-type-owners | ResourceBaseType/TextureValueError GPU, BufferReadError GPU, ReadbackError Compute, StagingError and GenerateError Armor already canonical. Reuse aliases of actual owners; no duplicate error/count/device/type. | D-STAGING-CALLERS/D-GENERATED consumed without compatibility types/helpers. |
| type-ownership | TryFrom associated Error is trait-owned API spelling; TextureValueResult remains legitimate nontrait owner alias. Staging decoding and facade GenerateError distinct families. RS-R1 confirmed; RS-S1 focused optional alias, mixed-error exception explicit. | Trait Self::Error explicit; facade readbacks keep GenerateError; no imports of facade-private Result. |
| type-granularity | TextureValue checked scalar+closed lane shape, session-branded staged ticket, host byte-error payloads and distinct operation errors express different concerns. No extra wrappers, generic facade error or repeated new native owners. | Only result-style tokens change; preserve aggregate private state and error variants. |
| enum-shapes | TextureComponents owns scalar/pair/triple/quad; ResourceBaseType already owns scalar/vector/resource role. StagingError distinct invalid ticket/overflow/decode cases, Decode owns BufferReadError cause. Current enum forms carry appropriate payloads; no Boolean or string-driven status replacement in owned files. | No enum/schema/serialized role changes for style repair; pending producer status decoding stays #780 whole function. |
| admission-contracts | TryFrom sole checked TextureValue entry; Staging session check precedes index, zero-sized prefix rejected, checked multiplication/range/allocation before copy. No new bypass or native bound observed in focused owner sources. | Preserve exact precedence, zero acceptance, requested/available typed byte payloads; downstream callers propagate errors. |
| aggregate-states | TextureValue scalar/components tied by sole constructor; staged session/order/index/bytes tied by private fields and Arc::ptr_eq. Results publish only after grouped read succeeds. No fresh duplicate authority; stale caller API is D-STAGING-CALLERS and prevents whole-family clearance. | No alias cleanup changes session identity/publication; consume exact source-associated canonical readback/callers. |
| invariant-operations | Checked count*element_bytes; checked byte prefix; fallible host allocation; status fixed index proved by prefix. Native read/submit/mapping errors retained. Existing owning arithmetic inspected; alias change requires no arithmetic or runtime repair. | No new unwrap/expect/unsafe, truncation/padding synthesis, allocation/copy/wait or assertion. |
| redundant-authority | Alias is a name for the same result/error owner, not a second state or compatibility path. ResourceBaseType is actual projected native type; session is actual membership authority. Keep TextureValueResult for consumers, optional single StagingResult, no facade-error alias replacement. | No duplicate same-concept types/constants or independently cached type/format/session metadata. |
| producer-propagation | GPU type -> checked TextureValue -> fetch/store generated source -> mixed builder error; staged Buffer -> canonical grouped readback -> session results -> borrowed fallible prefix/status -> facade/application source chain. Direct producers/consumers inspected; D-STAGING-CALLERS and D-GENERATED required incomplete paths. | No source-complete/compile verdict until all direct old callers and canonical exports/APIs/locks integrated. |
| behavior-preservation | Trait return spelling and optional alias are semantically equivalent type names; no native order or observable data contract should change. RS-R1 is style, not a claimed runtime bug; RS-S1 literal mixed-error permission retained. | Exact body/error/doc/fixture/native-call identity, existing tests only; no broadened renderer/art/native policy. |
| derivability | TextureValue/Components derive Clone/Copy/Debug; StagingSession/Session containers derive existing appropriate traits; StagingError derives Error with #[from] source. GPU lacks thiserror dependency; manual TextureValueError Display/Error preserves exact diagnostic. No requirement to add a derive crate or manually expand derived types. Result alias adds no trait semantics. | No new dependencies/trait bounds/serde/type construction routes, tests/helpers/assertions; maintain item order alias before structs and no-growth ceilings. |

All thirteen focused discovery lenses are inventoried before edits. The JSON
has one shared exact-hash source table with whole-file/function/range limits,
owner-to-caller traces and required dependencies. This is not compilation,
whole migration or final clearance.
