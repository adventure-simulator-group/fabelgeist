# Bounded 059 q2308 evidence packet

`REPORT.md` records the source conclusion, historical reviewer attribution,
related records, recovery qualifications and open questions. `READ-LIMITS.json`
contains this reviewer's exact displayed source intervals and unread
complements; full source files are preserved separately from read credit.
Original 6533 historical plans, receipts, root adoption, verification and the
two exact locator-qualification files are included without replacement.

`FAILED-CONNECTOR-CAPTURE.json` records the first unsaved connector responses.
`TRUNCATED-METADATA-VIEW.json` records the filename-only truncated display,
whose complete saved listing is retained. `LEDGER-METADATA-REPLAY.json`
identifies a reconstructed metadata view; the exact time-bound ledger snapshot
is retained under `ledger-snapshot/` to replay the literal search. That snapshot
is evidence only and is not a new ledger assignment. The historical final 3821
ledger and queue remain unrecovered in this evidence chain.

## Verify exact evidence

The raw commit anchors are deliberately separate because they contain
identity metadata. Supply those two anchors and the complete original and
reconstructed 059 queues alongside this packet. From this directory run:

```sh
python3 verify-evidence.py \
  --current-commit CURRENT_95E_COMMIT_FILE \
  --historical-commit HISTORICAL_6533_COMMIT_FILE \
  --original-queue ORIGINAL_QUEUE_JSON \
  --reconstructed-queue RECONSTRUCTED_QUEUE_JSON
```

The verifier checks both full queue hashes and exact selected entry bytes;
historical receipt and qualification hashes; both commit-to-tree-to-source
paths; the captured connector tree responses and Git object hashes; registered
source displays and complements; and both bounded metadata-search replays.
`MANIFEST.json` covers all packet regular files except itself. The full queues
and raw commits are not included in the scanned packet; without them the
commit roots and queue slices cannot be independently verified.

The privacy scans are bounded indicators, not a guarantee. No product code,
parser, build, test, fixture, or runtime probe is executed by this evidence
verification.
