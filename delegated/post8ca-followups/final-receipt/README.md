# Final fixed-cutoff process custody receipt

This receipt records same-machine authentication, fresh restore and comparison
of 23 fixed process roots plus one encrypted pre-capture control root.
`python3 -B VERIFY-RECEIPT.py` checks receipt bytes. Optional `--ciphertexts
DIRECTORY` checks the bound ciphertext identities without the key; optional
`--prior-inventory INVENTORY` compares their nonces with 703 supplied values.
Neither repeats authenticated restore or proves prior-inventory provenance.

The key, original evidence, prior ciphertexts and Git objects remain separate
same-machine dependencies. This is not independent backup, full Git closure,
original checkpoint 3821 recovery, product/source validation, repair or lens
closure. Guard, installation, publication, post-build controls and subsequent
source work remain outside the fixed cutoff.
