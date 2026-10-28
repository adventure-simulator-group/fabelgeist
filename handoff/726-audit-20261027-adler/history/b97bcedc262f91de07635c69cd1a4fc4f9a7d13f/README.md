# Fabelgeist continuation handoff to Adler Halbe

Continue the unfinished bespoke-type and structured-error audit in #726 from
this checkpoint. **Do not begin any code repairs until the audit ledger is
complete.** The current task is source discovery and ledger work only.

Adler may hand the audit back to Bruno while it remains incomplete. Ledger
completion is required before code repairs, not before handing the work back.
Return an updated handoff branch with exact checkpoints and next actions.

This is a handoff branch, not a product change to merge into main. It follows
the previous handoff layout: this README is the central entry document,
`CURRENT.json` records the current state, `AUDIT-RESUME.md` owns the detailed
audit frontiers, and `SOURCE-LOCATORS.json` explains portable source access.

## Read first

1. Read [CURRENT.json](CURRENT.json), then
   [AUDIT-RESUME.md](AUDIT-RESUME.md) and
   [SOURCE-LOCATORS.json](SOURCE-LOCATORS.json).
2. Use [PROMPT.md](PROMPT.md) as Adler's continuation prompt. Its no-repair
   gate and permission to return an incomplete audit are explicit.
3. `evidence/` mirrors the ignored `target/recovery-20261025/` directory.
   Restore the required evidence into a preserved working directory. Do not
   overwrite a newer checkpoint. Original absolute paths are provenance;
   resolve them using `SOURCE-LOCATORS.json`.
4. Read the authoritative ledger at
   `evidence/continuation-segovb/ledger.json`, its exact selected states and the
   three paused auditor checkpoints listed in `AUDIT-RESUME.md`.
5. Follow the current repository and crate `AGENTS.md` rules. `skills/`
   preserves the focused audit instructions; this handoff's explicit no-repair
   gate overrides any later implementation steps in those workflows.

`CURRENT.json` and this README supersede earlier resume/status aliases.
Historical receipts remain evidence. A completed proposal is not automatically
root-adopted, a source-accounted role is not repaired current code, and neither
hash verification nor a previous review establishes global audit completion.

## Published implementation

PR #992 is already squash-merged as
`7a9d92ad7357779e9280146c521bd8588b86fcdb`. Its product tree is
`9ad4f2300a9255bebc06f3aa1b15e58f69382130`. The former local checkout,
`5376d149da40b9ba3afd93d86827832eab1f1f89`, has the same product tree.
The working tree was clean at this handoff.

That PR carries typed fractions through locomotion and dive blending. Bruno
explicitly requested its completion and merge while the broader audit was
unfinished. That exception is closed. Same-head CI and relevant hosted
baseline reductions were deferred; they are not claimed passing. Fetch and
inspect current main when resuming, without starting another code repair.

## Next work

Review pending complete auditor proposals against their exact receipts, retain
all partial-read qualifications, then continue the next bounded unread scope.
Do not restart qualified completed source work or transfer one lens's verdict
to another. Full frontiers, missing material and required exclusions are in
`AUDIT-RESUME.md`.

All thirteen lenses, registered source populations and twenty migration
families remain subject to the ledger's completion gate. Connect findings
through actual producers, internal APIs, retained state, consumers and native
boundaries. Record canonical ownership, repair dependencies and acceptance
requirements before implementation. Preserve conditional cases, policy
questions, exact aliases and rejected suspicions honestly.

## Working constraints

- No product edits, tests, fixtures, helpers, assertions, doctests, builds,
  lints, scanners, generators, runtime probes or CI fixes during discovery.
  Existing test source may be read. Git/source and evidence bookkeeping are
  allowed.
- Backward compatibility is unsupported before release. Do not preserve old
  downcasts, obsolete error prose, invalid behavior or replaced APIs for
  compatibility. Later repairs must use clean final schemas and APIs.
- Keep tactical tick state in transient Bevy. Regenerate database bindings
  from their owning schema later; never edit generated bindings by hand.
- Preserve unrelated work and the four local stashes in `CURRENT.json`.
  Their contents were not transported. No public or player database resets.
- No root README or wiki prose changes belong to this audit.
- Keep receipts in ignored local output. Publishing this expressly requested
  handoff does not authorize ordinary iteration reports on main.

## Evidence and return handoff

[MANIFEST.json](MANIFEST.json) records exact payload paths, byte sizes and
SHA-256 hashes. [SOURCE-BYTE-VERIFICATION.json](SOURCE-BYTE-VERIFICATION.json)
identifies the source authority checks;
[TRANSPORT-VERIFICATION.json](TRANSPORT-VERIFICATION.json) records Git blob
readback. Git text conversion is disabled in this subtree. These checks prove
transport integrity, not audit coverage, CI or runtime behavior.

Bruno explicitly authorized public disclosure of the complete audit state,
source locators, hashes and workflow constraints. The evidence and skills are
included on this branch. No ZIP, new product worktree, database operation or
conversation snapshot is needed.

When handing back to Bruno, publish an updated handoff branch with this same
central-document structure. Include the updated ledger, exact evidence,
root-adopted versus proposed states, partial-read ranges, mandatory
qualifications and each active lens's next bounded action. An incomplete
handoff is valid. Keep the no-code-repairs-until-ledger-complete gate explicit.
Do not merge the handoff branch into main or use an issue as its entry point.
