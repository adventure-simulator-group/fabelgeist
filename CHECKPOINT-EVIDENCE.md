# Checkpoint evidence checks

Run the two evidence-only commands below from this directory. They verify the
captured snapshots, exact transports, additive status and review corrections,
queue and ledger counts, bounded scan receipts and same-machine custody
receipts. The full predecessor driver covered the earlier 173-file snapshot;
the later files have separate checks. Ciphertext and unknown values remain
outside the bounded privacy scans.

```sh
python3 -B verify_postf2ec_checkpoint.py
python3 -B delegated/postf2ec-followups/final-guard/VERIFY-CLOSING.py --publication .
```

The latest bracketed main observation is in
`delegated/postf2ec-followups/final-guard/PREPUSH-MAIN.json`. The original
checkpoint 3821 queue and ledger remain missing; all thirteen lenses and 73
nontext/no-positive-EOF contracts remain open. Full recovery requires the
same-machine key, prior queue anchors and documented nested keys; custody is not
independent. No product repair, build, test or runtime probe was performed.
