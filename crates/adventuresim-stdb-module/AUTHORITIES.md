# Semantic authorities and retained projections

A repeated value is retained only when its owner, purpose, lifetime, and update
path are explicit. A view, snapshot, storage adapter, or cache does not acquire
permission to mutate its source. Shared domain vocabulary belongs in the lowest
shared crate; generated SDK types belong at the transport boundary.

## Identity and enrollment

| Representation | Owner and purpose | Lifetime and update path | Invariant coverage |
|---|---|---|---|
| `Character.name` | `CharacterNameIdentity` owns the semantic identity; this field is its persisted native-culture everyday display projection. | `assign_character_name_identity` prepares rendering and serialization before either write in the reducer transaction. The shared identity validator checks all catalog references and native form/family agreement before rendering. Persistent creation, generated names, newborn names, and authored renames call this owner. Tactical temporary bot labels have no durable personal identity. | Starter rendering tests and isolated authority integration checks cover creation, rename, invalid names, surname lookup, invalid hereditary metadata, mismatched family/form references, and rollback. |
| Starter claim request key, character ID, and owner key | The request key identifies generation coordinates; the ID binds the materialized character; the owner authorizes retry. These express different relationships. | Claims are immutable. A retry regenerates the ID, checks character existence and owner, and reuses the character. Renaming never overwrites or invalidates a claim. Generation coordinates have no separately stored copies. | Isolated integration checks exercise retries after rename and ownership rejection. |
| Tactical participant IDs and launch count | `TacticalPartyRoster` validates captured enrollment authority: nonempty, unique, and within the receipt limit. | Request and server rows carry only the captured IDs for that mission. The dispatcher derives the scalar CLI launch argument. `Character.server` owns assignment; zero means unassigned. Enrollment checks active conflicts before reclaiming an absent server. Leave and teardown clear assignment. | Shared roster tests cover bounds and count; isolated integration checks exercise enrollment, leave, cleanup, conflicts, stale assignment, and strategic restrictions. |
| Operator names, shop names, settlement scale, scene and environment data | The request captures strategic facts atomically; the dispatcher owns materialization into an immutable mission scene. `ShopName::for_operator` derives branding from the captured name and building use. | Mission lifetime. Renaming or changing strategic facts does not rewrite an active scene. Later requests capture fresh facts. Scene validation checks operator, brand, business, and building agreement. | Scene establishment validation tests and dispatcher establishment-binding tests. |
| Resident presentations | `Character` owns current name and age; `CharacterPersonality` owns sex and presentation; resident profiles own local appearance and occupation metadata. | Reducer-local and gateway views join those current components. Age bands use `AgeBand::for_years`. Restricted public presentations deliberately expose less than private personality. | Age-classifier tests and isolated resident projection checks after rename and chronology changes. |
| Resident generation explanations | Immutable `PersistedGenerationExplanation` preserves historical inputs and weighted decisions. | Population generation records the explanation once. Its input owns the seed; no separate seed column exists. It does not determine current identity or demographics. | Population generation determinism and explanation tests; integration checks compare historical provenance before and after current-state changes. |

## Storage and progression

| Representation | Owner and purpose | Lifetime and update path | Invariant coverage |
|---|---|---|---|
| Primitive index keys and traversal IDs | Semantic IDs own identity. `CaseSiteAuthority.id_key`, incident index keys, and resident `projection_id` are SpacetimeDB index or bounded-view adapters. | Constructors derive index keys from semantic IDs and traversal IDs from the row ID; deletion removes the row as a unit. These fields never select a second semantic identity. | Existing materialization and ownership validation checks key equality; isolated resident checks cover traversal equality. |
| Quest manifests, indexed metadata, context commitments, and factor traces | `QuestGenerationAuthority` owns immutable replay evidence. Indexed columns permit lookup without decoding the manifest. | Created atomically with generated cases. `validate_quest_generation_authority` checks context, manifest, catalog revision, IDs, seed, settlement, and trace consistency before replay-sensitive use. | Existing quest replay, manifest consistency, and commitment tests. |
| Official minute projection and personal clocks | The wall-clock epoch anchors official time; actor activity owns personal chronology. These are different timelines. | `initialize_time` pins the epoch; `refresh_clock` calculates and refreshes the official minute. Personal initialization and activity/family advancement own character minutes and lifecycle settlement. | Existing clock, advancement, chronology, and family lifecycle tests. |
| Weather interval/cell and captured atmosphere | Core deterministic weather owns calculation from version, absolute time, and position. Snapshots pin the result for a journey, incident, or tactical handoff. | Current queries resample the owner. Route cache keys include weather/version/interval inputs; stored scene snapshots retain their captured historical purpose. | Existing deterministic weather, route-cache, and scene snapshot tests. |
| Birth minute and age years | Character chronology owns age. The bounded year projection supports gameplay and demographic display. | Creation establishes birth facts; personal-clock advancement refreshes age through chronology settlement. Missing birth authority fails eligibility checks instead of falling back to cached age. | Existing chronology and lifecycle tests; shared age classifier covers band boundaries. |
| XP, level, training hours, and derived capability | Progression owns XP and hours. Levels and capability are gameplay projections, not independent grants. | XP award updates level; equipment, training, personality, and condition changes refresh capability through the existing progression and refresh helpers. | Existing progression, equipment, training, and capability tests. |
| Personality-dependent stats and condition projections | Private continuous personality scores (with lossy trait labels) and durable needs, wounds, illness, and morale stimuli own inputs. Public strategic condition and stats serve bounded gameplay queries. | Character creation initializes projections; personality/equipment updates and `refresh_character_strategic_condition` refresh dependents. Chronological settlement refreshes time-dependent condition. | Existing personality, morale, condition, and equipment-refresh coverage. |
| Bound mission enemy roster | `MissionAuthority::capture` owns immutable combat/loot and exact enemy identity snapshots. `MissionEnemyRoster` validates nonempty, unique, transport-representable IDs; binding checks live group cardinality once. | Ordinary and standalone paths share capture. Consumption validates retained IDs and referenced characters, derives counts, and never refreshes from later group escalation. Tactical required-kill fields remain explicit launch/victory transport projections. Party receipt limits are separate. | Shared roster tests cover ordering, uniqueness, emptiness, count overflow, and larger-than-party rosters. Guarded normal/standalone binding, live planning drift, malformed/missing IDs, launch counts, and autoresolve checks. |
| Terminal case outcomes, finale phases, and execution receipts | CaseOutcome captures terminal party/status/path/time and selected finale. Finale phases describe selection eligibility; CaseFinaleExecution owns completed execution identity/source/party/time. | Resolution captures the outcome. Selection updates phases; execution commits effects, phase, and immutable receipt atomically. Exact retries check receipt first; no execution flag is stored on CaseOutcome. | Guarded absent/selected/executed finale and conflicting retry checks; separate world-event checks cover one-time fame and local-problem effects. Full generated-quest finale integration is not rerun by these fixtures. |
| Offense identity, legal settlement, and arrest charges | Offense creation owns immutable offense facts under implemented fine/arrest policy. No per-offense capital-eligibility projection exists. Legal settlement is downstream mutable state; charges capture the offenses admitted to an arrest. | World-event preflight compares authored offense facts and ignores later settlement. Settlement updates offense state; later offenses require later charges. | Guarded preflight identity mismatch, exact/conflicting event retries, fine policy, charge capture, and replay after settlement. |
| Context membership intervals | `CharacterContextMembership.entered_at` and `left_at` own the interval. Open membership is derived; no active flag is persisted. | `close_context_membership_at` closes once and advances revision. Roster teardown and patient recovery use this owner; later closure cannot rewrite history. Historical consumers use the interval at their observer minute. | Isolated context acceptance checks cover open/closed queries, interval boundaries, and repeated closure. |
| Party contact authority and mission awareness | Row existence owns completed contact and mutual awareness for the party/context pair. Encounter awareness and choices are current projections; mission contact/surprise fields are immutable enrollment snapshots. | Contact updates revision, encounter awareness, and retry receipt in one reducer transaction. Exact retries return before live checks; conflicting reuse and stale revisions fail. New mission binding captures row presence. | Guarded first contact, party scoping, removal of sneak, exact and conflicting retries, stale revisions, and subsequent contact revision checks. |
| Leisure accrual checkpoints | `conception_quantum_plan` owns the conserved joint-minute remainder and next trial ordinal. Overlap rows record processed interval pairs; trial receipts preserve deterministic crossings. | Spouse leisure settlement processes each overlap once and persists the returned checkpoint. No cumulative total is stored. | Core partition-invariance tests and guarded checkpoint, crossing, and repeated-settlement checks. |
| Witness claim resolution and social receipts | Optional typed `WitnessClaimResolution` owns claim resolution and captures realized affinity feedback. Heard text and authored responses preserve the encounter snapshot. | Resolution is written once with relationship effects. Admission derives resolution from presence. Immutable action receipts allow exact retry without effects and reject conflicting reuse. Later relationship changes do not rewrite feedback. | Core outcome/serialization tests, browser projection tests, and guarded unresolved/resolved admission, both outcomes, replay, and collision checks. |
| Automatic social preferences | Preference row presence owns actor/target opt-in. Reducer/form bools request a change; browser checked state derives presence. | Enable upserts one pair; disable deletes it. Party/death cleanup deletes invalid pairs; execution checks live co-location and eligibility and bounds attempts per downtime. | Guarded enable/re-enable, disable, single attempt, same-party/self rejection, and party/death cleanup checks. |
| Puzzle and road challenge completion | Puzzle solve timestamp and selected road choice own their respective resolution. Gateway openness/solved fields are derived transport projections. | Wrong puzzle answers advance revision without completing; successful answers and road choices commit immutable receipts with resolution. Camp binding changes locators. Delayed combat appends results without reopening the choice. Case and observer eligibility remain separate. | Guarded wrong/correct answers, exact/conflicting retries, closure, retained camp binding, and delayed combat follow-up checks. |
| Treatment request receipts | Receipt existence records a committed treatment request; immutable request parameters own replay identity. | Inserted after treatment effects in the reducer transaction. Interrupted waits return before receipt insertion, preserving the existing retry lifecycle. No constant completion flag is stored. | Guarded completed/interrupted treatment, exact replay, unchanged clocks/supplies, and conflicting parameters. |
| Ingredient attempt liveness and material receipts | Preparation receipts capture immutable terminal attempts. The private attempt cursor publishes the next generation after interruption without exposing receipt details. | Complete preparation changes material revision; clipped attempts preserve material, mark cursor incomplete, and advance generation. Exact replay reads the receipt before live authority. | Core planner/material tests and guarded cut/grind conservation, digest vectors, replay, collision, and clipped-generation checks. |
| Tincture readiness and materialization | Shared world time owns eligibility; `TinctureProcess.matured` records installation of the pinned medicinal component. | `materialize_mature_tincture` checks readiness and commits the component with the checkpoint once. Dosing materializes before requiring the component. Readiness and completed installation describe different stages. | Existing herbalism source-contract checks describe the lifecycle. This audit verified the owner and update path; no isolated maturation probe was run in this pass. |
| Corpse names, injuries, and pathology | Corpse creation captures identity and committed injury/pathology evidence at death or discovery. | Later names, injuries, and live condition changes do not rewrite historical evidence. Handling owners change custody, examination, burial, and cremation state. | Existing autopsy observation tests; seven native autopsy tests passed in this implementation. |
| Foraging seed, input digest, cursor, and receipts | Foraging owns the deterministic resolution seed and immutable request evidence. Digests commit inputs; generation cursors govern attempt liveness. | `resolution_seed` uses its named hash domain; the input digest commits the seed with planner inputs. Immutable receipts make old retries inert while cursors admit later attempts. | Existing core planner and database source-contract checks cover digest, replay, interruption, and private projection contracts. This audit did not rerun foraging-specific checks. |
| Evaluated weapon and holder recipes | Recipe evaluation owns immutable design hash, version, mass, and dimensions. Weapon and holder evaluation are separate geometry contracts. | Constructors derive every scalar from the encoded evaluation. Consumption decodes, checks the catalog identity, re-evaluates, and compares the whole row; later catalog defaults cannot replace the instance. | Existing evaluation tests corrupt every persisted projection and recipe; connected appearances fail closed. These native database tests are not executable under the crate's disabled unit-test configuration. |
| Fireplace dishes, food lots, and contamination provenance | Cooking owns a pinned ingredient and process snapshot; the retrieved lot owns its remaining food. Hidden contamination remains separate from inspectable provenance. | Starting cooking consumes ingredients into the dish. Retrieval derives elapsed-time effects, transfers captured provenance into a lot, and deletes the dish. Consumption and preparation conserve remaining material. Later names or recipes do not rewrite captured inputs. | Existing food timing, contamination, conservation, and fireplace custody coverage; guarded preparation checks. |
| Organization role, dues state, and public presentation | `CharacterOrganizationRole` owns the current role; membership owns dues and accrual. Public presentation is the character's selected affiliation. | Role mutations update the canonical assignment; gateway membership views join it. Dues payment/settlement refresh status, and current eligibility also checks the paid-through deadline. Present/clear and lapse reconciliation own the public selection. | Existing organization privilege, deadline, role-assignment, and presentation tests. |
| Whole row count and remaining material | Inventory owns stack count; amount/food-lot owners track remaining material in individual rows whose quantity stays one. | Constructors enforce individual rows. Consumption reduces the amount or lot, removing exhausted rows. Holder transfer moves remaining state in the same transaction. | Existing measured-row and food conservation/source-contract coverage; guarded preparation conservation checks. Fresh tracing added no new amount-transfer probe. |
| Party stakes, reserve, and pooled currency | Inventory-trade transactions own contribution entitlements and communal unallocated value. Physical coin/item rows own current holdings. | Deposits, withdrawals, purchases, rewards, and disband settlement update their corresponding ledgers. Disband validates final pooled currency against reserve after member stakes are settled. | Existing inventory-trade source contracts; fresh tracing added no new ledger acceptance probe. |
| Food batch provenance and material totals | Cooking/harvesting owns the captured ingredients, flavour mass, nutrition, value, and preparation result; these are material facts rather than a live recipe view. | Cooking/harvesting initializes a batch revision. Consumption conserves remaining quantities and totals; preparation advances material revision. Catalog changes cannot reconstruct past cooking or partially consumed batches. | Existing conservation and preparation checks, including isolated preparation acceptance. No new cooking conservation probe was run in this pass. |
| Reputation events and local aggregates | Authoritative action events own spreading input; local fame/infamy rows are terminal projections. | `record_event` checks the retry-stable event identity and updates the event and affected aggregates transactionally. Aggregate rows never spread again. Character deletion removes events and aggregates. | Existing graph/contribution and event-idempotency tests; guarded world-event checks in the isolated suite. |
| World-event and operational-custody storage adapters | Shared typed core events and custody own semantics; persisted unions encode primitive schema boundary values. | Exhaustive encoding/decoding reconstructs semantic IDs. World-event admission validates the envelope and request/consequence binding before effects and receipts; custody admission validates location/fixture and actor ownership. | Existing adapter and semantic binding tests; guarded event replay and custody/preparation checks. |
| Residence current status, transition history, and billing | Residence lifecycle owns current legal status and historical transitions. Primary designation and physical occupancy describe independent rights/use. | Acquisition, recovery, dormancy, and relinquishment update status and transitions together. Historical eligibility folds transition history; billing advances processed-charge deadlines. Gateway summary booleans derive current/observer-relative owners. | Existing residence lifecycle/source-contract coverage; no new isolated residence probe in this pass. |
| Objective continuity completion and social testimony release | Continuity ingestion owns a processed-fact checkpoint; witness social action owns session-local emission and assessment completion. | Deadline passage alone does not commit the objective fact. Learned testimony can survive a session, while its emission event and passive assessment commit once with that session's checkpoint. | Existing continuity/source-contract and social acceptance checks. Fresh audit traced these owners without adding new probes. |
| Outbreak health cleanup and historical presence | Infection/life/remediation chronology owns observer-relative suppression. `health_active` checkpoints pending recovery/death cleanup, separate from context release. | Remediation closes context without clearing health cleanup. Recovery/death closes the interval and settles health/presence effects once. Historical views reconstruct chronology instead of latest flags. | Shared suppression chronology tests and guarded single-patient materialization, context release while ill, recovery/death cleanup, repeated closure, and historical suppression checks. The fixture exercises the patient owner rather than full generated-quest binding. |
| Indexed outcome fact identities | Typed OutcomeFact owns semantic content; scalar identity columns adapt it into primary, case/party traversal, and unique-source indexes. | `ingest_case_outcome_fact` derives and inserts keys with the immutable payload. Retry validates encoded fact and source binding; evaluation decodes the payload. | Guarded payload timestamp and index-attribution checks, window deadline evaluation, and exact/conflicting source retries. Time exists only in the typed payload; no nonindexed timestamp copy is persisted. |
| Historical conception eligibility | CharacterBirth and CharacterDeath own age and life status at the trial minute. Current age/alive fields, pregnancy birth receipts, and corpse snapshots do not select a second chronology. | Conception queries the same `effective_age_years` and `character_alive_at` owners used by relationship lifecycle. | Isolated conception checks cover birth, adulthood, death, and stale current projections without pregnancy/corpse records. |
| Party, household, and organization membership projections | Membership rows own participation. Party leader/solo state and character party pointers support membership queries. | Membership mutation and party refresh update pointers and aggregates together; household and organization lifecycle owners settle their own participation. | Existing membership, governance, residence, and lifecycle checks. |
| Flat lifecycle columns and parsed states | Shared `strategic_state::vocabulary` and `case::ContractStatus` own discriminants. Parsed states validate combinations of status and optional fields. | Flat columns exist for storage/queryability; parsed sum types exist only while consuming a row. No parallel handwritten enum definitions or one-to-one domain copies remain. | Shared lifecycle parser tests cover accepted and contradictory rows, and enum serialization tests cover boundary vocabulary. |

## Shared vocabulary

Storefront, simple attribute, chivalric virtue, contextual role, authored
answers, and interaction presentation vocabularies each have one owning
definition in core. Authoring schemas and persistence import those definitions;
build-time catalog validation compiles the same framework-neutral vocabulary
files. Generated SDK enums remain transport bindings regenerated from the
schema.

`adventuresim_puzzles::Sigil` and `WitnessPath` own fixed puzzle submission
identifiers and typed parsing. Web options submit those identifiers and display
labels separately. Captured puzzle projections still own their offered choices;
submissions remain validated against the puzzle authority. The CLI explicitly
accepts case-insensitive identifiers and an optional path suffix. Serialization,
rendered form conversion, malformed-input, and CLI tests cover these boundaries.

`social::SocialActionKind` owns reducer keys, topic eligibility, and executing
`Skill` selection, including concern-dependent commiseration. Skill labels
derive from `Skill`. Persisted interaction receipts and cooldown keys are
historical captures maintained by the social action transaction. Shared
vocabulary tests and guarded execution, receipt, and cooldown checks cover all
request keys; automatic care retains its separate preference and downtime
lifecycle owner.

`social::ClaimChallengeApproach` owns witness request identifiers and parsing.
Its social-action mapping derives the executing skill from `SocialActionKind`.
HTTP and reducer admission share this vocabulary; HTTP preserves its bad-request
status and reducers format typed errors at the boundary. Authored response
availability, witness leverage, truthful-claim behavior, and morale attribution
retain their witness policy owners. Receipt keys capture the admitted identifier
for exact replay before mutable revision or presence checks. Shared parser
tests, HTTP dispatch tests, and guarded execution and retry checks cover all
approaches.

The embedded quest catalog owns complete threat and report membership.
`bestiary` builds transient ID views from catalog monsters; named `ThreatId`
constants refer to particular entries. `Catalog::descriptions` owns complete
report enumeration in its authored order. Bestiary quality validation,
generation, and editor options consume this population; a new unsupported-report
fixture proves additions reach ambiguity validation. Runtime item-reference
validation enumerates that same catalog for loot references, alongside mandatory
gameplay, currency, and treatment references. Catalog-wide profile checks and an
added-threat fixture cover automatic inclusion without a second roster edit.

`investigation_action::InvestigationActionKind` and `Terrain` own method and
action-environment keys and typed parsing. Capability strings adapt those types
for storage and observer transport; issuance writes canonical keys. Generated
capabilities pin immutable manifest bindings, while prerequisite admission
checks current ownership and completion. Shared serialization and guarded
storage, generated binding, prerequisite, unknown-key, and rollback checks cover
these contracts. Action environment remains distinct from terrain-pack surfaces.

`personality::Presentation` and `Inclination` own starter and live personality
vocabulary. Starter snapshots serialize canonical snake_case variants and derive
names from identity. Resident public rows reuse Presentation while retaining an
explicit restricted demographic shape. Generated SDK variants are adapted at the
browser boundary. Starter round-trip and candidate conversion tests cover all
combinations; guarded resident checks cover the live presentation join.

`disease::DiseaseId` owns canonical snake_case disease keys. Typed parsing uses
the enum's existing serde vocabulary; persistence, medical presentation, and
public treatment policy consume that owner directly. Unknown keys fail closed.
Its uppercase `stable_variant_id` remains a distinct deterministic coordinate
representation. Shared all-disease serialization/parsing tests, public
unknown-key behavior, and guarded stored-episode/rollback checks cover the
contract. Public differentials still expose only authorized knowledge.
Settlement exposure selection preserves community-source chronological ordering,
bounded problem-source identity ordering, mitigation, and end-time clipping. The
temporary typed source list owns no persisted facts. The simulator illness
fixture remains an owned disposable reducer using ordinary lifecycle hooks.

`ingredient_preparation::IngredientPreparationAction` owns Cut/Grind operation
vocabulary, public identifiers, and stable hash bytes for food and medicinal
planning. Material preparation states describe the resulting material. Request,
attempt, and authority digests retain their distinct versioned hash domains.

Combat scheduling and replay records use the same `BattleAttackKind` and
`MeleeMovementAction`. Movement records capture the requested action alongside
separate realized displacement and velocity. Replay records are immutable;
existing movement and combat behavior tests and stable serialization tests cover
this retention.

`projectile::ProjectileKind` owns physical projectile vocabulary. Combat logs
capture each hit; `commit_hit_injury` preserves the kind in retained projectile
records when damage is positive. Damage and extraction difficulty remain injury
facts. Serialization and guarded checks cover both kinds and zero damage.

`item_classification::CatalogItemKind` owns the materialized classification used
by persistence and native presentation. Authored `item_catalog::ItemKind` also
carries weapon, armor, shield, and container parameters; catalog projection owns
the lowering into item stats and classification. Capabilities and equipment
topology remain independent. Economic `CatalogKind` deliberately groups
containers with general goods through `economy_kind`. Generated SDK adapters
preserve transport boundaries. All-kind serialization, browser conversion, and
guarded catalog projection checks cover these contracts.

`LoadoutSlot` is the shared restricted vocabulary for initial player and enemy
loadouts. Full `EquipmentLocation` describes additional pockets, belts, and
attachment topology; receipt body parts describe combat outcome validation.
These scopes remain distinct. Native loadout fixtures and starter snapshots use
one serialization contract. Authored interaction answers and observer-specific
request/emergency affordances also remain distinct policy stages.

## Transport and caches

| Representation | Owner and purpose | Lifetime and update path | Invariant coverage |
|---|---|---|---|
| Generated SDK bindings, browser models, connected tactical views | Database schema owns wire shapes; consumers adapt to domain types at trust boundaries. Public demographic projections intentionally restrict information. | Regenerate bindings with `just generate-db-client`. Subscription updates and boundary conversions refresh views. Transient tactical positions, HP, enemies, and damage remain in Bevy. | `verify-db-client`, gateway authorization tests, transport conversion tests, and existing tactical projection tests. |
| Subscription cache and query metrics | SpacetimeDB subscriptions own cached rows; `LiveState` owns readiness and invalidation, and query metrics own observations. | Cache is usable only after subscription application; connect/failure clears readiness. Row events advance revisions and invalidate rendered data. Metrics are observations, not row authority. | Existing cache readiness, completeness, invalidation, and query-budget tests. |
| Tactical surface transport vocabulary | Terrain-pack cells own routing surface data; the dispatcher owns exhaustive conversion into the independent tactical scene schema. | Captured environment samples are immutable scene data. Tactical core/clients consume snapshots without the terrain package runtime and never update its routing cells. This handoff is distinct from shared domain identifier parsers. | Existing scene/grid validation and snapshot determinism coverage; the adapter is exhaustive. No new terrain acceptance probe in this pass. |
| Terrain route and chunk caches | Immutable `TerrainPack` owns terrain; `TerrainPlanner` owns route computation. | Chunk cache uses bounded bytes and LRU clocks. Route keys include coordinates, skill profile, weather rules/interval/moisture/snow, and snow skill. Bounded eviction or dropping the package/planner removes cached results. | Existing package validation, route-key, eviction, and weather-sensitive route tests. |
| Compiled bestiary profiles | Immutable embedded quest catalog owns threat profiles; `bestiary::profile` owns the process-lifetime compiled lookup. | `OnceLock` builds all profiles from catalog monsters once. A new process/catalog build replaces the cache; consumers receive immutable values. | Catalog-wide identity, profile, and loot consistency tests; an added authored threat exercises profile compilation and loot-reference validation. |
| Rendering and asset caches | Recipe, material, appearance, and asset owners determine generated geometry and images. | Shop-sign paint keys include shop text, font, finish, emblem, and texture height; weapon mesh/icon keys include appearance and design inputs. Scene-scoped building/vegetation caches and studio caches are rebuilt or dropped when their owning scene, recipe, or appearance changes. | Existing sign cache, weapon appearance, scene completeness, and studio invalidation coverage. |

## Distinct concepts and remaining audit scope

Courtship intent and completed outcome describe different stages. Equipment
locations and receipt body parts serve different contracts. Independent named
random streams isolate deterministic decisions; equal seed values do not make
them duplicate authorities. NPC public presentation remains a deliberate trust
projection. None of these should be unified merely because variants resemble
each other.

Issue #726 retains unrelated domain-type and generic-error opportunities:
primitive party/character references outside the naming and enrollment boundary,
inventory services and broad `Result<_, String>` services outside the helpers
changed here. These require their own authority and error-contract review. This
guide records the scope locally; it does not publish external audit comments or
imply those remaining areas were converted.

Run the isolated persistence acceptance checks with `python3
scripts/check_semantic_authorities.py`. The runner derives a disposable profile,
verifies local process ownership, and uses the guarded reset workflow. It never
accepts an arbitrary database target. Validation logs belong in ignored
`target/` output.
