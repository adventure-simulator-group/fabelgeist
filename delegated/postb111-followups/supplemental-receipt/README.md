# Supplemental process-evidence restoration receipt

The receipt records a completed same-machine decryption and byte/mode
comparison, including nine evidence-only packet replays. It is not itself a
fresh restoration or independent custody. The group preserves the first 46
ciphertexts and adds the primary receipt-scan subtree plus three pinned archive
replicas. Other restored/public trees, the separately held primary key,
historical Git objects, q2812 anchors, nested keys and earlier ciphertext
remain dependencies. Ownership and timestamps were normalized.

Run `python3 -B VERIFY.py --ciphertexts ENCRYPTED_GROUP` to verify object
identities. `--predecessor PREVIOUS_GROUP` checks the preserved objects;
`--git-checkout EVIDENCE_REPO --primary PRIMARY_GROUP` also repeats the external
nonce comparison. Fresh plaintext verification or restoration requires the
separately held key and the encrypted group's own verifier.
