# q2321 portable source-evidence packet

Read `REPORT.md`, `READ-LIMITS.json`, `READ-QUALIFICATIONS.json`, and
`PRIVACY-QUALIFICATION.json` before reuse. Historical originals preserve their
revision, reviewer, catalog read gap, and distinct discovery/reuse adoption
status. Source copies and registered-but-undisplayed ranges confer no personal
read credit. The separately authored q2320 assessment is related context, not
this reviewer's coverage. The final checkpoint 3821 queue and ledger remain
missing.

`MANIFEST.json` binds every packet file except itself. `verify-evidence.py`
checks the packet file set, selected queue byte slices, directly referenced
historical records and source objects, selected attribution/discovery pointers,
commit/tree/blob paths, source displays, related q2320 report digest and
bounded ledger searches. It performs evidence processing only. Full original
and reconstructed queues and raw commits remain external to avoid publishing
unnecessary private queue content and commit identity metadata:

```
python3 verify-evidence.py \
  --current-commit /path/to/95e63628f9d70b8dd6e6fede64334696c6995009.commit \
  --historical-commit /path/to/6533e444fa37552927070cc8d8190907fd3b413e.commit \
  --original-queue /path/to/CLOSURE-QUEUE-100.json \
  --reconstructed-queue /path/to/baseline-059/queue.json
```

`FINAL-GIT-PROOF-REGISTRATION.json` records local relative anchor custody;
recorded hashes permit independent matching. Captured tree objects and source
files permit offline replay. Privacy scans are bounded and the external final
receipt binds the completed manifest without a self-referential digest.
