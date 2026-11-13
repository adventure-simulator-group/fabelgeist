# Process evidence custody receipt

The receipt records same-machine authentication, fresh restore and comparison
of 36 fixed process roots plus one encrypted pre-capture control root.
`python3 -B VERIFY-RECEIPT.py` checks the sealed receipt bytes. Optional
`--ciphertexts DIRECTORY` checks bound ciphertext identities and auxiliary
metadata without the key; optional `--prior-inventory INVENTORY` compares
their nonces with 650 supplied prior values. Neither repeats authenticated
restoration or proves prior-inventory provenance.

The key, original evidence, earlier ciphertexts, Git objects, and later
application/installation controls or source work remain separately held
dependencies. This is not independent backup, full Git closure, recovery of
the original checkpoint 3821 queue or ledger, product/source validation,
repair or lens closure.

The included predecessor is unchanged; CORRECTION.json records the corrected
Windows-path scan and its single- and double-backslash positive controls.
