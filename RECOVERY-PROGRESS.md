# #726 recovery progress

## Quantitative summary

- **36,487** original handoff files restored and hash-verified.
- **7,196** public progress files checked against the pinned manifest.
- **212** historical JSON file versions reconstructed to recorded path and
  SHA-256 references; their latest-state authority and dependencies remain open.
- **89** other file-change candidates preserved; **5** copied-source
  or local-path candidates remain excluded.
- **310** historical Python literal-write drafts preserved, including **5**
  with paths redacted; validation remains unverified.
- **2,363** unverified helper candidates preserved as inert text, plus
  **23** with machine-specific paths redacted; their originals remain private.
- **155** historical Markdown write fragments preserved, overlapping with
  other recovered notes; appends are individual fragments.
- **19** late audit notes and **5** queue or lens values preserved as
  transcript-derived text or normalized values, not original file bytes.
- **3,146** normalized printed contract values spanning **1,722** distinct
  contract IDs preserved; values overlap and may be field projections.
- **538** normalized printed audit documents preserved; **8** values with
  personal local paths remain private.
- **268** source identities, representing **264** distinct blobs, verified
  against Git and recoverable through the source-object restorer.

These categories overlap. They are not a percentage of the audit completed or
an independently verified number of unique findings.

## What remains

The continuation's final ledger, closure and delta queues, selected-state
pointers, complete registrations and receipts, and exact checkpoint 3821
remain unrecovered. A reported checkpoint or source hash alone cannot restore
its contents. The private transcript snapshot contains additional candidate
file changes, command outputs and literal writes; not all have been examined
or safely published. The original audit goal is paused and no repair work or
new audit credit was performed during recovery.

The [recovery README](README.md) and [resume instructions](AUDIT-RESUME.md)
explain provenance, restrictions and restoration. The older [progress
report](PROGRESS.md) is a historical audit report, not this recovery report.
A fresh shallow clone of commit `a53d425cff091ae19833e16e2b9d235109911b7c`
verified 236 individually manifested files, all 3,146 contract values and all
486 printed documents. A separate original-source clone restored 268 source
identities and rejected an existing destination. These checks do not restore
or verify the missing final ledger.

Next, continue tracing transcript outputs and remaining file-change candidates
against known hashes and references. Publish reviewed recoverable evidence at
each checkpoint, then verify it from an independent clone. Preserve the raw
transcript privately and retain each record's historical source revision.
