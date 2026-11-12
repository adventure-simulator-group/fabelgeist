# NPZ declared size, CRC and resource policy: q2316

`REPORT.json` records the bounded 95e source and dependency findings, original
059/e275 attribution, conditional reachability and policy questions. The
original final checkpoint 3821 ledger and queue remain missing.

Prospective registrations and read receipts identify exact source and
historical windows. `FAILED-METADATA-ATTEMPT.json`,
`UNREGISTERED-METADATA-EVENT.json` and `READ-LIMITS.json` disclose two
unregistered navigation fragments and the unexecuted source-test lines. The
archive pre-scan and its unsupported nested formats are recorded in
`ARCHIVE-PRIVACY-QUALIFICATION.json`.

The current child manifest/lock, registered first-party source and canonical
Git trees are included. `ANCHORS.json` identifies separately held raw Git
commit anchors. Three exact registry crate archives remain separately held:
their checksums match the current child lock, but a bounded archive scan found
email candidates and unscanned nested ZIP/JAR members. The full flate2
Cargo.toml member is also excluded from this safe packet because it contains
email candidates. `PUBLIC-SUCCESSOR-QUALIFICATION.json` records its exact
full-file identity and the two unmodified registered line excerpts; all other
selected library source files remain byte-exact copies. Original scratch and
archives remain intact in restricted same-machine custody.

To replay all checks, move the packet with the separately controlled anchors
and three original crate archives, or point at the earlier publication's
anchors:

```text
python3 verify-evidence.py --anchors-dir PATH_TO_COMMITS --archives-dir PATH_TO_CRATES
python3 verify-evidence.py --publication-root PATH_TO_PUBLICATION --archives-dir PATH_TO_CRATES
```

Add `--original-queue PATH_TO_ORIGINAL_059_QUEUE` to verify the full original
queue and byte slice; otherwise that private full queue is not independently
checked. `--root-lock PATH` replays the separately preserved root lock metadata
when not using a publication root. `--ledger PATH` replays the time-bound
metadata search only against the exact registered ledger bytes. These scripts
perform evidence restoration and integrity checks, never parser or product
execution. Moving the public-safe packet alone is insufficient for full
archive, anchor or original-queue verification.

`MANIFEST.json` lists every packet file except itself. A later bounded scan is
retained separately. It does not scan excluded archives, anchors, original
queue, previous Git history or unknown formats, and cannot establish general
privacy or independent-machine custody.
