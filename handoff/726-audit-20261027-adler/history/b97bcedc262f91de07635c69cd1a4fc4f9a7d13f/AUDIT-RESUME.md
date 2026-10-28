# Audit resume points

The audit remains incomplete. Resume qualified source discovery from the saved
frontiers; no code repairs may begin until the ledger is complete. Adler can
return the incomplete audit to Bruno with an updated handoff branch.

## Authoritative and proposed frontiers

All indices below are zero-based. Counts describe scoped discovery with
findings retained, not repaired or validated code.

| Lens | Root-adopted frontier | Later evidence awaiting root review |
| --- | --- | --- |
| 08: behavior preservation | 2,853 original roles, all 701 affected frozen base comparisons, 246 overlays, 38 merged paths and 196 shared scopes accounted | New extensions, current source, twenty families, aliases and connected acceptance remain pending |
| 11: type granularity | 1,195 original roles; next 1195; STATE-proposal-101 | Complete packets 102-107 reach 1207; packet 108 is partial at original 1207 through line 2160; next line 2161 |
| 12: magic strings | 1,079 original roles; next 1079; STATE-proposal-228 | Complete proposals 229-233 reach 1093; original 1093 was read through 1800, but persisted read log covers through 1200; reconcile this disclosure before assigning coverage |
| 13: magic numbers | 350 original roles; next 350; STATE-061-proposal | Complete packet 223-230 / STATE-062 proposes 360; in-flight 360-364 reads remain qualified in the checkpoint |

The other nine lenses retain their saved states and qualifications in the
ledger. Their original/overlay/merged/shared accounting does not clear new
extensions, current source, family reconciliation or deduplication. In
particular, inherited count variants and report-only rows must retain their
existing qualifications; do not flatten them into an all-green table.

Paused checkpoints, under `evidence/continuation-segovb/audit-resume-20261027/`:

- `lens11/HANDOFF-paused-20261027-001.json`
- `lens12/HANDOFF-paused-adler-001.json`
- `lens13/HANDOFF-ADLER-20261027-001.json`

Latest root adoptions are ADOPTION-066 (lens 13), ADOPTION-067 (lens 11), and
ADOPTION-068/069 (lens 12). Lens 08's latest state is STATE-129; its
batch-130 receipt consolidates exact same-lens evidence for the 701 original
base comparisons. It does not complete current-source or extension discovery.

## Scope and ledger obligations

The original scope is `evidence/audit05/original-scope-072.json`, SHA-256
`d1c798df997434ae8e89e433237f798de5c87a38d634f3a7d2f1b0f8af9b1f0f`.
The 246 overlays are `evidence/audit05/raw-overlay-registration-000.jsonl`,
SHA-256 `c26e2def8f8520c870f359ef66919ce730fcddb65beb051365fce42371a7bc68`.
The original shared registry is
`evidence/shared-material-successor/source-extensions-196.json`.
The 38 frozen merged roles and twenty migration families are separate axes.

`REGISTRATION-156-extension-selector-corrections.json` fixes two stale
historical JSON pointers without rewriting their records. Its exclusions are
mandatory. `EXTENSION-NAVIGATION-001.json` inventories 525 registration
authorities and 1,607 candidate source-variant locator groups. Those figures
are navigation, not a new unique population or exhaustive qualified selector.
Repeated identities, aliases, no-range entries, superseded scopes and
incidental test exclusions still need reconciliation. A locator never grants
another lens body coverage.

`LEDGER-CHAINS-001.json` and `LEDGER-CHAINS-002.json` add concrete chains to the
master ledger. Recent retained source cases include elapsed training
normalization, contact-revision time authority and wedding capacity errors
after writes. Later pending proposals add stack-mass propagation, party
approval scope and recruitment-rating validation concerns. Read their actual
records and qualifications before treating them as current bugs or aliases.
A possible personality projection-ID mismatch was rejected by actual
normalization; tiny coefficient division does not alone prove an infinite
pose because concrete recipients may clamp or reject it.

## Portable evidence and source access

See [SOURCE-LOCATORS.json](SOURCE-LOCATORS.json) for the path projection and
exact source archive pins.

`evidence/` is a byte-preserving snapshot of the ignored local
`target/recovery-20261025/` tree. Restore that projection only into a fresh or
appropriately preserved workspace. Do not overwrite another continuation's
receipts. The manifest records every transported file and SHA-256.
Historical receipt hashes remain valid because source/evidence bytes were not
rewritten to change local provenance paths.

Old absolute paths are provenance, not instructions to require the original
machine. Resolve `target/recovery-20261025/...` through this evidence tree.
For historical worktree paths, use the original registered source hash and the
local transported object, or retrieve only the specific object from the
frozen archive at `f82adcabd9d89c4fa2dd7f99253a45297a0681ee`:
`handoff/774/audit/reconcile-adler-20261023/recovered-sources/objects/<sha256>`.
For other archived supplemental paths, preserve the exact recorded suffix and
consult the prior handoff's SOURCE-LOCATORS and portability records. Do not
extract or reread the entire older frozen tree. Missing base objects can be
retrieved by exact registered commit/path and verified against the expected
hash. Never substitute an arbitrary current file merely because its path fits.

The prior portable handoff is commit
`90b18956a366ab38f9cd79d856abbcce10396bb3`, under
`handoff/774-continuation-20261026/`. This new snapshot supersedes its resume
frontiers, while keeping earlier evidence and qualifications as history.

The `skills/` directory preserves the thirteen focused audit instructions and
the coordinating bespoke-audit instruction used for this work. The explicit
no-repair gate and no-backward-compatibility instruction in PROMPT override any
conflicting default workflow. Do not automatically begin an implementation or
validation wave from a skill's later steps.

No private conversation, environment file, credential file or stash content is
included. Publication pattern review found no credential candidates. Transport
verification is separate from source-body review, code validation and ledger
completion.
