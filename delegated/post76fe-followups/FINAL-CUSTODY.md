# Final review and preinstallation custody

`final-encrypted/` holds the two guard reviewers' restricted evidence, the
application custody control delivery, and the fixed application delivery,
receipt replay, preinstallation check and scan collections. Its embedded
control is a separate eighth collection. The recorded authenticated restore
matched 514 regular files and 45 directory or root modes. The captured scan
tree is the earlier, explicitly bounded snapshot; later stage additions are
covered by later scans, not retroactively by that snapshot.

`final-custody-receipt/` records the restore and the earlier finite scan.
`FINAL-CUSTODY-SCAN.json` records a later seven-metadata-file scan. The public
identity checks can be repeated from this directory:

```sh
python3 -B final-encrypted/VERIFY.py
python3 -B final-custody-receipt/VERIFY-CUSTODY.py --ciphertexts final-encrypted
```

These checks were run separately before stage capture. The checkpoint guard
binds their captured file bytes but does not invoke these two commands. The
optional prior-inventory check was also run against a separately held
638-value inventory; supplying an inventory does not establish its provenance.
Authenticated restoration needs the separately held key. The full
post-build control delivery and its own later verification outputs remain
outside this ciphertext's cutoff. Earlier ciphertexts, selected historical
Git objects and source anchors remain dependencies; the minimal Git subset
is not full Git closure. Same-machine custody is not an independent backup,
and finite scans cannot rule out unknown private values or encodings.

The installer, closing snapshot, publication and remote recovery occur after
this custody cutoff and have separate records. This packet does not recover
checkpoint 3821, close a lens, implement repairs or establish product behavior.
