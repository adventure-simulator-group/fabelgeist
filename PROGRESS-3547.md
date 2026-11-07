# Bespoke type refactor: progress

Updated: 2026-11-07T20:59:25.089127+00:00

Latest verified checkpoint: **3547**. Goal: **active**.

## Quantitative summary

| Category | Done or recorded | Remaining or status |
| --- | --- | --- |
| Original inventory roles, lenses 11â€“13 | 8,559 / 8,559 accounted | 0 original roles |
| Registered revision-delta lens/path checks | 3,172 / 3,172 closed | 0 in these registered deltas |
| Queue entries explicitly marked source-discovery closed | 578 / 3,860 marked | 3,282 other entries |
| Registered source ranges qualified, connected work open | 297 entries | Connected contracts remain open |
| Historical unread entries with recorded range evidence | 76 / 189 entries | 113 without these markers |
| Entries with explicit repair status | 441 recorded | Implementation gate not yet passed |
| Scoped repair obligations on entries | 75 recorded | 3 conditional; 10 policy/design |

These categories overlap and have different meanings. Original roles and
registered delta checks are not findings or completed repairs. Queue entries
include reminders and independent obligations; the other entries are not a count
of unique unfinished bugs. The source-closed count includes only explicit flags;
qualified-open entries use a separate status and are not completed contracts.
Historical range markers indicate evidence on an entry, not closure; other
historical entries may already have evidence in different forms. Scoped repair
obligations are counted separately and may overlap other entries. The full
audit, implementation, review and validation remain incomplete.

The refactor remains in the required source audit. Its 13 perspectives include
reuse of existing types and preservation of semantic values through native
boundaries. Completing a scoped question or correcting an old status note does
not complete the audit or authorize implementation. Checkpoint 3130 records an
evidence-integrity correction for recent work; adjacent checkpoints still
require review. Seven P02 note versions were recovered by exact hash; the
checkpoint 3122 version remains unresolved. A bounded scan checked 489 nearby
decision evidence references; seven mismatches all concern the same note;
checkpoint 3123 was subsequently recovered. The reducer adapter scope has been
reread after its registration; related contract work remains open. Web and
simulation error consumers have been reread after registration; runtime delivery
remains unverified. Strategic reducer error propagation has been reread after
registration; runtime delivery remains unverified. P02 mobility and timeline
scopes were reread after registration; the connected contract remains open. P02
scheduling was reread after registration; initialization failure behavior
remains open. The initialization caller owns its vectors and returns no outcome
on failure; other connected obligations remain open. The battle clock and joint
window scheduler were reread; contact batching policy remains an open question.
Existing test source does not explicitly settle near-equal contact batching.
Contact batch membership affects a defeated-attacker condition as well as
recorded telemetry; intended near-equal policy remains open. Authored initiative
delay varies; a supported near-equal contact witness and policy remain open.
Normal joint scheduling fixes new starts at its window endpoint; surprise-
opening timing remains to be traced. Surprise opening can retain out-of-window
timing in principle; a supported near-equal witness remains unproven. Retained
contact coordinates feed joint windows; supported near-equal reachability and
cancellation remain open. Same-weapon defensive cancellation clears retained
timing; supported near-equal reachability remains open. Scene-assets has a
browser generation consumer; independent tactical-server delivery remains open.
Dispatcher request minutes feed a separately materialized tactical-server scene;
their producer remains open. Tactical requests use a wilderness clock;
standalone bootstrap has a separate world-start fallback. Standalone bootstrap
initializes its local clock before the fallback; other clock producers remain
open. Wilderness elapsed updates saturate and project within the anchor day;
supported boundary reachability remains open. Inspected journey transitions
retain the clock outside settlements and clear it on settlement arrival.
Official clock conversion clamps negative elapsed and uses saturating
arithmetic; native extremes remain open. The native timestamp accessor returns
signed microseconds directly; runtime range and clock policy remain open. Scene
and displayed clocks handle missing or unrepresentable time differently;
supported failure policy remains open. Cloud seeds remain typed through
sampling; other generators and coordinate admission remain open. Cloud ray and
layer inputs are constrained; endpoint and wind admission remain open. Cloud
endpoints advance per completed bake; wind admission and supported long-session
behavior remain open. Validated scene wind speed is bounded; complete
environment-producer coverage remains open. Tactical startup validates scene
weather before snapshot; browser and alternate environment paths remain open.
Strategic retained scene parsing validates weather; pre-preparation and
alternate environment paths remain open. Standard browser pre-preparation
validates scene input before venue discovery; alternate entry paths remain open.
Tactical scene environments are inserted on replicated entities; client
admission and alternate producers remain open. Inspected viewer and demo
environment paths have bounded fixture wind or validate their input; other
producers remain open. Generated weather caps wind speed and normalizes
direction; other producers remain open. Strategic weather uses typed seeds and
fixed-point lattice coordinates; supported extreme arithmetic remains open. The
official clock is below the weather lattice cap; other clock producers remain
open. Journey elapsed time wraps within its stored anchor day; anchor provenance
and other weather callers remain open. Four more weather callers have distinct
time and coordinate paths; their upstream bounds remain open. Route planning and
reducer validation use distinct clock sources; divergence and intended behavior
remain open. A selected departure can differ from personal time and trigger a
stale route rejection; end-to-end admission remains to be confirmed. The planned
case-site payload carries personal-clock weather to the reducer, which can
reject it against the wilderness departure interval. Member approval reaches the
same planned route clock boundary; coordinate-frame questions remain separate.
Member approval reaches the same planned route clock boundary; coordinate-frame
questions remain separate. Forecast starting times come from independent
personal clocks; supported extreme bounds remain open. Newborn and encounter
character clocks derive from parent and encounter times; those producer bounds
remain open. Inspected road-cast clocks initialize from official time; newborn
chronology and mortality bounds remain open. Conception follows personal leisure
time and age display saturates; independent mortality bounds remain open. A
concrete representable weather input underflows the signed lattice; supported
gameplay reachability remains open. The lattice witness matches a byte-identical
frozen Lens 13 operation; independent qualification remains open. Lens 13
independently reread the matching current weather lattice; native admission
remains open. Lens 13 native weather paths use distinct official, personal,
wilderness and dispatcher clocks; admission remains open. Tactical request
weather time is bounded for officially initialized wilderness anchors; stored
provenance and personal clocks remain open. Current production anchor writes
initialize from official time or clear the anchor; personal-clock admission
remains open. Road-cast overrides use official time and one social action has an
eight-hour limit; cumulative personal time remains open. Personal and official
clocks have separate documented authority; extreme supported time policy remains
open. Full typed-input lattice indices fit i64 after wider intermediate
arithmetic; late-time rules behavior remains open. Weather rules version affects
route cache identity and route and tactical snapshot admission. Raw minimum
latitude can overflow cell reconstruction; supported coordinate admission
remains open. Inspected native abstract E7 conversions cannot reach the raw
i32-min latitude witness; other consumers remain open. Forecast endpoints are
geographic, but interpolation depends on itinerary movement invariants.
Generated itinerary segments constrain movement offsets; extreme-time and cast
conditions remain open. Itinerary forecasting can fail to advance at
StrategicMinute MAX; supported native reachability remains open. On-site
readiness passes personal time directly to itinerary forecasting; supported
clock exhaustion remains open. Correction: the full-range lattice bound includes
one-cell pressure offsets and still fits i128. Returned generated itineraries
accepted by the thermal horizon keep movement offsets and geographic
interpolation bounded. Thermal forecasts consume survival state and thresholds
while frostbite event consequences remain to be assessed. Frostbite events
require colder strain than the thermal unsafe threshold; safe forecasts cannot
contain an ignored event. Inspected readiness decisions combine thermal and
other safety checks and treat missing return projections as unsafe. Thermal
forecasts and wilderness exposure use different weather clocks; reachable
readiness impact remains open. Forecast weather clock differences feed seasonal
and lattice inputs; actual readiness impact remains open. A source-level
overnight itinerary yields different forecast and wilderness dates; admission
and readiness impact remain open. Camp rest samples wrapped wilderness weather
before advancing party elapsed; complete route impact remains open. Generated
site distances can exceed an eight-hour forecast leg; a concrete admitted route
remains open. Safe-departure selection configures journey-local time without
advancing personal time in the inspected method. Planned party movement lacks
the exposure call present in ordinary travel; outer admission remains to be
confirmed. Three party travel entry paths use disease-plan movement without the
ordinary exposure call; end-to-end validation remains pending. Planned movement
exposure repair must preserve shared disease-plan clipping and lifecycle
ordering. Movement exposure repair must preserve route and wilderness-time
cursors across chunks and settlement departure. Lens 12 source qualification
connects planned movement weather exposure to an existing open question; aliases
remain open. Lens 12 source qualification connects planned movement weather
exposure to an existing open question; aliases remain open. Three journey entry
paths use the planned party movement seam after encounter clipping; related
aliases remain open. Direct rest and stationary exposure calls cover their own
intervals; movement terminal ordering remains open. Weather exposure persists
thermal state and frostbite limb damage; movement terminal policy remains open.
Core exposure is minute-stepped; case-site recovery cause presentation has an
unverified thermal omission. A representable thermal-only incapacitation would
yield generic case-site recovery text if the shown notice path is reached; live
reachability remains open. The case-site portrait wheel source includes
temperature, narrowing the separate recovery wording question; live rendering
remains unverified. The movement exposure partial plan was scoped without
discarding independent battle-time and forecast obligations. Signed elevation
admission matches its historical owner at the current pin; related aliases
remain open. Signed elevation admission matches its historical owner at the
current pin; related aliases remain open. Compiled-default diagnostic source is
reconciled; dynamic configuration error policy remains separate. Compiled-
default diagnostic source is reconciled; dynamic configuration error policy
remains separate. Dynamic configuration loader and receiver paths are partially
traced; snapshot producer and outcome policy remain open. Server join and
reconnect publish the loaded configuration snapshot; delivery and rejection
consequences remain open. Checkpoint 3219 incorrectly closed the dynamic
configuration entry; checkpoint 3220 restores it to open and retains the prior
record. Checkpoint 3219 incorrectly closed the dynamic configuration entry;
checkpoint 3220 restored it to open and retained the prior record. Client
animation and server defense consume their respective config states; rejected-
snapshot policy remains open. A validated config digest has a distinct
serialization error; supported failure reachability remains open. The JSON
serializer maps nonfinite scalar floats to null; complete config admission and
serialization semantics remain open. A public AI facing threshold lacks a top-
level finite check; file admission and intended range remain open. YAML float
parsing admits nonfinite AI facing thresholds in the source-level path;
threshold policy and runtime effects remain open. UnitInterval is a possible AI
threshold type; finite out-of-range policy remains open. AI precision admission
is already owned separately and intentionally accepts every finite value. The AI
facing-threshold gap now has its own open entry; its numeric-lens qualification
remains separate. The AI facing-threshold gap now has its own open entry; its
numeric-lens qualification remains separate. AI facing admission now has
separate Lens 12 and Lens 13 source evidence; finite policy remains open. AI
facing admission has independent type, string and numeric lens evidence; finite
policy remains open. Ratio is unchecked; UnitInterval adds bounded admission, so
the threshold policy must decide the appropriate owner. Two historical range
markers now reference existing qualified reads; connected work remains open.
Native audio existing-type ownership is partially qualified under Lens 2;
project producers and other lenses remain open. Audio gain and pitch admission
is traced into selected native playback settings; other consumers remain open.
Native audio forwards composed playback speed; the existing pitch-product repair
remains independently owned. Checked UnitInterval is a reuse candidate for
bounded audio gains; native volume and pitch retain separate semantics. Combat
one-shots and rain-loop gain updates have been traced; remaining movement and
lifecycle scope is open. Movement and wind playback and the registered native
Volume tail have been read; alias and independent lens work remain open. Native
sink lifecycle is traced; the relationship between global volume and subsequent
ambience updates remains open. Native and browser startup validate audio config;
Bevy Pitch is a sine-wave source and global volume defaults to unity. Audio
decision templates contained unrelated inherited labels; checkpoint 3241
corrects the current record while preserving history. Physical areal density
does not own audio playback speed; checked pitch composition remains separately
tracked. Audio config errors, visual gating, pitch composition and scene
selection have distinct existing owners. Pinned rodio stores playback speed
without checking and applies it to a speed wrapper; remaining registered scopes
are open. The registered rodio player and spatial-player source ranges have been
read under Lens 2; independent contracts remain open. Procedural anticipation
source matches the prior assessed bytes at the current pin; exact aliases remain
open. UnitInterval lacks a fixed-step method; the anticipation repair must
preserve exact step arithmetic. The registered current procedural animation file
is read under Lens 2; its helper test does not cover the full anticipation
system. Jump anticipation is ordered between body response and later pose
refinements and fixed-tick stabilization. Jump charging admission is restricted
to grounded upright state without a posture transition. Strategic catalog
projection retains the ranged flag but has no projectile-kind field. Selected
bow and firearm catalog records lack explicit projectile or ammunition
compatibility. Tactical ranged attacks select arrows and record aggregate party
ammunition use; receipt representation remains open. Ranged contact loses
projectile identity before shared body-part injury aggregation; receipt-only
repair is insufficient. Tactical item projection carries the ranged flag and ID
but no projectile or ammunition compatibility. Tactical receipt validation
bounds records and checks party authority before strategic injury commitment.
The missing projectile-kind receipt question overlaps one existing Lens 12
entry; its wear and repair obligations remain distinct. Tactical injury
accumulation is limited per hit by current limb health; aggregate receipt
reachability remains unproven. Tactical projection copies snapshot limb health
without a local clamp; enemy scaling alone does not establish party receipt
reachability. Ordinary strategic limb creation starts at one and injury
projection clamps; other writers and finite admission remain open. Terminal
receipts are frozen before submission; projectile multiplicity needs an explicit
bounded policy. Strategic ranged combat counts and consumes arrows independently
of weapon kind; containment reachability remains conditional. Tactical receipt
refreshes capability before equipment wear; later lazy refresh and visible
effects remain to be traced. Web refresh errors are ignored before a stored
capability read; server readiness also reads stored combat power. Combat power
uses wear-sensitive precision and resistance; the located readiness caller
refreshes before reading. The registered settlement capability-consumer file is
read; unrelated displays use separate data paths. Condition loss can affect
effective equipment stats; a measurable post-receipt stale value remains
unproven. Live subscriptions invalidate on item-condition and capability
changes; render and refresh timing remain open. Melee contact force feeds wear;
a supported above-threshold contact and visible stale value remain unproven. An
authored longsword threshold and synthetic contact example are recorded; an end-
to-end contact remains unproven. Tactical contact aggregation and per-exchange
autoresolve can differ under the nonlinear wear formula; intended policy and
reachability remain open. Receipt participant truncation can conflict with
retained contact owners; supported overflow reachability remains open. Captured
tactical rosters share the 16-member receipt bound; temporary and tactical-side
provenance remain open. Party-side projection feeds routine and combat
consequences; strategic side provenance remains open. A temporary enemy creator
assigns characters without adding them to the captured enemy roster; invocation
and receipt reachability remain open. A party-classified temporary row needs a
matching loading player to spawn; actual invocation and receipt reachability
remain open. Before sealing, joins can create a matching loading record; row
delivery ordering and supported invocation remain open. The tactical client
sends its configured ID; a search found no non-generated temporary-creation
caller, while external invocation and delivery remain open. The SDK does not
replay an identical cached row as a new insert; actual view changes and timing
remain open. Receipt evidence overlaps three existing enrollment and projection
entries; their independent scopes remain open. A ranged miss flows through
sender-derived authority and consumes an available arrow after authorization;
terminal and impact traces remain open. Ranged windup gates authorization;
connected outcome presentation follows acceptance, while impact and strategic
checks remain open. Impact feedback transforms target-local contact data and
applies velocity to a separate recipient; timing and native boundaries remain
open. Impact surfaces use configured body-part boxes; config admission, native
timing and mission completion dependencies remain open. Normal server startup
validates combat hitboxes before insertion; mutation and native timing remain
open. No current mission candidate can yield strategic failure despite a
reported tactical success; reachability and presentation policy remain open.
Candidate validity depends on live capability, objective and custody state;
invalidation during an active mission remains unproven. Negotiation guards a
bound mission for the acting party; cross-party invalidation remains unproven.
Shared pre-combat resolution has no separate active-server check; second-party
admission remains unproven. Site arrival and generated approaches are evaluated
per party; a complete two-party admission sequence remains unproven. Exact site
disclosures are observer-scoped; a single accepted contract does not prove
second-party acquisition. Rumor receipts and generated graphs are observer-
scoped; repeated same-case admission and timing remain open. Rumor receipt
creation is observer-scoped and does not consume the problem; selection and
cross-party timing remain open. Admitted dialogue receives delivered observer-
owned rumors; generated site and cross-party timing remain open. Inspected
hostile finale sites are initially unknown; additional investigation is required
for the cross-party scenario. Generated routes can persist an exact finale lead
for the acting observer; success and cross-party timing remain unproven.
Investigation requires observer-owned live capabilities and completed activity
intervals; cross-party timing remains unproven. Tracking validates predecessor
position instead of requiring the undisclosed finale; success and cross-party
timing remain open. Generated action resolution reaches a guaranteed threshold
after five uninterrupted failures; admission and cross-party timing remain open.
Inspected readiness and encounter gates are acting-party scoped; other cross-
party admission and timing remain open. Arrival directly commits matching escort
objectives; continuity effects and cross-party timing remain open. Arrival
continuity guards cover survive and protect objectives, not the inspected defeat
and drive-off objectives. Withdrawal can pass its score threshold in principle;
jointly admitted inputs and timing remain unproven. Authored bandits are
negotiable with morale 50; admitted social and language inputs remain unproven.
Tactical assignment alone does not close hostile context membership; other
lifecycle and timing conditions remain open. Field counterparties receive
initialized vernacular exposure; actual pairwise language overlap remains
unproven. Life simulation carries native oral exposure forward; actual player
overlap and positive caps remain unproven. Default and generated players have
positive initial instinct and initialized oral skills; later state and complete
negotiation remain open. Settlement and journey paths train retained oral
skills; negotiation reads stored instinct rather than the inspected disease-
adjusted copy. Language training preserves existing finite nonnegative hours
above a lowered aptitude cap. Current immediate activities pass a normalized day
into ambient exposure; aptitude caps may limit the resulting gain. Other
training callers use elapsed-derived durations; convalescence and camp-priority
ambient exposure need policy. Lens 11 and Lens 13 identify the same immediate
ambient interval defect; independent lens work remains open. The Lens 13
immediate activity interval mismatch was independently reread on current source.
Existing strategic instant, day-position and injury recovery types have distinct
roles from general elapsed time. An existing Lens 12 disposition owns the same
immediate ambient duration defect and conditional repair. The day evaluation
basis preserves immediate activity and prayer allocations; changing it alone
would double-scale them. The day evaluation basis preserves immediate activity
and prayer allocations; changing it alone would double-scale them. Organization
and reading hours also rely on the schedule evaluation basis. The Lens 13
immediate defect is linked to the existing Lens 12 repair owner; broader
obligations remain open. The Lens 11 immediate defect is linked to the existing
Lens 12 repair owner; broader obligations remain open. Dialogue answer receipts
match prompt action kind before ballot comparison; ordinary browser submission
uses a fresh ID. Browser transport makes one fetch; dialogue receipts omit actor
and ballot while answer rows retain them. Gateway has one explicit send;
dialogue role maximum controls whether multiple eligible actors can share a
session. Authored prompts include respondent roles allowing four players; a
concrete joined pair and reused action ID remain conditional. Receipt retries
first require active membership, owning party, settlement, personal time and NPC
co-presence. Current session state has only Active; answer responses reconstruct
a current view rather than a receipt snapshot. Topic and join accept any
existing session action receipt; answer additionally checks prompt action kind.
A post-commit view failure leaves the inspected browser path without a newly
rendered view; other refresh paths remain open. NPC reselect starts a timestamp-
nonce encounter; the inspected browser client does not directly call the session
view endpoint. Dialogue social refresh reuses its existing view rather than
fetching a new session view. Ballots retain vector order; unanimous comparison
is order-sensitive and policy remains open. Dialogue receipt, case-fact replay,
startup identity and projection decoding have distinct failure boundaries. A
manual retry from an unchanged pre-commit view uses a fresh ID and encounters a
closed prompt or stale revision. Shared fetch throws plain errors before the
errantry retry helper can inspect HTTP status; answer uses a separate fresh-ID
path. Errantry maps database-call errors to 422, but the shared wrapper strips
numeric status before retry handling. Errantry exact retries bind actor,
session, action and current case-contract authority before presence checks.
Errantry HTTP status and retry ownership is recorded as a separate open
obligation. Errantry maps both gateway HTTP transport failures and database
error responses to 422. Pinned reqwest may return pre-dispatch errors or errors
while a request is in flight; commit outcome remains unproven. Pinned
SpacetimeDB reducer documentation says failed reducers roll back; HTTP failure
and commit correspondence remains open. Existing helper tests inject status
directly and do not exercise the shared fetch wrapper on conflict. Current
temporary dismissal transfers currency without clearing the positive stake
before its guard; reducer failure implies rollback under the documented
contract. Battle loot can credit recorded living participants without excluding
temporary characters; enrollment and liquidity remain open. Battle result
records living party members without a temporary exclusion; a temporary
character joining that party remains unproven. Party merge checks authority and
shared location; its inspected member move does not explicitly filter temporary
status. Browser join routes use session character IDs; inspected wounded and
social demo NPC constructors pass false for temporary status. Browser selection
and inspected grant paths reject temporary characters; non-browser producers and
later status transitions remain open. The inspected tactical temporary creator
makes a solo-party enemy; other membership producers and lifecycle paths remain
open. Stake value is not necessarily liquid coin; temporary settlement requires
enough currency stacks and can fail earlier. Temporary teardown refuses a party
containing a different member; inspected alternate paths create solo parties.
Disband independently requires stakes settled and the remaining pool liquidated
before reserve payout. Browser and approved-action removal share the reducer;
simulation also invokes inventory withdrawal for purchases. Three ledger entries
share the immediate conditional settlement mismatch; broader Lens 11 and 13
obligations remain open. Owned-item deposits can credit member stake; the
inspected encounter helper does not itself authenticate the actor. Owned-item
deposits can credit member stake; the inspected encounter helper does not itself
authenticate the actor. Browser and simulation deposit callers differ; the
inspected reducer entry does not invoke the strategic-character authority
helper. A public-interface database startup recipe exists; actual deployment
access and reducer invocation remain unverified. Current external reducer
documentation supports a callable-client concern but is not pinned to the
project native version. Pinned SpacetimeDB documentation describes client-
callable reducers and in-reducer identity checks; deployment access remains
open. Deposit caller authority is recorded separately from temporary stake
settlement; reachability and aliases remain open. Deposit validates character
and inventory association but the inspected entry has no explicit caller-
identity guard. Inspected deposit value, equipment and stake helpers do not add
a caller-identity check. Inspected deposit classification and object lookup
helpers do not compare caller identity. The inspected capability refresh does
not directly check caller identity; downstream loaders remain open. Personal
inventory and party membership schema rows are public; the full Character table
is private. Authored Rhenish gulden has currency kind and unit value one;
initialized deposit classification remains to be checked. Fresh item
initialization projects embedded catalog definitions with default omitted
equipment and capabilities. Authored currency is excluded from food and explicit
measured-material classifications under fresh initialization. Public startup
uses an all-interface bind; inspected disease and equipment loaders do not
directly compare caller identity. Public startup uses an all-interface bind;
inspected disease and equipment loaders do not directly compare caller identity.
Startup passes the requested bind to SpacetimeDB unless a process already
listens; actual deployment remains unverified. Transfer and discard bodies also
lack explicit caller checks in the inspected scope; each effect remains
independently qualified. Browser discard is self-only; browser transfer permits
the active character to be sender or recipient. The multi-item offer checks
same-party item relationships but has no actor parameter; intended consent
remains open. Party-offer UI has one staged confirmation; the inspected test
covers encounter guard presence, not consent. Both inventory rails intentionally
stage unequipped-item transfers in either direction; consent policy remains
open. The scripted firewall allows web and SSH inbound without a direct
database-port rule; actual deployment remains unverified. Game deployment
templates bind the database to loopback and restrict tactical proxy ports;
actual topology remains unverified. The game deployment script installs the
loopback service and restricted proxy templates; live topology remains
unverified. The game deployment script installs the loopback service and
restricted proxy templates; live topology remains unverified. Existing caller
authority accepts the strategic gateway or the owner of a registered simulation
character; multi-character scope needs explicit treatment. Party joins combine
actor authority and same-simulation-scope checks; inventory consent policy
remains distinct. Existing source-text authority tests cover listed recruitment
and mission reducers, not the inspected inventory mutations. Non-newborn
creation and simulation setup credit currency through a stack helper; runtime
effects remain unverified. Checked grants insert a non-individual stack;
currency bypasses weapon metadata and the wrapper separately discards errors.
Personal and party grant-error erasure have separate owned ledger repairs from
the caller-authority question. Browser withdrawal uses the session actor; the
inspected reducer validates stake and custody without an explicit caller check.
Simulation withdraws stake for first-aid and ammunition shortfalls; authorized
behavior must survive any authority repair. Corrected the current inventory
authority plan, which had inherited unrelated temporary-stake dismissal
acceptance. Corrected errantry retry acceptance inherited from the distinct
dialogue receipt plan. Corrected the temporary-stake plan observation while
retaining conditional reachability. A bounded check opened 555 current partial
plans and sampled five status labels; content correctness remains unproven. All
555 current partial-plan references matched their recorded hashes at queue 3724;
content correctness remains unproven. The temporary-stake current plan again
states the conditional settlement mismatch; reachability remains unproven. The
disease fixture creates missing party participants as non-temporary but may
reuse existing records; temporary-member reachability remains open. Debug item
physical reflectability, replication and iteration construction are qualified;
restore reconstruction remains open. The debug dump excludes physical equipment
data, and inspected item relationship hooks do not recreate it; other indirect
reconstruction remains open. Debug restore may also omit player equipment action
state; indirect initialization and runtime behavior remain open. The shared
attack-result handler requires wounds on both combatants; debug restore
initialization remains unproven. Existing dump and join tests do not assert
wound or equipment action state; combat fixture helpers explicitly provide
wounds. A separate provisional debug combat-state obligation preserves action
and wound state questions independently of fit identity. Related debug
environment and obstacle views share a snapshot boundary but have distinct first
contracts from combat state. Standalone join only installs a loading marker in
the inspected path; combat-state acceptance now covers action and wound
behavior. The Lens 1 schedule adapter and core numeric admission were reread;
connected policy and alias closure remains open. Checkpoint 3402 corrects an
unrelated debug-restore remaining-evidence line appended to the schedule entry
at checkpoint 3401; historical checkpoint retained. The schedule preview
validates its submitted form before organization eligibility filtering;
connected policy and alias closure remains open. Selected role cleanup paths
were traced; organization namespace and indirect-writer invariants remain open.
A bounded catalog ID census found no provisional family-prefix organization ID;
indirect-writer invariants remain open. A bounded catalog ID census found no
provisional family-prefix organization ID; indirect-writer invariants remain
open. Book target build and runtime restrictions were compared; connected
consumers and aliases remain open. Strategic reading and web preview use the
same selected terrain and ordinary-skill target mappings; other consumers remain
open. Book metadata was traced from catalog compilation to selected icon and
bookstore paths; exact aliases remain open. Selected book rank paths share
aptitude and cap helpers for equal source attributes; hours and timing parity
remain open. Selected ordinary and terrain book hours use matching fields and
shared projection for equal source data; family and timing parity remain open.
Selected web family-hour methods delegate to shared world-schema values;
transport and timing remain open. The item-schema current plan no longer
contains an unrelated schedule constraint; source and alias closure remains
open. Food class validation and runtime enum mapping were compared; independent
aliases and consumers remain open. Selected food lot creation maps ration class
to preserved preparation; independent aliases and other consumers remain open.
Party food-lot cause loss precedes an existing grant error-erasure boundary;
reachability and exact ownership remain open. The selected party grant creates
its stable object before the food lot; native visibility and failure
reachability remain open. Pinned table iteration documents visibility of same-
reducer inserts; other party food failure conditions remain open. Pinned unique
lookup delegates to a native index scan; same-reducer index visibility remains
unproven. Ten registered P02 source scopes were reread under Lens 11; broader
connected work remains open. Current narrative bridge cost specifies food that
the measured-item cost handler rejects; connected reachability remains open.
Current narrative bridge cost specifies food that the measured-item cost handler
rejects; connected reachability remains open. Catalog embedding and choice
view/reducer checks connect the authored bridge cost to its rejecting measured-
item handler under stated conditions. The bridge is eligible for weighted
errantry selection; existing measured food operations coordinate lot, fraction
and custody state. Narrative row-count checks do not inspect remaining food
fractions; whole-versus-partial cost policy and agreement between view and
reducer remain open. Pinned native reducer documentation establishes rollback
when the narrative cost error propagates; food authority requires conservation.
Scoped narrative measured-food cost discovery is complete; existing acceptance
requires measured availability and four whole portions, with partial-row
allocation left to repair. Current personal storefront merging bypasses the
stable-object predicate for existing non-food simple rows under stated
conditions. Simple item and container-capacity projection is explicit; a
simulation fixture creates a stocked general store with a present merchant.
Ordinary settlement import infers a general store and general-goods stock, while
actual trade still requires provider and transaction preconditions. General-shop
operator creation and provider selection are source-qualified, subject to
building limits and presence; failed reducer transactions roll back. Scoped
storefront stable-object discovery is complete; independent quantity-one objects
and fractions remain required in the repair. Scoped storefront stable-object
discovery is complete; independent quantity-one objects and fractions remain
required in the repair. The party grant wrapper discards checked errors; loot,
currency and merge callers can mutate source state afterward, subject to actual
failure reachability. Party merge may recreate partially consumed non-durable
food at whole catalog defaults; a separate conditional custody question is
registered. Party merge may recreate partially consumed non-durable food at
whole catalog defaults; a separate conditional custody question is registered.
Elapsed-needs consumption can leave a partially consumed party food row before a
later merge. Party food lot, amount and object are separate records; inspected
native delete wrapper does not specify companion cleanup. Join admission does
not explicitly exclude partial food; pinned indexed deletion does not specify
companion-table effects. A successful eligible party merge recreates partial
non-repairable rations at whole catalog defaults; old companion deletion
behavior remains unresolved. Party weapon initialization has fallible lookup,
evaluation and encoding after row insertion; an admitted failing default remains
unproven. Existing test source intends to evaluate and encode each melee
default; no passing test result is claimed. Scoped party-grant error discovery
is reconciled to its Lens 12 owner; a supported failing initializer remains
unproven. An adopted full frozen Lens 13 challenge read covers the extension
ranges; current reconciliation remains open. Four more frozen extension scopes
reuse adopted Lens 13 reads without new body credit. Eight frozen extension
scopes reuse adopted Lens 12 reads without new body credit. Twenty frozen
extension scopes reuse adopted Lens 11 reads without new body credit. Two
current Lens 11 extension ranges were reread after registration; connected work
remains open. Mission enrollment and terminal receipt submission paths are
source-qualified; roster and transport boundaries remain open. Strategic receipt
admission checks participant and contact bounds; the shared roster caps
membership at 16, with accumulator containment still open. Tactical consequence
producers filter on Party side; captured-roster containment and aggregate
numeric admission remain open. Normal non-temporary tactical admission checks
the captured roster; a temporary-character exception requires lifecycle
qualification. Temporary creation assigns a character, while Party-classified
tactical spawn requires a matching loading player; combined reachability remains
open. Tactical applied limb damage is clamped before injury aggregation;
complete receipt numeric bounds remain unproven. Character limb health
initializes at one and surgery projects clamped values; temporary and enemy
scaling remain separate. Receipt injury inputs are checked for finite bounds,
while stored injury components and other writers still require qualification.
Strategic autoresolve and other strategic injury writers are traced to their
immediate inputs; upstream values remain open. The frostbite increment is
finite; autoresolve and healing still require upstream numeric qualification.
Autoresolve logs and internal wounds use different cut/blunt proportions; the
intended contract and downstream effects remain open. Tactical consequences use
the weighted split; a separate conditional question records autoresolve logged
shares and durable injury semantics. Contact precision 1 produces equal cut and
blunt energy on an unarmored positive-energy hit; supported encounter
reachability remains open. An authored ranged weapon has positive precision and
its equipment projection carries that value into combat. Authored bandit
precision feeds strategic enemy weapon construction; encounter selection and
landed hits remain open. Journey encounter selection includes bandits and enemy
melee contacts target allies; particular contact conditions remain open. An
earlier Lens 1 question overlaps the damage recorder; its exact relationship
remains open. Default armor coverage is empty, allowing the unarmored contact
branch; actual battle reachability remains open. Allied equipment projection
retains empty coverage for unprotected body parts; contact timing remains open.
Intended weapon contact preserves weapon precision, while haft and pommel
contacts use unarmed precision. Existing test source describes positive-energy
sword contact; it does not prove a specific bandit hit. A hypothetical equal-
channel hit makes the two injury splits straddle the fracture threshold; runtime
occurrence remains unproved. Direct remote verification confirms the selected
current source pin despite a stale local tracking reference. Recent
qualification paths match the selected source tree despite the older checkout.
Historical errantry source is read under Lens 2; movement and elapsed time
owners need current reconciliation. Current movement and elapsed coordinates
have distinct semantics from absolute StrategicMinute. Nearby time owners have
different contracts from journey movement and elapsed coordinates. Existing
camp-coordinate owner question now includes generated client and web transport
evidence. Journey creation and redirect admit zero coordinates, including a real
movement-zero camp. Route totals are positive and bounded, while progress and
camp movement may be zero. Fallback travel, independent return routes and
redirects have distinct duration producers. Journey coordinate schema changes
must preserve gateway-authorized reducers and gateway-only views. Forecast
refresh rebases movement and elapsed coordinates independently and merges
contiguous camps at the same movement point. The full current errantry file is
read under Lens 2; connected owners and clue admission remain open. Raw clue
positions are validated at puzzle level and used directly for errantry lookup;
an existing Lens 3 admission question is retained. The gateway validates
retained puzzles before projection; direct public clue admission remains open.
Named puzzle choices do not themselves own ordinal clue positions; bounded
construction and serialization remain to be designed. Tactical insight stores a
raw threat string despite an existing ThreatId; inspected gateway output uses
only text. First-party symbol uses do not show a retained insight identifier
consumer; public serialization remains an open contract. Hostile-group
construction catalog-checks identity before retaining a string; insight
presentation parses it again. Hostile group writers include checked
materialization and a fixed-bandit standalone insertion. Selected escalation and
resolution updates preserve the stored enemy type. Case-site and canonical
journey-camp identities differ from the trial camp token; mapping remains open.
Trial binding has party and journey coordinates needed by the canonical camp
constructor; unbound state needs explicit treatment. The scoped challenge parser
discards the ordinal and cannot alone own complete retained identity. Unbound
trial tokens and zero departure sentinels represent related state; a conditional
zero-epoch question is retained. A freshly initialized wilderness anchor cannot
produce zero departure under the inspected clock calculation. Inspected
production anchor writes use the official clock and settlement arrival clears
the anchor. Canonical camp identity has validated versioned serialization
distinct from the current trial token. Frame and trial records are persisted;
challenge ID is used to select a trial. Road trial identity is checked through
its own authority, errantry list and private overlay. Puzzle and road camp
checks share identity but differ in destination and timing admission. Puzzle and
road caller guards and retry predicates have distinct scopes. Camp redirect
clears reached camp identity before continued travel. Authorized redirect clears
camp stops; later travel may record new stops. Departure and redirect derive
anchors from distinct clock sources. Character and party clocks have distinct
initialization and elapsed updates. Wilderness time wraps within the anchor day;
rest advances member clocks. Resolved interruptions establish a camp only at
coherent exact coordinates. Ordinary travel encounter scans select a boundary
after current movement. Direct camp rest records elapsed time without adding a
reached stop. Gateway and web routes exchange challenge IDs but not frame or
trial IDs. Challenge classifiers retain different subsets of the full identity.
Existing location identities do not encode the complete challenge key. Generated
authority and projection rows carry string challenge IDs. Challenge IDs flow
through receipts and solved case facts. Developer investigation quests reject
authored challenge objectives. Road authority accepts errantry and narrative
occurrence identity families. Road occurrence identity links deterministic
combat and follow-up records. Road occurrence identity also links actor context
and information sources. Information IDs can contain the delimiter used in
source keys. Current authored information IDs contain no colons although
admission allows them. Identity repair acceptance now spans creation,
boundaries, retries and outcomes. Finale-defense enum and retained JSON require
behavior reconciliation. Finale-defense enum and retained JSON require behavior
reconciliation. Inspected mission paths use neutral countermeasure multipliers.
Mission combat scale is captured from the hostile group separately. Finale group
values derive from the threat profile before mission capture. Pinned source
search found no other named finale-defense reader. Core and server presenter
enums overlap across a generated boundary. Core has an established conditional
database-type derive pattern. Presenter acceptance covers generated boundary,
name, slug and speech. Tactical insight stores a string derived from a typed
threat ID. Insight projection exposes text while public Serde remains an
admission boundary. Threat syntax admission is narrower than catalog membership
checking. Threat syntax admission is broader than catalog membership checking.
Threat syntax storage is reused by report and evidence vocabularies. Threat and
evidence ledger entries share syntax storage but remain distinct. Catalog syntax
and strategic-place component have different admission bounds. Shared catalog
syntax must also work in build-script validation. Public threat profile
admission is now a separately tracked question. Public threat profile admission
is now a separately tracked question. Public profile callers include both parsed
and direct typed inputs. Public profile callers include both parsed and direct
typed inputs. The normal quest solver parses catalog-backed hostile causes.
Authored profile compilation needs an ID before live catalog membership. Several
server consumers parse stored threats before profile lookup. Inspected manifest
paths deserialize catalog-checked threat IDs. The encounter panel catalog-parses
its threat before display. Threat admission acceptance spans public, catalog and
native boundaries. Existing invalid-data fallbacks are not compatibility duties.
Threat syntax and membership share one threat-named error. Authored threat
errors cross server and simulator wrappers. Threat error and parsing entries
remain independently scoped. Errantry trial order and selection need further
reconciliation. Ordered clue position admission remains under reconciliation.

| Phase | Status |
| --- | --- |
| Required audit | In progress |
| Product implementation | Not started |
| Independent review | Not started |
| Product and runtime validation | Deferred until audit completion; earlier prohibited probes are historical evidence only |

## Latest checkpoint

Qualified ordered clue position admission.

Validation dispatch, native boundaries and clean owner remain open.

## Working on now

Reconcile clue position validation and public ownership.

- Finish capture rotation, asset scale, report serialization and native boundary
  reconciliation.

- Resolve supported spring tuning and telemetry failure semantics, retaining
  independent numeric-lens duties.

- Reconcile MHR skin admission, shared zero-influence policy and corrective
  index conversion.

- Complete all 13 perspectives, 20 migration families and connected obligations
  before implementation.

- Implement the final types and complete independent review and validation.

## Work completed since the handoff

- Corrected an audit-evidence filename collision: checkpoints 2160 and 2161
  referred to different hashes at one local qualification path. Both exact
  original byte sequences were reconstructed and hash-verified under distinct
  RECOVERED-QUALIFICATION files; the shared path holds the latest historical
  version. Earlier checkpoint references retain the collision and require this
  correction when inspected.

- Traced capture rig scales and native quaternion decomposition; isolated probes
  distinguish degenerate and sheared transforms from observed supported assets.

- Confirmed isolated extreme spring tuning can yield nonfinite state and mask
  its telemetry; recorded scoped repair obligations without claiming live
  capture behavior.

- Rechecked checkpoint 1536 against the frozen selected pipeline revision after
  discovering the clean checkout was older. The substantive dispositions hold;
  validation-observation API and line positions differ. Future reads use frozen
  registered objects.

- Confirmed in isolated product probes that a constructed finite sloped support
  triangle can report zero grade and pass surface, bounded-terrain and collider
  construction; normal generation remains unverified.

- Confirmed in a selected-source module harness that a synthetic FBX rig retains
  a NaN skin weight, and that zero influence reduction yields NaN; downstream
  GPU behavior remains unverified.

- Completed original inventories for lenses 11, 12 and 13 while preserving
  historical attribution and separate connected obligations.

- Recorded source investigations across skill and book targets, rest and travel,
  identity codecs, injury clocks and strategic-to-tactical boundaries.

- Assessed all 403 lens/path checks in the latest 31-path source delta. The
  preceding 59-path delta has 767 of 767 checks assessed; connected source and
  acceptance obligations remain open. Earlier populations and connected work
  remain separate.

- Traced the replacement foliage renderer, shader layouts, material and loading
  readiness paths; retained admission, failure, lifecycle and visual acceptance
  questions.

- Read both registered lockfile revisions and both quality baseline revisions;
  reconciled the lockfile deletion and traced the quality baseline change to a
  function-length ceiling.

- Connected terrain admission, selected geographic input bounds, GPU limit
  selection and native/browser buffer error evidence to an existing open terrain
  allocation obligation.

- Preserved exact checkpoint history and published progress snapshots; the full
  audit evidence graph remains local.

- Assessed all 2,002 path-and-lens cells in the older source delta; connected
  and unchanged-source obligations remain open.

- Consolidated a source-backed attack-identity repair constraint: retain full
  participant and distinct attack instances through scheduling, cancellation,
  batching and diagnostic output.

- Correction to checkpoint 1673: loading resets the empty-party duration, but
  zero reconnect grace can still trigger abandonment. The separate zero-grace
  finding remains open.

- Traced mission enrollment, roster and timeout boundaries, including
  conditional stale membership and discarded enrollment acknowledgement; runtime
  and exact aliases remain open.

- Traced MHR skin errors and filesystem URI boundaries, including conditional
  missing-cluster, HTTP status, project-root and native path differences;
  supported policy and runtime remain open.

- Reconciled the Unix root round-trip observed at checkpoint 1694 with the pre-
  existing Lens 13 ledger entry; it is not a second unique finding.

- Traced the settlement Socializing button through the immediate-activity route:
  the submitted name is rejected. Scheduled Socializing has separate target,
  receipt and training semantics; browser runtime remains untested.

- Reconciled the profession-fame browser preview multiplier discrepancy with
  existing repair entry 2491; the related reward-enum conversion is entry 540.
  No additional unique finding is claimed.

- Traced focused capture plan completeness, absent body-response gates and the
  unsupported-scenario assertion; the assertion is already owned by entry 46.
  Prior report files remain on pre-admission failure.

- Traced pose-trace consumers and a conditional stale armor-readiness sidecar
  path when an output directory is reused for a no-harness capture. Runtime and
  exact alias reconciliation remain open.

- Traced a conditional capture readiness gap: a persistent unresolved playback
  can bypass the inspected frame timeout, while the separate watchdog covers
  armor readiness. External watchdogs and runtime behavior remain unverified.

- Traced a conditional isolated-simulation schedule admission mismatch: a non-
  quarter-hour allocation can pass its local total check, while stationary,
  settlement and location-aware preview consumers expect a validated schedule.
  Simulation commit and end-to-end reachability remain unverified.

- Confirmed in an isolated selected-method probe that an out-of-range anime
  breastplate gauge can panic before validation under checked arithmetic;
  unchecked arithmetic reaches the outer range error. Full loader and GPU
  execution remain untested.

## What remains

- Finish the required audit across all 13 lenses, the 20 migration families,
  connected source dependencies, exact canonical mappings, and current-source
  changes. The implementation gate is still incomplete.

- Resolve or explicitly disposition the remaining type ownership, bounds, units,
  frames, authority, construction, and policy questions. Preserve existing
  evidence without treating a narrow source result as whole-family completion.

- Implement the clean final bespoke types and APIs, update generated bindings
  through the prescribed generator where required, and synchronize the owning
  documentation.

- Complete independent review, remediation, required formatting and lint checks,
  applicable product tests, and isolated observable acceptance.

- Reconcile shared filesystem status, URI identity, containment and fallback
  policy with MHR and image consumers; validate platform behavior after the
  audit gate.

## Coverage and verification limits

The newest source-revision batch contains 31 changed paths and 403 lens/path
checks; 403 of those checks are closed. These are audit coverage obligations,
not bug counts. Earlier revision evidence remains separately qualified.

The canonical queue contains 3860 entries. Entries and views overlap, so this is
not a count of unique bugs or a meaningful completion percentage.

No product files have been changed in this continuation.

Earlier probes, including the checkpoint 2123 synthetic parser probe, ran
despite the handoff restriction. A later isolated exporter test build was
stopped before the test ran and its added fixture was removed. No product builds
or runtime probes will be run until the audit gate is complete; final validation
remains outstanding.

No subagents have been used in this continuation. Checkpoint verification is not
independent review.

## Evidence and report updates

Selected source revision: `ff9c1d7d0ea168b7e14eedc5e44879ee7b84fb04`.

- [Checkpoint 3547 publication record](CHECKPOINT-3547.json)
- [Saved report for checkpoint 3547](PROGRESS-3547.md)

PROGRESS.md is the latest working report. It is updated during ongoing work and
whenever a new checkpoint is published. PROGRESS-N.md preserves the report saved
for checkpoint N; later work updates the latest report without rewriting that
snapshot.
