# Final fixed-cutoff process evidence

This group preserves 23 frozen process roots and one pre-capture control root:
regular-file bytes and root, directory and file modes. Owners, timestamps,
ACLs, hardlink topology and later work are not preserved. One captured scan
collection contains only scripts and metadata; later scan logs are separate.

Run `python3 -B VERIFY.py` for no-key ciphertext identity checks. With the
separately held 32-byte key, `--key KEY --restore-root NEW_DESTINATION`
authenticates and restores to a fresh destination. An optional
`--prior-inventory PRIOR` compares nonces against 703 supplied values, but
cannot prove their provenance or completeness. With the key, an optional
`--plaintext-inventory INVENTORY` checks separately held archive identities;
source comparison remains a separate operation. The key, earlier ciphertexts,
original evidence and Git objects remain same-machine dependencies. No
independent backup, full Git closure, original checkpoint 3821 recovery,
product/source validation, repair or lens closure is established. Guard,
installation, publication and subsequent source work are separate dependencies.
