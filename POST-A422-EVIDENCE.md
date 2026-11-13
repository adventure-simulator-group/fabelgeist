# Evidence after checkpoint a422

`delegated/posta422-followups/INDEX.md` links the newly sealed source reviews,
independent reviews, restored historical qualification and reconstructed public
proposals. The proposals remain unadopted. Their draft-time review wording and
later independent reviews are reconciled by the separate chronology record.

The pinned working audit authority is source revision
`95e63628f9d70b8dd6e6fede64334696c6995009`, tree
`1eec97de340d9b7debbb30ca0c0fb2240a73b93f`; it remains incomplete. The ledger
has 1,103 entries and planning relationships, not unique defects. Of 3,713
historical queue records, 1,364 are partial and 2,349 unassessed; none has
semantic closure. All thirteen lenses and 73 nontext/no-positive-EOF
contracts remain open. Original checkpoint 3821 queue and ledger are missing.

Run the evidence-only checks from the publication checkout:

```sh
python3 -B delegated/posta422-followups/VERIFY-ADDENDUM.py --publication .
python3 -B delegated/posta422-followups/frozen/VERIFY.py
```

Recovering restricted originals requires the separately held same-machine key,
earlier q2812 queue anchors and documented nested keys; this is not
independent custody. No product
repair, build, test or runtime probe has been performed.
