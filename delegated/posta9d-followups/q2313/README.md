# Bounded 059 q2313 evidence packet

`REPORT.md` records the independent current source conclusion, historical e275
reviewer attribution, linked q2307/q2312/q2316 records, policy boundaries and
open questions. `READ-LIMITS.json` identifies exact source intervals personally
displayed and their complements. Copied full sources and original historical
reader ranges do not transfer personal read credit.

The complete original and reconstructed 059 queues, raw commit anchors and
checksum-bound source archive are separate inputs. The final checkpoint 3821
ledger and queue remain unrecovered in this evidence chain. The included exact
ledger snapshot is a time-bound search input, not a new ledger assignment.

## Verify exact evidence

Supply both raw commit anchors named in `ANCHORS.json` in one directory, the
`rawzip-0.5.2.crate` source archive in another, and the full original and
reconstructed 059 queues. From this directory run:

```sh
python3 verify-evidence.py \
  --anchors-dir ANCHORS_DIRECTORY \
  --archives-dir ARCHIVES_DIRECTORY \
  --original-queue ORIGINAL_QUEUE_JSON \
  --reconstructed-queue RECONSTRUCTED_QUEUE_JSON
```

The verifier checks the full queue hashes and selected original entry slices;
the linked entries against their prior packet manifests; the nine original
historical record hashes and projections; commit-to-tree-to-file membership;
the pinned native archive checksum and its named Rust source member; exact
registered displays and unread complements; and the literal ledger search.
`MANIFEST.json` covers all regular packet files except itself. Without the
separate raw commits, source archive and full queues, this packet alone cannot
prove their complete identity.

Raw commits contain author/committer email metadata. An earlier bounded scan of
the native source package found email candidates and nested ZIP/JAR signatures
whose contents were not traversed. They are excluded from this packet's scan
and require separate custody. The final privacy scans are bounded indicators,
not a general guarantee. The verifier reads only the named Rust source member
from the package; no product parser, archive fixture, build, test or runtime
probe is executed.
