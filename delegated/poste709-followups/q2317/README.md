# Portable q2317 source-evidence packet

Read `REPORT.md`, `READ-LIMITS.json`, `READ-QUALIFICATIONS.json`, and
`PRIVACY-QUALIFICATION.json` before using this packet. Historical records are
recovered originals with their own revision and attribution; current source
views are newly assessed at the 95e pinned baseline. Full source copies bind
exact bytes but grant fresh read credit only to numbered display receipts.
Original queue and reconstructed queue differ as full files, despite equal
selected entry bytes. No checkpoint 3821 final queue or ledger is recovered by
this packet. The current metadata snapshot is time-bound, and neither literal
searches nor this bounded source family establish global closure.

`MANIFEST.json` seals every packet file except itself. `verify-evidence.py`
checks the manifest, selected queue byte offsets and displays, historical
artifact digests and selected pointers, commit and tree identity, source blobs,
registered current displays, and corrected bounded ledger metadata searches.
It performs evidence processing only. Raw commits and whole queues are external
to avoid unnecessary identity metadata and private queue publication; supply
their exact local copies:

```
python3 verify-evidence.py \
  --current-commit /path/to/95e63628f9d70b8dd6e6fede64334696c6995009.commit \
  --historical-commit /path/to/6533e444fa37552927070cc8d8190907fd3b413e.commit \
  --original-queue /path/to/CLOSURE-QUEUE-100.json \
  --reconstructed-queue /path/to/baseline-059/queue.json
```

For this workspace the original queue is in the protected 28ba evidence root;
the reconstruction and historical raw commit are in the central publication;
the 95e raw commit is in the private q2306 anchor. `FINAL-GIT-PROOF-REGISTRATION.json` records the relative local anchor
references. Raw commit
SHA-256 values and queue digests in the packet permit independent matching.
Tree and GitHub source captures are included; no network is needed to verify.

The bounded privacy scan and its limits are documented separately. The packet
contains recovered historical records and source text, not a guarantee that all
potentially sensitive encodings or derived identities have been detected.
