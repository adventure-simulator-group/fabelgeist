# Bounded 059 q2312 evidence packet

`REPORT.md` states the current source findings, historical attribution,
relationships, open decisions, and limits. `READ-LIMITS.json` records exact
personally displayed source intervals and their unread complements. Numbered
source displays and their receipts are separate from the copied complete source
bytes. `HISTORY-FILE-REGISTRATION.json` and the matching exact historical files
preserve the original 059 reader, adoption, and proposal evidence.

The original queue entry and its path-redacted reconstruction are identical at
this entry only; the full files differ. No final checkpoint 3821 ledger or
queue was recovered. Earlier e275 work is not promoted into that checkpoint.
The prior q2307 and q2316 manifests and selected original queue entries are
included for their separately attributed relationships, not as a global alias
or unique-defect count.

`FAILED-GIT-PATH.json`, `FAILED-IDENTITY-ASSERTION.json`,
`SOURCE-EOF-QUALIFICATION.json`, and `READ-EVENT-QUALIFICATIONS.json` preserve
failed and reconstructed views. `PUBLIC-SUCCESSOR-QUALIFICATION.json`
identifies the one local custody path redacted from the metadata registration;
its exact original remains restricted and separately supplied to the verifier.

## Reproducing evidence verification

The portable packet does not itself contain all restricted inputs. Supply the
following alongside it for exact verification:

- The two raw Git commit anchors named by `ANCHORS.json` in one directory.
- The original checksum-bound `rawzip-0.5.2.crate` in another directory.
- The complete original 059 queue and its complete reconstructed counterpart.
- The exact original, unredacted `METADATA-REGISTRATION.json` from restricted
  custody identified by `PUBLIC-SUCCESSOR-QUALIFICATION.json`.

From this directory, run the evidence-only verifier with paths appropriate to
the destination:

```sh
python3 verify-evidence.py \
  --anchors-dir ANCHORS_DIRECTORY \
  --archives-dir ARCHIVES_DIRECTORY \
  --original-queue ORIGINAL_QUEUE_JSON \
  --reconstructed-queue RECONSTRUCTED_QUEUE_JSON \
  --original-registration ORIGINAL_REGISTRATION_JSON
```

The verifier reads only the two named Rust source members from the checksum-
bound source package; it does not read or execute embedded ZIP/JAR fixtures.
It checks the full queue hashes and selected entry slices, recorded historical
file hashes, raw Git object identity and tree-to-file paths, package checksum,
selected source members, displayed source receipts, and unread complements.
`MANIFEST.json` covers every regular file in the packet except itself.

The raw commit anchors contain author/committer email metadata. An earlier
bounded scan of native package archives found email candidates and 30 nested
ZIP/JAR signatures left unread. They are deliberately outside this scanned
packet and require appropriate separate custody. The final packet scans are
bounded indicators, not a general guarantee that all private material has been
found. No product, parser, test, fixture, build, shader, or runtime execution
is performed by this evidence procedure.
