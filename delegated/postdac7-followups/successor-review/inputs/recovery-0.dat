# #726 audit recovery handoff

## Recovery counts

**Partial recovery:** the user authorized publishing this package as the
handoff. The final authoritative ledger, queue and checkpoint remain missing.

- 2,846 original registry identities have verified source bytes; all seven
  deleted entries also have verified base versions.
- Eight draft trees have 528 verified changed-path entries and 602 distinct
  before-and-after blobs; this does not validate the drafts.
- Twelve late frontier observations and 755 displayed lines have portable
  source evidence; original registrations and final queue bytes remain missing.
- Nineteen Lens 13 adoption and state chains are exact; among the next 14
  roots, six qualifications and packets and one plan are exact. All 14 have
  printed packet and plan hashes; no adoption in that later span is recovered.
- 36,487 original evidence files restored and hash-verified.
- 7,196 later public progress files verified against the recovery manifest.
- Historical snapshot 440 reconstructed to its recorded SHA-256, with 13
  selected lens values and ten referenced state files verified; the complete
  final selection remains missing.
- One Lens 2 qualification recovered from six original receipts, matching its
  recorded hash and preserving 21 historical source rows without new credit.
- Five intermediate Lens 2 states (023–027) match recorded hashes; the
  later selected state 028 and final authority remain unavailable.
- One checkpoint writer traced through 19 recorded changes; a checkpoint 3802
  CURRENT prefix points to snapshot 440, while final CURRENT remains missing.
- Two historical capture repair contract versions recovered to cited hashes;
  their proposed repair and acceptance steps remain unimplemented.
- Nine historical program sources recovered against cited hashes and preserved
  as inert evidence.
- Eight pinned draft commits preserved in an incremental Git bundle;
  restoration requires the recorded base repository object store.
- 786 deduplicated Git source identities across seven manifests, representing
  691 distinct source blobs; restoration scripts are included.
- 83 distinct dependency or external source blobs preserved directly.
- Three connected current-source transitions are preserved, spanning 154,
  59 and 31 changed paths; the final closure queue remains unavailable.
- One historical Lens 4 receipt restored to its recorded SHA-256.
- 755 recorded late source lines across 12 observations matched to seven
  pinned sources, including 387 lines at the final frontier.
- 81 additional historical dependency files matched to cited hashes.
- In a limited check of references in the 212 JSON files, 528 distinct cited
  hashes were not located in the checked manifests or matched dependencies.
  Since that inventory, 44 have been matched; 484 remain unmatched within it.
- 212 historical JSON versions recovered from the transcript and matched to
  recorded path and SHA-256 references.
- 4,080 queue-related command outputs with context preserved;
  0 records remain deferred.
- 1,699 ledger-related command outputs preserved, including 22 with
  machine-specific paths redacted.
- 117 printed ledger-related JSON values and arrays preserved as historical fragments.
- 538 normalized historical audit documents preserved;
  8 additional values are published with personal paths redacted.
- 3,422 historical printed contract observations indexed, covering 1,722
  distinct contract IDs; 3,146 normalized values are preserved with
  unverified audit claims.
- 6,233 source/registration-related command and output records published;
  338 records have personal paths redacted.
- 57 overlapping late source-related records separately preserved.
- 5,623 audit write command bodies preserved as inert historical text;
  265 have machine-specific paths redacted.
- 359 late command/output records preserved, including 17 failures;
  the 70 late-audit write commands overlap.
- 37 unresolved patch fragments preserved for 30 historical paths.
- 54 additional Rust, JSON, TOML and JavaScript literal writes preserved.
- 89 other historical file-change candidates preserved; one personal-path candidate is preserved in redacted form; four
  copied-source candidates have verified upstream locators.
- 509 draft validation-related command candidates preserved,
  including failures and incomplete or redirected output.
- 287 implementation draft file-change records preserved across seven
  worktrees, with published-branch provenance and validation limits.
- 310 Python literal-write drafts preserved, including 5 path-redacted texts.
- 2,363 unverified helper candidates and 155 Markdown write fragments
  preserved as transcript-derived text; 23 more helpers have redacted paths.
- 19 late audit notes recovered as transcript text, with original bytes
  and supporting evidence unverified.
- 2,480 candidate file states and 519 literal-write records extracted for
  further investigation; 2,386 candidate states are Python scripts. These
  counts overlap and are not verified totals.

This is a **partial recovery**, not a lossless handoff of the latest audit
state. The continuation's final local ledger, queues and checkpoint 3821 evidence
were deleted before they were published. Some receipts and registrations have
since been reconstructed, but hashes and summaries alone cannot restore the
missing authority bytes. Do not represent this
package as the authoritative checkpoint 3821 or resume its writer from it.

The originating Codex chat's history tool was subsequently confirmed to
expose older turns, including some commands and tool outputs from before
compaction. A bounded private snapshot contains recorded commands, file changes
and outputs; its completeness remains unverified and the transcript itself is
not included in this package. Some outputs or argument previews may be
truncated. Review any recovered material for sensitive data and verify its provenance and hashes
before publishing or adopting it.

The recoverable material consists of:

- [Upstream source locators](recovered/upstream-source-locators/README.md)
  for four independently verified source-copy candidates.
- [Historical audit write command candidates](recovered/audit-write-command-candidates/README.md),
  preserved without replay.
- [Late checkpoint command history](recovered/late-command-history/README.md),
  including recorded failures.
- [Late-audit write commands](recovered/late-audit-write-commands/README.md),
  preserved without replaying them.
- [Unresolved historical patches](recovered/transcript-unresolved-patches/README.md)
  preserved without applying them.
- [Other literal-write drafts](recovered/transcript-other-writes/README.md).
- [Other file-change candidates](recovered/transcript-other-candidates/README.md),
  with limitations and exclusions.
- [Draft validation command candidates](recovered/draft-validation-records/README.md),
  with recorded results and limitations.
- [Historical implementation draft file changes](recovered/draft-filechanges/README.md),
  including their branch provenance and limitations.
- [Historical Python literal-write drafts](recovered/transcript-python-writes/README.md),
  with no acceptance claim.
- [Hash-matched historical dependency files](recovered/hash-matched-dependencies/README.md),
  with provenance and execution limitations.
- [Text matched to a printed filename and hash](recovered/printed-filename-text/README.md),
  an unreviewed historical helper preserved as inert evidence.
- [JSON matched to printed filenames and hashes](recovered/printed-filename-json/README.md),
  with path and authority limitations.
- [Linked literal evidence](recovered/linked-literal-evidence/README.md),
  including a historical checkpoint 3778 note.
- [JSON bodies recovered from linked references](recovered/recursive-printed-json/README.md),
  including earlier checkpoint evidence.
- [JSON bodies matched to printed references](recovered/printed-reference-json/README.md),
  with historical path and authority limitations.
- [Read and qualification records matched to printed hashes](recovered/printed-hash-audit-records/README.md),
  including 19 historical owner comparisons.
- [Registrations matched to printed hashes](recovered/printed-hash-registrations/README.md),
  reconstructed without running historical helpers.
- [Recursively recovered dependencies](recovered/recursive-dependencies/README.md),
  including historical selected-lens and lens-state snapshots.
- [Historical helper candidates](recovered/transcript-helper-candidates/README.md),
  explicitly unverified and not to be executed.
- A [corpus of Markdown write fragments](recovered/transcript-markdown-writes/README.md).
- [Source and registration output records](recovered/source-registration-outputs/README.md),
  with screening and truncation limits.
- The [final-frontier source trace](recovered/frontier-source-trace/README.md),
  with independently checked recorded lines.
- [Late source-related records](recovered/late-source-read-records/README.md),
  with their commands and output limits.
- [Queue-related command outputs](recovered/queue-command-outputs/README.md),
  with context and truncation caveats.
- [Ledger-related command outputs](recovered/ledger-command-outputs/README.md),
  with truncation caveats.
- [Printed ledger fragments](recovered/printed-ledger-fragments/README.md)
  from historical ledger-reading commands.
- A [collection of printed audit documents](recovered/printed-audit-documents/README.md).
- An [additional printed source identity manifest and restorer](recovered/printed-source-identities/README.md).
- A [registered source-object manifest and restorer](recovered/registered-source-objects/README.md).
- Two [preserved native Bevy source objects](recovered/native-bevy-source-objects/MANIFEST.json),
  checked against the recovered registration and crate checksum.
- Seven [partial-output native source objects](recovered/partial-output-native-source-objects/MANIFEST.json)
  checked against complete row fragments and four crate checksums.
- An [alternate-field source restorer](recovered/alternate-output-git-sources/README.md)
  for 15 recorded identities, including binary assets.
- [Selected-lens snapshot 440 values](recovered/selected-lens-440-values/MANIFEST.json),
  with explicit missing-state and authority limits.
- A [pinned draft Git bundle and restorer](recovered/draft-git-bundle/README.md),
  with its source-object-store dependency and validation limits.
- A [raw-output Git source restorer](recovered/raw-output-git-sources/README.md)
  and two separately [preserved external source files](recovered/raw-output-external-sources/MANIFEST.json).
- Three [rawzip native source objects](recovered/raw-output-native-source-objects/MANIFEST.json)
  checked against printed registration rows and the archive checksum.
- Fifty-three [printed-document native source objects](recovered/printed-native-source-objects/MANIFEST.json)
  checked against their citations and 27 crate archive checksums.
- Six [native tangent source objects](recovered/native-tangent-source-objects/MANIFEST.json)
  checked against their recovered registration and archive checksum.
- Ten [additional native source objects](recovered/native-archive-source-objects/MANIFEST.json)
  checked against recovered registrations and eight crate archives.
- A [supplemental source manifest and restorer](recovered/supplemental-registration-sources/README.md)
  for 43 identities linked to later recovered registrations.
- Five [historical queue and selected-lens values](recovered/transcript-queue-fragments/README.md)
  and an index of further printed contract candidates.
- Nineteen [late audit notes](recovered/transcript-notes/README.md)
  transcribed from recorded commands, with explicit evidence limitations.
- Historical [transcript-derived records](recovered/transcript-hash-matched/README.md)
  with matching later path and hash references. Their dependencies and
  authority as current state remain unverified.

- Segovb's original, lossless checkpoint 059 package at commit
  `eb407672813424d64ec31f208193969f5a62620c`. Its transport manifest covers
  36,487 evidence files and its own restoration verifier checks their hashes.
- The later public progress branch at commit
  `78b1d089cb5ee52df0b01c0bf6a020f049b8a126`, containing 3,249 numbered
  checkpoint summaries and reports through 3737, plus 348 incremental
  checkpoint summaries and reports through the reported audit checkpoint 3821.
  These are summaries, not the continuation's full evidence graph.
- Published implementation draft branches listed in `SOURCE-LOCATORS.json`.
  Their existence does not imply review, integration or current-head tests.

[Recovery progress](RECOVERY-PROGRESS.md) and `RECOVERY-CHECKPOINT.json`
record the latest recovery counts and remaining work. `CURRENT.json` records
the latest *reported* audit numbers and known missing state.
`RECOVERED-FRONTIER-3821.json` records the limited final-frontier evidence.
The [late queue entry trace](recovered/final-frontier-entry-2708/MANIFEST.json)
links one printed queue 4151 row to five later derived writer operations. The
final row is an inference with missing values, not restored queue authority.
`AUDIT-RESUME.md` and `PROMPT.md` explain how to proceed without overstating
recovery. The latest public [progress report](PROGRESS.md) and
[incremental checkpoint](CHECKPOINT-INCREMENT-348.json) are copied here for
convenience; their originals remain at the pinned progress commit. The original
handoff remains authoritative for its own checkpoint. See
[VALIDATION.md](VALIDATION.md) for the checks actually performed.

Future checkpoints must publish the complete
recoverable audit state to a remote branch, not just summaries. Rebase the
active working branch onto current `main` early and reconcile source changes
so the audit stays current.

The [original registry source archive](recovered/original-registry-source-archive/MANIFEST.json)
preserves the exact registry and 2,462 distinct hash-matched source objects
covering 2,846 identities. Restore them to a destination that does not exist:

```sh
python recovered/original-registry-source-archive/restore_objects.py NEW_DESTINATION
```

Seven rows describe deleted files, so they have no current file hash. Their
[seven verified base versions](recovered/deleted-source-base-objects/MANIFEST.json)
are also preserved as six distinct objects. These are historical base bytes,
not current snapshot files, audit credit or evidence of current behavior.

The [draft change source archive](recovered/draft-change-source-archive/MANIFEST.json)
contains all before-and-after contents for 528 changed paths across the eight
draft trees. Its complete Git tree objects verify the change lists and source
blobs without another repository. It omits unchanged source bytes and parent
history; the drafts remain unreviewed and unvalidated. Restore the 602 blobs
into a new content-addressed directory with:

```sh
python recovered/draft-change-source-archive/restore_objects.py NEW_DESTINATION
```

The separate incremental Git bundle still needs the recorded base repository
to restore complete draft commits and check out a full tree.

The [selected revision source archive](recovered/selected-revision-registry-sources/MANIFEST.json)
contains 1,824 Git blobs matching 2,193 registry paths at `ff9c1d7`. Another
660 registry rows have no blob at their paths in that revision. The included Git commit and tree objects prove membership for recovered paths
and establish the absent or non-blob paths. These bytes do not establish
reading or audit coverage.

To verify the full package and restore those objects to a new directory:

```sh
python recovered/selected-revision-registry-sources/restore_objects.py NEW_DESTINATION
```

For a Git bundle containing this recovery branch, specify the branch when
cloning: a bundle can lack a default `HEAD` and otherwise produce an empty
checkout. For example:

```sh
git -c core.autocrlf=false clone --branch codex/726-recovery-handoff-20261109 BUNDLE NEW_DESTINATION
```

The destination must not exist. Verify the expected commit and bundle hash
against the independently supplied checkpoint, then run the verifier below.
A bundle copied only to the same computer is not an independent backup.

The seven Git source collections in `SOURCE-COUNTS.json` each provide a
`restore_sources.py` that takes a source repository and a new destination.
A local restoration test is recorded in `SOURCE-RESTORATION-TEST.json`; it
requires the historical Git commits and is not an independent-machine test.

Run `python verify_recovered.py` to check the transcript-derived evidence.
Run `python build_recovery_index.py NEW_INDEX.sqlite` to build a local SQLite
full-text index; it refuses an existing destination. For example, query
`records` with `WHERE records MATCH 'terminal'`. Indexed records overlap and
remain historical and unverified unless their own provenance says otherwise.

Run `python restore.py DESTINATION` to clone the two pinned public sources,
verify the original manifests and restore its evidence, and verify every file
in the progress manifest. The destination must not already exist. The base
restoration requires at least 12 GB for extracted evidence plus space for the
Git clones and original transport package. Use `--verify-only` to verify the
transport without writing the extracted base evidence. Internet access, Git
and Python 3.9 or newer are required. Full extraction requires a
case-sensitive filesystem: the original manifest has two pairs of distinct
paths differing only in letter case. On a case-insensitive filesystem the
script refuses extraction before downloading; `--verify-only` remains
available. A failed or interrupted run leaves `RECOVERY-INCOMPLETE` and must
not be treated as a completed recovery. This does not recreate the deleted
post-handoff ledger or authorize code repair.

Before publication, review the handoff for credentials and private data.
Restoration does not merge a branch, contact a person, or change a database.
