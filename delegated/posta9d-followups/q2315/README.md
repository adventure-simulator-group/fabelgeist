# Bounded 059 q2315 evidence packet

`REPORT.md` records the current source mappings, historical e275 reviewer and
root attribution, linked q2307/q2313/q2316 records, presentation boundaries,
and open questions. `READ-LIMITS.json` identifies this reviewer's displayed
source intervals and their complements. Preserved full sources and historical
reader ranges are not transferred as personal review credit.

The complete original and reconstructed 059 queues, raw commit anchors and
checksum-bound rawzip source archive are separately held. The included exact
ledger snapshot is a time-bound literal search input, not a new ledger
assignment. The final checkpoint 3821 ledger and queue remain unrecovered in
this evidence chain.

## Verify exact evidence

Supply both raw commit anchors named by `ANCHORS.json` in one directory, the
`rawzip-0.5.2.crate` source package in another, and the two complete 059
queues. From this directory run:

```sh
python3 verify-evidence.py \
  --anchors-dir ANCHORS_DIRECTORY \
  --archives-dir ARCHIVES_DIRECTORY \
  --original-queue ORIGINAL_QUEUE_JSON \
  --reconstructed-queue RECONSTRUCTED_QUEUE_JSON
```

The verifier checks full queue hashes and selected entry slices; linked prior
entry and manifest hashes; historical record hashes and projections; Git
commit/tree/file paths; the locked rawzip archive checksum and its exact named
source members; registered source displays and unread complements; and the
literal ledger search. `MANIFEST.json` covers all regular packet files except
itself. Without the separate raw commits, complete queues and package, this
packet alone cannot prove their complete identity.

`PRIVATE-EXCLUSION-QUALIFICATION.json` records the sole excluded full source
member: rawzip's package `Cargo.toml` contains one email-pattern candidate. Its
original bytes are held in restricted scratch and in the separate unmodified
source archive. The exact displayed feature lines remain in this packet; the
verifier reads the full member from the supplied package to check their hash
and source receipt. Raw commits contain author/committer email metadata. The
prior source-archive scan also found email candidates and nested ZIP/JAR
signatures whose contents were not traversed. These excluded inputs are outside
the packet privacy scan. The bounded scans are not a general privacy guarantee.
No product, parser, ZIP/JAR fixture, build, test, or runtime probe is executed.
