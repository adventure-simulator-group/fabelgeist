# Independent q2968 type-ownership review

Reviewer: `/root/mhr_character`. The assessment concerns the registered source
scope only, at pinned product revision `95e63628f9d70b8dd6e6fede64334696c6995009`.
It was performed after the checkpoint then being frozen. No product, example,
fixture, test, renderer or probe was run, and no repair or publication edit was
made. This is not a queue, ledger, lens, or implementation-gate closure.

## Assigned scope and chronology

Recovered original 059 `/entries/2968`, contract
`OBLIGATION-ROOT-L3-EXT-0022-LENS-01`, specifies Lens 1 for
`crates/adventuresim-tactical-server-dispatcher/examples/validate-city-support.rs`
lines 1–140, historical 6533 and e275 identities. The original and reconstructed
selected entry bytes match (SHA-256
`9c93f18b314b3a1e9fc20f2a2e6ce1b12553d9252562c3ff9dbc1b60978ce43f`); their
whole queue streams do not. The pinned 6533, e275 and 95e files are the same
140-line blob `27475f5e3fad6f475cde51f1e8b2317feea85b95`, SHA-256
`92a90abaa4886c7e192fa799d406c9261c1879de3c97af39490d53c1dca4b707`.
Same-file identity is not a clean-tree or whole-tree claim: the historical
pr-working snapshot had unrelated modifications. I independently registered and
personally displayed 1–70 and 71–140 at each revision; see READ-RECEIPT.json.

The older Adler Lens 11 registration `REGISTRATION-ADLER-846.json`, hash
`e16ef24c1ddcb2771ca1bacd2af8dbdc1969e13d69390dd23f228ce96ad0f366`,
identifies original role 1995. The recovered `READ-L11-308.json` records fresh
1–36 and 37–140, attributed to the earlier Adler reviewer, not me. Its measured
hash is retained; I have not located a prior hash expectation authenticating that
receipt. Original 059 q2370 records Root's separate Lens 2 fresh 6533 full-file
read and e275 same-lens reuse, not a fresh e275 read. q3126 and recovered
`READ-ROOT-L3-EXT-020-027.json` separately record Root's fresh Lens 3 full read
at 6533; `READ-ROOT-L3-CURRENT-004.json` records qualified e275 reuse, not a
reread. Both 059-cited receipt hashes match the recovered originals. None of
those earlier reads grant my Lens 1 credit. Checkpoint 3821 remains unrecovered;
no earlier record is promoted into it.

## Scoped observations

- The optional selector is stored as `Option<u64>` in the parsed `Request`
  (assigned source 23–37). The same example strips both matching property IDs
  to `.0` to retain matching compounds and single properties (84–88). Current
  `city_layout/compound.rs:17–19,76–82` and
  `city_layout/compiled/single_support.rs:9–16` show both property IDs use
  `CityPropertyId`; `compiled/assembly.rs:46–64` and
  `compiled/property.rs:30–38` construct them from lot IDs. This is an
  independently corroborated, scoped opportunity to keep the existing typed
  city-property owner through request state and comparison after native CLI
  admission. It is the same first owner question recorded in historical 059
  q2370, `L2-CITY-SUPPORT-CLI-PROPERTY-SELECTOR-OWNER`; it is not an independently
  counted defect or a completed alias inventory. Actual parser adaptation is
  not established or run. `CityPropertyId(pub u64)` is transparent and public;
  this declaration does not itself prove a nonzero or restricted-bit invariant.
- Omission deliberately retains the all-properties mode; a selected ID yields an
  error unless the combined matching compound/single count is exactly one
  (35–37,84–89). Building members are collected as typed `SceneBuildingId`
  values and used to retain placements and related members (90–103). Future
  work must preserve the absent-selector behavior and selection/error semantics;
  runtime behavior or successful CLI parsing has not been verified.
- `adventuresim-core/src/settlement_property.rs:11–27` defines a different
  settlement-scoped string identity containing settlement and building parts.
  The inspected evidence does not establish it as interchangeable with
  `CityPropertyId`, so a shared name alone is no reason to merge the owners.
- Assigned lines 52–67 compare the staged scene-input digest with the result of
  `TacticalSceneInput::digest`, and separately require an imported-package source
  identity. `scene_input.rs:150–173` shows the former is a SHA-256 hex `String`
  after validation; `scene_input/terrain_samples.rs:1–14` and
  `adventuresim-world-schema/src/source_package.rs:1–44` show the latter is a
  checked `SourcePackageDigest`. These identify different things, so the
  source-package type is not a justified replacement for the scene digest just
  because both serialize as hex. Whether another existing exact scene-digest
  owner exists or should be introduced remains open. The selected windows do
  not prove stage-producer authenticity or all admission paths.
- The example checks input/source identity, chooses an explicit capture or near
  vista, recreates the layout, and carries an acceptance flag to its report
  (59–80,105–140). `validate_city_support/layout.rs:10–80` shows the inspected
  reproduction and membership equality checks. Its own report explicitly
  excludes production installation, collision/enclosure, complete access,
  capacity, visual and performance acceptance (127–132). Historical q3126 is
  a related admission question with its own source coverage; these observations
  neither close it nor prove any runtime or production acceptance.

## Historical and recovery limits

q2370 is an existing 6533/e275 Lens 2 reuse candidate with Root's own 6533
reads and qualified e275 proof, not a new finding here. q3126 is an independent
Lens 3 admission record. Historical q2238 is a separate CityPropertyId/opening
namespace question; no alias with the CLI selector is asserted. An original
059 structural search located these pointers; selected q2370 and q3126 raw
bytes match in the original 56f and reconstructed dac7d queues, but full streams
have different SHA-256 and blob identities. Historical sources and proposed
adoptions retain their own revision, reader, and status.

A bounded locator over the observed working 95e ledger searched the exact prior
IDs plus city-support terms and returned no direct matches among 1,084 entries;
that is not proof of no aliases elsewhere or of a clean committed checkout.
The exact observed ledger and raw original/reconstructed queues are retained
only in the restricted packet. No full body credit comes from this structural
search or from inspecting archived bytes. A first combined historical display
exceeded the output cap; only registered, later selected redisplays are used.
Two Git locators failed because the product checkout and partial replay store
lacked pinned objects; a later read-only retry used the complete publication
object store. A planned source-capture window 1–80 exceeded the digest file EOF
53 and aborted before body display/registration; the corrected window was
registered before display. The first receipt comparison also failed due to a
numbering-format assumption; the corrected check passed. All failed attempts are
qualifications, not evidence of missing source or product behavior.

Support windows and their unread complements are recorded in READ-RECEIPT.json.
Git tree proofs bind each captured source to a tree and blob; the commit-to-root
link requires a separately trusted pinned Git repository because no raw commit
or author metadata is copied into the public reconstruction. The public packet
summarizes and identifies historical records but does not contain their exact
private queue bytes. Product execution, repairs, global alias analysis, caller
reachability, and own-lens coverage beyond the listed windows remain open.
