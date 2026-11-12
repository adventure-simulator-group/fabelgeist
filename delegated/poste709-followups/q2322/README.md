# q2322 portable source-evidence packet

Read `REPORT.md`, `READ-LIMITS.json`, `READ-QUALIFICATIONS.json`, and
`PRIVACY-QUALIFICATION.json` before reuse. Historical records retain their
original revision, reviewer attribution, cumulative partial-union limitations,
and zero root reread credit. Fresh current read credit is limited to numbered
source display receipts. The missing checkpoint 3821 final queue and ledger
remain missing; the current ledger snapshot is time-bound.

`MANIFEST.json` binds every packet file except itself. The evidence-only
`verify-evidence.py` checks packet bytes, the selected entries against both
external queues, original historical files and pointers, exact commit/tree/blob
chains, registered source displays, and bounded ledger searches. It requires
separately retained raw commits and full queues to avoid publishing unnecessary
identity metadata or the entire private queue:

```
python3 verify-evidence.py \
  --current-commit /path/to/95e63628f9d70b8dd6e6fede64334696c6995009.commit \
  --historical-commit /path/to/6533e444fa37552927070cc8d8190907fd3b413e.commit \
  --original-queue /path/to/CLOSURE-QUEUE-100.json \
  --reconstructed-queue /path/to/baseline-059/queue.json
```

`FINAL-GIT-PROOF-REGISTRATION.json` records their local relative custody where
applicable; raw commit and queue hashes allow independent matching. Included
Git tree objects and exact source captures permit offline replay. The bounded
privacy scan is not a guarantee against unknown sensitive formats or values;
its external final receipt binds the completed manifest without self-reference.
