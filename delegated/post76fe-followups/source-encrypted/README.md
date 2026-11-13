# After-b111 restricted evidence custody

This group preserves 23 registered evidence collections and one custody-control
collection as authenticated ciphertext. The earlier source-review public
proposal remains separately preserved. Later central draft and review artifacts
are outside this cutoff; the minimal Git replay database is intentionally
excluded. The original queue, earlier ciphertexts, primary key, referenced Git
objects, and other captured dependencies remain separately held.

`VERIFY.py` checks ciphertext identity without the key; with the separately
held key it authenticates archives and can restore into a new destination.
Contents are protected at rest but the key and records are held on one machine.
This does not establish independent custody, full Git closure, recovery of
checkpoint 3821, semantic closure, or product behavior. File bytes and modes
are captured; ACLs, timestamps, and hard-link identities are not.
