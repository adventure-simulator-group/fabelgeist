# Second post-8b restoration receipt

The referenced group was authenticated and restored on the same machine. This
receipt records the subsequent file-byte and root, directory and file-mode
comparison; it is not a new restore or proof of independent custody.

`python3 -B VERIFY-RECEIPT.py` checks the receipt itself. Add `--ciphertexts`
with the group directory to check ciphertext identities without a key; add
`--prior-inventory` to compare its nonces against the separately held 818-value
inventory. Authentication and source comparison remain separately recorded.

The first comparison attempt failed on an extra metadata schema field in the
final control object. The corrected complete comparison passed; both attempts
and their exact child and outer streams are retained in restricted controls.
Key, source, prior inventory and ciphertext dependencies, same-machine limits,
cutoff exclusions, and unsupported metadata remain qualified in the receipt.
