# Verifying the successor checkpoint

From this checkpoint checkout, run the evidence-only checks:

```sh
python3 -B delegated/post5f6a-followups/final-guard/VERIFY.py \
  --publication .
python3 -B verify_post5f6a_checkpoint.py --predecessor /path/to/clean-5f6a
```

The optional predecessor argument supplies a separate clean checkout at commit
`5f6a6e829dca0f8b2a5efff9b6f17fab5db71ae0`; the second command replays its
three snapshot-specific checks there. Omitting that argument skips their replay.
The checks verify recorded evidence identities and structural application, not
the truth or completeness of every source assessment. The final encrypted
group is checked without decryption by the first command. Full authenticated
restoration needs the separately held key and documented evidence dependencies.

The main-branch observation is point-in-time. Checkpoint 3821's original queue
and ledger remain missing. Recovered original records, reconstructed streams,
historical summaries and new assessments remain distinct. All thirteen lenses,
all 3,713 historical queue entries and 73 nontext/no-positive-EOF obligations
remain open. This checkpoint applies six bounded partial assessments and three
planning relationships; it proves no semantic closure or product behavior.
