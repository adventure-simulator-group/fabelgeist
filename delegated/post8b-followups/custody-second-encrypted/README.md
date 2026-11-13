# Second fixed post-8b evidence custody group

This group holds 49 fixed source trees, three immediate-file snapshots, and one
pre-capture control collection. Original regular-file bytes and root, directory
and file modes are preserved. Owners, timestamps, ACLs, hardlink topology and
work after this cutoff are outside the claim. Previously public ciphertext and
receipt directories and the evolving public stage are not nested here.

`python3 -B VERIFY.py` checks ciphertext identities without a key. Supplying
`--prior-inventory` checks the 818 separately supplied nonce values without a
key. Supplying the separately held key with `--key` and a fresh non-existing
`--restore-root` authenticates and restores the archives. Optional restricted
plaintext identities can be checked with `--plaintext-inventory`.

The prior nonce inventory is not proof of every past use of the key. The key,
prior ciphertexts, historical Git objects and originals remain separately
held on the same machine. Independent custody, full Git closure, product
validation, repair, checkpoint 3821 recovery and audit closure are not implied.
