# Bounded NPY q2306 reconciliation

`REPORT.json` contains the current source trace, attribution, open questions,
and recovery qualifications. The current baseline is the pinned 95e revision;
the source-owning historical observations remain attributed to e275 and 059.
The original final checkpoint 3821 queue and ledger remain unavailable.

`SOURCE-REGISTRATION.json` precedes the nine numbered source displays. The
history registrations precede the credited original/reconstructed queue and
historical projection displays; `LOCATOR-QUALIFICATION.json` identifies
unregistered provenance metadata and a truncated locator display. Copies of
original historical files are exact, while JSON projection displays are
reconstructions. `QUEUE-EXTRACTION.json` separates the exact entry byte slice
from its different full original and public reconstructed queue containers.

`git-trees/` contains canonical raw tree objects, `connector-trees/` retains
normalized connector responses, and `sources/` contains the exact selected
source files. `ANCHORS.json` records the separately retained Git commit anchors
and the paths to their copies in the earlier publication. Commit anchors
contain author/committer email metadata and are omitted from this packet.
Thus an offline verifier must supply the two raw commit files separately, or
point to an existing publication containing the recorded anchors:

```text
python3 verify-evidence.py --anchors-dir PATH_TO_RAW_COMMIT_FILES
python3 verify-evidence.py --publication-root PATH_TO_PUBLICATION
```

To additionally verify the original full 059 queue hash and entry slice,
append `--original-queue PATH_TO_ORIGINAL_059_QUEUE`. That file remains in the
protected source extraction and contains paths outside this bounded slice, so
it is not included. Without it the verifier reports this explicit limitation.
These are evidence-only checks; they do not establish product behavior.

The manifest lists packet files, excluding itself. Bounded privacy scan receipts
are separately retained by the coordinator after manifest creation. They do not
cover Git history or prove general privacy. Exact raw anchor files and the
original queue remain separately controlled, with no claim of independent
machine custody.
