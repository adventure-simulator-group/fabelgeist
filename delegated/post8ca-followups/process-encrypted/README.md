# Encrypted process evidence

This group contains 36 frozen process roots and one encrypted pre-capture
control/inventory root. It preserves regular-file bytes and root, directory
and file modes; ownership, timestamps, ACLs, hardlink topology and later work
are not preserved. Later application/installation controls and source work
remain separate dependencies.

Run `python3 -B VERIFY.py` for no-key ciphertext identity checks. With the
separately held 32-byte key, `--key KEY --restore-root NEW_DESTINATION`
authenticates and restores to a fresh destination; source comparison requires
the separately held recorded inventory or control object. An optional
`--prior-inventory PRIOR` checks the new nonces against 650 supplied values,
but cannot prove that inventory's provenance. The key, old ciphertexts, Git
objects and original evidence remain separately held on the same machine.
There is no independent backup, full Git closure, original checkpoint 3821
recovery, product/source validation, repair or lens closure claim.
