# Application review custody at a later cutoff

`application-encrypted/` preserves the candidate, its separate predecessor and
successor evidence replays, the independent application review and replay, and
copies of the application and peer stage directories. The recorded
authenticated same-machine restore matched 12 collections, 109 regular files
and 20 directory modes, including roots. Its original restoration receipt and
both finite scan receipts are preserved in `application-custody-receipt/`.

The added receipt wrapper checks ciphertext and receipt identities and the
recorded results without a key. Without the separately held prior inventory, it
does not repeat the comparison against 626 earlier nonces. The independently
recorded authenticated restore, earlier ciphertext, source and historical Git
objects, separate key and later control delivery remain dependencies. The full
later control delivery is outside this ciphertext's earlier cutoff. Same-machine
custody is not independent backup, and the minimal Git objects do not constitute
a full source checkout or complete Git closure. Finite scans do not establish
the absence of unknown private data.

The captured application was a proposal at this cutoff. Installation,
publication, remote recovery and product behavior require separate evidence.
