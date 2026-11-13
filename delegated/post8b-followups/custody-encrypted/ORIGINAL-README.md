# Fixed post-8b evidence custody group

This group preserves 44 selected roots, an immediate stage-builder snapshot,
and a pre-capture control root. Regular-file bytes and root, directory and file
modes are preserved. Owners, timestamps, ACLs, hardlink topology and later work
are outside the claim.

`python3 -B VERIFY.py` checks ciphertext identities without a key. With the
separately held key, `--key KEY --restore-root NEW_DESTINATION` authenticates
and restores a fresh destination. `--prior-inventory INVENTORY` checks 772
supplied prior nonces; their provenance and completeness remain unproved.
`--plaintext-inventory INVENTORY` with the key checks separately held archive
identities; source comparison is separately recorded.

The omitted stage-public files match Git commit
`8b4b9b9aae3b9d9821418fa3ad5cff4a21e5384b` under the recorded prefix, using the
same-machine object store. The commit automatic-packing notice is preserved;
its effect on unreachable objects remains unknown. Key, prior ciphertexts,
Git objects and originals are separate same-machine dependencies. Independent
custody, complete Git closure, checkpoint 3821, product validation, repair and
audit closure are not established.
