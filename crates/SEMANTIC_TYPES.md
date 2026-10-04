# Semantic types and boundary contracts

The destination is bespoke types throughout handwritten function interfaces.
Types own identity, units, validation, states, indexes, counters, and the
quantities used by mathematical kernels. Remaining raw parameters are migration
debt, including local helpers. The rules in [AGENTS.md](AGENTS.md) apply to both
new code and conversions of existing code.

Constructors and representation conversions necessarily admit primitives.
Storage, external APIs, and external trait signatures may mandate raw values;
keep those adapters narrow and convert before calling domain functions. A type
alias does not establish a distinct identity. Define each concept at its shared
owner and migrate its producers and consumers together.

## Ownership

| Concept | Owner | Boundary representation |
| --- | --- | --- |
| Durable character and inventory-row identity | `adventuresim_core::identity` | Numeric `u64` |
| Exact item-definition identity and missing catalog references | `adventuresim_core::item_catalog` | Unmodified catalog/storage spelling; existence is checked separately |
| Inventory grant quantity and stack allocation | `adventuresim_core::inventory_measurement` | Native unsigned counts admit an explicit empty request or nonzero items; every issued row retains `ItemQuantity` |
| Party and settlement identity | `adventuresim_core::identity` | Canonical bounded string |
| Nonzero custody character and physical-object identity | `adventuresim_core::physical_object` | Numeric `u64`, checked on admission |
| Mission and investigation authority identities | Their core domain modules | Bounded strings with domain-specific validation |
| Generated quest identities and authored catalog keys | `adventuresim_core::quest_generation` | Bounded ASCII strings |
| Strategic instant and elapsed minutes | `adventuresim_world_schema::calendar` | Numeric minute counts |
| Numeric deterministic seed and exact textual root | `fabelgeist_determinism` | Number and string respectively |
| Tactical input and command serials | `adventuresim_tactical_core::protocol` | Numeric `u32` |
| Bounded probability and signed adjustment | `adventuresim_world_schema` | Numeric basis points |
| Buffer byte length, byte address, readback slot and device status word | `fabelgeist_gpu::data::gpu::buffer` | Device byte layout, decoded at readback |
| Directory child identity and lookup intent | `fabelgeist_fs` | Checked single-component name and the provider's creation flag |
| Complete immediate-child listings and enumeration failures | `fabelgeist_fs` | Native iterator/metadata or caught browser iterator protocol, admitted before exposing `Entry` |
| Entry display labels, provider namespaces, selected roots, and application-directory identity | `fabelgeist_fs` | Native OS filenames, browser handles, and platform directory identifiers, admitted at their owning boundary |
| Exact file contents, UTF-8 file text, and resource addresses | `fabelgeist_fs` | Serialized bytes/text and external address spelling, admitted at the owning boundary |
| Runtime character mesh detail, corrective policy/availability, and MHR asset roles | `fabelgeist_mhr` | Checked numeric detail, boolean policy, and authored asset filenames at their owning boundaries |
| Corrective rig topology, admitted network/basis dimensions, sparse coordinates, and network admission failures | `fabelgeist_mhr::correctives` | Shared character topology and checked array/coordinate encodings, retained through admission and translated only for device APIs |
| Model/rig/set parameter identities, joint rows and channels, model slots/counts, admitted limit intervals, and definition diagnostics | `fabelgeist_mhr::model_def` | Exact model-definition tokens and checked numeric intervals; identities and coordinates remain nominal through lookup, contribution building, and the creator's proportion binding |
| Model evaluation rows, pose/identity/expression column roles, broadcast admission, and evaluation failures | `fabelgeist_mhr::model` | Tensor SDK dimensions acquire their roles at layout construction; validation retains nominal counts and returns concrete variants, while pose columns remain distinct from the appended transform's total columns |
| Serialized byte lengths, addresses, checked spans, and borrowed views | `fabelgeist_storage` | Actual byte slices and checked little-endian representation conversions; format constructors assign their own tag/count roles immediately |
| Archive member identity, NumPy array lookup, ZIP tags, and archive admission failures | `fabelgeist_numpy_storage` | Exact filename bytes and checked little-endian protocol fields, admitted before member or array consumers |
| FBX version/layout, property and array counts, tags, framing, and decoder failures | `fabelgeist_fbx` | Checked binary fields and shared storage spans; node/property bounds are admitted before consumers and failures retain concrete causes |
| FBX scene identities and connection-property roles | `fabelgeist_fbx` | Signed integer/boolean identity properties and exact lossy UTF-8 connection names; shared identities remain nominal through graph, animation, rig traversal, skin lookup, and diagnostic provenance |
| Exact skeleton joint labels, ordered rig slots/counts, authored anatomical fragments, skin membership, and missing-label diagnostics | `fabelgeist_rig` | Exact model/glTF/Bevy labels and numeric GPU words; source normalization precedes admission, JSON stays scalar, and labels/slots remain nominal through lookup and binding |
| Animation clip/profile labels, loose retargeting keys, inference vocabulary, admission choices, and resolution failures | `fabelgeist_animation::animation` | Exact scalar JSON labels and boolean choices; loose matching keys remain separate from retained joint identity, and errors retain ordered roles and candidates |
| Skeleton/skin collections, local/model poses, chain positions, vertex bindings, and skinning failures | `fabelgeist_animation` | Numeric scalar slots and array poses retain JSON shape; rig, skin, clip-track and prepared-chain addresses stay distinct, and skinning reports nominal count mismatches |
| FBX normalized object labels and qualified animation targets | `fabelgeist_fbx` | First NUL/one suffix cutoff, lossy UTF-8 and last namespace-prefix removal; the two label authorities remain distinct |
| FBX record/subclass identities, property-table keys, and model roles | `fabelgeist_fbx` | Lossy UTF-8 record/class decoding and exact property-name bytes; selectors remain distinct through node lookup, animation, rig traversal, shape/skin selection, and diagnostic provenance |
| Rig admission diagnostics, polygon/array counts, vertex slots, object provenance, dense-shape counts, and model loading stages | `fabelgeist_mhr::character` and `fabelgeist_mhr::model` | Decoded rig metadata receives nominal diagnostic roles; structured loader errors retain provider/decoder causes and established admission order |
| NumPy shape, axis extents, element counts, occupancy, and storage width | `fabelgeist_numpy_storage::npy` | Supported header tokens and checked dimension products; metadata stays immutable after array admission and converts directly for device SDK layout |
| Decoded NumPy floating/signed values, payload-byte states, and value ordinals | `fabelgeist_numpy_storage::npy` | Exact stored words and preserved numeric casts; nominal values receive a model/game role before internal domain operations and convert to native representations only at SDK boundaries |
| Shader resource and flat pass-parameter identity, group/binding coordinates, and uniform ABI packing | `fabelgeist_gpu::data::gpu` | Exact authored/Naga names and numeric coordinates enter separate owners; byte addresses and lengths reuse buffer units, and scalar casts occur only at uniform serialization |
| General compute dispatch shape and surface attribute extraction | `fabelgeist_gpu::data::gpu::compute` and `fabelgeist_compute` | Three-axis workgroup words enter a private grid; surface extraction retains nominal byte lengths, pipeline resources and allocation/dispatch causes |
| Shader invocation dimensions, x item coverage, recorded dispatch counts, batch labels and copy bounds | `fabelgeist_gpu::data::gpu::compute` and `fabelgeist_compute::kernel` | Nonzero declarations remain distinct from grids and item counts; diagnostic labels borrow exact spelling, and copies retain shared byte units and logical range context |
| Radix-sort pair counts, key widths, tile layout, digit schedules, and staged failures | `fabelgeist_compute::sort` | Native counts and widths acquire distinct roles at construction; counts retain four-byte word layout, widths preserve clamping, and dispatch/copy failures retain concrete causes |
| Physical buffer allocation, retained logical byte results, offset writes and bounds admission | `fabelgeist_gpu::data::gpu::buffer` | Shared byte lengths and addresses remain nominal through allocation, readback, recording and checked writes; logical result admission checks the retained native allocation and preserves empty results |
| Borrowed buffer upload representations, empty binding occupancy, allocation labels and usage choices | `fabelgeist_gpu::data::gpu::buffer` | Exact `NoUninit` bytes enter their borrowed upload owner; diagnostic spelling and capability choices stay nominal through private descriptor selection and native allocation |
| Draw cardinalities and addresses, neighbor capacities, mesh admission and transfer failures | `fabelgeist_mesh` | Native index words acquire draw roles at decoding; edge construction retains those roles, record/line owners preserve device layouts, and errors retain attribute lengths and concrete causes |
| Quadratic bend points, weights, rest measures, and GPU records | `fabelgeist_shell` | Geometry admission owns ordered hinge points; records retain their eight-word layout through color permutation and upload |
| Constraint labels, parameter attachments, and record admission failures | `fabelgeist_xpbd` | Native `NoUninit` record serialization captures cardinality; internal binding keys stay nominal and allocation causes remain inspectable |
| Particle incidence, constraint identities, arity, graph colors, and solve intervals | `fabelgeist_xpbd` | Exact native particle words enter complete fixed records; original addresses and reordered slots remain distinct through coloring, permutation, and dispatch |
| Active particle counts, physical capacities, host input cardinalities, and particle-state failures | `fabelgeist_xpbd::particles` | Native unsigned narrowing and the sentinel allocation remain explicit; counts stay nominal through uniform/dispatch/sort encoding and errors retain admission order and concrete causes |
| Complete particle position/mass and velocity records | `fabelgeist_xpbd::particles` | Both private record roles retain the native sixteen-byte ABI; position construction admits matching mass cardinality and typed readback rejects partial elements |
| Surface resolution counts and host projection failures | `fabelgeist_shell::surface_contact` | Repeated pair resolutions retain their own count; failed interval admission and state reads retain nominal counts, stages and concrete causes |
| Particle mass, inverse mass, areal density and weighted corrections | `fabelgeist_xpbd::dynamics` | Native unit construction preserves float words; CPU admission and massless policy remain explicit; signed gradients and affine coordinates retain distinct operations and grouping |
| Full host particle-input addresses, kilogram mass counts, and shell admission failures | `fabelgeist_xpbd::particles` and `fabelgeist_shell` | Native host addresses retain their full width; mesh errors retain counts, reference and constraint families, input values and concrete decoder causes, while material errors retain the first failed parameter |
| Self-collision resource roles, construction/recording failures, shell construction and projection failures, and finite-state classification | `fabelgeist_shell` | Exact shader and buffer roles retain native layout and ordering; errors retain concrete lower causes and nominal capacities/stages, while position-state queries distinguish nonfinite state from readback failure |
| Full host collider counts, native allocation capacities, analytic/mesh shader roles, and body collision resource/update/recording failures | `fabelgeist_physics::contact` | Counts retain host width through diagnostics and their native narrowing through capacity/parameter operations; structured failures retain allocation capacity, shader role or previous-position copy with concrete causes |

| Solver frame/substep intervals, subdivision and sweep schedules, acceleration/drag/speed settings, compliance, and structured driver/hook failures | `fabelgeist_xpbd` | Native words enter unit construction; internal schedules retain distinct counts and indices, while uniform/serialized settings adapters encode directly and errors retain shader, constraint, hook phase, substep and concrete causes |

Reuse the shared owner in `adventuresim_world_schema::identity`. Core
aggregates this same `CharacterId`; the tactical component contains it, while
adding ECS component and reflection behavior. The creator's
local article identity belongs to an editor session; convert it explicitly when
building the durable equipment graph. Neither identity proves that a row
exists or that the caller may use it. Keep permission grants, custody proofs,
and authority validation separate from the shared identifier.

Different invariants may justify different types. A custody character must be
nonzero, whereas a durable character identity alone does not promise an
allocated row. Physical objects and inventory rows are different authorities;
do not equate their IDs merely because both use `u64`.

`item_catalog::ItemDefinitionId` distinguishes an item definition from a
character, inventory row, physical object, or display label. Its constructor
preserves every byte, including empty and unknown keys; it does not promise
catalog membership. Catalog and food lookup, forage yields, inventory issuance,
and food-lot creation retain the same identity. Convert native authored and row
keys on entry and encode them only for storage, source lookup, or presentation.
`MissingItemDefinitions` retains missing identities in input order, including
duplicates, and formats them at the diagnostic boundary.

`InventoryGrantQuantity` distinguishes an empty request from a nonzero quantity.
An empty personal grant returns before catalog admission or insertion.
`InventoryRowAllocation` chooses one stack or individual quantity-one rows; its
lazy row iterator conserves the requested quantity without allocating the rows
in advance. Shared personal issuance retains `CharacterId`, `ItemDefinitionId`,
and the resulting `InventoryItemId`. Food lots receive that same identity and
nonzero row quantity, and encode native values only in their persisted fields
and deterministic seed framing.

Inventory issuance retains concrete food-definition, object, weapon, and food
lot failures through `InventoryGrantError`. Weapon projection distinguishes
evaluation, encoding, transport size, and persisted field range failures;
`Error::source` preserves codec and nested inventory causes. Error presentation
must not replace classification or determine control flow. Existing internal
consumers that still return `String` remain migration debt, including explicit
conversions at those consumers; only mandated reducer presentation is a valid
final string boundary.

Stored weapon authentication distinguishes generator and digest admission,
recipe decoding, chassis identity, and projection mismatch. Holder lookup and
fitting retain these causes through object access. View projections may omit
invalid instances; a fitted holder's material query must report authentication
failure rather than silently use the catalog construction.

`ObjectCustodyError` retains core identity and place-decoding causes, separates
stored backing failures from containment and fixture failures, and carries the
failed custody where available. `InventoryContainerError` preserves those
causes alongside object admission and the shared containment graph's errors.
Neither error family grants authority: row identity, exact backing, ultimate
custody, and acting-character admission remain distinct checks in their existing
order.

Food lot identity is independent of the carried row that currently binds it.
`FoodLotId` names that material authority; `InventoryItemId` names its inventory
binding. Lot transfer, revision, and contamination admission retain concrete
failures through `FoodLotMutationError`, including food-object and medicinal
split causes. Measured amount transfers retain the missing source binding in
`InventoryAmountError`. Container rehoming preserves both families.
Native reducer presentation formats these errors once; unfinished internal
consumers remain explicit migration debt.

Persisted fireplace admission retains fixture decoding and custody failures in
`FireplaceCustodyError`. A canonical fixture must still have the fireplace role,
and station identity, return custody, object identity, and dish authority are
validated in their existing order. Character and temporary-party deletion share
`CharacterDeletionError`; refusal to delete a persistent character remains a
distinct failure from inventory cleanup. These changes do not move tactical
state into durable storage or authorize database resets.

Measured consumption and transfer retain `InventoryItemId` through mutation;
only native table lookup, row insertion, and deletion encode its numeric value.
`InventoryAmountError` owns the shared missing-state contract. Alcohol and soap
consumption preserve that cause in their own errors, with serving admission and
planned soap identity checks remaining separate. Best-effort ordinary alcohol
use retains its existing omission policy; explicit use reports its failure.

`MeasuredItemAmountMicros` is an aggregate stock amount across measured rows.
It is distinct from `ConsumableFractionMicros`, which cannot exceed one whole
row. Aggregate checked addition, saturating availability, bounded row requests,
and subtraction of actual consumption belong to the amount owner. Forge
quotation and consumption retain this amount and exact item-definition keys;
serialized quote keys and numeric values keep their existing JSON shape.
Quote duration and material mass still have primitive representations and
remain migration debt.

`ForgeQuoteError` distinguishes property derivation, material derivation,
untraded stock, and range/aggregate overflow. Its derivation aggregate preserves
every validation cause in order and exposes the first through `Error::source`.
Browser quote transport retains decoding, quotation, and encoding failures
until the mandated Wasm export formats the error. `MeasurementError` implements
the standard error contract so lower arithmetic causes remain inspectable.

`WaterMaterialContribution` couples exact volume with
`WaterContaminantMicrounits`, an extensive load rather than a concentration.
Its proportional sample preserves integer remainders at the source and samples
load from the rounded transferred volume. Persisted contribution transfer
retains `MaterialLotId` and `PhysicalObjectId`, and rejects zero stored lot
identity before mutating that row. `WaterContributionTransferError` preserves
the public-volume coordinates or concrete material admission cause. Invalid
stored identities are diagnostic boundary data, not usable domain identities.
`EpisodeDecodeError` admits persisted infection episodes in ruleset-version,
phenotype-key-version, then disease-key order. It retains `InfectionEpisodeId`,
the rejected stored value and the exact parser cause. `CharacterId` remains
nominal through episode reads, attribute impairment, capability admission,
equipment and mass reads, presence closing, and point-exposure protection.
Neither identity proves existence or permission.

Capability admission shares its ordered attribute, disease, skill, limb and
stat reads in `CapabilityInputs`. `CapabilityEvaluationError` retains the
missing character/component or episode cause. `DiseaseProtectionError` keeps
historical presence failure distinct from live capability failure; a failed
historical read does not silently select the live fallback. Disease notices
use a typed kind and source. Aggregate terminal failure has its own source
variant rather than a fabricated episode-zero identity; native notice keys
and messages retain their established encoding.

`StrategicConditionError` owns missing condition state, rejected stored religion
and threat keys, party membership, and concrete capability/clock causes.
Mental checks and body-state admission retain `CharacterId`. Condition
projection keeps the existing read order, morale mathematics, writes and
subsequent capability refresh in one transaction. `WorldClockError` retains
the official-clock singleton's failed initialization contract. Format these
errors at actual reducer presentation boundaries, and keep their causes
through internal consumers.

Readiness admission distinguishes individual and party-member refusals while
retaining `LivingCharacterError` and `StrategicConditionError` causes.
Projection
failures retain the requested character or the ordered participant list; a
failure in another member never replaces the requested actor's identity.
`PartyReadinessError` retains the participant error through whole-party
admission. Only incapacitation refuses ordinary action: staggered participants
remain eligible. Check every member's life state before party projection, and
preserve holy-day checks before each member's incapacitation refusal.

Departure revalidation retains a nominal location snapshot and character leader.
A missing or ambiguous location cannot authorize incapacitated withdrawal;
only an exact case-site origin qualifies. Pending incidents retain case-site
identity and duplicate occurrences: two incidents at the same site still refuse
departure. Check the synchronized party before reading members, stop member
reads at the first location refusal, and require ordinary readiness afterward.
Active-contract admission remains the responsibility of the contract owner.

Strategic character authority admits the registered gateway first, then the
owner of the exact disposable simulation character. `GatewayAdmissionError`
retains the recorded and submitting identities. The simulation owner check
retains those identities and the target character, and does not read membership
for another owner. `StrategicCharacterAuthorityError` retains both refused
alternatives, with the simulation refusal as its standard error cause. A
gateway grant avoids simulation reads entirely. These checks do not establish
character existence, liveness, nonce ownership or a new run-ID relationship.
Native reducers admit their character word once and format the refusal at their
presentation boundary. Primitive parent interfaces and generic internal
callers remain migration debt.

Encounter admission retains `PartyId` and distinguishes an awaiting choice from
an open road challenge bound to the party's current location. Read the bound
challenge before the encounter choice, retain the existing early-stop challenge
scan, and give awaiting choice precedence when both authorities are pending.
`PendingEncounterError` retains party context and decoding causes; malformed
stored keys are refused rather than interpreted as another party.

Investigation live prerequisites and position admission retain concrete
readiness, encounter, contact-decoding and action-vocabulary causes through
`InvestigationAdmissionError`. Stable coded refusals remain separate from human
text and preserve their reducer envelope. Member refusals retain nominal
identity, and insufficient membership retains the ordered selected members and
requested action. Validate readiness before encounters, then journey state,
co-location, predecessor completion, observer-safe leads and exact position.
The authorized executor checks live prerequisites again at the mutation
boundary. `InvestigationExecutionError` retains admission, living, custody,
rights-question and commit causes; the native reducer formats them. Generic
lower providers and planned-action callers remain visible migration debt.
Primitive parent keys, native row projections, catalog prerequisites, tracking
predicates and higher generic errors remain debt. Pure errors and source-order
assertions do not establish SDK reads or transactional behavior.

`DepartureRevalidationError` preserves readiness and stored-identity causes.
`DepartureClockError` retains the ordered requested members or failed party,
and preserves the official clock cause. Departure clock initialization retains
its native write order and canonical anchor without advancing subjective age.
It returns the established minute directly; no successful empty-clock outcome
exists. Travel retains these causes through `TravelError`, and native reducers
admit caller identities before invoking the handwritten travel implementations.
Format errors only at those reducer boundaries.

Route validation retains geometry, bound checks and admission order through
`RouteAdmissionError`. Stale departure weather retains the synchronized minute,
expected interval and provided snapshot interval. Camp redirection uses that
same weather rule. Invalid native destination identities retain their decoding
cause alongside the existing reducer refusal text. These migrations leave
lower travel services with generic errors, primitive route metadata, parent
keys and coordinates as visible debt. Pure admission and source assertions do
not establish SDK reads, writes or transaction rollback.

Condition projection, morale shares, religion context, personality reads and
neutral construction, language comparison and address pruning retain
`CharacterId`. Corpse projection stays isolated from living party support;
supplied member order, duplicates and empty membership retain their meaning.
Native row identity fields encode at their storage boundary. The living party
reader admits native row identities before selecting living
members and retains `CharacterId` through its ordered collection. Missing rows
are omitted, dead memberships remain stored, and duplicates are preserved.
Tactical roster validation, oral-language choices, leadership ballots, event
participants, disease timelines and presence projections retain the same
identity. Native index keys, row fields, event numbers and deterministic byte
frames encode explicitly at their owning boundaries. Identity does not prove
existence, life or permission; custody admission still rejects zero.

This identity work does not complete party text keys, lifecycle text selectors,
seeds, quantities, durations, predicates, higher generic errors or every
character producer. In particular, lifecycle target selection still relies on
text keys and a lexical tie break; replacing it with numeric ordering would
change that contract. Pure selection and admission tests do not establish
database reads, projection writes or transactions.

The official clock distinguishes `OfficialClockEpoch` from an observed
`UnixMicrosecondInstant`. Native Unix microseconds are admitted at database and
system-clock adapters. Elapsed real time uses `RealMicrosecondDuration`, while
strategic elapsed time uses `StrategicDuration`. The epoch owns rate conversion,
inverse shifts and projection to `StrategicMinute`. Signed subtraction still
saturates before future epochs clamp to zero; inverse shifts round up, and
targets before world start retain their existing world-start clamp.
`ClockEpochShiftError` preserves the epoch, both elapsed units and the concrete
narrowing cause or subtraction underflow. The bounded simulation clock advance
owns its nonzero, hundred-year limit and formats failures only in its native
reducer. These operations confer no authority to mutate a world.

Latest stored life admission retains `CharacterId` through the authoritative
character lookup. `LivingCharacterError` distinguishes a missing row from a
dead character and retains the requested identity. `StoredCharacterLifeState`
admits the native `alive` field once before applying the action policy. It does
not infer life at an observer's earlier personal frontier. Lookup precedes
life-state rejection, and zero or full-width identities retain their existing
lookup semantics. Native reducers format these errors at presentation.

Public settlement-rest admission retains the same identity through life and
settlement lookup. `RestServiceAdmissionError` keeps the concrete life cause,
absence from a settlement, missing settlement row and unavailable requested
service distinct. The service predicate continues to use the shared settlement
economy policy. Read order, refusal text and synchronization's availability
probe remain unchanged. Residence admission, rest execution, higher internal
life-error consumers and other stored life flags remain migration debt. Pure
policy tests do not establish database lookup or transaction behavior.

Personal clock reads retain `CharacterId` and report `CharacterClockError` with
that identity. `TemporalScope` carries its actor and, for canonical NPC or
exclusive shared admission, its required second participant. Soft scopes read
only the actor's frontier. Exclusive scopes check the other participant then
use official world time, preserving the existing order. `TemporalScopeError`
keeps personal and official clock causes separate from missing participants or
NPC policy. Courtship preserves those concrete causes through its admission
error; its remaining string state/rejection paths and other higher chronology
consumers remain migration debt. Initializing personal time and the default
schedule shares one implementation and propagates `WorldClockError`.

`StrategicDayIndex` owns the absolute zero-based day coordinate shared by
calendar, schedule, observation, NPC decisions and discovery. It retains day
identity through ordering and inclusive iteration, projects `StrategicWeekday`,
and owns saturating or checked conversion to minute boundaries. Its inclusive
iterator terminates after the final day, including the maximum stored index.
Native row and report integers are admitted or encoded at those boundaries.
The primitive day-start and Sunday helpers have been removed.
The calendar guard recognizes only the shared day owner's exact saturating
minute projection. Copies in consumers and other raw calendar operations in
that owner still fail the guard.

`EveningId` is distinct from an ordinary day index. It identifies the alcohol
boundary at 18:00 and preserves overnight history, the pre-boundary zero clamp,
and the strict rolling-week window. Nightly and crossed-evening iterators retain
that identity; ledger rows still encode their native numeric key. Calendar and
alcohol quantities, preference flags and higher errors remain migration debt.

Secret discovery keeps `CharacterId`, `StrategicDayIndex` and an explicit
`DiscoveryDayOutcome`. A missing personal frontier retains `CharacterClockError`
through pair and character settlement rather than becoming internal text. The
trial owner canonicalizes pair order and frames character/day coordinates as
the same fixed-width little-endian seed fields. Directed `CharacterAffinityKey`
keeps subject and actor distinct from an unordered pair and encodes the existing
native key. Affinity and effective-dated liveness reads retain character
identity; their primitive quantity and predicate results remain unfinished.

Father admission retains child and father `CharacterId` values through kinship
selection, canonical clock admission, effective-dated liveness, affinity and
currency lookup. A missing clock retains the child, requested minute and
concrete `CharacterClockError`; a frontier mismatch retains both identities and
the requested/observed minutes. Earlier and later frontiers both fail. Lookup
order and the absence/death policy are unchanged. Father identity encodes only
at the persisted courtship field or generated lookup boundary.

Courtship keeps the father failure and its source chain while exposing the same
stable father-approval code. Native player reducers encode that rejection once;
the development reducer preserves its existing plain diagnostic. Its other
state/rejection strings, courtship actor parameters, affinity magnitudes and
coin totals remain migration debt. An isolated test harness can import the
actual pure admission/error modules; those results verify the pure contracts,
not SDK reads or transaction behavior.

Character condition initialization is infallible in handwritten logic. It
accepts `CharacterId` and inserts the same missing condition, needs and exposure
rows in order; it does not invent a failure classification for an operation
that returned only success. Native database panic behavior is unchanged.

Core episode records and combat projection still contain primitive fields,
condition projection still has primitive quantities and some identity paths,
and familiarity reset still accepts a primitive identity. Disease interval,
water consumption and higher condition/error consumers remain unfinished.
Those interfaces and their internal string errors remain migration debt;
nominal admission in a lower reader does not complete the whole subsystem.

## Serialization and storage

Raw row fields, generated SDK bindings, reducer arguments, browser messages,
and external source records may keep their mandated scalar representations.
Convert when they enter domain logic and unwrap at the owning boundary. Do not
persist tactical tick state or use a database schema change to move tactical
authority into SpacetimeDB.

A validated constructor is insufficient if a decoder can bypass it. Serde,
SATS deserialization, SATS validation, and reflection must obey the same
invariant. Use a checked wire representation or `TryFrom` during decoding.
Validated values must not expose writable fields outside their owner. Opaque
reflection is appropriate when reflection must not mutate an inner identity.
Numeric identities and wrapping serials allow every value of their underlying
unsigned integer; deriving their scalar decoder is appropriate.

Opaque Bevy components that participate in scene serialization must register
`Serialize` and `Deserialize` reflection type data. Serde derives alone do not
provide that registration. Check both reflected round trips and rejection of
invalid encoded identities; reflection must retain the same scalar admission.

The checked SATS implementation preserves the existing named single-field
product and BSATN numeric encoding. Keep field names, product structure, enum
variant names, integer widths, and JSON scalar shapes under test. Regenerate
SpacetimeDB client bindings with `just generate-db-client` after schema changes;
never edit generated bindings directly.

Rejecting previously accepted invalid serialized data is an intentional
contract correction. This repository uses the clean final API and schema;
backward compatibility, dual decoding paths, and migrations are unsupported.
A deployment may need fresh isolated development data after a schema change.
That does not authorize deleting public or player-bearing data.

## Randomness, time, and counters

`Seed` contains the numeric SplitMix64 initial state. `SeedKey` retains exact
UTF-8 root material. Converting a textual root into a number, normalizing it,
concatenating context fields, or pre-hashing it changes deterministic output.
Preserve purpose names, length framing, little-endian numeric context, stable
ordinals, candidate order, and draw order during a type conversion. Current raw
numeric framing signatures are migration debt; their eventual typed interfaces
must preserve the same bytes and distinguish roots from context words.

Use `StrategicMinute` for an instant and `StrategicDuration` for elapsed time.
An action's nonzero requested duration has a stricter invariant than elapsed
time, which can be zero after clipping or interruption. Keep both concepts.
Collection lengths, loop counters, and indexes need types owned by their
collection or operation when passed through handwritten interfaces. Keep their
semantics separate from durations and protocol serials.

`foraging::ForageAttemptGeneration` owns the per-actor cursor of independent
forage attempts. Admit storage and reducer words once, retain the cursor through
replay comparison and planning, and encode it only for storage, JSON or
authority
digest framing. Its checked advancement rejects a stale cursor before testing
exhaustion and never wraps. Immutable receipt replay still precedes advancement.
The explicit snapshot conversion preserves the same revision word without
making other snapshot revisions interchangeable with forage generations.
`ForageGenerationError` retains submitted and expected cursors or the exhausted
cursor; only the native reducer boundary formats these failures as text.

`foraging::ForageExposure` keeps difficulty evidence coupled to a checked
stealth outcome. Unchecked exposure has no difficulty; checked exposure is
concealed or detected. Internal resolutions retain that owner. Convert its
native optional difficulty and success fields only when storing the private
receipt. Only detected exposure produces the existing Infamy consequence.

`ForagePublicLegalOutcome` owns the player-safe receipt vocabulary. Its explicit
projection preserves the existing noticed wording for an illegal unchecked
attempt; that public wording is not evidence that a detection check occurred.
Admit receipt words exactly before rendering or declaring submission results
visible. Unknown words return `ForageLegalOutcomeError`, whose diagnostic
retains
the input. `ForageResource::try_from` admits exact catalog keys and returns
`UnknownForageResource` for unrecognized spelling; consumers retain the admitted
catalog definition rather than displaying an unchecked key.

The web `ForageReceipt` admits the complete native result before rendering or
acknowledgement. It binds the response to the selected `CharacterId` and receipt
reference, checks parallel yield columns before pairing them, and retains
catalog resources with bounded `ForageYieldQuantity` values. Completed harvests
and interrupted elapsed time have distinct states; interrupted receipts cannot
carry a harvest. Empty quantities and repeated resources are rejected with
structured errors. Native wire fields remain in generated bindings.

`ForageYieldQuantity` refines the shared nonzero `ItemQuantity` to the forage
receipt's native range. Resolution preserves saturation at `u16::MAX` and
rejects an empty computed yield. Receipts reject zero without clamping. The
quantity stays nominal through resolution, inventory issuance and presentation;
its unit iterator conserves the count while issuing one inventory row per unit.
Encode the native `u16` only when constructing the stored receipt. Forage
resource catalog fields, shared inventory grant errors and neighboring
numeric helpers still have migration debt.

Input ticks, equipment commands, jump commands, and posture commands have
separate serial spaces. Equipment revisions are equality tokens. These values
wrap; do not derive `Ord` or compare them using ordinary numeric ordering.
`is_newer_than` accepts the forward half of the serial space and rejects the
ambiguous midpoint. Preserve serial state when reconnecting to a transient
player. Use `Option<InputTick>` to represent no accepted packet rather than a
numeric sentinel plus an initialization flag.

## States and errors

Directory lookup and deletion retain `EntryName` through both native and
browser backends. It admits one nonempty child component and rejects dot
traversal, path separators, drive prefixes, and NUL. It preserves accepted
spelling and Unicode; the filesystem remains the authority for existence and
access. `EntryLookupIntent` distinguishes lookup without creation from creation
if missing. Native file creation uses an exclusive create operation so another
creator cannot cause existing bytes to be truncated.

`EntryAccessError` retains the operation, child, native parent address when
available, and concrete provider cause. Browser failures also retain their
protocol stage, including method invocation, promise/handle decoding, and
asynchronous rejection. `BrowserIoCause` retains the original provider value;
`std::error::Error::source` exposes the concrete cause through its mandatory
borrowed trait interface. Formatting happens through `Display`.

Directory enumeration retains `DirectoryContents` through its consumers. Its
private collection exposes nominal `Entry` values in provider order and never
returns a successful partial listing after failure. `DirectoryListingError`
retains the directory, native enumeration phase or browser protocol stage, and
original provider cause. Native metadata failures retain the child address.
Browser iteration decodes completion according to the JavaScript iterator
contract and rejects unknown child kinds or malformed handles.

`EntryLabel` retains a provider display label, including native filename bytes;
presentation conversion does not confer child lookup or address authority.
`DirectoryNamespace` retains that authority as a native directory or browser
handle. `ProjectRoot` owns selection and persistence. Native saved-root records
preserve absolute OS path encoding without trimming or URI rewriting. Browser
records retain handles. `WorkspaceStore` owns the exact IndexedDB schema tokens
and waits for transaction completion before reporting successful reads/writes.
Database, transaction, and request owners manage connection/callback cleanup,
including caller cancellation. `WorkspaceStorageError` retains operation,
saved directory when writing, protocol stage, and original provider cause;
missing causes, incomplete reads, and interrupted drivers remain structured.
Permission queries and requests each admit a closed
`WorkspacePermission` before further policy runs. `WorkspaceAccessRestriction`
contains only prompt and denied states; granted access cannot appear in a
`PermissionNotGranted` restoration. `WorkspaceError` retains record/root
context, permission directory/operation/stage, and original provider causes.
`ApplicationIdentity` owns
platform directory selection and rejects invalid namespace inputs. See the
owning filesystem contract for storage and provider details.

Native path constructors admit external namespace addresses; they do not prove
that those namespaces exist. Browser handles and provider values retain their
originating-thread restriction through checked wrappers. Use their native path,
`AsRef`, and provider conversion adapters only at storage or protocol
boundaries.
See [the filesystem contract](fabelgeist-fs/README.md) for backend details.

Picker policy retains `FilePickerFilter`, `FileTypeFilter`, and admitted
`FileSuffixes` through selection. Suffix spelling converts only at the native
or browser protocol adapter. Reuse `ResourceMime` for accepted MIME types.
`PickerError` retains browser kind, stage, and original provider cause; the
native SDK's optional selection cannot provide a cause it does not expose.

File and resource loading retain `FileContents`, `FileText`, and
`ResourceLocator` through internal consumers. Representation adapters belong at
storage, serialization, and format-specific decoder construction. Resource
failures retain their address and operation; file failures retain their file.
Browser failures retain the method/stage and original provider value. Cached
blob failures preserve the original cause and identify reuse. A project URI
requires its configured namespace rather than using the process directory.

`CharacterLod` owns the three runtime levels and admits numbers and spellings
at construction, CLI, and serde boundaries. Loading, studio controls, cache
identity, and exports retain that shared type. `PoseCorrectivePolicy` separates
the loading/evaluation choice from `PoseCorrectiveAvailability`, the fact that
a loaded model has a network. Their wire representation remains numeric detail
and boolean policy. `MhrAsset` owns exact file roles; `MhrAssetDirectory` owns
native directory selection and distinguishes absence from inspection failure.
Asset errors retain role, directory or resource addresses, and original causes.
NumPy archive/array admission retains nominal values and concrete causes. MHR
evaluation failures retain distinct batch and coefficient-count roles. FBX
framing retains nominal counts, tags, and positions and returns concrete decoder
failures. Character interpretation and model loading retain concrete staged
errors, nominal rejected quantities and original causes. FBX scene/property and
animation interfaces and remaining model-definition, geometry, tensor and
mathematical interfaces are still recorded migration debt.

Use an enum with required payload when flags and optional data describe one
state. `PopulationRole::ServiceProvider` carries its profession and optional
public service identity. `BridgeRequirement::Required` carries the bridge that
must admit an outcome. Independent environmental, suppression, permission, and
rendering facts can remain independent states when all their combinations are
meaningful. Give each fact a semantic type when it crosses a handwritten
function interface; do not invent a combined state machine for unrelated facts.

Use domain errors for recoverable classification and invariant failures.
Equipment placement, containment, population generation, starting requests,
strategic state parsing, scene validation, terrain construction, weapon
construction, authoring controls, and texture transfer have their own errors.
Keep nested sampling, JSON, I/O, recipe, and construction errors as typed
sources. Include meaningful context such as a component, frame, channel, or
relation in the error rather than encoding it into prose.

The creator's `device_underlayer` owns `FitStatus` and `FitFailure`. Decode the
GPU status word with `TryFrom` immediately after readback. Known flags retain
their protocol priority and produce a classifiable failure; unknown flags are
an invalid device response, including when mixed with known failures. Preserve
the typed cause when adding application-level presentation context.

GPU readback owns `BufferByteLength`, `BufferByteOffset`, `ReadbackRange`, and
`ReadbackSlot`. Copy ranges check addition, alignment, physical storage, host
addressability, and element layout before exposing initialized host values.
An empty logical buffer is valid; a zero-sized element layout is not. Padding
needed for device copies does not become logical data. Slots retain one shared
identity through batched compute, armor staging, and creator frame consumers.
`ReadbackStatusWord` represents an encoded device response; the owning protocol
still validates its individual flags.

`BufferReadback` reserves its host destination before acquiring the device
view. Batched readback reserves every destination and the result collection
before obtaining that view, so WASM memory growth cannot detach it during the
copy. `ReadbackMapping` releases pending requests on cancellation or failure,
and its views borrow the owner that eventually unmaps them. Layout conversions
use `AnyBitPattern` and initialize all bytes before publishing typed elements.
Their representation ports and wgpu mapping callbacks are exact boundary
exceptions, rather than exemptions for GPU or mathematical helpers.

`ReadbackError` keeps allocation, polling, mapping, range, and layout failures
classifiable. Armor, metal baking, and creator consumers preserve that source.
Metal relief loading keeps I/O or image-decoder errors with source context;
decoration libraries retain the affected name or path in their error enums.
Cached device-open failures retain their typed causes through shared ownership.
Kernel source admission, compilation, and caching retain `ShaderSource`,
`ShaderEntryPoint`, and structured staged errors. Resource labels use
`ShaderBindingName` where admitted; remaining reflection and binding interfaces
still need conversion. Cache identity preserves exact source bytes. Cache
poisoning is classifiable rather than a panic. Owned and borrowed device
contexts keep distinct `ValidationReadback` policy. See the
[GPU source contract](fabelgeist-gpu/README.md) and
[kernel cache contract](fabelgeist-compute/README.md).
Other GPU allocation, dispatch, and execution providers still return generic
errors; those providers and `MetalError::Device` remain migration debt, not
permanent boundary exceptions.

Convert an error to text once at a reducer, JavaScript, CLI, or UI presentation
boundary. A mandated `Result<_, String>` at that boundary is acceptable. Do not
use display text to drive retry, authority, or gameplay decisions. Aggregated
catalog and build diagnostics can remain prose when the sole consumer displays
the complete report and no control flow depends on its wording. Introduce
structured paths or codes when tools need to select individual diagnostics.

The strategic HTTP/SATS boundary separates remote request rejection, response
decoding, malformed wire structure, generated-row admission, row cardinality,
and explicit view projection. Preserve concrete decoder and body-read causes.
Decode reducer codes only when admitting a reducer response; SQL diagnostics
must not acquire reducer semantics. Field projections retain their field role
and encoding/admission stage. SQL row and column cardinalities, product field
cardinality, tagged-sum arity, row/column addresses, and full-width wire tags
are
distinct roles. Complete SQL statements remain nominal through execution and
cardinality diagnostics, then encode directly for the HTTP provider. SQL request
counts, accumulated microseconds, and one send-to-headers latency have distinct
owners; keep native truncation, wrapping, snapshot saturation, and the shared
warning policy intact. Query key identities and literal fragments remain debt
until their own producers and consumers are migrated.

Character query selectors retain the shared `CharacterId`. Its private word and
transparent serialization preserve the complete unsigned storage range; the
identity itself does not prove existence, permission, or nonzero admission.
Convert native row, route, or session keys when constructing that identity.
Keep it through query producers and execution, and format it directly in
structured diagnostics. Export a native key with the owning `From` conversion
only where storage or protocol encoding requires it. Remaining raw-ID helpers
and other query-key families are still migration debt.

The strategic character loaders and shared-cache lookup retain that identity.
Observation chronology owns distinct mutable-access, frontier-alignment, and
life decisions. Missing clocks deny cross-character mutable access and do not
establish alignment. A known death becomes visible at its effective minute;
unavailable death chronology preserves life availability. Resident observation
retains the subject and observer identities through that failure policy and
formats concrete query errors only when logging. Native fields in presentation
rows and other primitive helpers remain debt.

Party readiness retains the observer and member identities through projected
membership, condition refresh, and condition lookup. Its error identifies the
failed query stage or missing/incapacitated member and retains the concrete
query cause. Readiness excludes corpses without removing their presentation or
history. Party-action execution and approval retain actor/leader identities,
checked queued party identity, and a closed request kind. Complete travel
payloads
stay owned through execution and approval; protocol array indexing does not
cross those operations. Action, location, profile and departure errors preserve
their stage, identity and provider cause until presentation. Other action
fields,
terrain arithmetic, coordinate tuples and neighboring primitive/error helpers
remain explicit migration debt.
Chat, vicinity and forage failures also retain their authority read stage and
concrete cause. Selected actors and decoded chat subjects retain CharacterId;
resident/presence evidence reads the same subject key in order. Forage feedback
is a closed allowlist, distinct from diagnostic error text. Format chat failures
only at HTTP output and render forage notices only after token admission.
See the [strategic query contract](strategic-web/README.md#generated-row-admission).

## Mathematical products and residuals

The compute host-float library owns WGSL `ProductExpansion` and
`ProductResidual`. A product expansion retains the high and low significands
and their shared binary exponent. Normalizing finite factors before splitting
them avoids intermediate overflow and underflow; nonzero normalized product
magnitudes lie in `[1/4, 1)`. The low part is computed using fenced partial
products, so correctness does not depend on a device's `fma` being fused.

Quotient and square-root correction construct a `ProductResidual` with an
addend that cancels the leading product. This cancellation condition is part
of its contract; it is not a general multiply-add operation. The product's
primitive constructor inputs are narrow mathematical representation ports.
Conversion preserves the nominal `DoubleFloat` through arithmetic and generated
polynomial evaluation. Final scaling or conversion can still flush subnormal
values on a device.

`DoubleFloat` represents an unevaluated sum of a leading word and its rounding
error. For finite results, its constructors normalize the words; addition,
multiplication, division, and negation retain this type. A two-component vector
cannot stand in for it. `QuarterTurnReduction` pairs the angular remainder
with its quarter-turn count. `TrigonometricComponent` selects sine or cosine
without a positional boolean argument. WGSL does not provide private struct
members; source consumers must use the owning constructors and operations.
Arithmetic accepts nominal expansions on both sides, including operands
constructed from a single scalar. The owning operations preserve the scalar
rounding order when an operand's error word is zero.

`df_from_scalar`, `df_two_sum`, and `df_quick_two_sum` construct expansions
from scalar representations. The quick constructor checks operand magnitudes
and uses the general transformation when they are in the opposite order.
`df_round` converts an expansion back to the rounded scalar representation.
The Rust generator owns closed `PolynomialSeries` definitions. Each definition
couples its WGSL symbol, truncation, and coefficient law. `PolynomialTerms`
admits `SeriesTerm` values from either end without duplicating a term;
`SeriesCoefficient` retains each coefficient through Horner source generation.
`DoubleFloatLibrary` owns assembly of the operations, constants, series, and
functions. It serializes source through `Display`.

`WgslFloatLiteral::try_from(f32)` is a binary32 serialization admission port:
it rejects nonfinite words and uses WGSL's direct-to-float suffix.
`DoubleFloatLiteral::try_from(f64)` admits a leading word and its nearest
residual together. It distinguishes nonfinite input, leading-word overflow,
and a nonzero input that would become two zero words. These representations
remain private. The closed series produce finite representable coefficients;
changing their truncation or laws requires revisiting that bound. Preserve
ascending factorial multiplication and coefficient encoding when changing the
generator. Its `Display` signatures follow the standard formatting trait.

The remaining scalar helper interfaces are migration debt. Shader fragments
require the same semantic review as other handwritten code; an opaque census
entry does not exempt their mathematical interfaces. Preserve binary rounding
order, signed zero,
cancellation, scale, and iteration bounds when extending these types through
the remaining consumers.

Preserve native arithmetic operation choices as well as iteration order.
Replacing a standard floating-point sum with a loop can change NaN payloads
despite agreement on finite inputs. Retain the original operations when a
semantic conversion does not require changing the algorithm.

GPU resource labels and parameter keys have different roles. `ShaderBindingName`
retains a resource identity and borrows its `PassParameterName` lookup spelling.
Uniform members use that same flat lookup namespace without claiming resource
identity. Keep nominal keys through producer maps, secondary-resource caches,
and dispatch; do not introduce a parallel raw map or primitive forwarding API.

`BindGroupIndex` addresses a resource group; `BindingIndex` addresses one slot
within it. Neither promises membership in a particular layout. Reflected byte
addresses and sizes reuse `BufferByteOffset` and `BufferByteLength`. Native
Naga and wgpu metadata stay within their exact representation adapters.

`UniformNumber` and `UniformUnsigned` encode the uniform ABI's scalar values.
They do not establish arbitrary gameplay units. `UniformBytes` serializes their
private representations and vector/matrix components at reflected addresses.
The general and cached packing policies retain different admission contracts;
do not silently broaden the cached subset or reject previously omitted general
values. Preserve column padding, rounding, signed zero, and nonfinite casts.
The cached serializer must not allocate per scalar or vector. Resource and
packing errors retain nominal context and concrete causes through generation.

Radix sorting belongs to `fabelgeist_compute::sort`. `SortItemCount` counts
key/payload pairs; `SortKeyWidth` owns the digit schedule and retains width
clamping. Keep tile counts, pass counts, and digit shifts distinct until native
parameter encoding. Preserve early no-op admission, sentinel capacity,
histogram/scan/scatter order, stable payloads, and odd-pass copies. Structured
failures retain resource and digit-stage context. See the [sorting
contract](fabelgeist-compute/README.md#radix-sorting) for the boundary policy.

Constraint graph addresses belong to `fabelgeist_xpbd`. Retain `ParticleIndex`
through incidence, `ConstraintIndex` through producer-order lookup, and
`ConstraintSlot` through the reordered solve schedule. Nonzero arity and
complete fixed records are admitted before coloring. Variable-arity conflict
graphs may contain empty records or repeated addresses. Preserve greedy
selection, stable within-color order, native integer narrowing, and the empty
binding sentinel. `ConstraintCount` owns graph and attachment cardinality; do
not define another per-constraint count beside it. See the [XPBD
contract](fabelgeist-xpbd/README.md) for admission and error ownership.

Particle cardinality also belongs to `fabelgeist_xpbd`. Keep active
`ParticleCount`, sentinel-bearing `ParticleCapacity`, and full host
`ParticleInputCount` distinct. Input length diagnostics retain the host count
before native GPU narrowing. Internal consumers request record extents,
invocations, parameter binding, or sorting quantities from the owning type;
do not unwrap a count and pass it to another handwritten primitive API.
Mass-length admission precedes capacity, and replacement admission precedes
mutation. Retain concrete causes and buffer roles through particle operations.

Position/mass records and velocity records retain separate owners through host
projection, native upload, and typed readback. Their private representations
preserve native float bits; velocity uploads clear the unused word. Reject
partial records at provider admission rather than treating complete scalar
words as a complete particle array. Native layout does not establish physical
mass validity or authorize scalar internal APIs.

Particle mass, inverse mass and areal density retain kilograms, inverse
kilograms and kilograms per square metre. Keep those owners through geometry
production, fabric presets, particle records, host state and CPU correction.
Native float construction does not promise physical validity; apply each owning
CPU admission policy explicitly and preserve signed-zero and massless behavior.

Signed separation gradients and affine barycentric coordinates remain distinct.
Their response and correction operations preserve the original grouping,
thresholds and units. Private native-term conversion may feed only the original
standard floating-point sum, immediately restoring the accumulated unit. Such a
port does not authorize scalar forwarding through handwritten domain functions.

Surface resolution cardinality remains separate from active particle count.
Count each successful positional correction, including another correction of
that pair in a later pass. Preserve the original traversal, accumulation and
arithmetic grouping. Host projection admits its interval length before state
readback and retains the failing read or encoding stage with concrete causes.
Format messages only when presenting them; inspect variants to classify errors.

Shell rest and simulation admission remain distinct. Empty rest geometry stays
constructible; simulation admission rejects it before other contracts. Retain
the original particle, constraint-length, reference and rest-data failure order.
Host input addresses and mass counts keep their full width in diagnostic owners.
Do not narrow malformed input merely to report it with a device particle index.
Mesh decoding and material admission retain concrete failures and causes;
standard source inspection is the only dynamic error erasure at these owners.

Self-collision and shell construction retain lower allocation, sort, cache,
constraint, particle and projection causes with their resource or stage role.
Preserve native admission order, sentinel capacity, shader selection, dispatch
sequence and numerical operations while lifting those errors. Nominal finite
state classification must remain distinct from a failed state read.

Handwritten callback traits are internal interfaces. `SubstepHook` declares its
associated error carrier; composition retains each participant's cause and the
solver adds phase and substep context. Solver and chain entry points borrow the
domain trait interface while retaining concrete, standard `Error` constrained
cause types. Already selected participants and nested chains use that same API.
Do not classify an internal callback protocol as an externally mandated
exception or reduce its causes to strings. Other unproven generic participants
and associated projections remain census work.

Terminal submission separates synchronous enqueue failure, remote reducer
rejection and SDK callback failure. Queue success cannot authorize outcome
presentation. The generated callback admits its native nested result once;
the mailbox and retry policy retain structured failures. Remote rejection text
keeps its exact protocol spelling without deciding local error classification.
Cause inspection preserves the SDK error object, and enqueue errors retain the
submitted resolution. Neither failure changes the frozen consequence receipt.

Analytic collider list cardinality and allocated device records have different
roles. Retain full host counts for rejected updates and the established native
narrowing for capacity selection and shader encoding. Count rejection precedes
upload or list mutation; capacity growth retains its original allocation order.
Body collision recording retains analytic/mesh dispatch roles and the concrete
previous-position copy failure. Ordinary disabled collision and explicit
push-out keep their distinct admission policies. See the [body collision
contract](fabelgeist-physics/README.md).

Procedural mesh choices belong to their geometry owners. Keep sphere radius,
latitude rings, longitude sectors, and depth displacement scale distinct;
a uniform scalar representation does not replace those roles. Plane cell
counts travel separately from its geometric basis. Preserve existing float
admission and arithmetic order when introducing these types: a stronger
allocation or finiteness contract would be a separate behavior change.
Displacement operations retain the GPU context that compiled their reusable
pipeline and preserve compilation causes through their operation error.

Bending weights and their rest target travel as a `BendRecord`, including
through
graph-color permutation and staged seam closure. Native serialization admits
complete records, not a parallel flattened float stream. Keep the empty-set
sentinel decision tied to record cardinality: a nonempty zero-sized native
record still has the original allocation failure. Constraint attachment errors
retain set labels, parameter identities, counts, and concrete allocation causes.

## Verification

Test the mistake the type prevents: mixed identity families, invalid decoded
values, source inspection after wrapping, replay rejection, rollover,
reconnection, deterministic generation, and unchanged boundary encoding.
Compile-fail examples can demonstrate family separation. Existing generation
and geometry regression tests are useful when a type conversion must preserve
output; avoid assertions that only repeat constants or enum definitions.

Audit handwritten code, including separate Cargo workspaces and boundary
consumers. Generated bindings and authored catalog data are not independent
owners. Review repeated-value census output as evidence, not a demand to wrap
every repeated literal immediately. Track raw interface parameters as migration
debt and add mechanical enforcement for each converted family. Run applicable
package tests, `just fmt-check`, and `just lint`.
Extract responsibilities and lower relevant quality ceilings after a split;
do not raise ceilings to accommodate a conversion.

`cargo run -p fabelgeist-rust-quality -- interfaces .` lists Rust signature
debt across the crate tree, including separate workspaces and test helpers.
It follows nested types, primitive receivers and `Self` returns, aliases,
imports, re-exports, generic alias arguments, and default arguments. Generic
error representations also appear in this inventory. Literal fixture, catalog,
and boundary scopes do not suppress interface findings.

The `interfaces` tables in `rust-quality-baseline.toml` record exact debt by
source file, owning item, signature, and occurrence count. The existing `check`
gate rejects new entries and changed counts, and requires removed findings to
be deleted from the baseline. Generate a snapshot with `baseline`, then apply
only the reductions belonging to the completed family. Keep reviewed mandated
interfaces in narrowly reasoned exceptions; do not blanket-exempt helpers or
mathematical modules.

An `exceptions` entry in `rust-quality.toml` names the exact rule, file, item,
signature fingerprint, and occurrence count, with its boundary reason. Remove
that finding from the debt baseline so the exception is its sole accounting
owner. The checker rejects stale exceptions and changed counts. Constructor
inputs and standard trait formatters still appear in the census; an exception
records why that specific interface must retain its mandated representation.

Freeze behavior evidence in ignored local output. Give each isolated Rust copy
a distinct package identity when sharing a build directory with the live
workspace or another copy. Verify that the executed test names and counts match
the intended source; a successful exit with no selected tests is not evidence.

Unresolved findings are review work, not proof of a primitive or permission to
ignore a signature. They include external type paths, generic parameters,
inferred closure interfaces, include-site uncertainty, item macros, and
handwritten macro invocations. Opaque macros and inferred closure bodies carry
stable token fingerprints: edits require another review instead of silently
retaining the old debt. A nominal declaration alone does not prove correct
semantic ownership or invariants. Review that contract alongside the mechanical
inventory. Browser and script interfaces have a separate language-aware census
and exact debt gate; see the
[script interface guide](../scripts/SEMANTIC_INTERFACES.md)
for coverage, commands, and unresolved language boundaries. Both gates run
through `just lint`.

The syntax inventory distinguishes a static `std::error::Error` capability from
`dyn Error` or opaque `impl Error` returns. A static standard error bound
excludes
scalar carriers; the compiler enforces that exclusion. It does not establish
that arbitrary other bounds exclude scalars. Unproven parameters and generic
projections remain visible.
Concrete non-generic associated bindings resolve at their owning implementation;
scalar bindings still report raw debt. Ambiguous and generic bindings remain
unresolved. Standard cause-inspection adapters remain exact boundary exceptions.

Qualified associated names such as `Self::Cell` are not standard `Cell`
containers. Keep them unresolved until their binding is known. External types
that share a container's leaf name likewise remain visible. Ridge and profile
plate fields share `PlateCell` as their contour-face identity. Its private
station and band retain each field's original station-first ordering through
grouping and refinement. Scalar sampling and other generic bindings remain
migration debt.

Review unresolved provider aliases against their locked SDK source. A narrow
adapter may receive the provider's result and immediately retain its concrete
cause in a domain error. Document only that exact provider leaf as a boundary;
generic success values and scalar consumers remain separate inventory debt.
The NetCDF adapters retain `netcdf_reader::Error` in the importer's `Netcdf`
variant with the source path. An unknown alias alone does not justify a generic
error exception.
