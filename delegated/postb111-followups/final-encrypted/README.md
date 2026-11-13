# Encrypted final process evidence

Run `python3 -B VERIFY.py` to check ciphertext identities. Supplying the
separately held same-machine primary key authenticates the plaintext; a new
`--restore-root` restores without overwriting. The encrypted final-control
object carries the restricted source map and nonce-scope receipt.

The earlier primary and supplemental ciphertext, historical Git objects,
q2812 anchors and nested keys remain separate dependencies. Source-root,
directory and file modes are preserved; ownership and timestamps were
normalized. This is not independent custody, missing checkpoint 3821
recovery, product validation or semantic audit closure. Ciphertext is
opaque to public privacy scans.
