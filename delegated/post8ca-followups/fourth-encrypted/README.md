# Fourth fixed-cutoff evidence group

This group contains 44 fixed source/process roots and one pre-capture control
root. It preserves regular-file bytes and root, directory and file modes. It
does not preserve owners, timestamps, ACLs, hardlink topology or later work.
Scan and installation roots contain their script/metadata cutoff; later results
are separate.

`python3 -B VERIFY.py` checks ciphertext identities without a key. With the
separately held 32-byte key, `--key KEY --restore-root NEW_DESTINATION`
authenticates and restores to a fresh destination. `--prior-inventory PRIOR`
optionally checks against 727 supplied values but cannot prove their provenance
or completeness. With the key, `--plaintext-inventory INVENTORY` compares
separately held archive identities; source comparison is separate. The key,
earlier ciphertexts, original records and Git objects remain same-machine
dependencies. Independent custody, full Git closure, missing checkpoint 3821,
product validation, repairs and lens closure are not established.
